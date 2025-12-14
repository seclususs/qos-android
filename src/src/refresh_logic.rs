//! Author: [Seclususs](https://github.com/seclususs)

use crate::ffi;
use crate::system_utils;
use crate::traits::EventHandler;
use crate::error::QosError;
use std::os::fd::{RawFd, AsRawFd, OwnedFd, FromRawFd};
use std::time::{Duration, Instant};

const K_TOUCH_PATH: &str = "/dev/input/event3";
const K_IDLE_TIMEOUT: Duration = Duration::from_millis(5000);

#[derive(PartialEq, Clone, Copy, Debug)]
enum DisplayMode { LowPower, Smooth }

pub struct RefreshManager {
    fd: OwnedFd,
    current_mode: DisplayMode,
    last_interaction: Instant,
    cached_prop_val: String, 
}

impl RefreshManager {
    pub fn new() -> Result<Self, QosError> {
        log::info!("RefreshManager: Initializing Display Service...");
        let raw_fd = ffi::open_touch_device(K_TOUCH_PATH);
        if raw_fd < 0 { 
            return Err(QosError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                format!("Failed to open input device: {}", K_TOUCH_PATH)
            ))); 
        }
        let mut manager = Self { 
            fd: unsafe { OwnedFd::from_raw_fd(raw_fd) },
            current_mode: DisplayMode::LowPower,
            last_interaction: Instant::now(),
            cached_prop_val: String::new(),
        };
        manager.apply_mode(DisplayMode::LowPower, true);
        Ok(manager)
    }
    fn apply_mode(&mut self, mode: DisplayMode, force: bool) {
        let val = match mode { 
            DisplayMode::Smooth => "90.0", 
            DisplayMode::LowPower => "60.0" 
        };
        if force || self.cached_prop_val != val {
            if force {
                log::debug!("RefreshManager: Force Mode -> {:?}", mode);
            }
            system_utils::set_android_setting("system", "min_refresh_rate", val);
            self.cached_prop_val = val.to_string();
        }
        self.current_mode = mode;
    }
}

impl EventHandler for RefreshManager {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) {
        ffi::read_touch_events(self.fd.as_raw_fd());
        self.last_interaction = Instant::now();
        if self.current_mode != DisplayMode::Smooth {
            self.apply_mode(DisplayMode::Smooth, false);
        }
    }
    fn get_timeout_ms(&self) -> i32 {
        if self.current_mode == DisplayMode::Smooth {
            let elapsed = self.last_interaction.elapsed();
            if elapsed >= K_IDLE_TIMEOUT { 0 } else { (K_IDLE_TIMEOUT - elapsed).as_millis() as i32 }
        } else {
            -1
        }
    }
    fn on_timeout(&mut self) {
        if self.current_mode == DisplayMode::Smooth {
            if self.last_interaction.elapsed() >= K_IDLE_TIMEOUT {
                log::debug!("RefreshManager: Idle detected -> Dropping to 60Hz");
                self.apply_mode(DisplayMode::LowPower, false);
            }
        }
    }
}