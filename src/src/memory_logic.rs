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
const K_PSI_UP_TO_YELLOW: f64 = 6.0;
const K_PSI_UP_TO_RED: f64 = 18.0;
const K_PSI_DOWN_TO_YELLOW: f64 = 12.0;
const K_PSI_DOWN_TO_GREEN: f64 = 3.0;
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";

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
            ffi::log_info("MemoryManager: (Power Save). Zone: GREEN");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_GREEN);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_CACHE_PRESSURE_GREEN);
        }
        PressureZone::Yellow => {
            ffi::log_info("MemoryManager: (Balanced). Zone: YELLOW");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_YELLOW);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_CACHE_PRESSURE_YELLOW);
        }
        PressureZone::Red => {
            ffi::log_info("MemoryManager: (Performance). Zone: RED");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_RED);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_CACHE_PRESSURE_RED);
        }
        PressureZone::Unknown => {}
    }
    *current_zone = new_zone;
}

pub fn monitor_memory(shutdown_requested: &AtomicBool) {
    ffi::log_info("MemoryManager: Starting monitoring...");
    let mut current_zone = PressureZone::Unknown;
    while !shutdown_requested.load(Ordering::Acquire) {
        let psi_value = ffi::get_memory_pressure();
        if psi_value < 0.0 {
            ffi::log_error("MemoryManager: Failed to read. Retrying in 5s...");
            thread::sleep(Duration::from_secs(5));
            continue;
        }
        ffi::log_debug(&format!("MemoryManager: {:.2}% | Zone: {:?}", psi_value, current_zone));
        let new_zone = match current_zone {
            PressureZone::Green => {
                if psi_value > K_PSI_UP_TO_RED { PressureZone::Red }
                else if psi_value > K_PSI_UP_TO_YELLOW { PressureZone::Yellow }
                else { PressureZone::Green }
            },
            PressureZone::Yellow => {
                if psi_value > K_PSI_UP_TO_RED { PressureZone::Red }
                else if psi_value < K_PSI_DOWN_TO_GREEN { PressureZone::Green }
                else { PressureZone::Yellow }
            },
            PressureZone::Red => {
                if psi_value < K_PSI_DOWN_TO_GREEN { PressureZone::Green }
                else if psi_value < K_PSI_DOWN_TO_YELLOW { PressureZone::Yellow }
                else { PressureZone::Red }
            },
            PressureZone::Unknown => {
                if psi_value >= K_PSI_UP_TO_RED { PressureZone::Red }
                else if psi_value >= K_PSI_UP_TO_YELLOW { PressureZone::Yellow }
                else { PressureZone::Green }
            }
        };
        apply_pressure_response(new_zone, &mut current_zone);
        thread::sleep(Duration::from_secs(1));
    }
    ffi::log_info("MemoryManager: Monitoring stopped.");
}