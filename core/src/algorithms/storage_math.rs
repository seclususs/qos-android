//! Author: [Seclususs](https://github.com/seclususs)

use crate::monitors::disk_monitor::IoStats;

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
    pub min_req_size_kb: f64,
    pub max_req_size_kb: f64,
    pub queue_pressure_low: f64,
    pub queue_pressure_high: f64,
    pub smoothing_factor: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IoDelta {
    pub throughput_read: f64,
    pub throughput_write: f64,
    pub service_time_ms: f64,
    pub delta_read_ios: f64,
    pub delta_read_merges: f64,
    pub delta_read_sectors: f64,
}

pub struct WorkloadState {
    pub sequentiality_smoothed: f64,
}

impl Default for WorkloadState {
    fn default() -> Self {
        Self {
            sequentiality_smoothed: 0.0,
        }
    }
}

pub fn is_congestion_critical(psi_avg10: f64, in_flight: f64, tunables: &StorageTunables) -> bool {
    psi_avg10 > tunables.critical_threshold_psi || in_flight > tunables.queue_pressure_high
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

pub fn calculate_io_deltas(current: &IoStats, prev: &IoStats, dt_sec: f64) -> IoDelta {
    if dt_sec <= 0.0 {
        return IoDelta::default();
    }
    let delta_read_ios = current.read_ios.saturating_sub(prev.read_ios) as f64;
    let delta_read_merges = current.read_merges.saturating_sub(prev.read_merges) as f64;
    let delta_read_sectors = current.read_sectors.saturating_sub(prev.read_sectors) as f64;
    let delta_write_ios = current.write_ios.saturating_sub(prev.write_ios) as f64;
    let delta_write_ticks = current.write_ticks.saturating_sub(prev.write_ticks) as f64;
    let delta_read_ticks = current.read_ticks.saturating_sub(prev.read_ticks) as f64;
    let total_ios = delta_read_ios + delta_write_ios;
    let total_ticks = delta_read_ticks + delta_write_ticks;
    let service_time_ms = if total_ios > 0.0 {
        total_ticks / total_ios
    } else {
        0.0
    };
    IoDelta {
        throughput_read: delta_read_ios / dt_sec,
        throughput_write: delta_write_ios / dt_sec,
        service_time_ms,
        delta_read_ios,
        delta_read_merges,
        delta_read_sectors,
    }
}

pub fn calculate_request_size_score(delta: &IoDelta, tunables: &StorageTunables) -> f64 {
    if delta.delta_read_ios <= 0.0 {
        return 0.0;
    }
    let avg_size_kb = (delta.delta_read_sectors / delta.delta_read_ios) * 0.5;
    let range = tunables.max_req_size_kb - tunables.min_req_size_kb;
    if range <= 0.0 {
        return 0.0;
    }
    let score = (avg_size_kb - tunables.min_req_size_kb) / range;
    score.clamp(0.0, 1.0)
}

pub fn calculate_merge_ratio(delta: &IoDelta) -> f64 {
    let total_submissions = delta.delta_read_merges + delta.delta_read_ios;
    if total_submissions <= 0.0 {
        return 0.0;
    }
    delta.delta_read_merges / total_submissions
}

pub fn calculate_pressure_score(in_flight: f64, tunables: &StorageTunables) -> f64 {
    let range = tunables.queue_pressure_high - tunables.queue_pressure_low;
    if range <= 0.0 {
        return 0.0;
    }
    let score = (in_flight - tunables.queue_pressure_low) / range;
    score.clamp(0.0, 1.0)
}

pub fn resolve_sequentiality_factor(
    state: &mut WorkloadState,
    req_size_score: f64,
    merge_ratio: f64,
    pressure_score: f64,
    tunables: &StorageTunables,
) -> f64 {
    let shape_factor = req_size_score.max(merge_ratio);
    let raw_sequentiality = shape_factor * pressure_score;
    let alpha = tunables.smoothing_factor;
    let smoothed = (raw_sequentiality * alpha) + (state.sequentiality_smoothed * (1.0 - alpha));
    state.sequentiality_smoothed = smoothed;
    smoothed
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

pub fn calculate_target_read_ahead(sequentiality: f64, tunables: &StorageTunables) -> f64 {
    let range = tunables.max_read_ahead - tunables.min_read_ahead;
    tunables.min_read_ahead + (range * sequentiality)
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

pub fn calculate_fifo_batch(current_nr_requests: f64, tunables: &StorageTunables) -> f64 {
    let nr_range = tunables.max_nr_requests - tunables.min_nr_requests;
    if nr_range <= 0.0 {
        return tunables.min_fifo_batch;
    }
    let batch_ratio = (current_nr_requests - tunables.min_nr_requests) / nr_range;
    let batch_range = tunables.max_fifo_batch - tunables.min_fifo_batch;
    tunables.min_fifo_batch + (batch_range * batch_ratio)
}