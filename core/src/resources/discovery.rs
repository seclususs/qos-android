//! Author: [Seclususs](https://github.com/seclususs)

use std::{collections, fs, path, sync};

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
            if file_name.starts_with("thermal_zone")
                && let Ok(content) = fs::read_to_string(path.join("type"))
            {
                let type_name = content.trim().to_string();
                zones_map.insert(type_name, file_name.to_string());
            }
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
