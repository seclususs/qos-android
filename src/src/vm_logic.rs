//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::traits::EventHandler;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};

const K_DIRTY_RATIO: &str = "/proc/sys/vm/dirty_ratio";
const K_DIRTY_BG_RATIO: &str = "/proc/sys/vm/dirty_background_ratio";
const K_WRITEBACK_CENTISECS: &str = "/proc/sys/vm/dirty_writeback_centisecs";
const K_DIRTY_EXPIRE_CENTISECS: &str = "/proc/sys/vm/dirty_expire_centisecs";
const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";

const THRESHOLD_UP_TO_BALANCED: f64 = 10.0;
const THRESHOLD_DOWN_TO_IDLE: f64 = 5.0;
const THRESHOLD_UP_TO_PRESSURE: f64 = 40.0;
const THRESHOLD_DOWN_TO_BALANCED: f64 = 20.0;
const MONITORING_INTERVAL_MS: i32 = 6000;

#[derive(Debug, PartialEq, Copy, Clone)]
enum VmState {
    Idle,
    Balanced,
    Pressure,
}

struct VmConfigCache {
    dirty_ratio: String,
    dirty_bg_ratio: String,
    writeback: String,
    expire: String,
}

impl VmConfigCache {
    fn new() -> Self {
        Self {
            dirty_ratio: String::new(),
            dirty_bg_ratio: String::new(),
            writeback: String::new(),
            expire: String::new(),
        }
    }
}

pub struct VmManager {
    fd: OwnedFd,
    current_state: VmState,
    cache: VmConfigCache,
}

impl VmManager {
    pub fn new() -> Result<Self, String> {
        ffi::log_info("VmManager: Initializing...");
        let mut manager = Self {
            fd: unsafe {
                let raw = ffi::register_psi_trigger(K_PSI_MEMORY_PATH, 100000, 1000000);
                if raw < 0 { return Err("Failed to register VM PSI trigger".to_string()); }
                OwnedFd::from_raw_fd(raw)
            },
            current_state: VmState::Idle,
            cache: VmConfigCache::new(),
        };
        manager.apply_state(VmState::Idle, true);
        Ok(manager)
    }
    fn evaluate_next_state(&self, max_psi: f64) -> VmState {
        match self.current_state {
            VmState::Idle => {
                if max_psi > THRESHOLD_UP_TO_BALANCED {
                    if max_psi > THRESHOLD_UP_TO_PRESSURE {
                        VmState::Pressure
                    } else {
                        VmState::Balanced
                    }
                } else {
                    VmState::Idle
                }
            },
            VmState::Balanced => {
                if max_psi > THRESHOLD_UP_TO_PRESSURE {
                    VmState::Pressure
                } else if max_psi < THRESHOLD_DOWN_TO_IDLE {
                    VmState::Idle
                } else {
                    VmState::Balanced
                }
            },
            VmState::Pressure => {
                if max_psi < THRESHOLD_DOWN_TO_BALANCED {
                    if max_psi < THRESHOLD_DOWN_TO_IDLE {
                        VmState::Idle
                    } else {
                        VmState::Balanced
                    }
                } else {
                    VmState::Pressure
                }
            },
        }
    }
    fn apply_state(&mut self, new_state: VmState, force: bool) {
        let (ratio, bg_ratio, interval) = match new_state {
            VmState::Idle => ("40", "20", "4500"),
            VmState::Balanced => ("30", "15", "3000"),
            VmState::Pressure => ("20", "10", "2000"),
        };
        if force || self.cache.dirty_ratio != ratio {
            ffi::log_debug(&format!("VmManager: Set Dirty Ratio -> {}", ratio));
            if ffi::apply_tweak(K_DIRTY_RATIO, ratio) {
                self.cache.dirty_ratio = ratio.to_string();
            }
        }
        if force || self.cache.dirty_bg_ratio != bg_ratio {
            ffi::log_debug(&format!("VmManager: Set Dirty BG Ratio -> {}", bg_ratio));
            if ffi::apply_tweak(K_DIRTY_BG_RATIO, bg_ratio) {
                self.cache.dirty_bg_ratio = bg_ratio.to_string();
            }
        }
        if force || self.cache.writeback != interval {
            ffi::log_debug(&format!("VmManager: Set Writeback -> {}", interval));
            if ffi::apply_tweak(K_WRITEBACK_CENTISECS, interval) {
                self.cache.writeback = interval.to_string();
            }
        }
        if force || self.cache.expire != interval {
            ffi::log_debug(&format!("VmManager: Set Dirty Expire -> {}", interval));
            if ffi::apply_tweak(K_DIRTY_EXPIRE_CENTISECS, interval) {
                self.cache.expire = interval.to_string();
            }
        }
        if self.current_state != new_state {
            ffi::log_info(&format!("VM State Transition: {:?} -> {:?}", self.current_state, new_state));
            self.current_state = new_state;
        }
    }
    fn process_logic(&mut self) {
        let psi = ffi::get_memory_pressure();
        let next_state = self.evaluate_next_state(psi);
        if next_state != self.current_state {
            self.apply_state(next_state, false);
        }
    }
}

impl EventHandler for VmManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) {
        self.process_logic();
    }
    fn on_timeout(&mut self) {
        self.process_logic();
    }
    fn get_timeout_ms(&self) -> i32 {
        match self.current_state {
            VmState::Idle => -1,
            _ => MONITORING_INTERVAL_MS,
        }
    }
}