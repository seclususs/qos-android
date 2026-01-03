pub mod api_entry;
pub mod sys;

use crate::daemon::types::QosError;

use std::ffi::CString;

pub fn to_cstring(s: &str) -> Result<CString, QosError> {
    CString::new(s).map_err(|e| QosError::InvalidInput(format!("String contains null byte: {}", e)))
}