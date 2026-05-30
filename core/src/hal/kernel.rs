//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::{cstr, sys};
use crate::daemon::types;

use std::io;

pub fn register_psi_trigger(
    path: &str,
    threshold_us: i32,
    window_us: i32,
) -> Result<i32, types::QosError> {
    cstr::with_cstr(path, |c_path| {
        let fd = unsafe { sys::register_psi_trigger(c_path, threshold_us, window_us) };
        if fd < 0 {
            Err(types::QosError::IoError(io::Error::last_os_error()))
        } else {
            Ok(fd)
        }
    })
    .and_then(|r| r)
}
