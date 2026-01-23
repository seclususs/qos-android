//! Author: [Seclususs](https://github.com/seclususs)

use crate::monitors::vm_monitor;

const QUEUE_HISTORY_CAP: usize = 32;

#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryKernelLimits {
    pub min_swappiness: f32,
    pub max_swappiness: f32,
    pub min_vfs: f32,
    pub max_vfs: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMathConfig {
    pub pressure_kp: f32,
    pub pressure_kd: f32,
    pub inefficiency_cost: f32,
    pub pressure_vfs_k: f32,
    pub fragmentation_impact_k: f32,
    pub wss_cost_factor: f32,
    pub zram_thermal_cost: f32,
    pub general_smooth_factor: f32,
    pub queue_history_size: usize,
    pub queue_smoothing_alpha: f32,
    pub residence_time_threshold: f32,
    pub protection_curve_k: f32,
    pub congestion_scaling_factor: f32,
}

impl Default for MemoryMathConfig {
    fn default() -> Self {
        Self {
            pressure_kp: 0.8,
            pressure_kd: 0.2,
            inefficiency_cost: 25.0,
            pressure_vfs_k: 0.10,
            fragmentation_impact_k: 2.0,
            wss_cost_factor: 3.0,
            zram_thermal_cost: 1.5,
            general_smooth_factor: 0.20,
            queue_history_size: 16,
            queue_smoothing_alpha: 0.2,
            residence_time_threshold: 30.0,
            protection_curve_k: 3.0,
            congestion_scaling_factor: 2.5,
        }
    }
}

#[derive(Debug, Default)]
pub struct ActivityState {
    pub efficiency: f32,
    pub refault_index: f32,
    pub scan_rate: f32,
}

pub struct QueueState {
    pub rate_history: [f32; QUEUE_HISTORY_CAP],
    pub head: usize,
    pub count: usize,
    pub smoothed_rate: f32,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            rate_history: [0.0; QUEUE_HISTORY_CAP],
            head: 0,
            count: 0,
            smoothed_rate: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SwappinessInput<'a> {
    pub p_mem: f32,
    pub dp_dt: f32,
    pub activity: &'a ActivityState,
    pub cpu_temp: f32,
    pub io_sat: f32,
    pub queue_correction: f32,
}

pub fn calculate_active_set(stats: &vm_monitor::VmStats) -> f32 {
    (stats.nr_active_anon + stats.nr_inactive_anon + stats.nr_active_file + stats.nr_inactive_file)
        as f32
}

pub fn calculate_pressure_level(current: f32, avg10: f32) -> f32 {
    current.max(avg10)
}

pub fn calculate_pressure_derivative(current_psi: f32, prev_psi: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        0.0
    } else {
        (current_psi - prev_psi) / dt
    }
}

pub fn smooth_value(current: f32, target: f32, alpha: f32) -> f32 {
    current * (1.0 - alpha) + target * alpha
}

pub fn calculate_activity_state(
    current: &vm_monitor::VmStats,
    prev: &vm_monitor::VmStats,
    dt_sec: f32,
) -> ActivityState {
    if dt_sec <= 0.0 {
        return ActivityState::default();
    }
    let delta_scan = current.pgscan.saturating_sub(prev.pgscan) as f32;
    let delta_steal = current.pgsteal.saturating_sub(prev.pgsteal) as f32;
    let delta_refault = current
        .workingset_refault
        .saturating_sub(prev.workingset_refault) as f32;
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
    active_set: f32,
    rate_raw: f32,
    math_config: &MemoryMathConfig,
) -> f32 {
    if state.smoothed_rate == 0.0 {
        state.smoothed_rate = rate_raw;
    } else {
        state.smoothed_rate = math_config.queue_smoothing_alpha * rate_raw
            + (1.0 - math_config.queue_smoothing_alpha) * state.smoothed_rate;
    }
    let safe_rate = state.smoothed_rate.max(1.0);
    let residence_time = active_set / safe_rate;
    let history_limit = math_config.queue_history_size.min(QUEUE_HISTORY_CAP);
    state.rate_history[state.head] = rate_raw;
    state.head = (state.head + 1) % history_limit;
    if state.count < history_limit {
        state.count += 1;
    }
    let n = state.count as f32;
    let variability_factor = if n > 2.0 {
        let mut sum = 0.0;
        for i in 0..state.count {
            sum += state.rate_history[i];
        }
        let mean = sum / n;
        let mut variance_sum = 0.0;
        for i in 0..state.count {
            let v = state.rate_history[i];
            variance_sum += (v - mean).powi(2);
        }
        let std_dev = (variance_sum / n).sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };
        1.0 + (cv * math_config.congestion_scaling_factor)
    } else {
        1.0
    };
    let thrash_threshold = math_config.residence_time_threshold;
    let risk_ratio = if residence_time > 0.0 {
        thrash_threshold / residence_time
    } else {
        100.0
    };
    let protection_factor = 1.0 / (1.0 + risk_ratio.powf(math_config.protection_curve_k));
    let final_correction_factor = protection_factor / variability_factor;
    final_correction_factor.clamp(0.0, 1.5)
}

pub fn calculate_swappiness(
    input: SwappinessInput,
    math_config: &MemoryMathConfig,
    kernel_limits: &MemoryKernelLimits,
) -> f32 {
    let base_swap = kernel_limits.min_swappiness;
    let p_term = math_config.pressure_kp * input.p_mem;
    let d_term = math_config.pressure_kd * input.dp_dt;
    let inefficiency = math_config.inefficiency_cost * (1.0 - input.activity.efficiency);
    let target_swap_raw = base_swap + p_term + d_term + inefficiency;
    let target_swap_corrected = (target_swap_raw - base_swap) * input.queue_correction + base_swap;
    let thermal_stress = (input.cpu_temp - 50.0).max(0.0) / 20.0;
    let thermal_throttle = (1.0 - (thermal_stress * math_config.zram_thermal_cost)).clamp(0.0, 1.0);
    let io_throttle = (1.0 - (input.io_sat * 0.6)).clamp(0.2, 1.0);
    let mut final_swap = target_swap_corrected * thermal_throttle * io_throttle;
    let thrashing_cost = (input.activity.refault_index * math_config.wss_cost_factor).powi(2);
    let wss_preservation = (1.0 - thrashing_cost).clamp(0.0, 1.0);
    final_swap *= wss_preservation;
    final_swap.clamp(kernel_limits.min_swappiness, kernel_limits.max_swappiness)
}

pub fn calculate_vfs_pressure(
    p_mem: f32,
    math_config: &MemoryMathConfig,
    kernel_limits: &MemoryKernelLimits,
) -> f32 {
    let range = kernel_limits.max_vfs - kernel_limits.min_vfs;
    let decay = (-math_config.pressure_vfs_k * p_mem).exp();
    let inverse_decay = 1.0 - decay;
    let vfs = kernel_limits.min_vfs + (range * inverse_decay);
    vfs.clamp(kernel_limits.min_vfs, kernel_limits.max_vfs)
}
