//! Author: [Seclususs](https://github.com/seclususs)

use libc::{c_char, c_int};
use std::ffi::CString;

#[allow(dead_code)]
#[link(name = "c")]
unsafe extern "C" {
    fn cpp_log_info(message: *const c_char);
    fn cpp_log_debug(message: *const c_char);
    fn cpp_log_error(message: *const c_char);
    fn cpp_notify_service_death(context: *const c_char);
    fn cpp_open_touch_device(path: *const c_char) -> c_int;
    fn cpp_read_touch_events(fd: c_int);
    fn cpp_register_psi_trigger(path: *const c_char, threshold_us: c_int, window_us: c_int) -> c_int;
}

#[inline]
pub fn raw_log_info(msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_default();
    unsafe { cpp_log_info(c_msg.as_ptr()) }
}

#[inline]
pub fn raw_log_debug(msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_default();
    unsafe { cpp_log_debug(c_msg.as_ptr()) }
}

#[inline]
pub fn raw_log_error(msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_default();
    unsafe { cpp_log_error(c_msg.as_ptr()) }
}

pub fn notify_service_death(context: &str) {
    let c_context = CString::new(context).unwrap_or_default();
    unsafe { cpp_notify_service_death(c_context.as_ptr()) }
}

pub fn open_touch_device(path: &str) -> i32 {
    let c_path = CString::new(path).unwrap_or_default();
    unsafe { cpp_open_touch_device(c_path.as_ptr()) }
}

pub fn read_touch_events(fd: i32) {
    unsafe { cpp_read_touch_events(fd) }
}

pub fn register_psi_trigger(path: &str, threshold_us: i32, window_us: i32) -> i32 {
    let c_path = CString::new(path).unwrap_or_default();
    unsafe { cpp_register_psi_trigger(c_path.as_ptr(), threshold_us, window_us) }
}