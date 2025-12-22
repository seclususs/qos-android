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
const MAX_LATENCY_NS: u64 = 9_000_000;
const MIN_GRANULARITY_NS: u64 = 6_000_000;
const MAX_GRANULARITY_NS: u64 = 7_000_000;
const MIN_WAKEUP_NS: u64 = 2_000_000;
const MAX_WAKEUP_NS: u64 = 3_000_000;
const MIN_MIGRATION_COST: u64 = 500_000;
const MAX_MIGRATION_COST: u64 = 600_000;
const MIN_PERF_CPU_PERCENT: u64 = 1;
const MAX_PERF_CPU_PERCENT: u64 = 15;
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
    last_psi_raw: f64,
    smoothed_psi: f64,
    current_latency: f64,
    current_min_gran: f64,
    current_wakeup: f64,
    current_migration: f64,
    current_perf_percent: f64,
    cache: KernelConfigCache,
}

impl CpuController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("CpuController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_CPU_PATH, 300000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Failed to register CPU PSI trigger: {}", e)))?;
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
            last_psi_raw: 0.0,
            smoothed_psi: 0.0,
            current_latency: MIN_LATENCY_NS as f64, 
            current_min_gran: MIN_GRANULARITY_NS as f64,
            current_wakeup: MAX_WAKEUP_NS as f64,
            current_migration: MIN_MIGRATION_COST as f64,
            current_perf_percent: MAX_PERF_CPU_PERCENT as f64,
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
    fn get_adaptive_smoothed_psi(&mut self, raw_psi: f64) -> f64 {
        let delta = raw_psi - self.last_psi_raw;
        let alpha = if delta > 1.0 { 0.8 } else { 0.2 };
        self.smoothed_psi = alpha * raw_psi + (1.0 - alpha) * self.smoothed_psi;
        self.last_psi_raw = raw_psi;
        self.smoothed_psi
    }
    fn inverse_sigmoid(&self, psi: f64, min: f64, max: f64, midpoint: f64, steepness: f64) -> f64 {
        let denominator = 1.0 + (steepness * (psi - midpoint)).exp();
        min + ((max - min) / denominator)
    }
    fn parabolic_migration(&self, psi: f64) -> f64 {
        let min_val = MIN_MIGRATION_COST as f64;
        let max_val = MAX_MIGRATION_COST as f64;
        if psi < 25.0 {
            let start = (min_val + max_val) / 2.0; 
            let t = (psi / 25.0).clamp(0.0, 1.0);
            start - (start - min_val) * t
        } else {
            let t = ((psi - 25.0) / 35.0).clamp(0.0, 1.0);
            min_val + (max_val - min_val) * t
        }
    }
    fn update_dynamics_hybrid(&mut self, raw_psi: f64) {
        let psi = self.get_adaptive_smoothed_psi(raw_psi);
        update_cpu_pressure(psi);
        let target_latency = self.inverse_sigmoid(psi, MIN_LATENCY_NS as f64, MAX_LATENCY_NS as f64, 25.0, 0.15);
        let t_factor = (target_latency - MIN_LATENCY_NS as f64) / (MAX_LATENCY_NS as f64 - MIN_LATENCY_NS as f64);
        let target_min_gran = MIN_GRANULARITY_NS as f64 + t_factor * (MAX_GRANULARITY_NS as f64 - MIN_GRANULARITY_NS as f64);
        let target_wakeup = self.inverse_sigmoid(psi, MIN_WAKEUP_NS as f64, MAX_WAKEUP_NS as f64, 20.0, 0.25);
        let target_migration = self.parabolic_migration(psi);
        let target_perf = self.inverse_sigmoid(psi, MIN_PERF_CPU_PERCENT as f64, MAX_PERF_CPU_PERCENT as f64, 30.0, 0.3);
        self.current_latency = target_latency;
        self.current_min_gran = target_min_gran;
        self.current_wakeup = target_wakeup;
        self.current_migration = target_migration;
        self.current_perf_percent = target_perf;
        self.apply_values(false);
    }
    fn apply_values(&mut self, force: bool) {
        let lat_u64 = self.current_latency as u64;
        let gran_u64 = self.current_min_gran as u64;
        let wake_u64 = self.current_wakeup as u64;
        let mig_u64 = self.current_migration as u64;
        let perf_u64 = self.current_perf_percent as u64;
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
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            update_cpu_pressure(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics_hybrid(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        POLLING_INTERVAL_MS as i32
    }
}