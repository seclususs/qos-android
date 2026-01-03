//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;
use crate::hal::monitored_file::MonitoredFile;

#[derive(Debug, Clone, Copy, Default)]
pub struct VmStats {
    pub pgscan: u64,
    pub pgsteal: u64,
    pub pswpout: u64,
    pub workingset_refault: u64,
    pub fragmentation_index: f64,
}

pub struct VmMonitor {
    vmstat_monitor: MonitoredFile<8192>,
    buddy_monitor: MonitoredFile<8192>,
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
            let mut scan_direct = 0;
            let mut scan_kswapd = 0;
            let mut steal_direct = 0;
            let mut steal_kswapd = 0;
            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
                    let val = val_str.parse::<u64>().unwrap_or(0);
                    match key {
                        "pgscan_direct" => scan_direct = val,
                        "pgscan_kswapd" => scan_kswapd = val,
                        "pgsteal_direct" => steal_direct = val,
                        "pgsteal_kswapd" => steal_kswapd = val,
                        "workingset_refault" => stats.workingset_refault = val,
                        "pswpout" => stats.pswpout = val,
                        _ => {}
                    }
                }
            }
            stats.pgscan = scan_direct + scan_kswapd;
            stats.pgsteal = steal_direct + steal_kswapd;
        }
        if let Ok(content) = self.buddy_monitor.read_value() {
            let mut total_free_pages = 0u64;
            let mut huge_pages = 0u64;
            for line in content.lines() {
                if let Some(pos) = line.find("Normal") {
                    let numbers_part = &line[pos + 6..];
                    for (order, count_str) in numbers_part.split_whitespace().enumerate() {
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
                stats.fragmentation_index = 1.0 - (huge_pages as f64 / total_free_pages as f64);
            }
        }
        Ok(stats)
    }
}