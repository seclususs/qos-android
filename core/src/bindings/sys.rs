//! Author: [Seclususs](https://github.com/seclususs)

use libc::{c_char, c_int, size_t};

#[link(name = "c")]
unsafe extern "C" {
    pub fn notify_service_death(context: *const c_char);
    pub fn register_psi_trigger(
        path: *const c_char,
        threshold_us: c_int,
        window_us: c_int,
    ) -> c_int;
    pub fn set_system_property(key: *const c_char, value: *const c_char) -> c_int;
    pub fn get_system_property(key: *const c_char, value: *mut c_char, max_len: size_t) -> c_int;
    pub fn mallopt(param: c_int, value: c_int) -> c_int;
    pub fn readlink(
        pathname: *const core::ffi::c_char,
        buf: *mut core::ffi::c_char,
        bufsiz: usize,
    ) -> isize;
}
