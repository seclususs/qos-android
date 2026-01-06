//! Author: [Seclususs](https://github.com/seclususs)

use crate::monitors::vm_monitor::VmStats;

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct MemoryTunables {
    pub min_swappiness: f64,
    pub max_swappiness: f64,
    pub min_dirty_expire: f64,
    pub max_dirty_expire: f64,
    pub min_stat_interval: f64,
    pub max_stat_interval: f64,
    pub min_watermark_scale: f64,
    pub max_watermark_scale: f64,
    pub min_extfrag_threshold: f64,
    pub max_extfrag_threshold: f64,
    pub min_dirty: f64,
    pub max_dirty: f64,
    pub min_dirty_bg: f64,
    pub max_dirty_bg: f64,
    pub min_dirty_writeback: f64,
    pub max_dirty_writeback: f64,
    pub min_page_cluster: f64,
    pub max_page_cluster: f64,
    pub min_vfs: f64,
    pub max_vfs: f64,
    pub pressure_kp: f64,
    pub pressure_kd: f64,
    pub inefficiency_penalty: f64,
    pub thermal_vfs_k: f64,
    pub fragmentation_impact_k: f64,
    pub wss_penalty_factor: f64,
    pub zram_thermal_penalty: f64,
    pub general_smooth_factor: f64,
    pub watermark_smooth_factor: f64,
    pub queue_history_size: usize,
    pub queue_smoothing_alpha: f64,
    pub residence_time_threshold: f64,
    pub protection_curve_k: f64,
    pub congestion_scaling_factor: f64,
}

#[derive(Default)]
pub struct ActivityState {
    pub efficiency: f64,
    pub refault_index: f64,
    pub scan_rate: f64,
}

pub struct QueueState {
    pub lambda_history: VecDeque<f64>,
    pub smoothed_lambda: f64,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            lambda_history: VecDeque::with_capacity(32),
            smoothed_lambda: 0.0,
        }
    }
}

pub fn calculate_activity_state(current: &VmStats, prev: &VmStats, dt_sec: f64) -> ActivityState {
    if dt_sec <= 0.0 {
        return ActivityState::default();
    }
    let delta_scan = current.pgscan.saturating_sub(prev.pgscan) as f64;
    let delta_steal = current.pgsteal.saturating_sub(prev.pgsteal) as f64;
    let delta_refault = current
        .workingset_refault
        .saturating_sub(prev.workingset_refault) as f64;
    let efficiency = if delta_scan > 0.0 {
        delta_steal / (delta_scan + 0.001)
    } else {
        1.0
    };
    let refault_index = if delta_scan > 0.0 || delta_refault > 0.0 {
        delta_refault / (delta_scan + delta_refault + 0.001)
    } else {
        0.0
    };
    ActivityState {
        efficiency: efficiency.clamp(0.0, 1.0),
        refault_index: refault_index.clamp(0.0, 1.0),
        scan_rate: delta_scan / dt_sec,
    }
}

pub fn update_congestion_model(
    state: &mut QueueState,
    active_set: f64,
    lambda_raw: f64,
    tunables: &MemoryTunables,
) -> f64 {
    if state.smoothed_lambda == 0.0 {
        state.smoothed_lambda = lambda_raw;
    } else {
        state.smoothed_lambda = tunables.queue_smoothing_alpha * lambda_raw
            + (1.0 - tunables.queue_smoothing_alpha) * state.smoothed_lambda;
    }
    let safe_lambda = state.smoothed_lambda.max(1.0);
    let residence_time = active_set / safe_lambda;
    if state.lambda_history.len() >= tunables.queue_history_size {
        state.lambda_history.pop_front();
    }
    state.lambda_history.push_back(lambda_raw);
    let n = state.lambda_history.len() as f64;
    let variability_penalty = if n > 2.0 {
        let mean: f64 = state.lambda_history.iter().sum::<f64>() / n;
        let variance_sum: f64 = state
            .lambda_history
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum();
        let std_dev = (variance_sum / n).sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };
        1.0 + (cv * tunables.congestion_scaling_factor)
    } else {
        1.0
    };
    let thrash_threshold = tunables.residence_time_threshold;
    let risk_ratio = if residence_time > 0.0 {
        thrash_threshold / residence_time
    } else {
        100.0
    };
    let protection_factor = 1.0 / (1.0 + risk_ratio.powf(tunables.protection_curve_k));
    let final_correction_factor = protection_factor / variability_penalty;
    final_correction_factor.clamp(0.0, 1.5)
}

pub fn calculate_pressure_derivative(current_psi: f64, prev_psi: f64, dt: f64) -> f64 {
    if dt <= 0.0 {
        0.0
    } else {
        (current_psi - prev_psi) / dt
    }
}

pub fn calculate_swappiness(
    p_mem: f64,
    dp_dt: f64,
    activity: &ActivityState,
    cpu_temp: f64,
    io_sat: f64,
    queue_correction: f64,
    tunables: &MemoryTunables,
) -> f64 {
    let base_swap = tunables.min_swappiness;
    let p_term = tunables.pressure_kp * p_mem;
    let d_term = tunables.pressure_kd * dp_dt;
    let inefficiency = tunables.inefficiency_penalty * (1.0 - activity.efficiency);
    let target_swap_raw = base_swap + p_term + d_term + inefficiency;
    let target_swap_corrected = (target_swap_raw - base_swap) * queue_correction + base_swap;
    let thermal_stress = (cpu_temp - 50.0).max(0.0) / 20.0;
    let thermal_throttle = (1.0 - (thermal_stress * tunables.zram_thermal_penalty)).clamp(0.0, 1.0);
    let io_throttle = (1.0 - (io_sat * 0.6)).clamp(0.2, 1.0);
    let mut final_swap = target_swap_corrected * thermal_throttle * io_throttle;
    let thrashing_penalty = (activity.refault_index * tunables.wss_penalty_factor).powi(2);
    let wss_protection = (1.0 - thrashing_penalty).clamp(0.0, 1.0);
    final_swap *= wss_protection;
    final_swap.clamp(tunables.min_swappiness, tunables.max_swappiness)
}

pub fn calculate_vfs_pressure(p_mem: f64, tunables: &MemoryTunables) -> f64 {
    let range = tunables.max_vfs - tunables.min_vfs;
    let decay = (-tunables.thermal_vfs_k * p_mem).exp();
    let inverse_decay = 1.0 - decay;
    let vfs = tunables.min_vfs + (range * inverse_decay);
    vfs.clamp(tunables.min_vfs, tunables.max_vfs)
}

pub fn calculate_dirty_limits(io_sat: f64, tunables: &MemoryTunables) -> (f64, f64) {
    let throughput_capacity = (1.0 - io_sat).clamp(0.1, 1.0);
    let target_dirty = tunables.max_dirty * throughput_capacity;
    let target_dirty_bg = tunables.max_dirty_bg * throughput_capacity;
    (
        target_dirty.clamp(tunables.min_dirty, tunables.max_dirty),
        target_dirty_bg.clamp(tunables.min_dirty_bg, tunables.max_dirty_bg),
    )
}

pub fn calculate_dirty_time(io_sat: f64, tunables: &MemoryTunables) -> f64 {
    let t = io_sat.clamp(0.0, 1.0);
    let expire =
        tunables.min_dirty_expire + (tunables.max_dirty_expire - tunables.min_dirty_expire) * t;
    expire.clamp(tunables.min_dirty_expire, tunables.max_dirty_expire)
}

pub fn calculate_dirty_writeback(target_expire: f64, tunables: &MemoryTunables) -> f64 {
    let t_wb = (target_expire - tunables.min_dirty_expire)
        / (tunables.max_dirty_expire - tunables.min_dirty_expire);
    let wb = tunables.min_dirty_writeback
        + (tunables.max_dirty_writeback - tunables.min_dirty_writeback) * t_wb;
    wb.clamp(tunables.min_dirty_writeback, tunables.max_dirty_writeback)
}

pub fn calculate_watermark_scale(p_mem: f64, fragmentation: f64, tunables: &MemoryTunables) -> f64 {
    let pressure_factor = (p_mem / 100.0).clamp(0.0, 1.0);
    let fragmentation_impact = tunables.fragmentation_impact_k * fragmentation * pressure_factor;
    let target_wm = tunables.min_watermark_scale * (1.0 + fragmentation_impact);
    target_wm.clamp(tunables.min_watermark_scale, tunables.max_watermark_scale)
}

pub fn calculate_extfrag_threshold(p_cpu: f64, tunables: &MemoryTunables) -> f64 {
    if p_cpu > 50.0 {
        tunables.max_extfrag_threshold
    } else {
        tunables.min_extfrag_threshold
    }
}

pub fn calculate_clustering_factor(p_cpu: f64, tunables: &MemoryTunables) -> f64 {
    if p_cpu > 25.0 {
        tunables.min_page_cluster
    } else {
        1.0
    }
}

pub fn calculate_sampling_rate(p_mem: f64, tunables: &MemoryTunables) -> f64 {
    let urgency = (p_mem / 50.0).clamp(0.0, 1.0);
    let interval = tunables.max_stat_interval
        - (urgency * (tunables.max_stat_interval - tunables.min_stat_interval));
    interval.clamp(tunables.min_stat_interval, tunables.max_stat_interval)
}

pub fn smooth_value(current: f64, target: f64, alpha: f64) -> f64 {
    current * (1.0 - alpha) + target * alpha
}