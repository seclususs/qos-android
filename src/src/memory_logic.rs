//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::system_utils;
use crate::traits::EventHandler;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};

const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";

const THRESHOLD_GREEN_TO_YELLOW: f64 = 8.0;
const THRESHOLD_YELLOW_TO_GREEN: f64 = 3.0;
const THRESHOLD_YELLOW_TO_RED: f64 = 35.0;
const THRESHOLD_RED_TO_YELLOW: f64 = 15.0;
const MONITORING_INTERVAL_MS: i32 = 60000;

#[derive(Debug, PartialEq, Copy, Clone)]
enum MemoryState {
    Idle,
    Balanced,
    Pressure,
}

struct KernelConfigCache {
    swappiness: String,
    vfs_cache_pressure: String,
}

impl KernelConfigCache {
    fn new() -> Self {
        Self {
            swappiness: String::new(),
            vfs_cache_pressure: String::new(),
        }
    }
}

pub struct MemoryManager {
    fd: OwnedFd, 
    current_state: MemoryState,
    cache: KernelConfigCache,
}

impl MemoryManager {
    pub fn new() -> Result<Self, String> {
        info!("MemoryManager: Initializing...");
        let mut manager = Self {
            fd: unsafe {
                let raw = ffi::register_psi_trigger(K_PSI_MEMORY_PATH, 80000, 1000000);
                if raw < 0 { return Err("Failed to register Memory PSI trigger".to_string()); }
                OwnedFd::from_raw_fd(raw)
            },
            current_state: MemoryState::Idle,
            cache: KernelConfigCache::new(),
        };
        manager.apply_state(MemoryState::Idle, true);
        Ok(manager)
    }
    fn evaluate_next_state(&self, psi: f64) -> MemoryState {
        match self.current_state {
            MemoryState::Idle => {
                if psi > THRESHOLD_GREEN_TO_YELLOW { MemoryState::Balanced } else { MemoryState::Idle }
            },
            MemoryState::Balanced => {
                if psi > THRESHOLD_YELLOW_TO_RED { MemoryState::Pressure } 
                else if psi < THRESHOLD_YELLOW_TO_GREEN { MemoryState::Idle } 
                else { MemoryState::Balanced }
            },
            MemoryState::Pressure => {
                if psi < THRESHOLD_RED_TO_YELLOW { MemoryState::Balanced } else { MemoryState::Pressure }
            },
        }
    }
    fn apply_state(&mut self, new_state: MemoryState, force: bool) {
        let (target_swap, target_cache) = match new_state {
            MemoryState::Idle => ("30", "50"),
            MemoryState::Balanced => ("40", "80"),
            MemoryState::Pressure => ("70", "120"),
        };
        if force || self.cache.swappiness != target_swap {
            if system_utils::write_to_file(K_SWAPPINESS_PATH, target_swap) {
                debug!("MemoryManager: Set Swappiness -> {}", target_swap);
                self.cache.swappiness = target_swap.to_string();
            }
        }
        if force || self.cache.vfs_cache_pressure != target_cache {
            if system_utils::write_to_file(K_VFS_CACHE_PRESSURE_PATH, target_cache) {
                debug!("MemoryManager: Set VFS Cache -> {}", target_cache);
                self.cache.vfs_cache_pressure = target_cache.to_string();
            }
        }
        if self.current_state != new_state {
            debug!("Memory State Transition: {:?} -> {:?}", self.current_state, new_state);
            self.current_state = new_state;
        }
    }
    fn process_logic(&mut self) {
        let psi_value = system_utils::parse_psi_avg10(K_PSI_MEMORY_PATH);
        let next_state = self.evaluate_next_state(psi_value);
        if next_state != self.current_state {
            self.apply_state(next_state, false);
        }
    }
}

impl EventHandler for MemoryManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) { self.process_logic(); }
    fn on_timeout(&mut self) { self.process_logic(); }
    fn get_timeout_ms(&self) -> i32 {
        match self.current_state {
            MemoryState::Idle => -1,
            _ => MONITORING_INTERVAL_MS,
        }
    }
}