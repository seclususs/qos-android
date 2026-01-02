//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::kalman_math::{KalmanConfig, KalmanFilter};
use crate::daemon::types::QosError;
use crate::hal::filesystem::open_file_for_read;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::Instant;

const BUFFER_SIZE: usize = 512;

#[derive(Debug, Clone, Copy)]
pub struct PsiTrend {
    pub current: f64,
    pub avg10: f64,
    pub avg60: f64,
    pub avg300: f64,
    pub total: u64,
}

impl Default for PsiTrend {
    fn default() -> Self {
        Self {
            current: 0.0,
            avg10: 0.0,
            avg60: 0.0,
            avg300: 0.0,
            total: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsiData {
    pub some: PsiTrend,
    pub full: PsiTrend,
}

pub struct PsiMonitor {
    file: File,
    buffer: [u8; BUFFER_SIZE],
    last_read_time: Instant,
    last_some_total: u64,
    last_full_total: u64,
    first_run: bool,
    filter_some: KalmanFilter,
    filter_full: KalmanFilter,
}

impl PsiMonitor {
    pub fn new(path: &str) -> Result<Self, QosError> {
        let file = open_file_for_read(path)?;
        let config = KalmanConfig {
            q_base: 2.0,
            r_base: 10.0,
            fading_factor: 1.05,
        };
        Ok(Self {
            file,
            buffer: [0u8; BUFFER_SIZE],
            last_read_time: Instant::now(),
            last_some_total: 0,
            last_full_total: 0,
            first_run: true,
            filter_some: KalmanFilter::new(config),
            filter_full: KalmanFilter::new(config),
        })
    }
    pub fn read_state(&mut self) -> Result<PsiData, QosError> {
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(QosError::IoError)?;
        let bytes_read = match self.file.read(&mut self.buffer) {
            Ok(n) => n,
            Err(e) => return Err(QosError::IoError(e)),
        };
        if bytes_read == 0 {
            return Err(QosError::PsiParseError("Empty PSI file".to_string()));
        }
        let content = std::str::from_utf8(&self.buffer[..bytes_read])
            .map_err(|_| QosError::PsiParseError("Invalid UTF-8".to_string()))?;
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
            data.full.current = self.filter_full.update(raw_full, dt_sec);
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