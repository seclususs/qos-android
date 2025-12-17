use std::fmt;

#[derive(Debug)]
pub enum QosError {
    IoError(std::io::Error),
    SystemCheckFailed(String),
    PermissionDenied(String),
    InvalidPath(String),
    PsiParseError(String),
    FfiError(String),
}

impl fmt::Display for QosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QosError::IoError(e) => write!(f, "I/O Error: {}", e),
            QosError::SystemCheckFailed(s) => write!(f, "System Check Failed: {}", s),
            QosError::PermissionDenied(s) => write!(f, "Permission Denied: {}", s),
            QosError::InvalidPath(s) => write!(f, "Invalid/Unsafe Path: {}", s),
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