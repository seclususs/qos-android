//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::{cstr, sys};
use crate::daemon::types;

use std::{fs, os};

#[inline]
fn validate_fd_secure(fd: os::fd::RawFd) -> Result<(), types::QosError> {
    let mut path_buf = [0u8; 32];
    path_buf[0..14].copy_from_slice(b"/proc/self/fd/");

    let mut itoa_buf = itoa::Buffer::new();
    let fd_bytes = itoa_buf.format(fd).as_bytes();
    let end = 14 + fd_bytes.len();

    if end >= path_buf.len() {
        return Err(types::QosError::InvalidPath("FD too large".into()));
    }

    path_buf[14..end].copy_from_slice(fd_bytes);
    path_buf[end] = b'\0';

    let mut target_buf = [0u8; 256];
    let res = unsafe {
        sys::readlink(
            path_buf.as_ptr().cast::<core::ffi::c_char>(),
            target_buf.as_mut_ptr().cast::<core::ffi::c_char>(),
            target_buf.len(),
        )
    };

    if res < 0 {
        return Err(types::QosError::InvalidPath("FD resolution failed".into()));
    }

    let target_slice = &target_buf[..res as usize];

    if target_slice.starts_with(b"/proc/") || target_slice.starts_with(b"/sys/") {
        Ok(())
    } else {
        Err(types::QosError::PermissionDenied("Access denied".into()))
    }
}

pub fn open_file_for_write(path: &str) -> Result<fs::File, types::QosError> {
    let fd = rustix::fs::openat(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| types::QosError::IoError(e.into()))?;

    validate_fd_secure(os::fd::AsRawFd::as_raw_fd(&fd))?;

    Ok(fd.into())
}

pub fn open_file_for_read(path: &str) -> Result<fs::File, types::QosError> {
    let fd = rustix::fs::openat(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| types::QosError::IoError(e.into()))?;

    validate_fd_secure(os::fd::AsRawFd::as_raw_fd(&fd))?;

    Ok(fd.into())
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
            log::debug!("Write via rustix::pwrite failed: {e}");
            types::QosError::IoError(e.into())
        })?;
    } else {
        let fd = os::fd::AsFd::as_fd(file);

        rustix::io::pwrite(fd, bytes, 0).map_err(|e| {
            log::debug!("Write via rustix::pwrite failed: {e}");
            types::QosError::IoError(e.into())
        })?;
    }

    Ok(())
}

pub fn write_to_file(path: &str, value: &str) -> Result<(), types::QosError> {
    if !cstr::validate_value(value) {
        return Err(types::QosError::SystemCheckFailed(
            format!("Invalid characters in value for {path}: '{value}'").into(),
        ));
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
        rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| {
        log::debug!("Openat failed for {path}: {e}");
        types::QosError::IoError(e.into())
    })?;

    validate_fd_secure(os::fd::AsRawFd::as_raw_fd(&fd))?;

    rustix::io::write(&fd, final_slice).map_err(|e| {
        log::debug!("Write raw failed '{value}' -> {path}: {e}");
        types::QosError::IoError(e.into())
    })?;

    Ok(())
}

#[inline]
fn check_absolute(current: u64, target: u64, threshold: u64) -> bool {
    if current == target {
        return false;
    }

    current.abs_diff(target) >= threshold
}

#[inline]
fn check_relative(current: u64, target: u64, tolerance_per_mille: u64) -> bool {
    if current == target {
        return false;
    }

    if current == 0 {
        return target != 0;
    }

    let diff = current.abs_diff(target);
    (diff as u128 * 1000) >= (current as u128 * tolerance_per_mille as u128)
}

pub enum CheckStrategy {
    Absolute(u64),
    Relative(u64),
    Strict,
}

pub struct CachedFile {
    file: Option<fs::File>,
    last_value: u64,
}

impl CachedFile {
    pub fn new(file: fs::File, initial_value: u64) -> Self {
        Self {
            file: Some(file),
            last_value: initial_value,
        }
    }

    pub fn new_opt(file: Option<fs::File>, initial_value: u64) -> Self {
        Self {
            file,
            last_value: initial_value,
        }
    }

    pub fn is_active(&self) -> bool {
        self.file.is_some()
    }

    pub fn update(&mut self, new_value: u64, force: bool, strategy: &CheckStrategy) {
        let Some(ref mut file) = self.file else {
            return;
        };

        let needs_update = if force {
            true
        } else {
            match strategy {
                CheckStrategy::Absolute(threshold) => {
                    check_absolute(self.last_value, new_value, *threshold)
                }
                CheckStrategy::Relative(tolerance) => {
                    check_relative(self.last_value, new_value, *tolerance)
                }
                CheckStrategy::Strict => self.last_value != new_value,
            }
        };

        if needs_update && write_to_stream(file, new_value).is_ok() {
            self.last_value = new_value;
        }
    }
}

pub struct MonitoredFile<const BUFFER_SIZE: usize> {
    file: fs::File,
    buffer: [u8; BUFFER_SIZE],
}

impl<const BUFFER_SIZE: usize> MonitoredFile<BUFFER_SIZE> {
    pub fn new(path: &str) -> Result<Self, types::QosError> {
        let file = open_file_for_read(path)?;

        Ok(Self {
            file,
            buffer: [0u8; BUFFER_SIZE],
        })
    }

    pub fn read_value(&mut self) -> Result<&str, types::QosError> {
        let bytes_read = os::unix::fs::FileExt::read_at(&self.file, &mut self.buffer, 0)
            .map_err(types::QosError::IoError)?;

        if bytes_read == 0 {
            return Ok("");
        }

        unsafe { Ok(std::str::from_utf8_unchecked(&self.buffer[..bytes_read])) }
    }

    pub fn read_bytes_raw(&mut self) -> Result<&[u8], types::QosError> {
        let bytes_read = os::unix::fs::FileExt::read_at(&self.file, &mut self.buffer, 0)
            .map_err(types::QosError::IoError)?;

        Ok(&self.buffer[..bytes_read])
    }
}
