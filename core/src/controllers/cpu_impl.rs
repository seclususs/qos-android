//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::cached_file::{CachedFile, CheckStrategy};
use crate::hal::filesystem;
use crate::hal::kernel;
use crate::hal::thermal::ThermalSensor;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::*;
use crate::config::tunables::*;
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::algorithms::cpu_math::{self, CpuTunables};
use crate::algorithms::thermal_math::{ThermalManager, ThermalTunables};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::daemon::state::{update_cpu_pressure, get_memory_pressure}; 
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;

use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

pub struct CpuController {
    fd: File,
    latency: CachedFile,
    min_gran: CachedFile,
    wakeup: CachedFile,
    migration: CachedFile,
    psi_monitor: PsiMonitor,
    thermal_manager: ThermalManager,
    thermal_tunables: ThermalTunables,
    cpu_sensor: ThermalSensor,
    battery_sensor: ThermalSensor,
    current_latency: f64,
    current_min_gran: f64,
    current_wakeup: f64,
    current_migration: f64,
    prev_impulse_smooth: f64,
    tunables: CpuTunables,
    poller: AdaptivePoller,
    next_wake_ms: i32,
}

impl CpuController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("CpuController: Initializing...");
        let raw_fd = kernel::register_psi_trigger(K_PSI_CPU_PATH, 100000, 1000000)
            .map_err(|e| QosError::FfiError(format!("CPU Trigger Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let latency = CachedFile::new(filesystem::open_file_for_write(K_SCHED_LATENCY_NS)?, 0);
        let min_gran = CachedFile::new(filesystem::open_file_for_write(K_SCHED_MIN_GRANULARITY_NS)?, 0);
        let wakeup = CachedFile::new(filesystem::open_file_for_write(K_SCHED_WAKEUP_GRANULARITY_NS)?, 0);
        let migration = CachedFile::new_opt(filesystem::open_file_for_write(K_SCHED_MIGRATION_COST_NS).ok(), 0);
        let psi_monitor = PsiMonitor::new(K_PSI_CPU_PATH)?;
        let cpu_sensor = ThermalSensor::new(K_CPU_TEMP_PATH, 70.0);
        let battery_sensor = ThermalSensor::new(K_BATTERY_TEMP_PATH, 35.0);
        let tunables = CpuTunables {
            min_latency_ns: MIN_LATENCY_NS as f64,
            max_latency_ns: MAX_LATENCY_NS as f64,
            min_granularity_ns: MIN_GRANULARITY_NS as f64,
            max_granularity_ns: MAX_GRANULARITY_NS as f64,
            min_wakeup_ns: MIN_WAKEUP_NS as f64,
            max_wakeup_ns: MAX_WAKEUP_NS as f64,
            min_migration_cost: MIN_MIGRATION_COST as f64,
            max_migration_cost: MAX_MIGRATION_COST as f64,
            trend_factor: 0.2,
            alpha_smooth: 0.5,
            burst_threshold: 25.0,
            sigmoid_k: 0.15,
            sigmoid_mid: 20.0,
            decay_coeff: 0.05,
            latency_gran_ratio: 0.75,
            memory_migration_alpha: 1.0,
            memory_granularity_scaling: 0.6,
            memory_burst_penalty: 1.0,
        };
        let thermal_tunables = ThermalTunables {
            pid_kp: 0.06,
            pid_ki: 0.002,
            pid_kd: 0.22,
            target_headroom: 30.0,
            hard_limit_cpu: 70.0,
            hard_limit_bat: 40.0,
            leakage_k: 0.12,
            leakage_start_temp: 58.0,
            bucket_capacity: 450.0,
            bucket_leak_base: 6.0,
            psi_threshold: 30.0,
            psi_strength: 0.2,
        };
        let thermal_manager = ThermalManager::new();
        let poller = AdaptivePoller::new(1.0, 2.5);
        let mut controller = Self {
            fd,
            latency,
            min_gran,
            wakeup,
            migration,
            psi_monitor,
            thermal_manager,
            thermal_tunables,
            cpu_sensor,
            battery_sensor,
            current_latency: MIN_LATENCY_NS as f64, 
            current_min_gran: MIN_GRANULARITY_NS as f64,
            current_wakeup: MIN_WAKEUP_NS as f64,
            current_migration: MIN_MIGRATION_COST as f64,
            prev_impulse_smooth: 0.0,
            tunables,
            poller,
            next_wake_ms: MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_dynamics(&mut self) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let memory_psi = get_memory_pressure();
        let raw_p = some.current.max(some.avg10);
        let cpu_temp = self.cpu_sensor.read();
        let bat_temp = self.battery_sensor.read();
        let damping_factor = self.thermal_manager.update(cpu_temp, bat_temp, raw_p, &self.thermal_tunables);
        let trend_gain = cpu_math::calculate_trend_gain(some.avg10, some.avg60, memory_psi, &self.tunables);
        let p_eff = raw_p * trend_gain * damping_factor;
        update_cpu_pressure(p_eff);
        self.next_wake_ms = self.poller.calculate_next_interval(p_eff, some.avg300) as i32;
        let delta_raw = some.current - some.avg10;
        let delta_smooth = cpu_math::smooth_delta(delta_raw, self.prev_impulse_smooth, &self.tunables);
        self.prev_impulse_smooth = delta_smooth;
        let target_migration = cpu_math::calculate_migration_cost(delta_smooth, p_eff, memory_psi, &self.tunables);
        let thermal_floor = cpu_math::calculate_thermal_floor(some.avg60, &self.tunables);
        let (target_latency, target_min_gran) = cpu_math::calculate_latency_and_granularity(
            p_eff, 
            some.avg10, 
            some.avg300, 
            thermal_floor, 
            memory_psi, 
            &self.tunables
        );
        let target_wakeup = cpu_math::calculate_wakeup_granularity(p_eff, &self.tunables);
        self.current_latency = target_latency;
        self.current_min_gran = target_min_gran;
        self.current_wakeup = target_wakeup;
        self.current_migration = target_migration;
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let lat_u64 = crate::algorithms::sanitize_to_u64(
            self.current_latency,
            self.tunables.max_latency_ns as u64
        );
        let gran_u64 =crate::algorithms::sanitize_to_u64(
            self.current_min_gran,
            self.tunables.max_granularity_ns as u64
        );
        let wake_u64 = crate::algorithms::sanitize_to_u64(
            self.current_wakeup,
            self.tunables.max_wakeup_ns as u64
        );
        let mig_u64 = crate::algorithms::sanitize_to_u64(
            self.current_migration,
            self.tunables.min_migration_cost as u64
        );
        self.latency.update(lat_u64, force, CheckStrategy::Relative(0.05));
        self.min_gran.update(gran_u64, force, CheckStrategy::Relative(0.05));
        self.wakeup.update(wake_u64, force, CheckStrategy::Relative(0.10));
        self.migration.update(mig_u64, force, CheckStrategy::Absolute(10000));
    }
}

impl EventHandler for CpuController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Err(e) = self.update_dynamics() {
            log::warn!("Cpu Logic Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_dynamics() {
            log::warn!("Cpu Timeout Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::PRI | rustix::event::epoll::EventFlags::ERR
    }
}