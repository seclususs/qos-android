//! Author: [Seclususs](https://github.com/seclususs)


use libc::{c_char, c_int};
use std::ffi::CString;

#[allow(dead_code)]
#[link(name = "c")]
unsafe extern "C" {
    fn cpp_apply_tweak(path: *const c_char, value: *const c_char) -> bool;
    fn cpp_set_system_prop(key: *const c_char, value: *const c_char);
    fn cpp_set_android_setting(property: *const c_char, value: *const c_char) -> bool;
    fn cpp_log_info(message: *const c_char);
    fn cpp_log_debug(message: *const c_char);
    fn cpp_log_error(message: *const c_char);
    fn cpp_close_fd(fd: c_int);
    fn cpp_get_free_ram_percentage() -> c_int;
    fn cpp_create_netlink_socket() -> c_int;
    fn cpp_poll_fd(fd: c_int, timeout_ms: c_int) -> c_int;
    fn cpp_read_netlink_event(fd: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int;
    fn cpp_open_touch_device(path: *const c_char) -> c_int;
    fn cpp_read_touch_events(fd: c_int);
}

pub fn log_info(msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_default();
    unsafe { cpp_log_info(c_msg.as_ptr()) }
}

pub fn log_debug(msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_default();
    unsafe { cpp_log_debug(c_msg.as_ptr()) }
}

pub fn log_error(msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_default();
    unsafe { cpp_log_error(c_msg.as_ptr()) }
}

pub fn close_fd(fd: i32) {
    unsafe { cpp_close_fd(fd) }
}

pub fn apply_tweak(path: &str, value: &str) -> bool {
    let c_path = CString::new(path).unwrap_or_default();
    let c_value = CString::new(value).unwrap_or_default();
    unsafe { cpp_apply_tweak(c_path.as_ptr(), c_value.as_ptr()) }
}

pub fn set_android_setting(property: &str, value: &str) -> bool {
    let c_prop = CString::new(property).unwrap_or_default();
    let c_value = CString::new(value).unwrap_or_default();
    unsafe { cpp_set_android_setting(c_prop.as_ptr(), c_value.as_ptr()) }
}

pub fn get_free_ram_percentage() -> i32 {
    unsafe { cpp_get_free_ram_percentage() }
}

pub fn create_netlink_socket() -> i32 {
    unsafe { cpp_create_netlink_socket() }
}

pub fn read_netlink_event(fd: i32, buffer: &mut [u8]) -> Option<&str> {
    unsafe {
        let len = cpp_read_netlink_event(fd, buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_int);
        if len > 0 {
            Some(std::str::from_utf8_unchecked(&buffer[..len as usize]))
        } else {
            None
        }
    }
}

pub fn open_touch_device(path: &str) -> i32 {
    let c_path = CString::new(path).unwrap_or_default();
    unsafe { cpp_open_touch_device(c_path.as_ptr()) }
}

pub fn poll_fd(fd: i32, timeout_ms: i32) -> i32 {
    unsafe { cpp_poll_fd(fd, timeout_ms) }
}

pub fn read_touch_events(fd: i32) {
    unsafe { cpp_read_touch_events(fd) }
}