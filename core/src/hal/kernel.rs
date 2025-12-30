//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::sys;
use crate::daemon::types::QosError;

use std::ffi::CString;
use std::io;

fn to_cstring(s: &str) -> Result<CString, QosError> {
    CString::new(s).map_err(|e| QosError::InvalidInput(format!("String contains null byte: {}", e)))
}

pub fn register_psi_trigger(
    path: &str,
    threshold_us: i32,
    window_us: i32,
) -> Result<i32, QosError> {
    let c_path = to_cstring(path)?;
    let fd = unsafe { sys::cpp_register_psi_trigger(c_path.as_ptr(), threshold_us, window_us) };
    if fd < 0 {
        Err(QosError::IoError(io::Error::last_os_error()))
    } else {
        Ok(fd)
    }
}