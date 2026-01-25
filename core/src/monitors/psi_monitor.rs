//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::filter_math;
use crate::daemon::types;
use crate::utils::monitored_file;

use std::time;

#[derive(Debug, Clone, Copy)]
pub struct PsiTrend {
    pub current: f32,
    pub velocity: f32,
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
            velocity: 0.0,
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
    fn parse_f32_bytes(buffer: &[u8], start: usize) -> (f32, usize) {
        let mut idx = start;
        let mut val = 0.0;
        let mut fraction = 0.0;
        let mut divisor = 1.0;
        let mut in_fraction = false;
        while idx < buffer.len() {
            let b = buffer[idx];
            if b.is_ascii_digit() {
                if !in_fraction {
                    val = val * 10.0 + (b - b'0') as f32;
                } else {
                    fraction = fraction * 10.0 + (b - b'0') as f32;
                    divisor *= 10.0;
                }
            } else if b == b'.' {
                in_fraction = true;
            } else {
                break;
            }
            idx += 1;
        }
        (val + (fraction / divisor), idx)
    }
    fn parse_u64_bytes(buffer: &[u8], start: usize) -> (u64, usize) {
        let mut idx = start;
        let mut val = 0;
        while idx < buffer.len() {
            let b = buffer[idx];
            if b.is_ascii_digit() {
                val = val * 10 + (b - b'0') as u64;
                idx += 1;
            } else {
                break;
            }
        }
        (val, idx)
    }
    pub fn read_state(&mut self) -> Result<PsiData, types::QosError> {
        let buffer = self.monitor.read_bytes_raw()?;
        if buffer.is_empty() {
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
        let mut some_trend = PsiTrend::default();
        let mut cursor = 0;
        let len = buffer.len();
        while cursor < len {
            if cursor + 5 < len && &buffer[cursor..cursor + 5] == b"some " {
                cursor += 5;
                while cursor < len && buffer[cursor] != b'\n' {
                    while cursor < len && buffer[cursor] == b' ' {
                        cursor += 1;
                    }
                    if cursor + 6 < len && &buffer[cursor..cursor + 6] == b"avg10=" {
                        let (val, next) = Self::parse_f32_bytes(buffer, cursor + 6);
                        some_trend.avg10 = val;
                        cursor = next;
                    } else if cursor + 6 < len && &buffer[cursor..cursor + 6] == b"avg60=" {
                        let (val, next) = Self::parse_f32_bytes(buffer, cursor + 6);
                        some_trend.avg60 = val;
                        cursor = next;
                    } else if cursor + 7 < len && &buffer[cursor..cursor + 7] == b"avg300=" {
                        let (val, next) = Self::parse_f32_bytes(buffer, cursor + 7);
                        some_trend.avg300 = val;
                        cursor = next;
                    } else if cursor + 6 < len && &buffer[cursor..cursor + 6] == b"total=" {
                        let (val, next) = Self::parse_u64_bytes(buffer, cursor + 6);
                        some_trend.total = val;
                        cursor = next;
                    } else {
                        while cursor < len && buffer[cursor] != b' ' && buffer[cursor] != b'\n' {
                            cursor += 1;
                        }
                    }
                }
                break;
            } else {
                while cursor < len && buffer[cursor] != b'\n' {
                    cursor += 1;
                }
                if cursor < len {
                    cursor += 1;
                }
            }
        }
        if !self.first_run {
            let delta_some = some_trend.total.saturating_sub(self.last_some_total) as f32;
            let raw_some = delta_some / dt_calc * 100.0;
            some_trend.current = self.filter_some.update(raw_some, dt_sec);
            some_trend.velocity = self.filter_some.get_velocity();
            some_trend.nis = self.filter_some.get_last_nis();
        } else {
            some_trend.current = some_trend.avg10;
            some_trend.velocity = 0.0;
            self.filter_some.reset();
            self.filter_some.update(some_trend.avg10, 1.0);
            self.first_run = false;
        }
        self.last_read_time = now;
        self.last_some_total = some_trend.total;
        Ok(PsiData { some: some_trend })
    }
}
