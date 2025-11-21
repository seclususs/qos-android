//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
const K_READ_AHEAD_GREEN: &str = "128";
const K_READ_AHEAD_YELLOW: &str = "256";
const K_READ_AHEAD_RED: &str = "512";
const K_PSI_UP_TO_YELLOW: f64 = 2.0;
const K_PSI_DOWN_TO_GREEN: f64 = 0.5;
const K_PSI_UP_TO_RED: f64 = 5.0;
const K_PSI_DOWN_TO_YELLOW: f64 = 3.5;
const K_BURST_SUSTAIN_SECONDS: i32 = 8;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
enum IoPressureZone {
    Green,
    Yellow,
    Red,
    Unknown,
}

fn apply_io_response(new_zone: IoPressureZone, current_zone: &mut IoPressureZone) {
    if new_zone == *current_zone {
        return;
    }
    match new_zone {
        IoPressureZone::Green => {
            if new_zone != *current_zone {
                ffi::log_info("StorageManager: (Green Zone) -> ReadAhead 128KB");
            }
            ffi::apply_tweak(K_READ_AHEAD_PATH, K_READ_AHEAD_GREEN);
        }
        IoPressureZone::Yellow => {
            if new_zone != *current_zone {
                ffi::log_info("StorageManager: (Yellow Zone) -> ReadAhead 256KB");
            }
            ffi::apply_tweak(K_READ_AHEAD_PATH, K_READ_AHEAD_YELLOW);
        }
        IoPressureZone::Red => {
            if new_zone != *current_zone {
                ffi::log_info("StorageManager: (Red Zone) -> ReadAhead 512KB");
            }
            ffi::apply_tweak(K_READ_AHEAD_PATH, K_READ_AHEAD_RED);
        }
        IoPressureZone::Unknown => {}
    }
    *current_zone = new_zone;
}

pub fn monitor_storage(shutdown_requested: &AtomicBool) {
    ffi::log_info("StorageManager: Starting monitoring...");
    let mut current_zone = IoPressureZone::Unknown;
    let mut sustain_counter = 0;
    while !shutdown_requested.load(Ordering::Acquire) {
        let psi_value = ffi::get_io_pressure();
        if psi_value < 0.0 {
            ffi::log_error("StorageManager: Failed to read. Retrying...");
            thread::sleep(Duration::from_secs(5));
            continue;
        }
        let target_zone = match current_zone {
            IoPressureZone::Green => {
                if psi_value > K_PSI_UP_TO_RED { IoPressureZone::Red }
                else if psi_value > K_PSI_UP_TO_YELLOW { IoPressureZone::Yellow }
                else { IoPressureZone::Green }
            },
            IoPressureZone::Yellow => {
                if psi_value > K_PSI_UP_TO_RED { IoPressureZone::Red }
                else if psi_value < K_PSI_DOWN_TO_GREEN { IoPressureZone::Green }
                else { IoPressureZone::Yellow }
            },
            IoPressureZone::Red => {
                if psi_value < K_PSI_DOWN_TO_GREEN { IoPressureZone::Green }
                else if psi_value < K_PSI_DOWN_TO_YELLOW { IoPressureZone::Yellow }
                else { IoPressureZone::Red }
            },
            IoPressureZone::Unknown => {
                if psi_value >= K_PSI_UP_TO_RED { IoPressureZone::Red }
                else if psi_value >= K_PSI_UP_TO_YELLOW { IoPressureZone::Yellow }
                else { IoPressureZone::Green }
            }
        };
        if target_zone > current_zone {
            sustain_counter = K_BURST_SUSTAIN_SECONDS;
            apply_io_response(target_zone, &mut current_zone);
        } else if target_zone < current_zone {
            if sustain_counter > 0 {
                sustain_counter -= 1;
                ffi::log_debug(&format!("StorageManager: Sustaining zone {:?} for {}s", current_zone, sustain_counter));
            } else {
                apply_io_response(target_zone, &mut current_zone);
            }
        } else {
            if current_zone != IoPressureZone::Green {
                sustain_counter = K_BURST_SUSTAIN_SECONDS;
            }
        }
        if psi_value > 0.0 {
            ffi::log_debug(&format!("StorageManager: IO {:.2}% | Zone: {:?}", psi_value, current_zone));
        }
        thread::sleep(Duration::from_secs(1));
    }
    ffi::log_info("StorageManager: Monitoring stopped.");
}