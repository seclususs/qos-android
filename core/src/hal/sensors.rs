//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::sysfs;

const TEMP_SCALE_MILLI_THRESHOLD: f32 = 10_000.0;
const TEMP_SCALE_DECI_THRESHOLD: f32 = 100.0;
const SCALE_MULTIPLIER_MILLI: f32 = 0.001;
const SCALE_MULTIPLIER_DECI: f32 = 0.1;

pub struct ThermalSensor {
    monitor: Option<sysfs::MonitoredFile<16>>,
    default_val: f32,
}

impl ThermalSensor {
    pub fn new(path: &str, default_val: f32) -> Self {
        let monitor = sysfs::MonitoredFile::new(path).ok();
        Self {
            monitor,
            default_val,
        }
    }

    pub fn read(&mut self) -> f32 {
        let Some(monitor) = self.monitor.as_mut() else {
            return self.default_val;
        };

        let Ok(bytes) = monitor.read_bytes_raw() else {
            return self.default_val;
        };

        let mut parsed_val: i32 = 0;
        let mut sign = 1;
        let mut has_digits = false;

        for &byte in bytes {
            if byte.is_ascii_digit() {
                parsed_val = parsed_val
                    .wrapping_mul(10)
                    .wrapping_add(i32::from(byte - b'0'));

                has_digits = true;
                continue;
            }

            if byte == b'-' && !has_digits {
                sign = -1;
                continue;
            }

            if has_digits {
                break;
            }
        }

        if !has_digits {
            return self.default_val;
        }

        let final_temp = (parsed_val * sign) as f32;
        let abs_temp = final_temp.abs();

        if abs_temp >= TEMP_SCALE_MILLI_THRESHOLD {
            final_temp * SCALE_MULTIPLIER_MILLI
        } else if abs_temp >= TEMP_SCALE_DECI_THRESHOLD {
            final_temp * SCALE_MULTIPLIER_DECI
        } else {
            final_temp
        }
    }
}

pub struct BatterySensor {
    monitor: Option<sysfs::MonitoredFile<16>>,
}

impl BatterySensor {
    pub fn new(path: &str) -> Self {
        let monitor = sysfs::MonitoredFile::new(path).ok();
        Self { monitor }
    }

    pub fn read(&mut self) -> f32 {
        self.monitor
            .as_mut()
            .and_then(|m| m.read_value().ok())
            .and_then(|content| content.trim().parse::<f32>().ok())
            .unwrap_or(100.0)
    }
}
