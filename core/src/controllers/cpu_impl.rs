//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::{cpu_math, poll_math, thermal_math};
use crate::config::{loop_settings, tunables};
use crate::daemon::{state, traits, types};
use crate::hal::{battery, cached_file, filesystem, kernel, thermal};
use crate::monitors::psi_monitor;
use crate::resources::sys_paths;

use std::{fs, io, os, time};

pub struct CpuController {
    fd: fs::File,
    latency: cached_file::CachedFile,
    min_gran: cached_file::CachedFile,
    wakeup: cached_file::CachedFile,
    migration: cached_file::CachedFile,
    walt_init: cached_file::CachedFile,
    uclamp_min: cached_file::CachedFile,
    psi_monitor: psi_monitor::PsiMonitor,
    thermal_manager: thermal_math::ThermalManager,
    thermal_config: thermal_math::ThermalConfig,
    cpu_sensor: thermal::ThermalSensor,
    battery_sensor: thermal::ThermalSensor,
    battery_capacity_sensor: battery::BatterySensor,
    current_latency: f32,
    current_min_gran: f32,
    current_wakeup: f32,
    current_migration: f32,
    current_walt_init: f32,
    current_uclamp_min: f32,
    load_state: cpu_math::LoadState,
    last_tick: time::Instant,
    prev_delta_smooth: f32,
    tunables: cpu_math::CpuTunables,
    poller: poll_math::AdaptivePoller,
    next_wake_ms: i32,
}

impl CpuController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("CpuController: Initializing...");
        let config = tunables::GlobalConfig::default();
        let raw_fd = kernel::register_psi_trigger(sys_paths::K_PSI_CPU_PATH, 100000, 1000000)
            .map_err(|e| types::QosError::FfiError(format!("CPU Trigger Error: {}", e)))?;
        let fd = unsafe { os::fd::FromRawFd::from_raw_fd(raw_fd) };
        let latency = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_LATENCY_NS).ok(),
            0,
        );
        let min_gran = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_MIN_GRANULARITY_NS).ok(),
            0,
        );
        let wakeup = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_WAKEUP_GRANULARITY_NS).ok(),
            0,
        );
        let migration = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_MIGRATION_COST_NS).ok(),
            0,
        );
        let walt_init = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_WALT_INIT_TASK_LOAD_PCT).ok(),
            config.cpu.min_walt_init_pct,
        );
        let uclamp_min = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_UCLAMP_UTIL_MIN).ok(),
            config.cpu.min_uclamp_min,
        );
        if !latency.is_active()
            && !min_gran.is_active()
            && !wakeup.is_active()
            && !migration.is_active()
            && !walt_init.is_active()
            && !uclamp_min.is_active()
        {
            return Err(types::QosError::SystemCheckFailed(
                "No supported CPU kernel knobs found.".to_string(),
            ));
        }
        let psi_monitor = psi_monitor::PsiMonitor::new(sys_paths::K_PSI_CPU_PATH)?;
        let cpu_path = sys_paths::get_cpu_temp_path();
        let cpu_sensor = thermal::ThermalSensor::new(cpu_path.to_str().unwrap_or_default(), 70.0);
        let battery_sensor = thermal::ThermalSensor::new(sys_paths::K_BATTERY_TEMP_PATH, 35.0);
        let battery_capacity_sensor =
            battery::BatterySensor::new(sys_paths::K_BATTERY_CAPACITY_PATH);
        let thermal_config = thermal_math::ThermalConfig::default();
        let tunables = cpu_math::CpuTunables {
            min_latency_ns: config.cpu.min_latency_ns as f32,
            max_latency_ns: config.cpu.max_latency_ns as f32,
            min_granularity_ns: config.cpu.min_granularity_ns as f32,
            max_granularity_ns: config.cpu.max_granularity_ns as f32,
            min_wakeup_ns: config.cpu.min_wakeup_ns as f32,
            max_wakeup_ns: config.cpu.max_wakeup_ns as f32,
            min_migration_cost: config.cpu.min_migration_cost as f32,
            max_migration_cost: config.cpu.max_migration_cost as f32,
            min_walt_init_pct: config.cpu.min_walt_init_pct as f32,
            max_walt_init_pct: config.cpu.max_walt_init_pct as f32,
            min_uclamp_min: config.cpu.min_uclamp_min as f32,
            max_uclamp_min: config.cpu.max_uclamp_min as f32,
            latency_gran_ratio: config.cpu.latency_gran_ratio,
            decay_coeff: config.cpu.decay_coeff,
            uclamp_k: config.cpu.uclamp_k,
            uclamp_mid: config.cpu.uclamp_mid,
            response_gain: config.cpu.response_gain,
            stability_ratio: config.cpu.stability_ratio,
            stability_margin: config.cpu.stability_margin,
            gain_scheduling_alpha: config.cpu.gain_scheduling_alpha,
            alpha_smooth: config.cpu.alpha_smooth,
            sigmoid_k: config.cpu.sigmoid_k,
            sigmoid_mid: config.cpu.sigmoid_mid,
            lookahead_time: config.cpu.lookahead_time,
            variance_sensitivity: config.cpu.variance_sensitivity,
            efficiency_gain: config.cpu.efficiency_gain,
            trend_amplification: config.cpu.trend_amplification,
            surge_threshold: config.cpu.surge_threshold,
            surge_gain: config.cpu.surge_gain,
            transient_rate_threshold: config.cpu.transient_rate_threshold,
            transient_diff_threshold: config.cpu.transient_diff_threshold,
            transient_poll_interval: config.cpu.transient_poll_interval,
            nis_threshold: config.cpu.nis_threshold,
            safe_temp_limit: config.cpu.safe_temp_limit,
            max_temp_limit: config.cpu.max_temp_limit,
            temp_cost_weight: config.cpu.temp_cost_weight,
            bat_temp_weight: config.cpu.bat_temp_weight,
            bat_level_weight: config.cpu.bat_level_weight,
            integral_acc_rate: config.cpu.integral_acc_rate,
            memory_migration_alpha: config.cpu.memory_migration_alpha,
            memory_granularity_scaling: config.cpu.memory_granularity_scaling,
            memory_volatility_cost: config.cpu.memory_volatility_cost,
        };
        let thermal_manager = thermal_math::ThermalManager::default();
        let poller = poll_math::AdaptivePoller::new(1.0, 2.5, poll_math::PollerConfig::default());
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
            thermal_config,
            cpu_sensor,
            battery_sensor,
            battery_capacity_sensor,
            current_latency: config.cpu.min_latency_ns as f32,
            current_min_gran: config.cpu.min_granularity_ns as f32,
            current_wakeup: config.cpu.min_wakeup_ns as f32,
            current_migration: config.cpu.min_migration_cost as f32,
            current_walt_init: config.cpu.min_walt_init_pct as f32,
            current_uclamp_min: config.cpu.min_uclamp_min as f32,
            load_state: cpu_math::LoadState::default(),
            last_tick: time::Instant::now(),
            prev_delta_smooth: 0.0,
            tunables,
            poller,
            next_wake_ms: loop_settings::MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_dynamics(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<(), types::QosError> {
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
                .update(cpu_temp, bat_temp, target_psi, &self.thermal_config);
        let trend_factor =
            cpu_math::calculate_trend_gain(some.avg10, some.avg60, memory_psi, &self.tunables);
        let now = time::Instant::now();
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
            .update(lat_u64, force, cached_file::CheckStrategy::Relative(0.05));
        self.min_gran
            .update(gran_u64, force, cached_file::CheckStrategy::Relative(0.05));
        self.wakeup
            .update(wake_u64, force, cached_file::CheckStrategy::Relative(0.10));
        self.migration
            .update(mig_u64, force, cached_file::CheckStrategy::Absolute(50000));
        self.walt_init
            .update(walt_u64, force, cached_file::CheckStrategy::Absolute(3));
        self.uclamp_min
            .update(uclamp_u64, force, cached_file::CheckStrategy::Absolute(32));
    }
}

impl traits::EventHandler for CpuController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.fd)
    }
    fn on_event(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        let mut buf = [0u8; 8];
        let _ = io::Read::read(&mut self.fd, &mut buf);
        if let Err(e) = self.update_dynamics(context) {
            log::warn!("Cpu Logic Error: {}", e);
        }
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        if let Err(e) = self.update_dynamics(context) {
            log::warn!("Cpu Timeout Error: {}", e);
        }
        Ok(traits::LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::PRI | rustix::event::epoll::EventFlags::ERR
    }
}
