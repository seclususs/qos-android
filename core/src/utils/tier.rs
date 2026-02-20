//! Author: [Seclususs](https://github.com/seclususs)

use std::{fs, sync};

static CURRENT_TIER: sync::OnceLock<DeviceTier> = sync::OnceLock::new();

const FREQ_FLAGSHIP_MIN: u64 = 2_800_000;
const FREQ_BIG_CORE_MIN: u64 = 2_100_000;
const FREQ_DUAL_CLUSTER_MID: u64 = 2_450_000;
const RAM_FLAGSHIP_MIN: u64 = 5_500;
const RAM_MID_REQ: u64 = 3_800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceTier {
    LowEnd,
    MidRange,
    Flagship,
}

impl DeviceTier {
    #[inline]
    pub fn get() -> Self {
        *CURRENT_TIER.get_or_init(detect_hardware_capabilities)
    }
}

struct CpuStats {
    max_freq_khz: u64,
    big_cores_count: u8,
}

fn detect_hardware_capabilities() -> DeviceTier {
    let stats = get_cpu_stats();
    let total_ram_mb = get_total_ram_mb();
    if stats.max_freq_khz >= FREQ_FLAGSHIP_MIN && total_ram_mb >= RAM_FLAGSHIP_MIN {
        return DeviceTier::Flagship;
    }
    if stats.big_cores_count >= 4 && total_ram_mb >= RAM_MID_REQ {
        return DeviceTier::MidRange;
    }
    if stats.max_freq_khz >= FREQ_DUAL_CLUSTER_MID && total_ram_mb >= RAM_MID_REQ {
        return DeviceTier::MidRange;
    }
    DeviceTier::LowEnd
}

fn get_cpu_stats() -> CpuStats {
    let mut max_freq = 0;
    let mut big_cores = 0;
    for i in 0..16 {
        let path_info = format!("/sys/devices/system/cpu/cpu{i}/cpufreq/cpuinfo_max_freq");
        let path_scaling = format!("/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_max_freq");
        let content = fs::read_to_string(&path_info).or_else(|_| fs::read_to_string(&path_scaling));
        if let Ok(val_str) = content {
            if let Ok(freq) = val_str.trim().parse::<u64>() {
                if freq > max_freq {
                    max_freq = freq;
                }
                if freq >= FREQ_BIG_CORE_MIN {
                    big_cores += 1;
                }
            }
        } else if i >= 8 {
            break;
        }
    }
    if max_freq == 0 {
        max_freq = 2_000_000;
    }
    CpuStats {
        max_freq_khz: max_freq,
        big_cores_count: big_cores,
    }
}

fn get_total_ram_mb() -> u64 {
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb = parts[1].parse::<u64>().unwrap_or(0);
                    return kb / 1024;
                }
            }
        }
    }
    3072
}
