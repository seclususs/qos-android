//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::sys;
use crate::daemon::types;

use std::io;

pub fn set_refresh_rate(param: i32) -> Result<(), types::QosError> {
    let result = unsafe { sys::cpp_set_refresh_rate(param) };
    if result < 0 {
        Err(types::QosError::IoError(io::Error::last_os_error()))
    } else {
        Ok(())
    }
}
