//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::traits::EventHandler;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};

const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";

#[derive(Debug, PartialEq, Copy, Clone)]
enum PressureZone { Green, Yellow, Red, Unknown }

pub struct MemoryManager {
    fd: OwnedFd, 
    current_zone: PressureZone,
}

impl MemoryManager {
    pub fn new() -> Result<Self, String> {
        ffi::log_info("MemoryManager: Initializing...");
        Self::apply_tweak(PressureZone::Green, PressureZone::Unknown);
        let raw_fd = ffi::register_psi_trigger(K_PSI_MEMORY_PATH, 60000, 1000000);
        if raw_fd < 0 { return Err("Failed to register Memory PSI".to_string()); }
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        Ok(Self { fd, current_zone: PressureZone::Green })
    }
    fn apply_tweak(new_zone: PressureZone, old_zone: PressureZone) {
        if new_zone == old_zone { return; }
        let (swap, cache, label) = match new_zone {
            PressureZone::Green => ("30", "50", "(Power Save) -> GREEN"),
            PressureZone::Yellow => ("60", "100", "(Balanced) -> YELLOW"),
            PressureZone::Red => ("100", "150", "(Performance) -> RED"),
            _ => return,
        };
        ffi::log_info(&format!("MemoryManager: {}", label));
        ffi::apply_tweak(K_SWAPPINESS_PATH, swap);
        ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, cache);
    }
}

impl EventHandler for MemoryManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) {
        let psi_value = ffi::get_memory_pressure();
        let new_zone = if psi_value > 15.0 { PressureZone::Red } else { PressureZone::Yellow };
        if self.current_zone != new_zone {
            ffi::log_debug(&format!("MemoryManager: PSI {:.2}%", psi_value));
            Self::apply_tweak(new_zone, self.current_zone);
            self.current_zone = new_zone;
        }
    }
}