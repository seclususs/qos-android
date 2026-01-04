//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::cpu_math::{self, CpuTunables, PhysicsState};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::algorithms::thermal_math::{ThermalManager, ThermalTunables};
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::config::tunables::*;
use crate::daemon::state::{get_io_pressure, get_memory_pressure, update_cpu_pressure};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::cached_file::{CachedFile, CheckStrategy};
use crate::hal::filesystem;
use crate::hal::kernel;
use crate::hal::thermal::ThermalSensor;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::*;

use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::time::Instant;

pub struct CpuController {
    fd: File,
    latency: CachedFile,
    min_gran: CachedFile,
    wakeup: CachedFile,
    migration: CachedFile,
    nr_migrate: CachedFile,
    walt_init: CachedFile,
    uclamp_min: CachedFile,
    psi_monitor: PsiMonitor,
    thermal_manager: ThermalManager,
    thermal_tunables: ThermalTunables,
    cpu_sensor: ThermalSensor,
    battery_sensor: ThermalSensor,
    current_latency: f64,
    current_min_gran: f64,
    current_wakeup: f64,
    current_migration: f64,
    current_nr_migrate: f64,
    current_walt_init: f64,
    current_uclamp_min: f64,
    physics_state: PhysicsState,
    last_physics_tick: Instant,
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
        let min_gran = CachedFile::new(
            filesystem::open_file_for_write(K_SCHED_MIN_GRANULARITY_NS)?,
            0,
        );
        let wakeup = CachedFile::new(
            filesystem::open_file_for_write(K_SCHED_WAKEUP_GRANULARITY_NS)?,
            0,
        );
        let migration = CachedFile::new_opt(
            filesystem::open_file_for_write(K_SCHED_MIGRATION_COST_NS).ok(),
            0,
        );
        let nr_migrate = CachedFile::new_opt(
            filesystem::open_file_for_write(K_SCHED_NR_MIGRATE).ok(),
            MAX_NR_MIGRATE,
        );
        let walt_init = CachedFile::new_opt(
            filesystem::open_file_for_write(K_SCHED_WALT_INIT_TASK_LOAD_PCT).ok(),
            MIN_WALT_INIT_PCT,
        );
        let uclamp_min = CachedFile::new_opt(
            filesystem::open_file_for_write(K_SCHED_UCLAMP_UTIL_MIN).ok(),
            MIN_UCLAMP_MIN,
        );
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
            min_nr_migrate: MIN_NR_MIGRATE as f64,
            max_nr_migrate: MAX_NR_MIGRATE as f64,
            min_walt_init_pct: MIN_WALT_INIT_PCT as f64,
            max_walt_init_pct: MAX_WALT_INIT_PCT as f64,
            min_uclamp_min: MIN_UCLAMP_MIN as f64,
            max_uclamp_min: MAX_UCLAMP_MIN as f64,
            nr_migrate_k: 0.15,
            uclamp_k: 0.35,
            uclamp_mid: 45.0,
            trend_factor: 0.3,
            alpha_smooth: 0.5,
            spring_stiffness: 92.0,
            damping_ratio: 1.15,
            gain_scheduling_alpha: 1.85,
            sigmoid_k: 0.20,
            sigmoid_mid: 25.0,
            decay_coeff: 0.08,
            latency_gran_ratio: 0.75,
            memory_migration_alpha: 1.0,
            memory_granularity_scaling: 0.6,
            memory_burst_penalty: 1.0,
            trend_boost_intensity: 0.2,
            animation_vel_threshold: 0.1,
            animation_pos_threshold: 0.5,
            animation_poll_interval: 20.0,
            impulse_threshold: 30.0,
            impulse_factor: 0.45,
            variance_sensitivity: 0.15,
            lookahead_time: 0.05,
            efficiency_gain: 1.5,
        };
        let thermal_tunables = ThermalTunables {
            pid_kp: 0.075,
            pid_ki: 0.003,
            pid_kd: 0.22,
            hard_limit_cpu: 75.0,
            hard_limit_bat: 40.0,
            dth_start_temp: 35.0,
            dth_k_thermal: 3.0,
            tga_k_anticipation: 8.0,
            leakage_k: 0.15,
            leakage_start_temp: 50.0,
            bucket_capacity: 400.0,
            bucket_leak_base: 5.0,
            psi_threshold: 20.0,
            psi_strength: 0.40,
        };
        let thermal_manager = ThermalManager::new();
        let poller = AdaptivePoller::new(1.0, 2.5);
        let mut controller = Self {
            fd,
            latency,
            min_gran,
            wakeup,
            migration,
            nr_migrate,
            walt_init,
            uclamp_min,
            psi_monitor,
            thermal_manager,
            thermal_tunables,
            cpu_sensor,
            battery_sensor,
            current_latency: MIN_LATENCY_NS as f64,
            current_min_gran: MIN_GRANULARITY_NS as f64,
            current_wakeup: MIN_WAKEUP_NS as f64,
            current_migration: MIN_MIGRATION_COST as f64,
            current_nr_migrate: MAX_NR_MIGRATE as f64,
            current_walt_init: MIN_WALT_INIT_PCT as f64,
            current_uclamp_min: MIN_UCLAMP_MIN as f64,
            physics_state: PhysicsState::default(),
            last_physics_tick: Instant::now(),
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
        let io_psi = get_io_pressure();
        let target_psi = some.current.max(some.avg10);
        let cpu_temp = self.cpu_sensor.read();
        let bat_temp = self.battery_sensor.read();
        let damping_factor =
            self.thermal_manager
                .update(cpu_temp, bat_temp, target_psi, &self.thermal_tunables);
        let trend_gain =
            cpu_math::calculate_trend_gain(some.avg10, some.avg60, memory_psi, &self.tunables);
        let now = Instant::now();
        let dt_duration = now.duration_since(self.last_physics_tick);
        self.last_physics_tick = now;
        let dt_sec = dt_duration.as_secs_f64().clamp(0.000001, 0.1);
        let physics_urgency = cpu_math::calculate_physics_urgency(
            &mut self.physics_state,
            target_psi,
            dt_sec,
            damping_factor,
            trend_gain,
            &self.tunables,
        );
        let p_eff = cpu_math::calculate_effective_pressure(
            physics_urgency,
            trend_gain,
            memory_psi,
            io_psi,
            &self.tunables,
        );
        update_cpu_pressure(p_eff);
        let mut calculated_poll = self.poller.calculate_next_interval(p_eff, some.avg300) as i32;
        if cpu_math::is_animating(&self.physics_state, target_psi, &self.tunables) {
            calculated_poll = calculated_poll.min(self.tunables.animation_poll_interval as i32);
        }
        self.next_wake_ms = calculated_poll;
        let thermal_floor_ns = cpu_math::calculate_thermal_floor(damping_factor, &self.tunables);
        let (target_latency, target_min_gran) = cpu_math::calculate_latency_and_granularity(
            p_eff,
            physics_urgency,
            thermal_floor_ns,
            memory_psi,
            &self.tunables,
        );
        let delta_raw = some.current - some.avg10;
        let delta_smooth =
            cpu_math::smooth_delta(delta_raw, self.prev_impulse_smooth, &self.tunables);
        self.prev_impulse_smooth = delta_smooth;
        let target_migration =
            cpu_math::calculate_migration_cost(delta_smooth, p_eff, memory_psi, &self.tunables);
        let target_wakeup = cpu_math::calculate_wakeup_granularity(p_eff, &self.tunables);
        let target_nr_migrate = cpu_math::calculate_nr_migrate(p_eff, &self.tunables);
        let target_walt_init = cpu_math::calculate_walt_init(p_eff, &self.tunables);
        let target_uclamp = cpu_math::calculate_uclamp_min(p_eff, damping_factor, &self.tunables);
        self.current_latency = target_latency;
        self.current_min_gran = target_min_gran;
        self.current_wakeup = target_wakeup;
        self.current_migration = target_migration;
        self.current_nr_migrate = target_nr_migrate;
        self.current_walt_init = target_walt_init;
        self.current_uclamp_min = target_uclamp;
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let lat_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_latency,
            self.tunables.max_latency_ns as u64,
            50_000,
        );
        let gran_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_min_gran,
            self.tunables.max_granularity_ns as u64,
            50_000,
        );
        let wake_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_wakeup,
            self.tunables.max_wakeup_ns as u64,
            50_000,
        );
        let mig_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_migration,
            self.tunables.min_migration_cost as u64,
            50_000,
        );
        let nr_u64 = crate::algorithms::sanitize_to_u64(
            self.current_nr_migrate,
            self.tunables.min_nr_migrate as u64,
        );
        let walt_u64 = crate::algorithms::sanitize_to_u64(
            self.current_walt_init,
            self.tunables.min_walt_init_pct as u64,
        );
        let uclamp_u64 = crate::algorithms::sanitize_to_u64(
            self.current_uclamp_min,
            self.tunables.min_uclamp_min as u64,
        );
        self.latency
            .update(lat_u64, force, CheckStrategy::Relative(0.05));
        self.min_gran
            .update(gran_u64, force, CheckStrategy::Relative(0.05));
        self.wakeup
            .update(wake_u64, force, CheckStrategy::Relative(0.10));
        self.migration
            .update(mig_u64, force, CheckStrategy::Absolute(50000));
        self.nr_migrate
            .update(nr_u64, force, CheckStrategy::Absolute(2));
        self.walt_init
            .update(walt_u64, force, CheckStrategy::Absolute(3));
        self.uclamp_min
            .update(uclamp_u64, force, CheckStrategy::Absolute(32));
    }
}

impl EventHandler for CpuController {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
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