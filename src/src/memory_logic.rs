//! Author: [Seclususs](https://github.com/seclususs)


use crate::ffi;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const K_SWAPPINESS_GREEN: &str = "30";
const K_CACHE_PRESSURE_GREEN: &str = "50";
const K_SWAPPINESS_YELLOW: &str = "100";
const K_CACHE_PRESSURE_YELLOW: &str = "120";
const K_SWAPPINESS_RED: &str = "190";
const K_CACHE_PRESSURE_RED: &str = "200";
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";
const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
const K_PSI_THRESHOLD_US: i32 = 60000; 
const K_PSI_WINDOW_US: i32 = 1000000;
const K_EPOLL_TIMEOUT_MS: i32 = 5000;

#[derive(Debug, PartialEq, Copy, Clone)]
enum PressureZone {
    Green,
    Yellow,
    Red,
    Unknown,
}

fn apply_pressure_response(new_zone: PressureZone, current_zone: &mut PressureZone) {
    if new_zone == *current_zone {
        return;
    }
    match new_zone {
        PressureZone::Green => {
            ffi::log_info("MemoryManager: (Power Save) -> GREEN");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_GREEN);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_CACHE_PRESSURE_GREEN);
        }
        PressureZone::Yellow => {
            ffi::log_info("MemoryManager: (Balanced) -> YELLOW");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_YELLOW);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_CACHE_PRESSURE_YELLOW);
        }
        PressureZone::Red => {
            ffi::log_info("MemoryManager: (Performance) -> RED");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_RED);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_CACHE_PRESSURE_RED);
        }
        PressureZone::Unknown => {}
    }
    *current_zone = new_zone;
}

pub fn monitor_memory(shutdown_requested: &AtomicBool) {
    ffi::log_info("MemoryManager: Initializing Event-Driven PSI...");
    let epoll_fd = ffi::register_psi_trigger(K_PSI_MEMORY_PATH, K_PSI_THRESHOLD_US, K_PSI_WINDOW_US);
    if epoll_fd < 0 {
        ffi::log_error("MemoryManager: Failed to register PSI trigger. Falling back to simple polling.");
        thread::sleep(Duration::from_secs(10)); 
        return;
    }
    ffi::log_info("MemoryManager: PSI Trigger Registered. Entering event loop.");
    let mut current_zone = PressureZone::Unknown;
    apply_pressure_response(PressureZone::Green, &mut current_zone);
    while !shutdown_requested.load(Ordering::Acquire) {
        let result = ffi::wait_for_psi_event(epoll_fd, K_EPOLL_TIMEOUT_MS);
        match result {
            1 => {
                let psi_value = ffi::get_memory_pressure();
                let new_zone = if psi_value > 15.0 {
                    PressureZone::Red
                } else {
                    PressureZone::Yellow
                };
                if current_zone != new_zone {
                    ffi::log_debug(&format!("MemoryManager: PSI Event! Value: {:.2}%", psi_value));
                    apply_pressure_response(new_zone, &mut current_zone);
                }
            }
            0 => {
                if current_zone != PressureZone::Green {
                    ffi::log_debug("MemoryManager: System Idle (Timeout). Reverting to GREEN.");
                    apply_pressure_response(PressureZone::Green, &mut current_zone);
                }
            }
            _ => {
                ffi::log_error("MemoryManager: Epoll error.");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    ffi::close_fd(epoll_fd);
    ffi::log_info("MemoryManager: Monitoring stopped.");
}