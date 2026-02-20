//! Author: [Seclususs](https://github.com/seclususs)

use crate::utils::monitored_file;

pub struct ThermalSensor {
    monitor: Option<monitored_file::MonitoredFile<16>>,
    default_val: f32,
}

impl ThermalSensor {
    pub fn new(path: &str, default_val: f32) -> Self {
        let monitor = monitored_file::MonitoredFile::new(path).ok();
        Self {
            monitor,
            default_val,
        }
    }
    pub fn read(&mut self) -> f32 {
        let Some(monitor) = self.monitor.as_mut() else {
            return self.default_val;
        };
        match monitor.read_bytes_raw() {
            Ok(bytes) => {
                let mut val: i32 = 0;
                let mut sign = 1;
                let mut started = false;
                for &b in bytes {
                    if b.is_ascii_digit() {
                        val = val.wrapping_mul(10).wrapping_add((b - b'0') as i32);
                        started = true;
                    } else if b == b'-' {
                        if !started {
                            sign = -1;
                        }
                    } else if started {
                        break;
                    }
                }
                if !started {
                    return self.default_val;
                }
                let final_val = (val * sign) as f32;
                let abs = final_val.abs();
                if abs >= 10_000.0 {
                    final_val / 1000.0
                } else if abs >= 100.0 {
                    final_val / 10.0
                } else {
                    final_val
                }
            }
            _ => self.default_val,
        }
    }
}
