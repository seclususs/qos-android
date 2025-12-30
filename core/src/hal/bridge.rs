//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::sys;

use std::ffi::CString;

pub fn notify_service_death(context: &str) {
    let c_context_opt = CString::new(context);
    let ptr = match c_context_opt {
        Ok(ref c) => c.as_ptr(),
        Err(_) => c"Service Death".as_ptr(),
    };
    unsafe { sys::cpp_notify_service_death(ptr) }
}