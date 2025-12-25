//! Author: [Seclususs](https://github.com/seclususs)

use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crate::config::loop_settings::*;

const MIN_EFFECTIVE_DT_MS: u64 = 500;

pub struct AdaptivePoller {
    current_interval: u64,
    last_pressure: f64,
    last_tick: Instant,
    target_interval: u64,
    weight_pressure: f64,
    weight_derivative: f64,
    rng_state: u64,
}

impl AdaptivePoller {
    pub fn new(weight_pressure: f64, weight_derivative: f64) -> Self {
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
        if range == 0 { return 0; }
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let limit = range * 2;
        self.rng_state % (limit + 1)
    }
    pub fn calculate_next_interval(&mut self, current_pressure: f64) -> u64 {
        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.last_tick).as_millis() as u64;
        if elapsed_ms > (self.current_interval + SLEEP_TOLERANCE_MS) {
            log::debug!("Time Discontinuity (Sleep?): {}ms.", elapsed_ms);
            self.last_pressure = current_pressure;
            self.last_tick = now;
            self.current_interval = MIN_POLLING_MS;
            return MIN_POLLING_MS;
        }
        let effective_dt_ms = elapsed_ms.max(MIN_EFFECTIVE_DT_MS);
        let dt_sec = effective_dt_ms as f64 / 1000.0;
        let velocity = (current_pressure - self.last_pressure) / dt_sec;
        let prediction = current_pressure + (velocity * 1.0); 
        let p_term = prediction * self.weight_pressure;
        let d_term = velocity.abs() * self.weight_derivative;
        let urgency_score = (p_term + d_term).clamp(0.0, 100.0);
        let raw_interval = MAX_POLLING_MS as f64 - ((urgency_score / 100.0) * (MAX_POLLING_MS - MIN_POLLING_MS) as f64);
        let target = if raw_interval < self.target_interval as f64 {
            raw_interval 
        } else {
            (raw_interval * DECAY_COEFF) + (self.target_interval as f64 * (1.0 - DECAY_COEFF))
        };
        self.target_interval = target as u64;
        let diff = (self.target_interval as i64 - self.current_interval as i64).abs();
        self.last_pressure = current_pressure;
        self.last_tick = now;
        if diff < HYSTERESIS_THRESHOLD_MS as i64 {
            return self.apply_discrete_math_mut(self.current_interval);
        }
        self.current_interval = self.target_interval;
        self.apply_discrete_math_mut(self.current_interval)
    }
    fn apply_discrete_math_mut(&mut self, interval: u64) -> u64 {
        let quantized = ((interval as f64 / QUANTIZATION_STEP_MS as f64).round() * QUANTIZATION_STEP_MS as f64) as u64;
        let clamped = quantized.clamp(MIN_POLLING_MS, MAX_POLLING_MS);
        let jitter_range = (clamped * JITTER_PERCENT) / 100;
        let noise = self.next_random(jitter_range);
        let final_val = if noise > jitter_range {
            clamped + (noise - jitter_range)
        } else {
            clamped.saturating_sub(jitter_range - noise)
        };
        final_val.clamp(MIN_POLLING_MS, MAX_POLLING_MS)
    }
}