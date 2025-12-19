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
const MIN_LATENCY_NS: u64 = 7_000_000;
const MAX_LATENCY_NS: u64 = 9_000_000;
const MIN_GRANULARITY_NS: u64 = 5_000_000;
const MAX_GRANULARITY_NS: u64 = 7_000_000;
const MIN_WAKEUP_NS: u64 = 2_000_000;
const MAX_WAKEUP_NS: u64 = 3_000_000;
const PSI_MAX_SCALE: f64 = 40.0;
const BURST_THRESHOLD: f64 = 10.0;
const DECAY_FACTOR: f64 = 0.15;
const POLLING_INTERVAL_MS: u64 = 3000;

struct KernelConfigCache {
    latency: u64,
    min_granularity: u64,
    wakeup_granularity: u64,
}

pub struct CpuController {
    fd: File,
    latency_file: File,
    min_gran_file: File,
    wakeup_file: File,
    psi_monitor: PsiMonitor,
    current_latency: f64,
    current_min_gran: f64,
    current_wakeup: f64,
    last_psi: f64,
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
        let psi_monitor = PsiMonitor::new(K_PSI_CPU_PATH)?;
        let mut controller = Self {
            fd,
            latency_file,
            min_gran_file,
            wakeup_file,
            psi_monitor,
            current_latency: 8_000_000.0,
            current_min_gran: 6_000_000.0,
            current_wakeup: 3_000_000.0,
            last_psi: 0.0,
            cache: KernelConfigCache { latency: 0, min_granularity: 0, wakeup_granularity: 0 },
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn map_psi_to_value(&self, psi: f64, val_at_zero: u64, val_at_max: u64) -> f64 {
        let clamped_psi = psi.min(PSI_MAX_SCALE).max(0.0);
        let ratio = clamped_psi / PSI_MAX_SCALE;
        let diff = val_at_zero as f64 - val_at_max as f64;
        (val_at_zero as f64) - (diff * ratio)
    }
    fn update_dynamics(&mut self, psi: f64) {
        update_cpu_pressure(psi);
        let delta_psi = psi - self.last_psi;
        let is_burst = delta_psi > BURST_THRESHOLD;
        if is_burst {
            log::info!("CPU BURST DETECTED! Delta PSI: {:.2}.", delta_psi);
        }
        let effective_psi = if is_burst { PSI_MAX_SCALE } else { psi };
        let target_latency = self.map_psi_to_value(effective_psi, MAX_LATENCY_NS, MIN_LATENCY_NS);
        let target_min_gran = self.map_psi_to_value(effective_psi, MAX_GRANULARITY_NS, MIN_GRANULARITY_NS);
        let target_wakeup = self.map_psi_to_value(effective_psi, MAX_WAKEUP_NS, MIN_WAKEUP_NS);
        let apply_decay = |current: &mut f64, target: f64| {
            if target < *current {
                *current = target;
            } else {
                let diff = target - *current;
                *current += diff * DECAY_FACTOR;
            }
        };
        apply_decay(&mut self.current_latency, target_latency);
        apply_decay(&mut self.current_min_gran, target_min_gran);
        apply_decay(&mut self.current_wakeup, target_wakeup);
        self.last_psi = psi;
        self.apply_values(false);
    }
    fn apply_values(&mut self, force: bool) {
        let lat_u64 = self.current_latency as u64;
        let gran_u64 = self.current_min_gran as u64;
        let wake_u64 = self.current_wakeup as u64;
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
    }
}

impl EventHandler for CpuController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        POLLING_INTERVAL_MS as i32
    }
}