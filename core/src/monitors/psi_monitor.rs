//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::kalman_math::{KalmanConfig, KalmanFilter};
use crate::daemon::types::QosError;
use crate::hal::monitored_file::MonitoredFile;

use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct PsiTrend {
    pub current: f64,
    pub avg10: f64,
    pub avg60: f64,
    pub avg300: f64,
    pub total: u64,
    pub nis: f64,
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
    monitor: MonitoredFile<512>,
    last_read_time: Instant,
    last_some_total: u64,
    last_full_total: u64,
    first_run: bool,
    filter_some: KalmanFilter,
    filter_full: KalmanFilter,
}

impl PsiMonitor {
    pub fn new(path: &str) -> Result<Self, QosError> {
        let monitor = MonitoredFile::new(path)?;
        let config = KalmanConfig {
            q_base: 2.0,
            r_base: 10.0,
            fading_factor: 1.05,
            window_size: 10,
        };
        Ok(Self {
            monitor,
            last_read_time: Instant::now(),
            last_some_total: 0,
            last_full_total: 0,
            first_run: true,
            filter_some: KalmanFilter::new(config),
            filter_full: KalmanFilter::new(config),
        })
    }
    pub fn read_state(&mut self) -> Result<PsiData, QosError> {
        let content = self.monitor.read_value()?;
        if content.is_empty() {
            return Err(QosError::PsiParseError("Empty PSI file".to_string()));
        }
        let now = Instant::now();
        let elapsed_duration = now.duration_since(self.last_read_time);
        let dt_sec = if self.first_run {
            1.0
        } else {
            elapsed_duration.as_secs_f64().max(0.001)
        };
        let elapsed_micros = if self.first_run {
            1_000_000.0
        } else {
            elapsed_duration.as_micros() as f64
        };
        let dt_calc = if elapsed_micros < 1000.0 {
            1000.0
        } else {
            elapsed_micros
        };
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
            for token in line.split_whitespace() {
                if let Some(v) = token.strip_prefix("avg10=") {
                    target.avg10 = v.parse::<f64>().unwrap_or(0.0);
                } else if let Some(v) = token.strip_prefix("avg60=") {
                    target.avg60 = v.parse::<f64>().unwrap_or(0.0);
                } else if let Some(v) = token.strip_prefix("avg300=") {
                    target.avg300 = v.parse::<f64>().unwrap_or(0.0);
                } else if let Some(v) = token.strip_prefix("total=") {
                    target.total = v.parse::<u64>().unwrap_or(0);
                }
            }
        }
        if !self.first_run {
            let delta_some = data.some.total.saturating_sub(self.last_some_total) as f64;
            let delta_full = data.full.total.saturating_sub(self.last_full_total) as f64;
            let raw_some = delta_some / dt_calc * 100.0;
            let raw_full = delta_full / dt_calc * 100.0;
            data.some.current = self.filter_some.update(raw_some, dt_sec);
            data.some.nis = self.filter_some.get_last_nis();
            data.full.current = self.filter_full.update(raw_full, dt_sec);
            data.full.nis = self.filter_full.get_last_nis();
        } else {
            data.some.current = data.some.avg10;
            data.full.current = data.full.avg10;
            self.filter_some.reset();
            self.filter_full.reset();
            self.filter_some.update(data.some.avg10, 1.0);
            self.filter_full.update(data.full.avg10, 1.0);
            self.first_run = false;
        }
        self.last_read_time = now;
        self.last_some_total = data.some.total;
        self.last_full_total = data.full.total;
        Ok(data)
    }
}