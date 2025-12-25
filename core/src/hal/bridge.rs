//! Author: [Seclususs](https://github.com/seclususs)

use std::ffi::{CStr, CString};
use crate::bindings::sys;

pub fn notify_service_death(context: &str) {
    let c_context_opt = CString::new(context);
    let ptr = match c_context_opt {
        Ok(ref c) => c.as_ptr(),
        Err(_) => {
            unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Service Death\0").as_ptr()
            }
        }
    };
    unsafe { sys::cpp_notify_service_death(ptr) }
}