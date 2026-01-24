//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;
use crate::hal::filesystem;

use std::{fs, os};

pub struct MonitoredFile<const BUFFER_SIZE: usize> {
    file: fs::File,
    buffer: [u8; BUFFER_SIZE],
}

impl<const BUFFER_SIZE: usize> MonitoredFile<BUFFER_SIZE> {
    pub fn new(path: &str) -> Result<Self, types::QosError> {
        let file = filesystem::open_file_for_read(path)?;
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
