//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::filter_math;
use crate::daemon::types;
use crate::hal::monitored_file;

use std::time;

#[derive(Debug, Clone, Copy)]
pub struct PsiTrend {
    pub current: f32,
    pub avg10: f32,
    pub avg60: f32,
    pub avg300: f32,
    pub total: u64,
    pub nis: f32,
}

impl Default for PsiTrend {
    fn default() -> Self {
        Self {
            current: 0.0,
            avg10: 0.0,
            avg60: 0.0,
            avg300: 0.0,
            total: 0,
            nis: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsiData {
    pub some: PsiTrend,
    pub full: PsiTrend,
}

pub struct PsiMonitor {
    monitor: monitored_file::MonitoredFile<512>,
    last_read_time: time::Instant,
    last_some_total: u64,
    first_run: bool,
    filter_some: filter_math::KalmanFilter,
}

impl PsiMonitor {
    pub fn new(path: &str) -> Result<Self, types::QosError> {
        let monitor = monitored_file::MonitoredFile::new(path)?;
        let config = filter_math::KalmanConfig::default();
        Ok(Self {
            monitor,
            last_read_time: time::Instant::now(),
            last_some_total: 0,
            first_run: true,
            filter_some: filter_math::KalmanFilter::new(config),
        })
    }
    pub fn read_state(&mut self) -> Result<PsiData, types::QosError> {
        let content = self.monitor.read_value()?;
        if content.is_empty() {
            return Err(types::QosError::PsiParseError("Empty PSI file".to_string()));
        }
        let now = time::Instant::now();
        let elapsed_duration = now.duration_since(self.last_read_time);
        let dt_sec = if self.first_run {
            1.0
        } else {
            elapsed_duration.as_secs_f32().max(0.001)
        };
        let elapsed_micros = if self.first_run {
            1_000_000.0
        } else {
            elapsed_duration.as_micros() as f32
        };
        let dt_calc = elapsed_micros.max(1000.0);
        let mut data = PsiData {
            some: PsiTrend::default(),
            full: PsiTrend::default(),
        };
        for line in content.lines() {
            let is_some = line.starts_with("some ");
            let is_full = line.starts_with("full ");
            if !is_some && !is_full {
                continue;
            }
            let target = if is_some {
                &mut data.some
            } else {
                &mut data.full
            };
            for token in line.split_ascii_whitespace() {
                if let Some(v) = token.strip_prefix("avg10=") {
                    target.avg10 = v.parse::<f32>().unwrap_or(0.0);
                } else if is_some {
                    if let Some(v) = token.strip_prefix("avg60=") {
                        target.avg60 = v.parse::<f32>().unwrap_or(0.0);
                    } else if let Some(v) = token.strip_prefix("avg300=") {
                        target.avg300 = v.parse::<f32>().unwrap_or(0.0);
                    } else if let Some(v) = token.strip_prefix("total=") {
                        target.total = v.parse::<u64>().unwrap_or(0);
                    }
                }
            }
        }
        if !self.first_run {
            let delta_some = data.some.total.saturating_sub(self.last_some_total) as f32;
            let raw_some = delta_some / dt_calc * 100.0;
            data.some.current = self.filter_some.update(raw_some, dt_sec);
            data.some.nis = self.filter_some.get_last_nis();
        } else {
            data.some.current = data.some.avg10;
            self.filter_some.reset();
            self.filter_some.update(data.some.avg10, 1.0);
            self.first_run = false;
        }
        self.last_read_time = now;
        self.last_some_total = data.some.total;
        Ok(data)
    }
}
