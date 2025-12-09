//! Author: [Seclususs](https://github.com/seclususs)

#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub enum LogLevel {
    ErrorOnly = 1,
    Basic = 2,
    Full = 3,
}

#[cfg(debug_assertions)]
pub const ACTIVE_PROFILE: LogLevel = LogLevel::Full;

#[cfg(not(debug_assertions))]
pub const ACTIVE_PROFILE: LogLevel = LogLevel::Basic;

#[inline(always)]
pub fn enabled(target: LogLevel) -> bool {
    target <= ACTIVE_PROFILE
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if $crate::logger::enabled($crate::logger::LogLevel::Basic) {
            let msg = format!($($arg)*);
            $crate::ffi::raw_log_info(&msg);
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::logger::enabled($crate::logger::LogLevel::Full) {
            let msg = format!($($arg)*);
            $crate::ffi::raw_log_debug(&msg);
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if $crate::logger::enabled($crate::logger::LogLevel::ErrorOnly) {
            let msg = format!($($arg)*);
            $crate::ffi::raw_log_error(&msg);
        }
    }
}