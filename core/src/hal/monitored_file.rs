//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;
use crate::hal::filesystem;

use std::{fs, io};

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
        io::Seek::seek(&mut self.file, io::SeekFrom::Start(0)).map_err(types::QosError::IoError)?;
        let bytes_read =
            io::Read::read(&mut self.file, &mut self.buffer).map_err(types::QosError::IoError)?;
        if bytes_read == 0 {
            return Ok("");
        }
        unsafe { Ok(std::str::from_utf8_unchecked(&self.buffer[..bytes_read])) }
    }
}
