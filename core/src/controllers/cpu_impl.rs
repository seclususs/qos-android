//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::filesystem::{self, write_to_stream};
use crate::hal::kernel;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::*;
use crate::config::tunables::*;
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::algorithms::cpu_math::{self, CpuTunables};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::core::state::update_cpu_pressure;
use crate::core::interfaces::{EventHandler, LoopAction};
use crate::core::types::QosError;

use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

struct KernelConfigCache {
    latency: u64,
    min_granularity: u64,
    wakeup_granularity: u64,
    migration_cost: u64,
    perf_cpu_max_percent: u64,
}

pub struct CpuController {
    fd: File,
    latency_file: File,
    min_gran_file: File,
    wakeup_file: File,
    migration_file: Option<File>,
    perf_cpu_file: Option<File>,
    psi_monitor: PsiMonitor,
    current_latency: f64,
    current_min_gran: f64,
    current_wakeup: f64,
    current_migration: f64,
    current_perf_percent: f64,
    prev_impulse_smooth: f64,
    cache: KernelConfigCache,
    tunables: CpuTunables,
    poller: AdaptivePoller,
    next_wake_ms: i32,
}

impl CpuController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("CpuController: Initializing...");
        let raw_fd = kernel::register_psi_trigger(K_PSI_CPU_PATH, 120000, 1000000)
            .map_err(|e| QosError::FfiError(format!("CPU Trigger Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let latency_file = filesystem::open_file_for_write(K_SCHED_LATENCY_NS)?;
        let min_gran_file = filesystem::open_file_for_write(K_SCHED_MIN_GRANULARITY_NS)?;
        let wakeup_file = filesystem::open_file_for_write(K_SCHED_WAKEUP_GRANULARITY_NS)?;
        let migration_file = filesystem::open_file_for_write(K_SCHED_MIGRATION_COST_NS).ok();
        let perf_cpu_file = filesystem::open_file_for_write(K_PERF_CPU_TIME_MAX_PERCENT).ok();
        let psi_monitor = PsiMonitor::new(K_PSI_CPU_PATH)?;
        let tunables = CpuTunables {
            min_latency_ns: MIN_LATENCY_NS as f64,
            max_latency_ns: MAX_LATENCY_NS as f64,
            min_granularity_ns: MIN_GRANULARITY_NS as f64,
            max_granularity_ns: MAX_GRANULARITY_NS as f64,
            min_wakeup_ns: MIN_WAKEUP_NS as f64,
            max_wakeup_ns: MAX_WAKEUP_NS as f64,
            min_migration_cost: MIN_MIGRATION_COST as f64,
            max_migration_cost: MAX_MIGRATION_COST as f64,
            min_perf_cpu_percent: MIN_PERF_CPU_PERCENT as f64,
            max_perf_cpu_percent: MAX_PERF_CPU_PERCENT as f64,
            trend_factor: 0.5,
            critical_threshold: 50.0,
            alpha_smooth: 0.7,
            burst_threshold: 5.0,
            sigmoid_k: 0.15,
            sigmoid_mid: 25.0,
            decay_coeff: 0.05,
            latency_gran_ratio: 0.75,
        };
        let poller = AdaptivePoller::new(1.0, 2.5);
        let mut controller = Self {
            fd,
            latency_file,
            min_gran_file,
            wakeup_file,
            migration_file,
            perf_cpu_file,
            psi_monitor,
            current_latency: MIN_LATENCY_NS as f64, 
            current_min_gran: MIN_GRANULARITY_NS as f64,
            current_wakeup: MIN_WAKEUP_NS as f64,
            current_migration: MIN_MIGRATION_COST as f64,
            current_perf_percent: MAX_PERF_CPU_PERCENT as f64,
            prev_impulse_smooth: 0.0,
            cache: KernelConfigCache { 
                latency: 0,
                min_granularity: 0,
                wakeup_granularity: 0,
                migration_cost: 0,
                perf_cpu_max_percent: 0,
            },
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
        let raw_p = some.current.max(some.avg10);
        let k_trend = cpu_math::calculate_trend_gain(some.avg10, some.avg60, &self.tunables);
        let p_eff = raw_p * k_trend;
        update_cpu_pressure(p_eff);
        self.next_wake_ms = self.poller.calculate_next_interval(p_eff) as i32;
        let delta_raw = some.current - some.avg10;
        let delta_smooth = cpu_math::smooth_delta(delta_raw, self.prev_impulse_smooth, &self.tunables);
        self.prev_impulse_smooth = delta_smooth;
        let target_migration = cpu_math::calculate_migration_cost(delta_smooth, p_eff, &self.tunables);
        let thermal_floor = cpu_math::calculate_thermal_floor(some.avg60, &self.tunables);
        let (target_latency, target_min_gran) = cpu_math::calculate_latency_and_granularity(p_eff, thermal_floor, &self.tunables);
        let target_wakeup = cpu_math::calculate_wakeup_granularity(p_eff, &self.tunables);
        let target_perf = cpu_math::calculate_perf_limit(some.avg10, &self.tunables);
        self.current_latency = target_latency;
        self.current_min_gran = target_min_gran;
        self.current_wakeup = target_wakeup;
        self.current_migration = target_migration;
        self.current_perf_percent = target_perf;
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let lat_u64 = self.current_latency.round() as u64;
        let gran_u64 = self.current_min_gran.round() as u64;
        let wake_u64 = self.current_wakeup.round() as u64;
        let mig_u64 = self.current_migration.round() as u64;
        let perf_u64 = self.current_perf_percent.round() as u64;
        if force || self.cache.latency.abs_diff(lat_u64) > 100_000 {
            if write_to_stream(&mut self.latency_file, &lat_u64.to_string()).is_ok() {
                self.cache.latency = lat_u64;
            }
        }
        if force || self.cache.min_granularity.abs_diff(gran_u64) > 100_000 {
            if write_to_stream(&mut self.min_gran_file, &gran_u64.to_string()).is_ok() {
                self.cache.min_granularity = gran_u64;
            }
        }
        if force || self.cache.wakeup_granularity.abs_diff(wake_u64) > 100_000 {
            if write_to_stream(&mut self.wakeup_file, &wake_u64.to_string()).is_ok() {
                self.cache.wakeup_granularity = wake_u64;
            }
        }
        if let Some(ref mut f) = self.migration_file {
            if force || self.cache.migration_cost.abs_diff(mig_u64) > 50_000 {
                if write_to_stream(f, &mig_u64.to_string()).is_ok() {
                    self.cache.migration_cost = mig_u64;
                }
            }
        }
        if let Some(ref mut f) = self.perf_cpu_file {
            if force || self.cache.perf_cpu_max_percent != perf_u64 {
                if write_to_stream(f, &perf_u64.to_string()).is_ok() {
                    self.cache.perf_cpu_max_percent = perf_u64;
                }
            }
        }
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