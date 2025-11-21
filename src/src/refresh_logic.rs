//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const K_TOUCH_DEVICE_PATH: &str = "/dev/input/event3";
const K_LOW_REFRESH_RATE: &str = "60.0";
const K_HIGH_REFRESH_RATE: &str = "90.0";
const K_REFRESH_RATE_PROPERTY: &str = "min_refresh_rate";
const K_IDLE_TIMEOUT: Duration = Duration::from_secs(4);
const K_MAX_CONSECUTIVE_ERRORS: i32 = 10;
const K_CHECK_INTERVAL_MS: i32 = 100;

#[derive(Debug, PartialEq, Copy, Clone)]
enum RefreshRateMode {
    Low,
    High,
    Unknown,
}

fn set_refresh_rate(new_mode: RefreshRateMode, current_mode: &mut RefreshRateMode) {
    if new_mode == *current_mode {
        return;
    }
    let (rate_str, mode_str) = match new_mode {
        RefreshRateMode::High => (K_HIGH_REFRESH_RATE, "HIGH"),
        _ => (K_LOW_REFRESH_RATE, "LOW"),
    };
    ffi::log_debug(&format!("RefreshManager: Requesting switch to {} mode ({}Hz)", mode_str, rate_str));
    if ffi::set_android_setting(K_REFRESH_RATE_PROPERTY, rate_str) {
        *current_mode = new_mode;
    }
}

pub fn monitor_refresh_rate(shutdown_requested: &AtomicBool) {
    ffi::log_info(&format!("RefreshManager: Starting monitoring on: {}", K_TOUCH_DEVICE_PATH));
    ffi::log_info(&format!("RefreshManager: LOW mode: {}Hz, HIGH mode: {}Hz", K_LOW_REFRESH_RATE, K_HIGH_REFRESH_RATE));
    let fd = ffi::open_touch_device(K_TOUCH_DEVICE_PATH);
    if fd < 0 {
        ffi::log_error(&format!("RefreshManager: Failed to open {}. Exiting.", K_TOUCH_DEVICE_PATH));
        return;
    }
    let mut current_mode = RefreshRateMode::Unknown;
    set_refresh_rate(RefreshRateMode::Low, &mut current_mode);
    let mut last_touch_time = Instant::now();
    let mut consecutive_errors = 0;
    while !shutdown_requested.load(Ordering::Acquire) {
        let poll_result = ffi::poll_fd(fd, K_CHECK_INTERVAL_MS);
        match poll_result {
            1 => {
                consecutive_errors = 0;
                ffi::read_touch_events(fd);
                last_touch_time = Instant::now();
                if current_mode != RefreshRateMode::High {
                    ffi::log_info(&format!("Touch detected -> Switching to {}Hz.", K_HIGH_REFRESH_RATE));
                    set_refresh_rate(RefreshRateMode::High, &mut current_mode);
                }
            }
            0 => {
                let idle_duration = Instant::now().duration_since(last_touch_time);
                if idle_duration >= K_IDLE_TIMEOUT && current_mode == RefreshRateMode::High {
                    ffi::log_info(&format!("No activity -> Reverting to {}Hz.", K_LOW_REFRESH_RATE));
                    set_refresh_rate(RefreshRateMode::Low, &mut current_mode);
                }
            }
            _ => {
                consecutive_errors += 1;
                ffi::log_error(&format!("RefreshManager: poll() error, attempt {}/{}", consecutive_errors, K_MAX_CONSECUTIVE_ERRORS));
                if consecutive_errors >= K_MAX_CONSECUTIVE_ERRORS {
                    ffi::log_error("RefreshManager: Too many errors, stopping monitoring.");
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    ffi::log_info("RefreshManager: Monitoring stopped. Reverting to power-saving mode.");
    set_refresh_rate(RefreshRateMode::Low, &mut current_mode);
    ffi::close_fd(fd);
    ffi::log_debug("RefreshManager: Monitor thread exited.");
}