//! Author: [Seclususs](https://github.com/seclususs)

use crate::resources::sys_paths::K_BATTERY_TEMP_PATH;
use std::fs;

pub fn read_initial_thermal_state() -> f64 {
    if let Ok(content) = fs::read_to_string(K_BATTERY_TEMP_PATH) {
        if let Ok(val_raw) = content.trim().parse::<f64>() {
            let abs_val = val_raw.abs();
            let normalized_val = if abs_val >= 1000.0 {
                val_raw / 1000.0
            } else if abs_val >= 150.0 {
                val_raw / 10.0
            } else {
                val_raw
            };
            if normalized_val > -30.0 && normalized_val < 120.0 {
                return normalized_val;
            }
        }
    }
    34.0 
}