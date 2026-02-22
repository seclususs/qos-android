//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;
use crate::utils::strings;

use std::{fs, os, path};

const ALLOWED_PREFIXES: [&str; 2] = ["/proc/", "/sys/"];

fn validate_path_secure(path_str: &str) -> Result<(), types::QosError> {
    let path = path::Path::new(path_str);
    let canonical_path = fs::canonicalize(path).map_err(|e| {
        types::QosError::InvalidPath(format!("Path resolution failed for {path_str}: {e}"))
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
            "Access denied: {canonical_str}"
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
    let mut buffer = itoa::Buffer::new();
    let formatted_str = buffer.format(value);
    let mut write_buf = [0u8; 32];
    let bytes = formatted_str.as_bytes();
    let len = bytes.len();
    if len < write_buf.len() {
        write_buf[..len].copy_from_slice(bytes);
        write_buf[len] = b'\n';
        let fd = os::fd::AsFd::as_fd(file);
        rustix::io::pwrite(fd, &write_buf[..=len], 0).map_err(|e| {
            log::warn!("Write via rustix::pwrite failed: {e}");
            types::QosError::IoError(e.into())
        })?;
    } else {
        let fd = os::fd::AsFd::as_fd(file);
        rustix::io::pwrite(fd, bytes, 0).map_err(|e| {
            log::warn!("Write via rustix::pwrite failed: {e}");
            types::QosError::IoError(e.into())
        })?;
    }
    Ok(())
}

pub fn write_to_file(path: &str, value: &str) -> Result<(), types::QosError> {
    validate_path_secure(path)?;
    if !strings::validate_value(value) {
        return Err(types::QosError::SystemCheckFailed(format!(
            "Invalid characters in value for {path}: '{value}'"
        )));
    }
    let mut buffer = [0u8; 64];
    let val_bytes = value.as_bytes();
    if val_bytes.len() + 1 > buffer.len() {
        return Err(types::QosError::InvalidInput(
            "Value too long for stack buffer".into(),
        ));
    }
    buffer[..val_bytes.len()].copy_from_slice(val_bytes);
    buffer[val_bytes.len()] = b'\n';
    let final_slice = &buffer[..=val_bytes.len()];
    let fd = rustix::fs::openat(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::TRUNC | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| {
        log::debug!("Openat failed for {path}: {e}");
        types::QosError::IoError(e.into())
    })?;
    rustix::io::write(&fd, final_slice).map_err(|e| {
        log::debug!("Write raw failed '{value}' -> {path}: {e}");
        types::QosError::IoError(e.into())
    })?;
    Ok(())
}
