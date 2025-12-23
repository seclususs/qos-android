//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::{update_io_pressure, update_io_saturation};
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
const POLLING_INTERVAL_MS: u64 = 2000;
const EPSILON: f64 = 0.01;

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
        let raw_fd = ffi::register_psi_trigger(K_PSI_IO_PATH, 100000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Storage PSI Error: {}", e)))?;
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
    fn calculate_read_ahead(&self, p_curr: f64) -> f64 {
        if p_curr < 10.0 {
            MAX_READ_AHEAD as f64
        } else {
            let normalized_p = (p_curr - 10.0).max(EPSILON);
            let scaling = 20.0 / normalized_p; 
            let result = MIN_READ_AHEAD as f64 + (scaling * (MAX_READ_AHEAD - MIN_READ_AHEAD) as f64);
            result.clamp(MIN_READ_AHEAD as f64, MAX_READ_AHEAD as f64)
        }
    }
    fn update_fluid_dynamics(&mut self) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let full = data.full;
        update_io_pressure(some.avg10);
        let i_sat_raw = full.avg10 / (some.avg10 + EPSILON);
        let i_sat = i_sat_raw.clamp(0.0, 1.0);
        update_io_saturation(i_sat);
        let beta = 3.0;
        let sat_factor = i_sat.powf(beta);
        let target_nr = (MAX_NR_REQUESTS as f64 * (1.0 - sat_factor)) + 
                        (MIN_NR_REQUESTS as f64 * sat_factor);
        let target_fifo = (MAX_FIFO_BATCH as f64 * (1.0 - sat_factor)) + 
                          (MIN_FIFO_BATCH as f64 * sat_factor);
        let tactical_p = some.current.max(full.current * 2.0);
        let target_ra = self.calculate_read_ahead(tactical_p);
        self.current_read_ahead = target_ra;
        self.current_nr_requests = target_nr.clamp(MIN_NR_REQUESTS as f64, MAX_NR_REQUESTS as f64);
        self.current_fifo_batch = target_fifo.clamp(MIN_FIFO_BATCH as f64, MAX_FIFO_BATCH as f64);
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let ra_u64 = self.current_read_ahead.round() as u64;
        let nr_u64 = self.current_nr_requests.round() as u64;
        let fifo_u64 = self.current_fifo_batch.round() as u64;
        if force || self.cached_read_ahead != ra_u64 {
            if write_to_stream(&mut self.read_ahead_file, &ra_u64.to_string()).is_ok() {
                self.cached_read_ahead = ra_u64;
            }
        }
        if force || self.cached_nr_requests != nr_u64 {
            if write_to_stream(&mut self.nr_requests_file, &nr_u64.to_string()).is_ok() {
                self.cached_nr_requests = nr_u64;
            }
        }
        if let Some(ref mut f) = self.fifo_batch_file {
            if force || self.cached_fifo_batch != fifo_u64 {
                if write_to_stream(f, &fifo_u64.to_string()).is_ok() {
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
        if let Err(e) = self.update_fluid_dynamics() {
            log::warn!("Storage Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_fluid_dynamics() {
            log::warn!("Storage Timeout Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        POLLING_INTERVAL_MS as i32
    }
}