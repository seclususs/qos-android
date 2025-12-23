//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::update_cpu_pressure;
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

const K_PSI_CPU_PATH: &str = "/proc/pressure/cpu";
const K_SCHED_LATENCY_NS: &str = "/proc/sys/kernel/sched_latency_ns";
const K_SCHED_MIN_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_min_granularity_ns";
const K_SCHED_WAKEUP_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_wakeup_granularity_ns";
const K_SCHED_MIGRATION_COST_NS: &str = "/proc/sys/kernel/sched_migration_cost_ns";
const K_PERF_CPU_TIME_MAX_PERCENT: &str = "/proc/sys/kernel/perf_cpu_time_max_percent";
const MIN_LATENCY_NS: u64 = 8_000_000;
const MAX_LATENCY_NS: u64 = 10_000_000;
const MIN_GRANULARITY_NS: u64 = 6_000_000;
const MAX_GRANULARITY_NS: u64 = 8_000_000;
const MIN_WAKEUP_NS: u64 = 2_000_000;
const MAX_WAKEUP_NS: u64 = 4_000_000;
const MIN_MIGRATION_COST: u64 = 200_000;
const MAX_MIGRATION_COST: u64 = 400_000;
const MIN_PERF_CPU_PERCENT: u64 = 5;
const MAX_PERF_CPU_PERCENT: u64 = 25;
const POLLING_INTERVAL_MS: u64 = 2000;

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
}

impl CpuController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("CpuController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_CPU_PATH, 100000, 1000000)
            .map_err(|e| QosError::FfiError(format!("CPU Trigger Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let latency_file = fs::open_file_for_write(K_SCHED_LATENCY_NS)?;
        let min_gran_file = fs::open_file_for_write(K_SCHED_MIN_GRANULARITY_NS)?;
        let wakeup_file = fs::open_file_for_write(K_SCHED_WAKEUP_GRANULARITY_NS)?;
        let migration_file = fs::open_file_for_write(K_SCHED_MIGRATION_COST_NS).ok();
        let perf_cpu_file = fs::open_file_for_write(K_PERF_CPU_TIME_MAX_PERCENT).ok();
        let psi_monitor = PsiMonitor::new(K_PSI_CPU_PATH)?;
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
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn calculate_trend_gain(&self, avg10: f64, avg60: f64) -> f64 {
        let delta = avg10 - avg60;
        1.0 + 0.5 * delta.tanh()
    }
    fn calculate_thermal_floor(&self, avg300: f64) -> f64 {
        let fatigue = (avg300 / 100.0).clamp(0.0, 1.0);
        MIN_LATENCY_NS as f64 + (MAX_LATENCY_NS - MIN_LATENCY_NS) as f64 * fatigue
    }
    fn calculate_perf_limit(&self, avg10: f64) -> f64 {
        let critical_threshold = 50.0;
        let saturation = (avg10 / critical_threshold).clamp(0.0, 1.0);
        let min_limit = MIN_PERF_CPU_PERCENT as f64;
        let max_limit = MAX_PERF_CPU_PERCENT as f64;
        let limit = min_limit + (max_limit - min_limit) * (1.0 - saturation);
        limit.clamp(min_limit, max_limit)
    }
    fn update_dynamics(&mut self) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let raw_p = some.current.max(some.avg10);
        let k_trend = self.calculate_trend_gain(some.avg10, some.avg60);
        let p_eff = raw_p * k_trend;
        update_cpu_pressure(p_eff);
        let delta_raw = some.current - some.avg10;
        let alpha_smooth = 0.7;
        let delta_smooth = alpha_smooth * delta_raw + (1.0 - alpha_smooth) * self.prev_impulse_smooth;
        self.prev_impulse_smooth = delta_smooth;
        let threshold_burst = 5.0;
        let target_migration = if delta_smooth > threshold_burst {
            MIN_MIGRATION_COST as f64
        } else {
            let x = (p_eff / 100.0).clamp(0.0, 1.0);
            let raw_mig = MIN_MIGRATION_COST as f64 + (MAX_MIGRATION_COST - MIN_MIGRATION_COST) as f64 * (x * x);
            raw_mig.clamp(MIN_MIGRATION_COST as f64, MAX_MIGRATION_COST as f64)
        };
        let thermal_floor = self.calculate_thermal_floor(some.avg300);
        let k_sig = 0.15;
        let p_mid = 25.0;
        let denom = 1.0 + (k_sig * (p_eff - p_mid)).exp();
        let raw_latency = thermal_floor + ((MAX_LATENCY_NS as f64 - thermal_floor) / denom);
        let target_latency = raw_latency.clamp(thermal_floor, MAX_LATENCY_NS as f64);
        let raw_gran = target_latency * 0.75;
        let target_min_gran = raw_gran.clamp(MIN_GRANULARITY_NS as f64, MAX_GRANULARITY_NS as f64);
        let decay = (-0.05 * p_eff).exp();
        let raw_wake = MIN_WAKEUP_NS as f64 + (MAX_WAKEUP_NS - MIN_WAKEUP_NS) as f64 * decay;
        let target_wakeup = raw_wake.clamp(MIN_WAKEUP_NS as f64, MAX_WAKEUP_NS as f64);
        let target_perf = self.calculate_perf_limit(some.avg10);
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
        POLLING_INTERVAL_MS as i32
    }
}