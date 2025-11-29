//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::traits::EventHandler;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};
use std::time::{Duration, Instant};

const K_TOUCH_PATH: &str = "/dev/input/event3";
const K_IDLE_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(PartialEq, Clone, Copy)]
enum RefreshMode { Low, High }

pub struct RefreshManager {
    fd: OwnedFd,
    current_mode: RefreshMode,
    last_touch: Instant,
}

impl RefreshManager {
    pub fn new() -> Result<Self, String> {
        ffi::log_info("RefreshManager: Initializing...");
        Self::set_rate(RefreshMode::Low);
        let raw_fd = ffi::open_touch_device(K_TOUCH_PATH);
        if raw_fd < 0 { return Err(format!("Failed to open {}", K_TOUCH_PATH)); }
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        Ok(Self { fd, current_mode: RefreshMode::Low, last_touch: Instant::now() })
    }
    fn set_rate(mode: RefreshMode) {
        let val = match mode { RefreshMode::High => "90.0", RefreshMode::Low => "60.0" };
        ffi::set_android_setting("min_refresh_rate", val);
    }
}

impl EventHandler for RefreshManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) {
        ffi::read_touch_events(self.fd.as_raw_fd());
        self.last_touch = Instant::now();
        if self.current_mode != RefreshMode::High {
            ffi::log_info("Touch -> High Refresh Rate");
            Self::set_rate(RefreshMode::High);
            self.current_mode = RefreshMode::High;
        }
    }
    fn get_timeout_ms(&self) -> i32 {
        if self.current_mode == RefreshMode::High {
            let elapsed = self.last_touch.elapsed();
            if elapsed >= K_IDLE_TIMEOUT { 0 } else { (K_IDLE_TIMEOUT - elapsed).as_millis() as i32 }
        } else {
            -1
        }
    }
    fn on_timeout(&mut self) {
        if self.current_mode == RefreshMode::High {
            if self.last_touch.elapsed() >= K_IDLE_TIMEOUT {
                ffi::log_info("Idle -> Low Refresh Rate");
                Self::set_rate(RefreshMode::Low);
                self.current_mode = RefreshMode::Low;
            }
        }
    }
}