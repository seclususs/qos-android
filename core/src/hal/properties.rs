//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::bindings::sys;

use std::ffi::{CStr, CString};
use std::io;
use libc::c_char;

fn to_cstring(s: &str) -> Result<CString, QosError> {
    CString::new(s).map_err(|e| QosError::InvalidInput(format!("String contains null byte: {}", e)))
}

fn validate_value(value: &str) -> bool {
    value.chars().all(|c| 
        c.is_alphanumeric() || 
        c == '.' || c == '-' || c == '_' || c == '=' || c == ' '
    )
}

pub fn set_system_property(key: &str, value: &str) -> Result<(), QosError> {
    if !key.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') { 
        return Err(QosError::InvalidInput(format!("Invalid characters in key: '{}'", key)));
    }
    if !validate_value(value) {
         return Err(QosError::InvalidInput(format!("Invalid characters in value: '{}'", value)));
    }
    let c_key = to_cstring(key)?;
    let c_val = to_cstring(value)?;
    let res = unsafe { sys::cpp_set_system_property(c_key.as_ptr(), c_val.as_ptr()) };
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
        sys::cpp_get_system_property(c_key.as_ptr(), buffer.as_mut_ptr() as *mut c_char, PROP_VALUE_MAX) 
    };
    if len < 0 {
        return Ok(String::new());
    }
    let result = unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) };
    Ok(result.to_string_lossy().into_owned())
}