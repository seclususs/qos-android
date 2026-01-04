//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::monitored_file::MonitoredFile;

pub struct BatterySensor {
    monitor: Option<MonitoredFile<16>>,
}

impl BatterySensor {
    pub fn new(path: &str) -> Self {
        let monitor = MonitoredFile::new(path).ok();
        Self { monitor }
    }
    pub fn read(&mut self) -> f64 {
        if let Some(ref mut monitor) = self.monitor {
            if let Ok(content) = monitor.read_value() {
                return content.trim().parse::<f64>().unwrap_or(100.0);
            }
        }
        100.0
    }
}