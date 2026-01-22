//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;
use crate::hal::monitored_file;

#[derive(Debug, Clone, Copy, Default)]
pub struct VmStats {
    pub pgscan: u64,
    pub pgsteal: u64,
    pub workingset_refault: u64,
    pub nr_active_anon: u64,
    pub nr_inactive_anon: u64,
    pub nr_active_file: u64,
    pub nr_inactive_file: u64,
}

pub struct VmMonitor {
    vmstat_monitor: monitored_file::MonitoredFile<8192>,
}

impl VmMonitor {
    pub fn new(vmstat_path: &str) -> Result<Self, types::QosError> {
        Ok(Self {
            vmstat_monitor: monitored_file::MonitoredFile::new(vmstat_path)?,
        })
    }
    pub fn read_stats(&mut self) -> Result<VmStats, types::QosError> {
        let mut stats = VmStats::default();
        if let Ok(content) = self.vmstat_monitor.read_value() {
            for line in content.lines() {
                let mut parts = line.split_ascii_whitespace();
                if let Some(key) = parts.next() {
                    match key {
                        "pgscan_direct" | "pgscan_kswapd" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.pgscan = stats.pgscan.saturating_add(val);
                            }
                        }
                        "pgsteal_direct" | "pgsteal_kswapd" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.pgsteal = stats.pgsteal.saturating_add(val);
                            }
                        }
                        "workingset_refault" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.workingset_refault = val;
                            }
                        }
                        "nr_active_anon" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.nr_active_anon = val;
                            }
                        }
                        "nr_inactive_anon" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.nr_inactive_anon = val;
                            }
                        }
                        "nr_active_file" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.nr_active_file = val;
                            }
                        }
                        "nr_inactive_file" => {
                            if let Some(val_str) = parts.next()
                                && let Ok(val) = val_str.parse::<u64>()
                            {
                                stats.nr_inactive_file = val;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(stats)
    }
}
