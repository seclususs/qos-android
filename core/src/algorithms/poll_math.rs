//! Author: [Seclususs](https://github.com/seclususs)

use crate::config::loop_settings::{MAX_POLLING_MS, MIN_POLLING_MS};

use std::time::{Instant, SystemTime, UNIX_EPOCH};

const SLEEP_TOLERANCE_MS: u64 = 500;
const MIN_EFFECTIVE_DT_MS: u64 = 500;
const QUANTIZATION_STEP_MS: u64 = 100;
const HYSTERESIS_THRESHOLD_MS: u64 = 500;
const NOISE_PERCENT: u64 = 5;
const RISE_FACTOR: f32 = 1.0;
const FALL_FACTOR: f32 = 0.2;

pub struct AdaptivePoller {
    current_interval: u64,
    last_pressure: f32,
    last_tick: Instant,
    target_interval: u64,
    weight_pressure: f32,
    weight_derivative: f32,
    rng_state: u64,
}

impl AdaptivePoller {
    pub fn new(weight_pressure: f32, weight_derivative: f32) -> Self {
        let start_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            current_interval: MIN_POLLING_MS,
            last_pressure: 0.0,
            last_tick: Instant::now(),
            target_interval: MIN_POLLING_MS,
            weight_pressure,
            weight_derivative,
            rng_state: start_seed,
        }
    }
    fn next_random(&mut self, range: u64) -> u64 {
        if range == 0 {
            return 0;
        }
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let limit = range * 2;
        self.rng_state % (limit + 1)
    }
    pub fn calculate_next_interval(&mut self, current_pressure: f32, avg300: f32) -> u64 {
        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.last_tick).as_millis() as u64;
        if elapsed_ms > (self.current_interval + SLEEP_TOLERANCE_MS) {
            log::debug!("Time Discontinuity (Sleep?): {}ms.", elapsed_ms);
            self.last_pressure = current_pressure;
            self.last_tick = now;
            self.current_interval = MIN_POLLING_MS;
            return MIN_POLLING_MS;
        }
        let (dynamic_min, dynamic_max) = if avg300 < 2.0 && current_pressure < 10.0 {
            (6000u64, MAX_POLLING_MS)
        } else if avg300 > 20.0 {
            (MIN_POLLING_MS, 5000u64)
        } else {
            (MIN_POLLING_MS, MAX_POLLING_MS)
        };
        let effective_dt_ms = elapsed_ms.max(MIN_EFFECTIVE_DT_MS);
        let dt_sec = effective_dt_ms as f32 / 1000.0;
        let rate_change = (current_pressure - self.last_pressure) / dt_sec;
        let prediction = current_pressure + (rate_change * RISE_FACTOR);
        let p_term = prediction * self.weight_pressure;
        let d_term = rate_change.abs() * self.weight_derivative;
        let priority_score = (p_term + d_term).clamp(0.0, 100.0);
        let raw_interval =
            dynamic_max as f32 - ((priority_score / 100.0) * (dynamic_max - dynamic_min) as f32);
        let target = if raw_interval < self.target_interval as f32 {
            raw_interval
        } else {
            (raw_interval * FALL_FACTOR)
                + (self.target_interval as f32 * (RISE_FACTOR - FALL_FACTOR))
        };
        self.target_interval = target as u64;
        let diff = (self.target_interval as i64 - self.current_interval as i64).abs();
        self.last_pressure = current_pressure;
        self.last_tick = now;
        if diff < HYSTERESIS_THRESHOLD_MS as i64 {
            return self.apply_discrete_math_mut(self.current_interval, dynamic_min, dynamic_max);
        }
        self.current_interval = self.target_interval;
        self.apply_discrete_math_mut(self.current_interval, dynamic_min, dynamic_max)
    }
    fn apply_discrete_math_mut(&mut self, interval: u64, min_limit: u64, max_limit: u64) -> u64 {
        let quantized = ((interval as f32 / QUANTIZATION_STEP_MS as f32).round()
            * QUANTIZATION_STEP_MS as f32) as u64;
        let clamped = quantized.clamp(min_limit, max_limit);
        let noise_amplitude = (clamped * NOISE_PERCENT) / 100;
        let noise = self.next_random(noise_amplitude);
        let final_val = if noise > noise_amplitude {
            clamped + (noise - noise_amplitude)
        } else {
            clamped.saturating_sub(noise_amplitude - noise)
        };
        final_val.clamp(min_limit, max_limit)
    }
}