//! Author: [Seclususs](https://github.com/seclususs)

use std::ffi::NulError;
use std::fmt;

#[derive(Debug)]
pub enum QosError {
    IoError(std::io::Error),
    SystemCheckFailed(String),
    PermissionDenied(String),
    InvalidPath(String),
    InvalidInput(String),
    PsiParseError(String),
    FfiError(String),
}

impl fmt::Display for QosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QosError::IoError(e) => write!(f, "I/O Error: {}", e),
            QosError::SystemCheckFailed(s) => write!(f, "System Check Failed: {}", s),
            QosError::PermissionDenied(s) => write!(f, "Permission Denied: {}", s),
            QosError::InvalidPath(s) => write!(f, "Invalid Path: {}", s),
            QosError::InvalidInput(s) => write!(f, "Invalid Input: {}", s),
            QosError::PsiParseError(s) => write!(f, "PSI Parse Error: {}", s),
            QosError::FfiError(s) => write!(f, "FFI Error: {}", s),
        }
    }
}

impl From<std::io::Error> for QosError {
    fn from(err: std::io::Error) -> Self {
        QosError::IoError(err)
    }
}

impl From<NulError> for QosError {
    fn from(err: NulError) -> Self {
        QosError::InvalidInput(format!("String contains null byte: {}", err))
    }
}