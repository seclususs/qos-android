//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::traits::EventHandler;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};

const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";

const IO_UP_TO_BUSY: f64 = 12.0;
const IO_DOWN_TO_IDLE: f64 = 4.0;
const IO_UP_TO_CONGESTED: f64 = 60.0;
const IO_DOWN_TO_BUSY: f64 = 25.0;
const POLLING_INTERVAL_MS: i32 = 6000;

#[derive(Debug, PartialEq, Copy, Clone)]
enum IoState {
    Idle,
    Busy,
    Congested,
}

pub struct StorageManager {
    fd: OwnedFd,
    current_state: IoState,
    cached_read_ahead: String,
}

impl StorageManager {
    pub fn new() -> Result<Self, String> {
        ffi::log_info("StorageManager: Starting I/O optimization service...");
        let mut manager = Self { 
            fd: unsafe { 
                let raw = ffi::register_psi_trigger(K_PSI_IO_PATH, 120000, 1000000);
                if raw < 0 { return Err("Failed to register Storage PSI".to_string()); }
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
                if psi > IO_UP_TO_CONGESTED {
                    IoState::Congested
                } else if psi < IO_DOWN_TO_IDLE {
                    IoState::Idle
                } else {
                    IoState::Busy
                }
            },
            IoState::Congested => {
                if psi < IO_DOWN_TO_BUSY { IoState::Busy } else { IoState::Congested }
            },
        }
    }
    fn apply_state(&mut self, new_state: IoState, force: bool) {
        let target_val = match new_state {
            IoState::Idle => "384",
            IoState::Busy => "256",
            IoState::Congested => "192",
        };
        if force || self.cached_read_ahead != target_val {
            ffi::log_debug(&format!("StorageManager: Changing ReadAhead -> {}kb (PSI State: {:?})", target_val, new_state));
            if ffi::apply_tweak(K_READ_AHEAD_PATH, target_val) {
                self.cached_read_ahead = target_val.to_string();
            }
        }
        self.current_state = new_state;
    }
    fn update(&mut self) {
        let psi = ffi::get_io_pressure();
        let next = self.evaluate_state(psi);
        if next != self.current_state {
            self.apply_state(next, false);
        }
    }
}

impl EventHandler for StorageManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) {
        self.update();
    }
    fn on_timeout(&mut self) {
        self.update();
    }
    fn get_timeout_ms(&self) -> i32 {
        if self.current_state != IoState::Idle {
            POLLING_INTERVAL_MS
        } else {
            -1
        }
    }
}