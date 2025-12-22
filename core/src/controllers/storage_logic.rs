//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::{update_io_pressure, get_cpu_pressure};
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
const K_NR_REQUESTS_PATH: &str = "/sys/block/mmcblk0/queue/nr_requests";
const K_FIFO_BATCH_PATH: &str = "/sys/block/mmcblk0/queue/iosched/fifo_batch";
const MAX_READ_AHEAD: u64 = 256;
const MIN_READ_AHEAD: u64 = 128;
const MAX_NR_REQUESTS: u64 = 256;
const MIN_NR_REQUESTS: u64 = 128;
const MIN_FIFO_BATCH: u64 = 8;
const MAX_FIFO_BATCH: u64 = 16;
const IO_PSI_CEILING: f64 = 60.0;
const DECAY_FACTOR: f64 = 0.1;
const POLLING_INTERVAL_MS: u64 = 2000;

pub struct StorageController {
    fd: File,
    read_ahead_file: File,
    nr_requests_file: File,
    fifo_batch_file: Option<File>,
    psi_monitor: PsiMonitor,
    current_read_ahead: f64,
    current_nr_requests: f64,
    current_fifo_batch: f64,
    cached_read_ahead: u64,
    cached_nr_requests: u64,
    cached_fifo_batch: u64,
}

impl StorageController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("StorageController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_IO_PATH, 120000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Failed to register Storage PSI: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let read_ahead_file = fs::open_file_for_write(K_READ_AHEAD_PATH)?;
        let nr_requests_file = fs::open_file_for_write(K_NR_REQUESTS_PATH)?;
        let fifo_batch_file = fs::open_file_for_write(K_FIFO_BATCH_PATH).ok();
        let psi_monitor = PsiMonitor::new(K_PSI_IO_PATH)?;
        let mut controller = Self { 
            fd,
            read_ahead_file,
            nr_requests_file,
            fifo_batch_file,
            psi_monitor,
            current_read_ahead: MAX_READ_AHEAD as f64,
            current_nr_requests: MAX_NR_REQUESTS as f64,
            current_fifo_batch: MAX_FIFO_BATCH as f64,
            cached_read_ahead: 0,
            cached_nr_requests: 0,
            cached_fifo_batch: 0,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn cubic_lerp(&self, psi: f64, max_scale: f64, start_val: u64, end_val: u64) -> f64 {
        let t = (psi / max_scale).clamp(0.0, 1.0);
        let t_cubic = t * t * t;
        let start = start_val as f64;
        let end = end_val as f64;
        start + (end - start) * t_cubic
    }
    fn update_dynamics(&mut self, psi: f64) {
        update_io_pressure(psi);
        let cpu_psi = get_cpu_pressure();
        let mut target_ra = self.cubic_lerp(psi, IO_PSI_CEILING, MAX_READ_AHEAD, MIN_READ_AHEAD);
        let mut target_nr = self.cubic_lerp(psi, IO_PSI_CEILING, MAX_NR_REQUESTS, MIN_NR_REQUESTS);
        let target_fifo = self.cubic_lerp(psi, IO_PSI_CEILING, MAX_FIFO_BATCH, MIN_FIFO_BATCH);
        const IO_CONGESTION: f64 = 10.0;
        if psi > IO_CONGESTION {
            if cpu_psi > 20.0 {
                target_ra = MIN_READ_AHEAD as f64;
                target_nr = MIN_NR_REQUESTS as f64;
            } else if cpu_psi < 10.0 {
                target_ra = MAX_READ_AHEAD as f64;
                target_nr = MAX_NR_REQUESTS as f64;
            }
        }
        let apply_smooth = |current: &mut f64, target: f64| {
            *current += (target - *current) * DECAY_FACTOR;
        };
        apply_smooth(&mut self.current_read_ahead, target_ra);
        apply_smooth(&mut self.current_nr_requests, target_nr);
        apply_smooth(&mut self.current_fifo_batch, target_fifo);
        self.apply_values(false);
    }
    fn apply_values(&mut self, force: bool) {
        let ra_u64 = self.current_read_ahead.round() as u64;
        if force || self.cached_read_ahead != ra_u64 {
            let s = ra_u64.to_string();
            if let Ok(_) = write_to_stream(&mut self.read_ahead_file, &s) {
                self.cached_read_ahead = ra_u64;
            }
        }
        let nr_u64 = self.current_nr_requests.round() as u64;
        if force || self.cached_nr_requests != nr_u64 {
            let s = nr_u64.to_string();
            if let Ok(_) = write_to_stream(&mut self.nr_requests_file, &s) {
                self.cached_nr_requests = nr_u64;
            }
        }
        if let Some(ref mut f) = self.fifo_batch_file {
            let fifo_u64 = self.current_fifo_batch.round() as u64;
            if force || self.cached_fifo_batch != fifo_u64 {
                let s = fifo_u64.to_string();
                if write_to_stream(f, &s).is_ok() {
                    self.cached_fifo_batch = fifo_u64;
                }
            }
        }
    }
}

impl EventHandler for StorageController {
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