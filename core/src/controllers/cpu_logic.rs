//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::{update_cpu_pressure, get_io_pressure};
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

const K_PSI_CPU_PATH: &str = "/proc/pressure/cpu";
const K_SCHED_LATENCY_NS: &str = "/proc/sys/kernel/sched_latency_ns";
const K_SCHED_MIN_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_min_granularity_ns";
const K_SCHED_WAKEUP_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_wakeup_granularity_ns";
const K_SCHED_MIGRATION_COST_NS: &str = "/proc/sys/kernel/sched_migration_cost_ns";
const K_PERF_CPU_TIME_MAX_PERCENT: &str = "/proc/sys/kernel/perf_cpu_time_max_percent";
const MIN_LATENCY_NS: u64 = 7_000_000;
const MAX_LATENCY_NS: u64 = 9_000_000;
const MIN_GRANULARITY_NS: u64 = 5_000_000;
const MAX_GRANULARITY_NS: u64 = 7_000_000;
const MIN_WAKEUP_NS: u64 = 2_000_000;
const MAX_WAKEUP_NS: u64 = 3_000_000;
const MIN_MIGRATION_COST: u64 = 400_000;
const MAX_MIGRATION_COST: u64 = 600_000;
const MIN_PERF_CPU_PERCENT: u64 = 1;
const MAX_PERF_CPU_PERCENT: u64 = 15;
const PSI_MAX_SCALE: f64 = 40.0;
const DECAY_FACTOR: f64 = 0.15;
const POLLING_INTERVAL_MS: u64 = 2000;
const PANIC_HOLD_DURATION: Duration = Duration::from_secs(5);

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
    last_psi: f64,
    panic_deadline: Option<Instant>,
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
            current_latency: MAX_LATENCY_NS as f64, 
            current_min_gran: MAX_GRANULARITY_NS as f64,
            current_wakeup: MAX_WAKEUP_NS as f64,
            current_migration: MIN_MIGRATION_COST as f64,
            current_perf_percent: MAX_PERF_CPU_PERCENT as f64,
            last_psi: 0.0,
            panic_deadline: None,
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
    fn enter_panic_mode(&mut self) {
        let now = Instant::now();
        self.panic_deadline = Some(now + PANIC_HOLD_DURATION);
        self.current_latency = MIN_LATENCY_NS as f64;
        self.current_min_gran = MIN_GRANULARITY_NS as f64;
        self.current_wakeup = MIN_WAKEUP_NS as f64;
        self.current_migration = MAX_MIGRATION_COST as f64;
        self.current_perf_percent = MIN_PERF_CPU_PERCENT as f64;
        log::info!("!!! CPU PANIC MODE ACTIVATED !!!");
        self.apply_values(true);
    }
    fn cubic_lerp(&self, psi: f64, max_scale: f64, start_val: u64, end_val: u64) -> f64 {
        let t = (psi / max_scale).clamp(0.0, 1.0);
        let t_cubic = t * t * t;
        let start = start_val as f64;
        let end = end_val as f64;
        start + (end - start) * t_cubic
    }
    fn update_dynamics_standard(&mut self, psi: f64) {
        update_cpu_pressure(psi);
        if let Some(deadline) = self.panic_deadline {
            if Instant::now() < deadline {
                return;
            } else {
                self.panic_deadline = None;
                log::info!("CPU Panic Mode Ended.");
            }
        }
        let mut target_latency = self.cubic_lerp(psi, PSI_MAX_SCALE, MAX_LATENCY_NS, MIN_LATENCY_NS);
        let mut target_min_gran = self.cubic_lerp(psi, PSI_MAX_SCALE, MAX_GRANULARITY_NS, MIN_GRANULARITY_NS);
        let target_wakeup = self.cubic_lerp(psi, PSI_MAX_SCALE, MAX_WAKEUP_NS, MIN_WAKEUP_NS);
        let mut target_migration = self.cubic_lerp(psi, PSI_MAX_SCALE, MIN_MIGRATION_COST, MAX_MIGRATION_COST);
        let target_perf = self.cubic_lerp(psi, PSI_MAX_SCALE, MAX_PERF_CPU_PERCENT, MIN_PERF_CPU_PERCENT);
        let io_psi = get_io_pressure();
        const CPU_LOAD_THRESHOLD: f64 = 15.0;
        const IO_CONGESTION: f64 = 20.0;
        if psi > CPU_LOAD_THRESHOLD {
            if io_psi < IO_CONGESTION {
                target_min_gran = MIN_GRANULARITY_NS as f64;
                target_migration = MAX_MIGRATION_COST as f64;
            } else {
                target_latency = 8_000_000.0; 
            }
        }
        let apply_smooth = |current: &mut f64, target: f64| {
            *current += (target - *current) * DECAY_FACTOR;
        };
        apply_smooth(&mut self.current_latency, target_latency);
        apply_smooth(&mut self.current_min_gran, target_min_gran);
        apply_smooth(&mut self.current_wakeup, target_wakeup);
        apply_smooth(&mut self.current_migration, target_migration);
        apply_smooth(&mut self.current_perf_percent, target_perf);
        self.last_psi = psi;
        self.apply_values(false);
    }
    fn apply_values(&mut self, force: bool) {
        let lat_u64 = self.current_latency as u64;
        let gran_u64 = self.current_min_gran as u64;
        let wake_u64 = self.current_wakeup as u64;
        let mig_u64 = self.current_migration as u64;
        let perf_u64 = self.current_perf_percent as u64;
        if force || self.cache.latency.abs_diff(lat_u64) > 100_000 {
            let s_val = lat_u64.to_string();
            if write_to_stream(&mut self.latency_file, &s_val).is_ok() {
                self.cache.latency = lat_u64;
            }
        }
        if force || self.cache.min_granularity.abs_diff(gran_u64) > 100_000 {
            let s_val = gran_u64.to_string();
            if write_to_stream(&mut self.min_gran_file, &s_val).is_ok() {
                self.cache.min_granularity = gran_u64;
            }
        }
        if force || self.cache.wakeup_granularity.abs_diff(wake_u64) > 100_000 {
            let s_val = wake_u64.to_string();
            if write_to_stream(&mut self.wakeup_file, &s_val).is_ok() {
                self.cache.wakeup_granularity = wake_u64;
            }
        }
        if let Some(ref mut f) = self.migration_file {
            if force || self.cache.migration_cost.abs_diff(mig_u64) > 50_000 {
                let s_val = mig_u64.to_string();
                if write_to_stream(f, &s_val).is_ok() {
                    self.cache.migration_cost = mig_u64;
                }
            }
        }
        if let Some(ref mut f) = self.perf_cpu_file {
            if force || self.cache.perf_cpu_max_percent != perf_u64 {
                let s_val = perf_u64.to_string();
                if write_to_stream(f, &s_val).is_ok() {
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
        self.enter_panic_mode();
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            update_cpu_pressure(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics_standard(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        POLLING_INTERVAL_MS as i32
    }
}