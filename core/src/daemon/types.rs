//! Author: [Seclususs](https://github.com/seclususs)

use std::{borrow, ffi, fmt};

pub type Result<T> = std::result::Result<T, QosError>;

#[derive(Debug)]
pub enum QosError {
    IoError(std::io::Error),
    SystemCheckFailed(borrow::Cow<'static, str>),
    PermissionDenied(borrow::Cow<'static, str>),
    InvalidPath(borrow::Cow<'static, str>),
    InvalidInput(borrow::Cow<'static, str>),
    PsiParseError(borrow::Cow<'static, str>),
    FfiError(borrow::Cow<'static, str>),
}

impl fmt::Display for QosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QosError::IoError(e) => write!(f, "I/O Error: {e}"),
            QosError::SystemCheckFailed(s) => write!(f, "System Check Failed: {s}"),
            QosError::PermissionDenied(s) => write!(f, "Permission Denied: {s}"),
            QosError::InvalidPath(s) => write!(f, "Invalid Path: {s}"),
            QosError::InvalidInput(s) => write!(f, "Invalid Input: {s}"),
            QosError::PsiParseError(s) => write!(f, "PSI Parse Error: {s}"),
            QosError::FfiError(s) => write!(f, "FFI Error: {s}"),
        }
    }
}

impl From<std::io::Error> for QosError {
    fn from(err: std::io::Error) -> Self {
        QosError::IoError(err)
    }
}

impl From<ffi::NulError> for QosError {
    fn from(err: ffi::NulError) -> Self {
        QosError::InvalidInput(format!("String contains null byte: {err}").into())
    }
}
