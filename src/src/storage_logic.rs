//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::traits::EventHandler;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};

const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";

#[derive(Debug, PartialEq, Copy, Clone)]
enum IoPressureZone { Green, Yellow, Red }

pub struct StorageManager {
    fd: OwnedFd,
    current_zone: IoPressureZone,
}

impl StorageManager {
    pub fn new() -> Result<Self, String> {
        ffi::log_info("StorageManager: Initializing...");
        Self::apply_tweak(IoPressureZone::Green);
        let raw_fd = ffi::register_psi_trigger(K_PSI_IO_PATH, 60000, 1000000);
        if raw_fd < 0 { return Err("Failed to register Storage PSI".to_string()); }
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        Ok(Self { fd, current_zone: IoPressureZone::Green })
    }
    fn apply_tweak(zone: IoPressureZone) {
        let val = match zone {
            IoPressureZone::Green => "512",
            IoPressureZone::Yellow => "256",
            IoPressureZone::Red => "128",
        };
        ffi::apply_tweak(K_READ_AHEAD_PATH, val);
    }
}

impl EventHandler for StorageManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) {
        let psi_value = ffi::get_io_pressure();
        let new_zone = if psi_value > 5.0 { IoPressureZone::Red } else { IoPressureZone::Yellow };
        if self.current_zone != new_zone {
            ffi::log_debug(&format!("StorageManager: PSI {:.2}%", psi_value));
            Self::apply_tweak(new_zone);
            self.current_zone = new_zone;
        }
    }
}