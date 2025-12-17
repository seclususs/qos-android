//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;
use std::time::{Instant, Duration};

const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
const IO_UP_TO_BUSY: f64 = 12.0;
const IO_DOWN_TO_IDLE: f64 = 4.0;
const IO_UP_TO_CONGESTED: f64 = 60.0;
const IO_DOWN_TO_BUSY: f64 = 25.0;
const QUICK_RECHECK_MS: u64 = 5000; 
const HYSTERESIS_DURATION: Duration = Duration::from_secs(5);

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum IoState { Idle, Busy, Congested }

pub struct StorageController {
    fd: File,
    read_ahead_file: File,
    current_state: IoState,
    cached_read_ahead: String,
    psi_monitor: PsiMonitor,
    last_state_change: Instant,
    next_check: Instant,
    trigger_active: bool,
}

impl StorageController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("StorageController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_IO_PATH, 120000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Failed to register Storage PSI: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let read_ahead_file = fs::open_file_for_write(K_READ_AHEAD_PATH)
            .map_err(|e| QosError::SystemCheckFailed(format!("Failed to open ReadAhead: {}", e)))?;
        let psi_monitor = PsiMonitor::new(K_PSI_IO_PATH)?;
        let mut manager = Self { 
            fd,
            read_ahead_file,
            current_state: IoState::Idle,
            cached_read_ahead: String::new(),
            psi_monitor,
            last_state_change: Instant::now() - HYSTERESIS_DURATION,
            next_check: Instant::now() + Duration::from_millis(QUICK_RECHECK_MS),
            trigger_active: true,
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
        let is_pressure_increase = new_state > self.current_state;
        if !force && !is_pressure_increase && self.last_state_change.elapsed() < HYSTERESIS_DURATION {
            return;
        }
        let target_val = match new_state {
            IoState::Idle => "256",
            IoState::Busy => "192",
            IoState::Congested => "128",
        };
        if force || self.cached_read_ahead != target_val {
            match write_to_stream(&mut self.read_ahead_file, target_val) {
                Ok(_) => self.cached_read_ahead = target_val.to_string(),
                Err(e) => log::error!("StorageController: Failed to write ReadAhead: {}", e),
            }
        }
        if self.current_state != new_state {
            log::info!("Storage State: {:?} -> {:?}", self.current_state, new_state);
            self.current_state = new_state;
            self.last_state_change = Instant::now();
        }
    }
    fn perform_polling_check(&mut self) {
        if self.last_state_change.elapsed() < HYSTERESIS_DURATION {
            return;
        }
        match self.psi_monitor.read_avg10() {
            Ok(psi) => {
                let next = self.evaluate_state(psi);
                if next != self.current_state {
                    log::info!("Storage Poll: PSI={:.2} trigger change {:?}->{:?}", 
                        psi, self.current_state, next);
                    self.apply_state(next, false);
                }
            },
            Err(e) => log::warn!("Storage Poll Error: {}", e),
        }
    }
}

impl EventHandler for StorageController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        let mut new_state = self.current_state;
        if self.current_state == IoState::Idle {
            new_state = IoState::Busy;
        } else if self.current_state == IoState::Busy {
            new_state = IoState::Congested;
        }
        if new_state != self.current_state {
            log::info!("Storage Trigger: Escalating to {:?}", new_state);
            self.apply_state(new_state, false);
            self.next_check = Instant::now() + Duration::from_millis(QUICK_RECHECK_MS);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        self.perform_polling_check();
        self.next_check = Instant::now() + Duration::from_millis(QUICK_RECHECK_MS);
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        if self.trigger_active && self.current_state == IoState::Idle {
            return -1;
        }
        let now = Instant::now();
        if now >= self.next_check {
            0
        } else {
            (self.next_check - now).as_millis() as i32
        }
    }
}