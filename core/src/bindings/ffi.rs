//! Author: [Seclususs](https://github.com/seclususs)

use crate::common::error::QosError;
use libc::{c_char, c_int, size_t};
use std::ffi::{CStr, CString};
use std::io;

#[link(name = "c")]
unsafe extern "C" {
    fn cpp_notify_service_death(context: *const c_char);
    fn cpp_register_psi_trigger(path: *const c_char, threshold_us: c_int, window_us: c_int) -> c_int;
    fn cpp_set_system_property(key: *const c_char, value: *const c_char) -> c_int;
    fn cpp_get_system_property(key: *const c_char, value: *mut c_char, max_len: size_t) -> c_int;
}

fn to_cstring(s: &str) -> Result<CString, QosError> {
    CString::new(s).map_err(|e| QosError::InvalidInput(format!("String contains null byte: {}", e)))
}

pub fn notify_service_death(context: &str) {
    let c_context_opt = CString::new(context);
    let ptr = match c_context_opt {
        Ok(ref c) => c.as_ptr(),
        Err(_) => {
            unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Service Death (Error in message generation)\0").as_ptr()
            }
        }
    };
    unsafe { cpp_notify_service_death(ptr) }
}

pub fn register_psi_trigger(path: &str, threshold_us: i32, window_us: i32) -> Result<i32, QosError> {
    let c_path = to_cstring(path)?;
    let fd = unsafe { cpp_register_psi_trigger(c_path.as_ptr(), threshold_us, window_us) };
    if fd < 0 {
        Err(QosError::IoError(io::Error::last_os_error()))
    } else {
        Ok(fd)
    }
}

pub fn set_system_property(key: &str, value: &str) -> Result<(), QosError> {
    let c_key = to_cstring(key)?;
    let c_val = to_cstring(value)?;
    let res = unsafe { cpp_set_system_property(c_key.as_ptr(), c_val.as_ptr()) };
    if res < 0 {
        Err(QosError::IoError(io::Error::last_os_error()))
    } else {
        Ok(())
    }
}

pub fn get_system_property(key: &str) -> Result<String, QosError> {
    let c_key = to_cstring(key)?;
    const PROP_VALUE_MAX: usize = 92;
    let mut buffer = vec![0u8; PROP_VALUE_MAX];
    let len = unsafe { 
        cpp_get_system_property(c_key.as_ptr(), buffer.as_mut_ptr() as *mut c_char, PROP_VALUE_MAX) 
    };
    if len < 0 {
        return Ok(String::new());
    }
    let result = unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) };
    Ok(result.to_string_lossy().into_owned())
}