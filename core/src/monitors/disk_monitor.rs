//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::monitored_file::MonitoredFile;

#[derive(Debug, Clone, Copy, Default)]
pub struct IoStats {
    pub read_ios: u64,
    pub read_merges: u64,
    pub read_sectors: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
    pub write_merges: u64,
    pub write_sectors: u64,
    pub write_ticks: u64,
    pub in_flight: u64,
}

pub struct DiskMonitor {
    monitor: MonitoredFile<512>,
}

impl DiskMonitor {
    pub fn new(path: &str) -> Result<Self, QosError> {
        Ok(Self {
            monitor: MonitoredFile::new(path)?,
        })
    }
    pub fn read_stats(&mut self) -> Result<IoStats, QosError> {
        let content = self.monitor.read_value()?;
        if content.is_empty() {
            return Err(QosError::SystemCheckFailed(
                "Empty diskstats file".to_string(),
            ));
        }
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() < 11 {
            return Err(QosError::SystemCheckFailed(
                "Incomplete diskstats format".to_string(),
            ));
        }
        Ok(IoStats {
            read_ios: parts[0].parse::<u64>().unwrap_or(0),
            read_merges: parts[1].parse::<u64>().unwrap_or(0),
            read_sectors: parts[2].parse::<u64>().unwrap_or(0),
            read_ticks: parts[3].parse::<u64>().unwrap_or(0),
            write_ios: parts[4].parse::<u64>().unwrap_or(0),
            write_merges: parts[5].parse::<u64>().unwrap_or(0),
            write_sectors: parts[6].parse::<u64>().unwrap_or(0),
            write_ticks: parts[7].parse::<u64>().unwrap_or(0),
            in_flight: parts[8].parse::<u64>().unwrap_or(0),
        })
    }
}