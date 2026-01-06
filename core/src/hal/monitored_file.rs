//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::filesystem::open_file_for_read;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct MonitoredFile<const BUFFER_SIZE: usize> {
    file: File,
    buffer: [u8; BUFFER_SIZE],
}

impl<const BUFFER_SIZE: usize> MonitoredFile<BUFFER_SIZE> {
    pub fn new(path: &str) -> Result<Self, QosError> {
        let file = open_file_for_read(path)?;
        Ok(Self {
            file,
            buffer: [0u8; BUFFER_SIZE],
        })
    }
    pub fn read_value(&mut self) -> Result<&str, QosError> {
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(QosError::IoError)?;
        let bytes_read = self
            .file
            .read(&mut self.buffer)
            .map_err(QosError::IoError)?;
        if bytes_read == 0 {
            return Ok("");
        }
        unsafe { Ok(std::str::from_utf8_unchecked(&self.buffer[..bytes_read])) }
    }
}