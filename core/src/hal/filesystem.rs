//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;

use std::{fs, io, os, path};

const ALLOWED_PREFIXES: [&str; 3] = ["/proc/", "/sys/", "/dev/"];

fn validate_path_secure(path_str: &str) -> Result<(), types::QosError> {
    let path = path::Path::new(path_str);
    let canonical_path = fs::canonicalize(path).map_err(|e| {
        types::QosError::InvalidPath(format!("Path resolution failed for {}: {}", path_str, e))
    })?;
    let canonical_str = canonical_path
        .to_str()
        .ok_or_else(|| types::QosError::InvalidPath("Non-UTF8 path".to_string()))?;
    if ALLOWED_PREFIXES
        .iter()
        .any(|&prefix| canonical_str.starts_with(prefix))
    {
        Ok(())
    } else {
        Err(types::QosError::PermissionDenied(format!(
            "Access denied: {}",
            canonical_str
        )))
    }
}

pub fn open_file_for_write(path: &str) -> Result<fs::File, types::QosError> {
    validate_path_secure(path)?;
    fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(types::QosError::IoError)
}

pub fn open_file_for_read(path: &str) -> Result<fs::File, types::QosError> {
    validate_path_secure(path)?;
    fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(types::QosError::IoError)
}

pub fn write_to_stream(file: &mut fs::File, value: u64) -> Result<(), types::QosError> {
    let mut buffer = [0u8; 24];
    let mut cursor = io::Cursor::new(&mut buffer[..]);
    io::Write::write_fmt(&mut cursor, format_args!("{}", value))
        .map_err(types::QosError::IoError)?;
    let len = cursor.position() as usize;
    let valid_slice = &buffer[..len];
    os::unix::fs::FileExt::write_all_at(file, valid_slice, 0).map_err(|e| {
        log::warn!("Write via pwrite failed: {}", e);
        types::QosError::IoError(e)
    })?;
    Ok(())
}

pub fn write_to_file(path: &str, value: &str) -> Result<(), types::QosError> {
    validate_path_secure(path)?;
    if !super::validate_value(value) {
        return Err(types::QosError::SystemCheckFailed(format!(
            "Invalid characters in value for {}: '{}'",
            path, value
        )));
    }
    let content = format!("{}\n", value);
    fs::write(path, content).map_err(|e| {
        log::debug!("Write failed '{}' -> {}: {}", value, path, e);
        types::QosError::IoError(e)
    })
}
