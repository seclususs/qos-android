//! Author: [Seclususs](https://github.com/seclususs)

use crate::monitors::disk_monitor::IoStats;

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct StorageTunables {
    pub min_read_ahead: f64,
    pub max_read_ahead: f64,
    pub min_nr_requests: f64,
    pub max_nr_requests: f64,
    pub min_fifo_batch: f64,
    pub max_fifo_batch: f64,
    pub write_cost_factor: f64,
    pub target_latency_base_ms: f64,
    pub hysteresis_threshold: f64,
    pub critical_threshold_psi: f64,
    pub urgent_poll_psi: f64,
    pub urgent_poll_inflight: f64,
    pub pattern_window_size: usize,
    pub pattern_threshold_h0: f64,
    pub pattern_margin_k: f64,
    pub pattern_decision_h: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IoDelta {
    pub throughput_read: f64,
    pub throughput_write: f64,
    pub service_time_ms: f64,
    pub avg_request_size_sectors: f64,
}

pub struct PatternState {
    pub history: VecDeque<f64>,
    pub c_plus: f64,
    pub c_minus: f64,
    pub is_sequential: bool,
}

impl Default for PatternState {
    fn default() -> Self {
        Self {
            history: VecDeque::with_capacity(64),
            c_plus: 0.0,
            c_minus: 0.0,
            is_sequential: false,
        }
    }
}

pub fn calculate_io_deltas(current: &IoStats, prev: &IoStats, dt_sec: f64) -> IoDelta {
    if dt_sec <= 0.0 {
        return IoDelta::default();
    }
    let delta_read_ios = current.read_ios.saturating_sub(prev.read_ios) as f64;
    let delta_write_ios = current.write_ios.saturating_sub(prev.write_ios) as f64;
    let delta_read_ticks = current.read_ticks.saturating_sub(prev.read_ticks) as f64;
    let delta_write_ticks = current.write_ticks.saturating_sub(prev.write_ticks) as f64;
    let delta_read_sectors = current.read_sectors.saturating_sub(prev.read_sectors) as f64;
    let delta_write_sectors = current.write_sectors.saturating_sub(prev.write_sectors) as f64;
    let total_ios = delta_read_ios + delta_write_ios;
    let total_ticks = delta_read_ticks + delta_write_ticks;
    let total_sectors = delta_read_sectors + delta_write_sectors;
    let service_time_ms = if total_ios > 0.0 {
        total_ticks / total_ios
    } else {
        0.0
    };
    let avg_request_size_sectors = if total_ios > 0.0 {
        total_sectors / total_ios
    } else {
        0.0
    };
    IoDelta {
        throughput_read: delta_read_ios / dt_sec,
        throughput_write: delta_write_ios / dt_sec,
        service_time_ms,
        avg_request_size_sectors,
    }
}

fn calculate_rs_statistic(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 2 {
        return 0.0;
    }
    let mean: f64 = data.iter().sum::<f64>() / n as f64;
    let mut sum_sq_diff = 0.0;
    let mut current_accum_dev = 0.0;
    let mut max_y = 0.0;
    let mut min_y = 0.0;
    for &x in data {
        let diff = x - mean;
        sum_sq_diff += diff * diff;
        current_accum_dev += diff;
        if current_accum_dev > max_y {
            max_y = current_accum_dev;
        }
        if current_accum_dev < min_y {
            min_y = current_accum_dev;
        }
    }
    let r = max_y - min_y;
    let s = (sum_sq_diff / n as f64).sqrt();
    if s < 1e-9 {
        return 0.0;
    }
    r / s
}

pub fn calculate_variance_index(
    state: &mut PatternState,
    new_sample: f64,
    tunables: &StorageTunables,
) -> f64 {
    if new_sample > 0.1 {
        if state.history.len() >= tunables.pattern_window_size {
            state.history.pop_front();
        }
        state.history.push_back(new_sample);
    }
    let n = state.history.len();
    if n < 16 {
        return 0.5;
    }
    let data_vec: Vec<f64> = state.history.iter().copied().collect();
    let rs_full = calculate_rs_statistic(&data_vec);
    let half_n = n / 2;
    let rs_half = calculate_rs_statistic(&data_vec[half_n..]);
    if rs_full <= 0.0 || rs_half <= 0.0 {
        return 0.5;
    }
    let log_rs_full = rs_full.ln();
    let log_rs_half = rs_half.ln();
    let log_n = (n as f64).ln();
    let log_half_n = (half_n as f64).ln();
    let h = (log_rs_full - log_rs_half) / (log_n - log_half_n);
    h.clamp(0.0, 1.0)
}

pub fn update_pattern_decision(
    state: &mut PatternState,
    variance_val: f64,
    tunables: &StorageTunables,
) {
    let target = tunables.pattern_threshold_h0;
    let margin = tunables.pattern_margin_k;
    let limit = tunables.pattern_decision_h;
    state.c_plus = (state.c_plus + variance_val - (target + margin)).max(0.0);
    state.c_minus = (state.c_minus + (target - margin) - variance_val).max(0.0);
    if state.c_plus > limit {
        state.is_sequential = true;
        state.c_plus = 0.0;
    } else if state.c_minus > limit {
        state.is_sequential = false;
        state.c_minus = 0.0;
    }
}

pub fn calculate_adaptive_read_ahead(state: &PatternState, tunables: &StorageTunables) -> f64 {
    let base = tunables.min_read_ahead;
    let range = tunables.max_read_ahead - tunables.min_read_ahead;
    let factor = if state.is_sequential { 1.0 } else { 0.0 };
    base + (range * factor)
}

pub fn calculate_weighted_throughput(delta: &IoDelta, tunables: &StorageTunables) -> f64 {
    delta.throughput_read + (tunables.write_cost_factor * delta.throughput_write)
}

pub fn calculate_effective_latency(delta: &IoDelta, lambda_eff: f64, in_flight: f64) -> f64 {
    if delta.service_time_ms > 0.0 {
        delta.service_time_ms
    } else if lambda_eff > 0.0 {
        (in_flight / lambda_eff) * 1000.0
    } else {
        0.0
    }
}

pub fn calculate_target_latency(psi_some_avg10: f64, tunables: &StorageTunables) -> f64 {
    let psi_ratio = (psi_some_avg10 / 100.0).clamp(0.0, 1.0);
    let target = tunables.target_latency_base_ms * (1.0 - psi_ratio);
    target.max(1.0)
}

pub fn calculate_next_queue_depth(
    lambda_eff: f64,
    current_latency_ms: f64,
    target_latency_ms: f64,
    current_nr_requests: f64,
    psi_full_avg10: f64,
    tunables: &StorageTunables,
) -> f64 {
    if psi_full_avg10 > tunables.critical_threshold_psi {
        return tunables.min_nr_requests;
    }
    if lambda_eff < 1.0 || current_latency_ms < 0.1 {
        return current_nr_requests;
    }
    let gradient = target_latency_ms / current_latency_ms;
    let next_nr;
    if gradient > 1.2 {
        next_nr = current_nr_requests + 1.0;
    } else if gradient < 0.8 {
        let smoothing_factor = gradient.sqrt();
        next_nr = current_nr_requests * smoothing_factor;
    } else {
        next_nr = current_nr_requests;
    }
    next_nr.clamp(tunables.min_nr_requests, tunables.max_nr_requests)
}

pub fn should_update_nr_requests(
    calculated: f64,
    current: f64,
    tunables: &StorageTunables,
) -> bool {
    let diff = (calculated - current).abs();
    let error_ratio = diff / current;
    error_ratio > tunables.hysteresis_threshold
        || calculated <= tunables.min_nr_requests
        || calculated >= tunables.max_nr_requests
}

pub fn calculate_fifo_batch(current_nr_requests: f64, tunables: &StorageTunables) -> f64 {
    let nr_range = tunables.max_nr_requests - tunables.min_nr_requests;
    if nr_range <= 0.0 {
        return tunables.min_fifo_batch;
    }
    let batch_ratio = (current_nr_requests - tunables.min_nr_requests) / nr_range;
    let batch_range = tunables.max_fifo_batch - tunables.min_fifo_batch;
    tunables.min_fifo_batch + (batch_range * batch_ratio)
}