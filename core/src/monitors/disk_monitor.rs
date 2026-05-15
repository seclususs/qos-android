//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;
use crate::utils::monitored_file;

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
    pub fn new(path: &str) -> types::Result<Self> {
        Ok(Self {
            monitor: monitored_file::MonitoredFile::new(path)?,
        })
    }

    pub fn read_stats(&mut self) -> types::Result<IoStats> {
        let buffer = self.monitor.read_bytes_raw()?;
        if buffer.is_empty() {
            return Err(types::QosError::SystemCheckFailed("Empty diskstats".into()));
        }

        let mut stats = IoStats::default();
        let mut field_idx = 0;
        let mut pos = 0;
        let len = buffer.len();

        while pos < len {
            let byte = buffer[pos];
            if byte == b' ' || byte == b'\t' || byte == b'\n' {
                pos += 1;
                continue;
            }

            let token_start = pos;
            let mut parsed_val: u64 = 0;

            while pos < len && buffer[pos].is_ascii_digit() {
                parsed_val = parsed_val * 10 + u64::from(buffer[pos] - b'0');
                pos += 1;
            }

            if pos == token_start {
                while pos < len && !buffer[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                continue;
            }

            match field_idx {
                0 => stats.read_ios = parsed_val,
                1 => stats.read_merges = parsed_val,
                2 => stats.read_sectors = parsed_val,
                3 => stats.read_ticks = parsed_val,
                4 => stats.write_ios = parsed_val,
                7 => stats.write_ticks = parsed_val,
                8 => stats.in_flight = parsed_val,
                _ => {}
            }

            field_idx += 1;
            if field_idx > 8 {
                break;
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
