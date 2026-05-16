//! Author: [Seclususs](https://github.com/seclususs)

use std::{collections, fs, path, sync};

static CURRENT_TIER: sync::OnceLock<DeviceTier> = sync::OnceLock::new();

const FREQ_FLAGSHIP_MIN: u64 = 2_800_000;
const FREQ_BIG_CORE_MIN: u64 = 2_100_000;
const FREQ_DUAL_CLUSTER_MID: u64 = 2_450_000;
const RAM_FLAGSHIP_MIN: u64 = 5_500;
const RAM_MID_REQ: u64 = 3_800;

const DEFAULT_MAX_FREQ: u64 = 2_000_000;
const MAX_CPUS: u64 = 16;
const FALLBACK_BREAK: u64 = 8;

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

    for i in 0..MAX_CPUS {
        let mut len = prefix.len();
        buf[..len].copy_from_slice(prefix);
        let mut itoa_buf = itoa::Buffer::new();
        let num_bytes = itoa_buf.format(i).as_bytes();
        buf[len..len + num_bytes.len()].copy_from_slice(num_bytes);
        len += num_bytes.len();

        let path_info_len = len + suffix1.len();
        buf[len..path_info_len].copy_from_slice(suffix1);
        let path_info = unsafe { std::str::from_utf8_unchecked(&buf[..path_info_len]) };

        let content = fs::read_to_string(path_info).or_else(|_| {
            let path_scaling_len = len + suffix2.len();
            buf[len..path_scaling_len].copy_from_slice(suffix2);
            let path_scaling = unsafe { std::str::from_utf8_unchecked(&buf[..path_scaling_len]) };
            fs::read_to_string(path_scaling)
        });

        let Ok(val_str) = content else {
            if i >= FALLBACK_BREAK {
                break;
            }
            continue;
        };

        let Ok(freq) = val_str.trim().parse::<u64>() else {
            continue;
        };

        if freq > max_freq {
            max_freq = freq;
        }

        if freq >= FREQ_BIG_CORE_MIN {
            big_cores += 1;
        }
    }

    if max_freq == 0 {
        max_freq = DEFAULT_MAX_FREQ;
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

static STORAGE_DEV: sync::OnceLock<String> = sync::OnceLock::new();
static READ_AHEAD_PATH: sync::OnceLock<path::PathBuf> = sync::OnceLock::new();
static NR_REQUESTS_PATH: sync::OnceLock<path::PathBuf> = sync::OnceLock::new();
static DISKSTATS_PATH: sync::OnceLock<path::PathBuf> = sync::OnceLock::new();
static CPU_ZONE_PATH: sync::OnceLock<path::PathBuf> = sync::OnceLock::new();

const THERMAL_PRIORITY_LIST: &[&str] = &[
    "cpu-1-0-usr",
    "cpu-1-1-usr",
    "cpu-1-2-usr",
    "cpu-1-3-usr",
    "cpu-0-0-usr",
    "cpu-0-1-usr",
    "big-core",
    "mid-core",
    "little-core",
    "cpu0_thermal",
    "cpu1_thermal",
    "mtktscpu",
    "mtk_ts_cpu",
    "mtkts_cpu",
    "thermal-cpuss-0",
    "thermal-cpuss-1",
    "exynos_thermal",
    "exynos_dev_thermal",
    "hisi_thermal",
    "mtktsAP",
    "mtk_ts_ap",
    "ap_cdev",
    "ap_thermal",
    "soc_thermal",
    "soc-thermal",
    "cpu_thermal",
    "cpu-thermal",
    "cpu",
    "tsens_tz_sensor10",
    "tsens_tz_sensor5",
    "tsens_tz_sensor0",
];

const THERMAL_BLACKLIST: &[&str] = &[
    "battery",
    "bms",
    "bat",
    "charger",
    "usb",
    "pa_therm",
    "pa-therm",
    "modem",
    "wifi",
    "wlan",
    "gpu",
    "camera",
    "flash",
    "led",
    "pmic",
    "buck",
    "ldo",
    "xo_therm",
    "quiet",
    "backlight",
];

pub(crate) fn get_storage_name() -> &'static str {
    STORAGE_DEV.get_or_init(detect_storage_device)
}

pub fn get_read_ahead_path() -> &'static path::Path {
    READ_AHEAD_PATH.get_or_init(|| {
        path::PathBuf::from(format!(
            "/sys/block/{}/queue/read_ahead_kb",
            get_storage_name()
        ))
    })
}

pub fn get_nr_requests_path() -> &'static path::Path {
    NR_REQUESTS_PATH.get_or_init(|| {
        path::PathBuf::from(format!(
            "/sys/block/{}/queue/nr_requests",
            get_storage_name()
        ))
    })
}

pub fn get_diskstats_path() -> &'static path::Path {
    DISKSTATS_PATH
        .get_or_init(|| path::PathBuf::from(format!("/sys/block/{}/stat", get_storage_name())))
}

pub fn get_cpu_temp_path() -> &'static path::Path {
    CPU_ZONE_PATH.get_or_init(detect_cpu_thermal_path)
}

fn detect_storage_device() -> String {
    let candidates = ["nvme0n1", "sda", "sdb", "mmcblk0"];
    for &dev in &candidates {
        if path::Path::new("/sys/block").join(dev).exists() {
            return dev.to_string();
        }
    }
    "mmcblk0".to_string()
}

fn detect_cpu_thermal_path() -> path::PathBuf {
    let base_dir = path::Path::new("/sys/class/thermal");
    let mut zones_map: collections::HashMap<String, String> = collections::HashMap::new();

    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            if !file_name.starts_with("thermal_zone") {
                continue;
            }

            let Ok(content) = fs::read_to_string(path.join("type")) else {
                continue;
            };

            let type_name = content.trim().to_string();
            zones_map.insert(type_name, file_name.to_string());
        }
    }

    for &target in THERMAL_PRIORITY_LIST {
        if let Some(filename) = zones_map.get(target) {
            return base_dir.join(filename).join("temp");
        }

        if let Some((_, filename)) = zones_map
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(target))
        {
            return base_dir.join(filename).join("temp");
        }
    }

    for (type_name, filename) in &zones_map {
        let name_lower = type_name.to_lowercase();
        let looks_like_cpu = name_lower.contains("cpu")
            || name_lower.contains("soc")
            || name_lower.contains("cluster")
            || name_lower.contains("ap");

        let is_safe = !THERMAL_BLACKLIST.iter().any(|&b| name_lower.contains(b));

        if looks_like_cpu && is_safe {
            return base_dir.join(filename).join("temp");
        }
    }

    base_dir.join("thermal_zone3").join("temp")
}
