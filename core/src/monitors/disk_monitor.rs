//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::monitored_file::MonitoredFile;

#[derive(Debug, Clone, Copy, Default)]
pub struct IoStats {
    pub read_ios: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
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