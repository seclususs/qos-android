//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::filter;
use crate::daemon::types;
use crate::hal::sysfs;

use std::time;

#[derive(Debug, Clone, Copy, Default)]
pub struct IoStats {
    pub read_ios: u64,
    pub read_merges: u64,
    pub read_sectors: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
    pub write_ticks: u64,
    pub in_flight: u64,
}

pub struct DiskMonitor {
    monitor: sysfs::MonitoredFile<512>,
}

impl DiskMonitor {
    pub fn new(path: &str) -> types::Result<Self> {
        Ok(Self {
            monitor: sysfs::MonitoredFile::new(path)?,
        })
    }

    pub fn read_stats(&mut self) -> types::Result<IoStats> {
        let buffer = self.monitor.read_bytes_raw()?;
        if buffer.is_empty() {
            return Err(types::QosError::SystemCheckFailed("Empty diskstats".into()));
        }

        let mut stats = IoStats::default();
        let mut field_idx = 0;
        let mut pos = 0;
        let len = buffer.len();

        while pos < len {
            let byte = buffer[pos];
            if byte == b' ' || byte == b'\t' || byte == b'\n' {
                pos += 1;
                continue;
            }

            let token_start = pos;
            let mut parsed_val: u64 = 0;

            while pos < len && buffer[pos].is_ascii_digit() {
                parsed_val = parsed_val * 10 + u64::from(buffer[pos] - b'0');
                pos += 1;
            }

            if pos == token_start {
                while pos < len && !buffer[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                continue;
            }

            match field_idx {
                0 => stats.read_ios = parsed_val,
                1 => stats.read_merges = parsed_val,
                2 => stats.read_sectors = parsed_val,
                3 => stats.read_ticks = parsed_val,
                4 => stats.write_ios = parsed_val,
                7 => stats.write_ticks = parsed_val,
                8 => stats.in_flight = parsed_val,
                _ => {}
            }

            field_idx += 1;
            if field_idx > 8 {
                break;
            }
        }

        if field_idx > 8 {
            Ok(stats)
        } else {
            Err(types::QosError::SystemCheckFailed(
                "Incomplete diskstats".into(),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsiTrend {
    pub current: f32,
    pub velocity: f32,
    pub avg10: f32,
    pub avg300: f32,
    pub nis: f32,
}

impl Default for PsiTrend {
    fn default() -> Self {
        Self {
            current: 0.0,
            velocity: 0.0,
            avg10: 0.0,
            avg300: 0.0,
            nis: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsiData {
    pub some: PsiTrend,
}

pub struct PsiMonitor {
    monitor: sysfs::MonitoredFile<512>,
    last_read_time: time::Instant,
    last_some_total: u64,
    first_run: bool,
    filter_some: filter::KalmanFilter,
}

impl PsiMonitor {
    pub fn new(path: &str) -> types::Result<Self> {
        let monitor = sysfs::MonitoredFile::new(path)?;
        let config = filter::KalmanConfig::default();

        Ok(Self {
            monitor,
            last_read_time: time::Instant::now(),
            last_some_total: 0,
            first_run: true,
            filter_some: filter::KalmanFilter::new(config),
        })
    }

    #[inline]
    fn parse_f32_bytes(buffer: &[u8], start_pos: usize) -> (f32, usize) {
        let mut pos = start_pos;
        let mut parsed_val = 0.0;
        let mut fraction_val = 0.0;
        let mut fraction_divisor = 1.0;
        let mut in_fraction = false;

        let len = buffer.len();

        while pos < len {
            let byte = buffer[pos];

            if byte.is_ascii_digit() {
                let digit = f32::from(byte - b'0');

                if in_fraction {
                    fraction_val = fraction_val * 10.0 + digit;
                    fraction_divisor *= 10.0;
                } else {
                    parsed_val = parsed_val * 10.0 + digit;
                }

                pos += 1;
                continue;
            }

            if byte == b'.' {
                in_fraction = true;
                pos += 1;
                continue;
            }

            break;
        }

        (parsed_val + (fraction_val / fraction_divisor), pos)
    }

    #[inline]
    fn parse_u64_bytes(buffer: &[u8], start_pos: usize) -> (u64, usize) {
        let mut pos = start_pos;
        let mut parsed_val = 0;

        let len = buffer.len();

        while pos < len {
            let byte = buffer[pos];

            if byte.is_ascii_digit() {
                parsed_val = parsed_val * 10 + u64::from(byte - b'0');
                pos += 1;
                continue;
            }
            break;
        }

        (parsed_val, pos)
    }

    pub fn read_state(&mut self) -> types::Result<PsiData> {
        let buffer = self.monitor.read_bytes_raw()?;
        if buffer.is_empty() {
            return Err(types::QosError::PsiParseError("Empty PSI file".into()));
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
        let mut current_total = 0u64;

        let mut pos = 0;
        let len = buffer.len();

        while pos < len {
            if pos + 5 > len || &buffer[pos..pos + 5] != b"some " {
                while pos < len && buffer[pos] != b'\n' {
                    pos += 1;
                }

                pos += 1;
                continue;
            }

            pos += 5;

            while pos < len && buffer[pos] != b'\n' {
                if buffer[pos] == b' ' {
                    pos += 1;
                    continue;
                }

                if pos + 6 <= len && &buffer[pos..pos + 6] == b"avg10=" {
                    let (parsed_val, next_pos) = Self::parse_f32_bytes(buffer, pos + 6);
                    some_trend.avg10 = parsed_val;
                    pos = next_pos;
                    continue;
                }

                if pos + 7 <= len && &buffer[pos..pos + 7] == b"avg300=" {
                    let (parsed_val, next_pos) = Self::parse_f32_bytes(buffer, pos + 7);
                    some_trend.avg300 = parsed_val;
                    pos = next_pos;
                    continue;
                }

                if pos + 6 <= len && &buffer[pos..pos + 6] == b"total=" {
                    let (parsed_val, next_pos) = Self::parse_u64_bytes(buffer, pos + 6);
                    current_total = parsed_val;
                    pos = next_pos;
                    continue;
                }

                while pos < len && buffer[pos] != b' ' && buffer[pos] != b'\n' {
                    pos += 1;
                }
            }
            break;
        }

        if self.first_run {
            some_trend.current = some_trend.avg10;
            some_trend.velocity = 0.0;

            self.filter_some.reset();
            self.filter_some.update(some_trend.avg10, 1.0);

            self.first_run = false;
        } else {
            let delta_some = current_total.saturating_sub(self.last_some_total) as f32;
            let raw_some = delta_some / dt_calc * 100.0;

            some_trend.current = self.filter_some.update(raw_some, dt_sec);
            some_trend.velocity = self.filter_some.get_velocity();
            some_trend.nis = self.filter_some.get_last_nis();
        }

        self.last_read_time = now;
        self.last_some_total = current_total;

        Ok(PsiData { some: some_trend })
    }
}
