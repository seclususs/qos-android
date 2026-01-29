//! Author: [Seclususs](https://github.com/seclususs)

use crate::monitors::disk_monitor;
use crate::utils::tier::DeviceTier;

#[derive(Debug, Clone, Copy, Default)]
pub struct StorageKernelLimits {
    pub min_read_ahead: f32,
    pub max_read_ahead: f32,
    pub min_nr_requests: f32,
    pub max_nr_requests: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct StorageMathConfig {
    pub min_req_size_kb: f32,
    pub max_req_size_kb: f32,
    pub write_cost_factor: f32,
    pub target_latency_base_ms: f32,
    pub hysteresis_threshold: f32,
    pub critical_threshold_psi: f32,
    pub queue_pressure_low: f32,
    pub queue_pressure_high: f32,
    pub smoothing_factor: f32,
}

impl Default for StorageMathConfig {
    fn default() -> Self {
        let tier = DeviceTier::get();
        match tier {
            DeviceTier::Flagship => Self {
                min_req_size_kb: 6.0,
                max_req_size_kb: 768.0,
                write_cost_factor: 3.5,
                target_latency_base_ms: 30.0,
                hysteresis_threshold: 0.25,
                critical_threshold_psi: 18.0,
                queue_pressure_low: 0.15,
                queue_pressure_high: 6.0,
                smoothing_factor: 0.55,
            },
            DeviceTier::MidRange => Self {
                min_req_size_kb: 7.0,
                max_req_size_kb: 640.0,
                write_cost_factor: 4.0,
                target_latency_base_ms: 35.0,
                hysteresis_threshold: 0.28,
                critical_threshold_psi: 19.5,
                queue_pressure_low: 0.2,
                queue_pressure_high: 5.5,
                smoothing_factor: 0.52,
            },
            DeviceTier::LowEnd => Self {
                min_req_size_kb: 8.0,
                max_req_size_kb: 512.0,
                write_cost_factor: 4.5,
                target_latency_base_ms: 50.0,
                hysteresis_threshold: 0.3,
                critical_threshold_psi: 22.0,
                queue_pressure_low: 0.25,
                queue_pressure_high: 5.0,
                smoothing_factor: 0.5,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IoDelta {
    pub throughput_read: f32,
    pub throughput_write: f32,
    pub service_time_ms: f32,
    pub delta_read_ios: f32,
    pub delta_read_merges: f32,
    pub delta_read_sectors: f32,
}

pub struct WorkloadState {
    pub sequentiality_smoothed: f32,
}

impl Default for WorkloadState {
    fn default() -> Self {
        Self {
            sequentiality_smoothed: 0.0,
        }
    }
}

pub fn is_congestion_critical(
    psi_pressure: f32,
    in_flight: f32,
    math_config: &StorageMathConfig,
) -> bool {
    psi_pressure > math_config.critical_threshold_psi || in_flight > math_config.queue_pressure_high
}

pub fn should_update_nr_requests(
    calculated: f32,
    current: f32,
    math_config: &StorageMathConfig,
    kernel_limits: &StorageKernelLimits,
) -> bool {
    let diff = (calculated - current).abs();
    let error_ratio = if current > 0.0 { diff / current } else { 1.0 };
    error_ratio > math_config.hysteresis_threshold
        || calculated <= kernel_limits.min_nr_requests
        || calculated >= kernel_limits.max_nr_requests
}

pub fn calculate_io_deltas(
    current: &disk_monitor::IoStats,
    prev: &disk_monitor::IoStats,
    dt_real: f32,
) -> IoDelta {
    if dt_real <= 0.000001 {
        return IoDelta::default();
    }
    let delta_read_ios = current.read_ios.saturating_sub(prev.read_ios) as f32;
    let delta_read_merges = current.read_merges.saturating_sub(prev.read_merges) as f32;
    let delta_read_sectors = current.read_sectors.saturating_sub(prev.read_sectors) as f32;
    let delta_write_ios = current.write_ios.saturating_sub(prev.write_ios) as f32;
    let delta_write_ticks = current.write_ticks.saturating_sub(prev.write_ticks) as f32;
    let delta_read_ticks = current.read_ticks.saturating_sub(prev.read_ticks) as f32;
    let total_ios = delta_read_ios + delta_write_ios;
    let total_ticks = delta_read_ticks + delta_write_ticks;
    let service_time_ms = if total_ios > 0.0 {
        total_ticks / total_ios
    } else {
        0.0
    };
    IoDelta {
        throughput_read: delta_read_ios / dt_real,
        throughput_write: delta_write_ios / dt_real,
        service_time_ms,
        delta_read_ios,
        delta_read_merges,
        delta_read_sectors,
    }
}

pub fn calculate_request_size_ratio(delta: &IoDelta, math_config: &StorageMathConfig) -> f32 {
    if delta.delta_read_ios <= 0.0 {
        return 0.0;
    }
    let avg_size_kb = (delta.delta_read_sectors / delta.delta_read_ios) * 0.5;
    let range = math_config.max_req_size_kb - math_config.min_req_size_kb;
    if range <= 0.0 {
        return 0.0;
    }
    let ratio = (avg_size_kb - math_config.min_req_size_kb) / range;
    ratio.clamp(0.0, 1.0)
}

pub fn calculate_merge_ratio(delta: &IoDelta) -> f32 {
    let total_submissions = delta.delta_read_merges + delta.delta_read_ios;
    if total_submissions <= 0.0 {
        return 0.0;
    }
    delta.delta_read_merges / total_submissions
}

pub fn calculate_pressure_ratio(in_flight: f32, math_config: &StorageMathConfig) -> f32 {
    let range = math_config.queue_pressure_high - math_config.queue_pressure_low;
    if range <= 0.0 {
        return 0.0;
    }
    let ratio = (in_flight - math_config.queue_pressure_low) / range;
    ratio.clamp(0.0, 1.0)
}

pub fn resolve_sequentiality_factor(
    state: &mut WorkloadState,
    req_size_ratio: f32,
    merge_ratio: f32,
    pressure_ratio: f32,
    math_config: &StorageMathConfig,
) -> f32 {
    let pattern_factor = req_size_ratio.max(merge_ratio);
    let raw_sequentiality = pattern_factor * pressure_ratio;
    let alpha = math_config.smoothing_factor;
    let smoothed = (raw_sequentiality * alpha) + (state.sequentiality_smoothed * (1.0 - alpha));
    state.sequentiality_smoothed = smoothed;
    smoothed
}

pub fn calculate_weighted_throughput(delta: &IoDelta, math_config: &StorageMathConfig) -> f32 {
    delta.throughput_read + (math_config.write_cost_factor * delta.throughput_write)
}

pub fn calculate_effective_latency(delta: &IoDelta, lambda_eff: f32, in_flight: f32) -> f32 {
    if delta.service_time_ms > 0.0 {
        delta.service_time_ms
    } else if lambda_eff > 0.0 {
        (in_flight / lambda_eff) * 1000.0
    } else {
        0.0
    }
}

pub fn calculate_target_latency(psi_some_avg10: f32, math_config: &StorageMathConfig) -> f32 {
    let psi_ratio = (psi_some_avg10 / 100.0).clamp(0.0, 1.0);
    let target = math_config.target_latency_base_ms * (1.0 - psi_ratio);
    target.max(1.0)
}

pub fn calculate_target_read_ahead(sequentiality: f32, kernel_limits: &StorageKernelLimits) -> f32 {
    let range = kernel_limits.max_read_ahead - kernel_limits.min_read_ahead;
    kernel_limits.min_read_ahead + (range * sequentiality)
}

pub fn calculate_next_queue_depth(
    lambda_eff: f32,
    current_latency_ms: f32,
    target_latency_ms: f32,
    current_nr_requests: f32,
    psi_pressure: f32,
    math_config: &StorageMathConfig,
    kernel_limits: &StorageKernelLimits,
) -> f32 {
    if psi_pressure > math_config.critical_threshold_psi {
        return kernel_limits.min_nr_requests;
    }
    if lambda_eff < 1.0 || current_latency_ms < 0.1 {
        return current_nr_requests;
    }
    let gradient = target_latency_ms / current_latency_ms.max(0.1);
    let next_nr;
    if gradient > 1.2 {
        next_nr = current_nr_requests + 2.0;
    } else if gradient < 0.8 {
        let smoothing_factor = gradient.sqrt();
        next_nr = current_nr_requests * smoothing_factor;
    } else {
        next_nr = current_nr_requests;
    }
    next_nr.clamp(kernel_limits.min_nr_requests, kernel_limits.max_nr_requests)
}
