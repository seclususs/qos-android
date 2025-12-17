//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::system_utils;
use crate::traits::EventHandler;
use crate::error::QosError;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};

const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";

const IO_UP_TO_BUSY: f64 = 12.0;
const IO_DOWN_TO_IDLE: f64 = 4.0;
const IO_UP_TO_CONGESTED: f64 = 60.0;
const IO_DOWN_TO_BUSY: f64 = 25.0;
const POLLING_INTERVAL_MS: i32 = 60000;

#[derive(Debug, PartialEq, Copy, Clone)]
enum IoState { Idle, Busy, Congested }

pub struct StorageManager {
    fd: OwnedFd,
    current_state: IoState,
    cached_read_ahead: String,
}

impl StorageManager {
    pub fn new() -> Result<Self, QosError> {
        log::info!("StorageManager: Starting I/O optimization service...");
        let mut manager = Self { 
            fd: unsafe { 
                let raw = ffi::register_psi_trigger(K_PSI_IO_PATH, 120000, 1000000);
                if raw < 0 { return Err(QosError::FfiError("Failed to register Storage PSI".to_string())); }
                OwnedFd::from_raw_fd(raw) 
            },
            current_state: IoState::Idle,
            cached_read_ahead: String::new(),
        };
        manager.apply_state(IoState::Idle, true);
        Ok(manager)
    }
    fn evaluate_state(&self, psi: f64) -> IoState {
        match self.current_state {
            IoState::Idle => {
                if psi > IO_UP_TO_BUSY { IoState::Busy } else { IoState::Idle }
            },
            IoState::Busy => {
                if psi > IO_UP_TO_CONGESTED { IoState::Congested } 
                else if psi < IO_DOWN_TO_IDLE { IoState::Idle } 
                else { IoState::Busy }
            },
            IoState::Congested => {
                if psi < IO_DOWN_TO_BUSY { IoState::Busy } else { IoState::Congested }
            },
        }
    }
    fn apply_state(&mut self, new_state: IoState, force: bool) {
        let target_val = match new_state {
            IoState::Idle => "256",
            IoState::Busy => "192",
            IoState::Congested => "128",
        };
        if force || self.cached_read_ahead != target_val {
            log::debug!("StorageManager: Set ReadAhead -> {}kb", target_val);
            match system_utils::write_to_file(K_READ_AHEAD_PATH, target_val) {
                Ok(_) => {
                    self.cached_read_ahead = target_val.to_string();
                },
                Err(e) => {
                    log::error!("StorageManager: Failed to set ReadAhead: {}", e);
                }
            }
        }
        self.current_state = new_state;
    }
    fn update(&mut self) {
        match system_utils::parse_psi_avg10(K_PSI_IO_PATH) {
            Ok(psi) => {
                let next = self.evaluate_state(psi);
                if next != self.current_state {
                    self.apply_state(next, false);
                }
            },
            Err(e) => log::warn!("StorageManager: PSI read failed: {}", e),
        }
    }
}

impl EventHandler for StorageManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) { self.update(); }
    fn on_timeout(&mut self) { self.update(); }
    fn get_timeout_ms(&self) -> i32 {
        if self.current_state != IoState::Idle { POLLING_INTERVAL_MS } else { -1 }
    }
}