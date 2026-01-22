pub mod api_entry;
pub mod sys;

use crate::daemon::types;

use std::ffi;

pub fn to_cstring(s: &str) -> Result<ffi::CString, types::QosError> {
    ffi::CString::new(s)
        .map_err(|e| types::QosError::InvalidInput(format!("String contains null byte: {}", e)))
}
