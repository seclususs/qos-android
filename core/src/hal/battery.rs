//! Author: [Seclususs](https://github.com/seclususs)

use crate::utils::monitored_file;

pub struct BatterySensor {
    monitor: Option<monitored_file::MonitoredFile<16>>,
}

impl BatterySensor {
    pub fn new(path: &str) -> Self {
        let monitor = monitored_file::MonitoredFile::new(path).ok();
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
