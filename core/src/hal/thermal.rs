//! Author: [Seclususs](https://github.com/seclususs)

use crate::resources::sys_paths::{K_BATTERY_TEMP_PATH, K_THERMAL_ZONE0_PATH};
use std::fs;

pub fn read_initial_thermal_state() -> f64 {
    let paths = [K_BATTERY_TEMP_PATH, K_THERMAL_ZONE0_PATH];
    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(val_raw) = content.trim().parse::<f64>() {
                let normalized_val = if val_raw > 1000.0 {
                    val_raw / 1000.0
                } else if val_raw > 150.0 {
                    val_raw / 10.0
                } else {
                    val_raw
                };
                if normalized_val > -20.0 && normalized_val < 120.0 {
                    return normalized_val;
                }
            }
        }
    }
    34.0 
}