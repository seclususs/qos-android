//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
const K_READ_AHEAD_GREEN: &str = "128";
const K_READ_AHEAD_YELLOW: &str = "256";
const K_READ_AHEAD_RED: &str = "512";
const K_PSI_IO_PATH: &str = "/proc/pressure/io";
const K_PSI_THRESHOLD_US: i32 = 60000; 
const K_PSI_WINDOW_US: i32 = 1000000;
const K_EPOLL_TIMEOUT_MS: i32 = 5000;
const K_PSI_UP_TO_RED: f64 = 5.0;

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
    ffi::log_info("StorageManager: Initializing Event-Driven PSI...");
    let epoll_fd = ffi::register_psi_trigger(K_PSI_IO_PATH, K_PSI_THRESHOLD_US, K_PSI_WINDOW_US);
    if epoll_fd < 0 {
        ffi::log_error("StorageManager: Failed to register PSI trigger. Aborting storage monitor.");
        return;
    }
    ffi::log_info("StorageManager: PSI Trigger Registered. Entering event loop.");
    let mut current_zone = IoPressureZone::Unknown;
    apply_io_response(IoPressureZone::Green, &mut current_zone);
    while !shutdown_requested.load(Ordering::Acquire) {
        let result = ffi::wait_for_psi_event(epoll_fd, K_EPOLL_TIMEOUT_MS);
        match result {
            1 => {
                let psi_value = ffi::get_io_pressure();
                let new_zone = if psi_value > K_PSI_UP_TO_RED {
                    IoPressureZone::Red
                } else {
                    IoPressureZone::Yellow
                };
                if current_zone != new_zone {
                    ffi::log_debug(&format!("StorageManager: PSI Event! Value: {:.2}%", psi_value));
                    apply_io_response(new_zone, &mut current_zone);
                }
            },
            0 => {
                if current_zone != IoPressureZone::Green {
                    ffi::log_debug("StorageManager: System Idle (Timeout). Reverting to GREEN.");
                    apply_io_response(IoPressureZone::Green, &mut current_zone);
                }
            },
            _ => {
                ffi::log_error("StorageManager: Epoll error.");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    ffi::close_fd(epoll_fd);
    ffi::log_info("StorageManager: Monitoring stopped.");
}