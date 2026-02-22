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
    let mut buf = [0u8; 128];
    let prefix = b"/sys/devices/system/cpu/cpu";
    let suffix1 = b"/cpufreq/cpuinfo_max_freq";
    let suffix2 = b"/cpufreq/scaling_max_freq";
    for i in 0..16 {
        let mut len = prefix.len();
        buf[..len].copy_from_slice(prefix);
        let mut itoa_buf = itoa::Buffer::new();
        let num_bytes = itoa_buf.format(i).as_bytes();
        buf[len..len + num_bytes.len()].copy_from_slice(num_bytes);
        len += num_bytes.len();
        let path_info_len = len + suffix1.len();
        buf[len..path_info_len].copy_from_slice(suffix1);
        let path_info = unsafe { std::str::from_utf8_unchecked(&buf[..path_info_len]) };
        let mut content = fs::read_to_string(path_info);
        if content.is_err() {
            let path_scaling_len = len + suffix2.len();
            buf[len..path_scaling_len].copy_from_slice(suffix2);
            let path_scaling = unsafe { std::str::from_utf8_unchecked(&buf[..path_scaling_len]) };
            content = fs::read_to_string(path_scaling);
        }
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
