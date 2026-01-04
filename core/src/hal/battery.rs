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
        self.monitor
            .as_mut()
            .and_then(|m| m.read_value().ok())
            .and_then(|content| content.trim().parse::<f64>().ok())
            .unwrap_or(100.0)
    }
}