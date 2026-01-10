//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::cpu_math::{self, CpuTunables, LoadState};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::algorithms::thermal_math::{ThermalManager, ThermalTunables};
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::config::tunables::*;
use crate::daemon::state::DaemonContext;
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::battery::BatterySensor;
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
    walt_init: CachedFile,
    uclamp_min: CachedFile,
    psi_monitor: PsiMonitor,
    thermal_manager: ThermalManager,
    thermal_tunables: ThermalTunables,
    cpu_sensor: ThermalSensor,
    battery_sensor: ThermalSensor,
    battery_capacity_sensor: BatterySensor,
    current_latency: f32,
    current_min_gran: f32,
    current_wakeup: f32,
    current_migration: f32,
    current_walt_init: f32,
    current_uclamp_min: f32,
    load_state: LoadState,
    last_tick: Instant,
    prev_delta_smooth: f32,
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
        let battery_capacity_sensor = BatterySensor::new(K_BATTERY_CAPACITY_PATH);
        let tunables = CpuTunables {
            min_latency_ns: MIN_LATENCY_NS as f32,
            max_latency_ns: MAX_LATENCY_NS as f32,
            min_granularity_ns: MIN_GRANULARITY_NS as f32,
            max_granularity_ns: MAX_GRANULARITY_NS as f32,
            min_wakeup_ns: MIN_WAKEUP_NS as f32,
            max_wakeup_ns: MAX_WAKEUP_NS as f32,
            min_migration_cost: MIN_MIGRATION_COST as f32,
            max_migration_cost: MAX_MIGRATION_COST as f32,
            min_walt_init_pct: MIN_WALT_INIT_PCT as f32,
            max_walt_init_pct: MAX_WALT_INIT_PCT as f32,
            min_uclamp_min: MIN_UCLAMP_MIN as f32,
            max_uclamp_min: MAX_UCLAMP_MIN as f32,
            latency_gran_ratio: 0.65,
            decay_coeff: 0.10,
            uclamp_k: 0.12,
            uclamp_mid: 25.0,
            response_gain: 50.0,
            stability_ratio: 1.40,
            stability_margin: 1.6,
            gain_scheduling_alpha: 1.2,
            alpha_smooth: 0.6,
            sigmoid_k: 0.20,
            sigmoid_mid: 30.0,
            lookahead_time: 0.06,
            variance_sensitivity: 0.10,
            efficiency_gain: 2.5,
            trend_amplification: 0.10,
            surge_threshold: 40.0,
            surge_gain: 0.30,
            transient_rate_threshold: 0.25,
            transient_diff_threshold: 1.5,
            transient_poll_interval: 50.0,
            nis_threshold: 8.0,
            safe_temp_limit: 60.0,
            max_temp_limit: 80.0,
            temp_cost_weight: 5.0,
            bat_temp_weight: 4.0,
            bat_level_weight: 60.0,
            integral_acc_rate: 0.2,
            memory_migration_alpha: 1.5,
            memory_granularity_scaling: 0.8,
            memory_volatility_cost: 1.5,
        };
        let thermal_tunables = ThermalTunables {
            hard_limit_cpu: 70.0,
            hard_limit_bat: 40.0,
            sched_temp_cool: 30.0,
            sched_temp_hot: 40.0,
            kp_base: 1.5,
            ki_base: 0.02,
            kd_base: 0.5,
            kp_fast: 5.0,
            ki_fast: 0.10,
            kd_fast: 3.0,
            anti_windup_k: 0.8,
            deriv_filter_n: 10.0,
            ff_gain: 1.5,
            ff_lead_time: 4.0,
            ff_lag_time: 2.0,
            smith_gain: 1.0,
            smith_tau: 10.0,
            smith_delay_sec: 3.0,
        };
        let thermal_manager = ThermalManager::default();
        let poller = AdaptivePoller::new(1.0, 2.5);
        let mut controller = Self {
            fd,
            latency,
            min_gran,
            wakeup,
            migration,
            walt_init,
            uclamp_min,
            psi_monitor,
            thermal_manager,
            thermal_tunables,
            cpu_sensor,
            battery_sensor,
            battery_capacity_sensor,
            current_latency: MIN_LATENCY_NS as f32,
            current_min_gran: MIN_GRANULARITY_NS as f32,
            current_wakeup: MIN_WAKEUP_NS as f32,
            current_migration: MIN_MIGRATION_COST as f32,
            current_walt_init: MIN_WALT_INIT_PCT as f32,
            current_uclamp_min: MIN_UCLAMP_MIN as f32,
            load_state: LoadState::default(),
            last_tick: Instant::now(),
            prev_delta_smooth: 0.0,
            tunables,
            poller,
            next_wake_ms: MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_dynamics(&mut self, context: &mut DaemonContext) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let memory_psi = context.pressure.memory_psi;
        let io_psi = context.pressure.io_psi;
        let target_psi = some.current.max(some.avg10);
        let is_break = some.nis > self.tunables.nis_threshold;
        let cpu_temp = self.cpu_sensor.read();
        let bat_temp = self.battery_sensor.read();
        let bat_level = self.battery_capacity_sensor.read();
        let thermal_scale =
            self.thermal_manager
                .update(cpu_temp, bat_temp, target_psi, &self.thermal_tunables);
        let trend_factor =
            cpu_math::calculate_trend_gain(some.avg10, some.avg60, memory_psi, &self.tunables);
        let now = Instant::now();
        let dt_duration = now.duration_since(self.last_tick);
        self.last_tick = now;
        let dt_sec = cpu_math::sanitize_dt(dt_duration.as_secs_f32());
        let (integral_total, integral_dot) = cpu_math::update_integral_params(
            &mut self.load_state,
            cpu_temp,
            bat_temp,
            bat_level,
            dt_sec,
            &self.tunables,
        );
        let demand_input = cpu_math::DemandInput {
            target_psi,
            dt_sec,
            thermal_scale,
            trend_factor,
            integral_total,
            integral_dot,
            is_structural_break: is_break,
        };
        let load_demand =
            cpu_math::calculate_load_demand(&mut self.load_state, demand_input, &self.tunables);
        let p_eff = cpu_math::calculate_effective_pressure(
            load_demand,
            trend_factor,
            memory_psi,
            io_psi,
            &self.tunables,
        );
        context.pressure.cpu_psi = p_eff;
        let mut calculated_poll = self.poller.calculate_next_interval(p_eff, some.avg300) as i32;
        if cpu_math::is_transient(&self.load_state, target_psi, &self.tunables) {
            calculated_poll = calculated_poll.min(self.tunables.transient_poll_interval as i32);
        }
        self.next_wake_ms = calculated_poll;
        let thermal_min_latency_ns =
            cpu_math::calculate_thermal_latency_limit(thermal_scale, &self.tunables);
        let (target_latency, target_min_gran) = cpu_math::calculate_latency_and_granularity(
            p_eff,
            load_demand,
            thermal_min_latency_ns,
            memory_psi,
            &self.tunables,
        );
        let delta_raw = some.current - some.avg10;
        let delta_smooth =
            cpu_math::smooth_delta(delta_raw, self.prev_delta_smooth, &self.tunables);
        self.prev_delta_smooth = delta_smooth;
        let target_migration =
            cpu_math::calculate_migration_cost(delta_smooth, p_eff, memory_psi, &self.tunables);
        let target_wakeup = cpu_math::calculate_wakeup_granularity(p_eff, &self.tunables);
        let target_walt_init = cpu_math::calculate_walt_init(p_eff, &self.tunables);
        let target_uclamp = cpu_math::calculate_uclamp_min(p_eff, thermal_scale, &self.tunables);
        self.current_latency = target_latency;
        self.current_min_gran = target_min_gran;
        self.current_wakeup = target_wakeup;
        self.current_migration = target_migration;
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
    fn on_event(&mut self, context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Err(e) = self.update_dynamics(context) {
            log::warn!("Cpu Logic Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self, context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_dynamics(context) {
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