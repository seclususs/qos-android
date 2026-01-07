//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::monitored_file::MonitoredFile;

pub struct ThermalSensor {
    monitor: Option<MonitoredFile<16>>,
    default_val: f32,
}

impl ThermalSensor {
    pub fn new(path: &str, default_val: f32) -> Self {
        let monitor = MonitoredFile::new(path).ok();
        Self {
            monitor,
            default_val,
        }
    }
    pub fn read(&mut self) -> f32 {
        let monitor = match self.monitor.as_mut() {
            Some(m) => m,
            None => return self.default_val,
        };
        match monitor.read_value() {
            Ok(content) => {
                let s = content.trim();
                match s.parse::<f32>() {
                    Ok(val) => {
                        let abs = val.abs();
                        if abs >= 10_000.0 {
                            val / 1000.0
                        } else if abs >= 100.0 {
                            val / 10.0
                        } else {
                            val
                        }
                    }
                    Err(_) => self.default_val,
                }
            }
            _ => self.default_val,
        }
    }
}