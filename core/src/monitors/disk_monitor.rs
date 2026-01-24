//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;
use crate::hal::monitored_file;

#[derive(Debug, Clone, Copy, Default)]
pub struct IoStats {
    pub read_ios: u64,
    pub read_merges: u64,
    pub read_sectors: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
    pub write_ticks: u64,
    pub in_flight: u64,
}

pub struct DiskMonitor {
    monitor: monitored_file::MonitoredFile<512>,
}

impl DiskMonitor {
    pub fn new(path: &str) -> Result<Self, types::QosError> {
        Ok(Self {
            monitor: monitored_file::MonitoredFile::new(path)?,
        })
    }
    pub fn read_stats(&mut self) -> Result<IoStats, types::QosError> {
        let buffer = self.monitor.read_bytes_raw()?;
        if buffer.is_empty() {
            return Err(types::QosError::SystemCheckFailed("Empty diskstats".into()));
        }
        let mut stats = IoStats::default();
        let mut field_idx = 0;
        let mut cursor = 0;
        while cursor < buffer.len() {
            while cursor < buffer.len()
                && (buffer[cursor] == b' ' || buffer[cursor] == b'\t' || buffer[cursor] == b'\n')
            {
                cursor += 1;
            }
            if cursor >= buffer.len() {
                break;
            }
            let mut val: u64 = 0;
            let start = cursor;
            while cursor < buffer.len() && buffer[cursor].is_ascii_digit() {
                val = val * 10 + (buffer[cursor] - b'0') as u64;
                cursor += 1;
            }
            if cursor > start {
                match field_idx {
                    0 => stats.read_ios = val,
                    1 => stats.read_merges = val,
                    2 => stats.read_sectors = val,
                    3 => stats.read_ticks = val,
                    4 => stats.write_ios = val,
                    7 => stats.write_ticks = val,
                    8 => stats.in_flight = val,
                    _ => {}
                }
                field_idx += 1;
                if field_idx > 8 {
                    break;
                }
            } else {
                while cursor < buffer.len() && !buffer[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
            }
        }
        if field_idx > 8 {
            Ok(stats)
        } else {
            Err(types::QosError::SystemCheckFailed(
                "Incomplete diskstats".into(),
            ))
        }
    }
}
