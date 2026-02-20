//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::{cpu_math, poll_math, thermal_math};
use crate::config::{kernel_limits, loop_settings};
use crate::daemon::{state, traits, types};
use crate::hal::{battery, filesystem, kernel, thermal};
use crate::monitors::psi_monitor;
use crate::resources::sys_paths;
use crate::utils::{cached_file, math};

use std::{fs, io, os, time};

pub struct CpuController {
    fd: fs::File,
    latency: cached_file::CachedFile,
    min_gran: cached_file::CachedFile,
    wakeup: cached_file::CachedFile,
    migration: cached_file::CachedFile,
    walt_init: cached_file::CachedFile,
    uclamp_min: cached_file::CachedFile,
    psi_cpu: psi_monitor::PsiMonitor,
    thermal_manager: thermal_math::ThermalManager,
    thermal_config: thermal_math::ThermalConfig,
    cpu_sensor: thermal::ThermalSensor,
    battery_sensor: thermal::ThermalSensor,
    battery_capacity_sensor: battery::BatterySensor,
    cached_bat_level: f32,
    cached_bat_temp: f32,
    last_bat_check: time::Instant,
    current_latency: f32,
    current_min_gran: f32,
    current_wakeup: f32,
    current_migration: f32,
    current_walt_init: f32,
    current_uclamp_min: f32,
    load_state: cpu_math::LoadState,
    cpu_math_config: cpu_math::CpuMathConfig,
    cpu_kernel_limits: cpu_math::CpuKernelLimits,
    last_tick: time::Instant,
    poller: poll_math::AdaptivePoller,
    next_wake_ms: i32,
}

impl CpuController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("CpuController: Initializing...");
        let config_limits = kernel_limits::GlobalConfig::default().cpu_config;
        let cpu_math_config = cpu_math::CpuMathConfig::default();
        let cpu_kernel_limits = cpu_math::CpuKernelLimits {
            min_latency_ns: config_limits.min_latency_ns as f32,
            max_latency_ns: config_limits.max_latency_ns as f32,
            min_granularity_ns: config_limits.min_granularity_ns as f32,
            max_granularity_ns: config_limits.max_granularity_ns as f32,
            min_wakeup_ns: config_limits.min_wakeup_ns as f32,
            max_wakeup_ns: config_limits.max_wakeup_ns as f32,
            min_migration_cost: config_limits.min_migration_cost as f32,
            max_migration_cost: config_limits.max_migration_cost as f32,
            min_walt_init_pct: config_limits.min_walt_init_pct as f32,
            max_walt_init_pct: config_limits.max_walt_init_pct as f32,
            min_uclamp_min: config_limits.min_uclamp_min as f32,
            max_uclamp_min: config_limits.max_uclamp_min as f32,
        };
        let raw_fd = kernel::register_psi_trigger(sys_paths::K_PSI_CPU_PATH, 100_000, 1_000_000)
            .map_err(|e| types::QosError::FfiError(format!("CPU Trigger Error: {e}")))?;
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
            config_limits.min_walt_init_pct,
        );
        let uclamp_min = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SCHED_UCLAMP_UTIL_MIN).ok(),
            config_limits.min_uclamp_min,
        );
        let psi_cpu = psi_monitor::PsiMonitor::new(sys_paths::K_PSI_CPU_PATH)?;
        let cpu_path = sys_paths::get_cpu_temp_path();
        let cpu_sensor = thermal::ThermalSensor::new(cpu_path.to_str().unwrap_or_default(), 70.0);
        let battery_sensor = thermal::ThermalSensor::new(sys_paths::K_BATTERY_TEMP_PATH, 35.0);
        let battery_capacity_sensor =
            battery::BatterySensor::new(sys_paths::K_BATTERY_CAPACITY_PATH);
        let thermal_config = thermal_math::ThermalConfig::default();
        let thermal_manager = thermal_math::ThermalManager::default();
        let poller = poll_math::AdaptivePoller::new(1.5, 0.05, poll_math::PollerConfig::default());
        let mut controller = Self {
            fd,
            latency,
            min_gran,
            wakeup,
            migration,
            walt_init,
            uclamp_min,
            psi_cpu,
            thermal_manager,
            thermal_config,
            cpu_sensor,
            battery_sensor,
            battery_capacity_sensor,
            cached_bat_level: 50.0,
            cached_bat_temp: 35.0,
            last_bat_check: time::Instant::now(),
            current_latency: config_limits.min_latency_ns as f32,
            current_min_gran: config_limits.min_granularity_ns as f32,
            current_wakeup: config_limits.min_wakeup_ns as f32,
            current_migration: config_limits.min_migration_cost as f32,
            current_walt_init: config_limits.min_walt_init_pct as f32,
            current_uclamp_min: config_limits.min_uclamp_min as f32,
            load_state: cpu_math::LoadState::default(),
            cpu_math_config,
            cpu_kernel_limits,
            last_tick: time::Instant::now(),
            poller,
            next_wake_ms: loop_settings::MIN_POLLING_MS as i32,
        };
        controller.cached_bat_level = controller.battery_capacity_sensor.read();
        controller.cached_bat_temp = controller.battery_sensor.read();
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_dynamics(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<(), types::QosError> {
        let data_cpu = self.psi_cpu.read_state()?;
        let some_cpu = data_cpu.some;
        let io_psi = context.pressure.io_psi;
        let target_psi = some_cpu.current;
        let is_break = some_cpu.nis > self.cpu_math_config.nis_threshold;
        let cpu_temp = self.cpu_sensor.read();
        let now = time::Instant::now();
        if now.duration_since(self.last_bat_check).as_secs() >= 5 {
            self.cached_bat_level = self.battery_capacity_sensor.read();
            self.cached_bat_temp = self.battery_sensor.read();
            self.last_bat_check = now;
        }
        let bat_level = self.cached_bat_level;
        let bat_temp = self.cached_bat_temp;
        let thermal_scale =
            self.thermal_manager
                .update(cpu_temp, bat_temp, target_psi, &self.thermal_config);
        let trend_factor = cpu_math::calculate_trend_gain(some_cpu.velocity);
        let dt_duration = now.duration_since(self.last_tick);
        self.last_tick = now;
        let dt_real = dt_duration.as_secs_f32().max(0.000_001);
        let dt_safe = cpu_math::sanitize_dt(dt_real);
        let (integral_total, integral_dot) = cpu_math::update_integral_params(
            &mut self.load_state,
            bat_level,
            dt_safe,
            &self.cpu_math_config,
        );
        let demand_input = cpu_math::DemandInput {
            target_psi,
            psi_velocity: some_cpu.velocity,
            dt_real,
            dt_safe,
            thermal_scale,
            trend_factor,
            integral_total,
            integral_dot,
            is_structural_break: is_break,
        };
        let load_demand = cpu_math::calculate_load_demand(
            &mut self.load_state,
            demand_input,
            &self.cpu_math_config,
        );
        let p_eff = cpu_math::calculate_effective_pressure(
            load_demand,
            trend_factor,
            io_psi,
            &self.cpu_math_config,
        );
        context.pressure.cpu_psi = p_eff;
        let mut calculated_poll =
            self.poller
                .calculate_next_interval(p_eff, some_cpu.avg300, some_cpu.velocity)
                as i32;
        if cpu_math::is_transient(&self.load_state, target_psi, &self.cpu_math_config) {
            calculated_poll =
                calculated_poll.min(self.cpu_math_config.transient_poll_interval as i32);
        }
        self.next_wake_ms = calculated_poll;
        let thermal_min_latency_ns =
            cpu_math::calculate_thermal_latency_limit(thermal_scale, &self.cpu_kernel_limits);
        let (target_latency, target_min_gran) = cpu_math::calculate_latency_and_granularity(
            p_eff,
            load_demand,
            thermal_min_latency_ns,
            &self.cpu_math_config,
            &self.cpu_kernel_limits,
        );
        let target_migration =
            cpu_math::calculate_migration_cost(some_cpu.velocity, p_eff, &self.cpu_kernel_limits);
        let target_wakeup = cpu_math::calculate_wakeup_granularity(
            p_eff,
            &self.cpu_math_config,
            &self.cpu_kernel_limits,
        );
        let target_walt_init = cpu_math::calculate_walt_init(p_eff, &self.cpu_kernel_limits);
        let target_uclamp = cpu_math::calculate_uclamp_min(
            p_eff,
            thermal_scale,
            &self.cpu_math_config,
            &self.cpu_kernel_limits,
        );
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
        let lat_u64 = math::sanitize_to_clean_u64(
            self.current_latency,
            self.cpu_kernel_limits.max_latency_ns as u64,
            50_000,
        );
        let gran_u64 = math::sanitize_to_clean_u64(
            self.current_min_gran,
            self.cpu_kernel_limits.max_granularity_ns as u64,
            50_000,
        );
        let wake_u64 = math::sanitize_to_clean_u64(
            self.current_wakeup,
            self.cpu_kernel_limits.max_wakeup_ns as u64,
            50_000,
        );
        let mig_u64 = math::sanitize_to_clean_u64(
            self.current_migration,
            self.cpu_kernel_limits.min_migration_cost as u64,
            50_000,
        );
        let walt_u64 = math::sanitize_to_u64(
            self.current_walt_init,
            self.cpu_kernel_limits.min_walt_init_pct as u64,
        );
        let uclamp_u64 = math::sanitize_to_u64(
            self.current_uclamp_min,
            self.cpu_kernel_limits.min_uclamp_min as u64,
        );
        self.latency
            .update(lat_u64, force, &cached_file::CheckStrategy::Relative(0.10));
        self.min_gran
            .update(gran_u64, force, &cached_file::CheckStrategy::Relative(0.10));
        self.wakeup
            .update(wake_u64, force, &cached_file::CheckStrategy::Relative(0.15));
        self.migration
            .update(mig_u64, force, &cached_file::CheckStrategy::Absolute(50000));
        self.walt_init
            .update(walt_u64, force, &cached_file::CheckStrategy::Absolute(5));
        self.uclamp_min
            .update(uclamp_u64, force, &cached_file::CheckStrategy::Absolute(32));
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
            log::warn!("Cpu Logic Error: {e}");
        }
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        if let Err(e) = self.update_dynamics(context) {
            log::warn!("Cpu Timeout Error: {e}");
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
