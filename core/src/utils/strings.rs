//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;

use std::ffi;

pub fn to_cstring(s: &str) -> Result<ffi::CString, types::QosError> {
    ffi::CString::new(s)
        .map_err(|e| types::QosError::InvalidInput(format!("String contains null byte: {e}")))
}

#[inline]
pub fn validate_value(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '=' || c == ' ')
}
