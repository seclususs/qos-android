//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::filesystem::open_file_for_read;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone, Copy, Default)]
pub struct IoStats {
    pub read_ios: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
    pub write_ticks: u64,
    pub in_flight: u64,
}

pub struct DiskMonitor {
    file: File,
    buffer: [u8; 512],
}

impl DiskMonitor {
    pub fn new(path: &str) -> Result<Self, QosError> {
        let file = open_file_for_read(path)?;
        Ok(Self {
            file,
            buffer: [0u8; 512],
        })
    }
    pub fn read_stats(&mut self) -> Result<IoStats, QosError> {
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(QosError::IoError)?;
        let bytes_read = match self.file.read(&mut self.buffer) {
            Ok(n) => n,
            Err(e) => return Err(QosError::IoError(e)),
        };
        if bytes_read == 0 {
            return Err(QosError::SystemCheckFailed(
                "Empty diskstats file".to_string(),
            ));
        }
        let content = std::str::from_utf8(&self.buffer[..bytes_read])
            .map_err(|_| QosError::InvalidInput("Invalid UTF-8 in diskstats".to_string()))?;
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() < 10 {
            return Err(QosError::SystemCheckFailed(
                "Incomplete diskstats format".to_string(),
            ));
        }
        let read_ios = parts[0].parse::<u64>().unwrap_or(0);
        let read_ticks = parts[3].parse::<u64>().unwrap_or(0);
        let write_ios = parts[4].parse::<u64>().unwrap_or(0);
        let write_ticks = parts[7].parse::<u64>().unwrap_or(0);
        let in_flight = parts[8].parse::<u64>().unwrap_or(0);
        Ok(IoStats {
            read_ios,
            read_ticks,
            write_ios,
            write_ticks,
            in_flight,
        })
    }
}