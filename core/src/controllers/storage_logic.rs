//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::update_io_pressure;
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
const MAX_READ_AHEAD: u64 = 256;
const MIN_READ_AHEAD: u64 = 128;
const IO_PSI_CEILING: f64 = 60.0;
const DECAY_FACTOR: f64 = 0.1;
const POLLING_INTERVAL_MS: u64 = 3000;

pub struct StorageController {
    fd: File,
    read_ahead_file: File,
    psi_monitor: PsiMonitor,
    current_read_ahead: f64,
    cached_read_ahead: u64,
}

impl StorageController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("StorageController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_IO_PATH, 120000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Failed to register Storage PSI: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let read_ahead_file = fs::open_file_for_write(K_READ_AHEAD_PATH)?;
        let psi_monitor = PsiMonitor::new(K_PSI_IO_PATH)?;
        let mut controller = Self { 
            fd,
            read_ahead_file,
            psi_monitor,
            current_read_ahead: MAX_READ_AHEAD as f64,
            cached_read_ahead: 0,
        };
        controller.apply_value(true);
        Ok(controller)
    }
    fn update_dynamics(&mut self, psi: f64) {
        update_io_pressure(psi);
        let ratio = (psi / IO_PSI_CEILING).clamp(0.0, 1.0);
        let diff = MAX_READ_AHEAD as f64 - MIN_READ_AHEAD as f64;
        let target_ra = (MAX_READ_AHEAD as f64) - (diff * ratio);
        if target_ra < self.current_read_ahead {
            self.current_read_ahead = target_ra;
        } else {
            self.current_read_ahead += (target_ra - self.current_read_ahead) * DECAY_FACTOR;
        }
        self.apply_value(false);
    }
    fn apply_value(&mut self, force: bool) {
        let val_u64 = self.current_read_ahead.round() as u64;
        if force || self.cached_read_ahead != val_u64 {
            let s = val_u64.to_string();
            if let Ok(_) = write_to_stream(&mut self.read_ahead_file, &s) {
                self.cached_read_ahead = val_u64;
            } else {
                log::error!("Failed to write ReadAhead value: {}", s);
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