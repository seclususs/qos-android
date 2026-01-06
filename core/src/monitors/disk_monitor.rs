//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::monitored_file::MonitoredFile;

#[derive(Debug, Clone, Copy, Default)]
pub struct IoStats {
    pub read_ios: u64,
    pub read_sectors: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
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
        let mut parts = content.split_whitespace();
        let read_ios = parts.next().and_then(|v| v.parse::<u64>().ok());
        let read_sectors = parts.next().and_then(|v| v.parse::<u64>().ok());
        let read_ticks = parts.next().and_then(|v| v.parse::<u64>().ok());
        let write_ios = parts.next().and_then(|v| v.parse::<u64>().ok());
        let write_sectors = parts.next().and_then(|v| v.parse::<u64>().ok());
        let write_ticks = parts.next().and_then(|v| v.parse::<u64>().ok());
        let in_flight = parts.next().and_then(|v| v.parse::<u64>().ok());
        if let (Some(ri), Some(rs), Some(rt), Some(wi), Some(ws), Some(wt), Some(infl)) = (
            read_ios,
            read_sectors,
            read_ticks,
            write_ios,
            write_sectors,
            write_ticks,
            in_flight,
        ) {
            Ok(IoStats {
                read_ios: ri,
                read_sectors: rs,
                read_ticks: rt,
                write_ios: wi,
                write_sectors: ws,
                write_ticks: wt,
                in_flight: infl,
            })
        } else {
            Err(QosError::SystemCheckFailed(
                "Incomplete diskstats format".to_string(),
            ))
        }
    }
}