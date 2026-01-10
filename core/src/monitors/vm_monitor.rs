//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::monitored_file::MonitoredFile;

#[derive(Debug, Clone, Copy, Default)]
pub struct VmStats {
    pub pgscan: u64,
    pub pgsteal: u64,
    pub workingset_refault: u64,
    pub fragmentation_index: f32,
    pub nr_active_anon: u64,
    pub nr_inactive_anon: u64,
    pub nr_active_file: u64,
    pub nr_inactive_file: u64,
}

pub struct VmMonitor {
    vmstat_monitor: MonitoredFile<8192>,
    buddy_monitor: MonitoredFile<1024>,
}

impl VmMonitor {
    pub fn new(vmstat_path: &str, buddy_path: &str) -> Result<Self, QosError> {
        Ok(Self {
            vmstat_monitor: MonitoredFile::new(vmstat_path)?,
            buddy_monitor: MonitoredFile::new(buddy_path)?,
        })
    }
    pub fn read_stats(&mut self) -> Result<VmStats, QosError> {
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
        if let Ok(content) = self.buddy_monitor.read_value() {
            let mut total_free_pages = 0u64;
            let mut huge_pages = 0u64;
            for line in content.lines() {
                if let Some(pos) = line.find("Normal") {
                    let numbers_part = &line[pos + 6..];
                    for (order, count_str) in numbers_part.split_ascii_whitespace().enumerate() {
                        let count = count_str.parse::<u64>().unwrap_or(0);
                        let pages = count * (1 << order);
                        total_free_pages += pages;
                        if order >= 9 {
                            huge_pages += pages;
                        }
                    }
                }
            }
            if total_free_pages > 0 {
                stats.fragmentation_index = 1.0 - (huge_pages as f32 / total_free_pages as f32);
            }
        }
        Ok(stats)
    }
}