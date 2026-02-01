//! Author: [Seclususs](https://github.com/seclususs)

use libc::{c_char, c_int, size_t};

#[link(name = "c")]
unsafe extern "C" {
    pub fn cpp_notify_service_death(context: *const c_char);
    pub fn cpp_register_psi_trigger(
        path: *const c_char,
        threshold_us: c_int,
        window_us: c_int,
    ) -> c_int;
    pub fn cpp_set_system_property(key: *const c_char, value: *const c_char) -> c_int;
    pub fn cpp_get_system_property(
        key: *const c_char,
        value: *mut c_char,
        max_len: size_t,
    ) -> c_int;
}
