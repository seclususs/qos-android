//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;

use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

const ALLOWED_PREFIXES: [&str; 2] = ["/proc/", "/sys/"];

fn validate_path_secure(path_str: &str) -> Result<(), QosError> {
    let path = Path::new(path_str);
    let canonical_path = fs::canonicalize(path)
        .map_err(|e| QosError::InvalidPath(format!("Path resolution failed for {}: {}", path_str, e)))?;
    let canonical_str = canonical_path.to_str()
        .ok_or_else(|| QosError::InvalidPath("Non-UTF8 path".to_string()))?;
    if ALLOWED_PREFIXES.iter().any(|&prefix| canonical_str.starts_with(prefix)) {
        Ok(())
    } else {
        Err(QosError::PermissionDenied(format!("Access denied: {}", canonical_str)))
    }
}

fn validate_value(value: &str) -> bool {
    value.chars().all(|c| 
        c.is_alphanumeric() || 
        c == '.' || c == '-' || c == '_' || c == '=' || c == ' '
    )
}

pub fn open_file_for_write(path: &str) -> Result<File, QosError> {
    validate_path_secure(path)?;
    fs::OpenOptions::new()
        .write(true) 
        .open(path)
        .map_err(|e| QosError::IoError(e))
}

pub fn open_file_for_read(path: &str) -> Result<File, QosError> {
    validate_path_secure(path)?;
    fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| QosError::IoError(e))
}

pub fn write_to_stream(file: &mut File, value: &str) -> Result<(), QosError> {
    if !validate_value(value) {
        return Err(QosError::InvalidInput(format!("Invalid characters in value: '{}'", value)));
    }
    let _ = file.seek(SeekFrom::Start(0));
    let content = format!("{}\n", value);
    file.write_all(content.as_bytes()).map_err(|e| {
        log::warn!("Write to stream failed: {}", e);
        QosError::IoError(e)
    })?;
    Ok(())
}

pub fn write_to_file(path: &str, value: &str) -> Result<(), QosError> {
    validate_path_secure(path)?;
    if !validate_value(value) {
        return Err(QosError::SystemCheckFailed(format!("Invalid characters in value for {}: '{}'", path, value)));
    }
    let content = format!("{}\n", value);
    fs::write(path, content).map_err(|e| {
        log::debug!("Write failed '{}' -> {}: {}", value, path, e);
        QosError::IoError(e)
    })
}