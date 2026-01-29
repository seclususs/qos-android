//! Author: [Seclususs](https://github.com/seclususs)

use crate::utils::tier::DeviceTier;

#[derive(Debug, Clone, Copy)]
pub struct LoadState {
    pub psi_value: f32,
    pub rate: f32,
    pub prev_integral: f32,
    pub first_run: bool,
}

impl Default for LoadState {
    fn default() -> Self {
        Self {
            psi_value: 0.0,
            rate: 0.0,
            prev_integral: 0.0,
            first_run: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CpuKernelLimits {
    pub min_latency_ns: f32,
    pub max_latency_ns: f32,
    pub min_granularity_ns: f32,
    pub max_granularity_ns: f32,
    pub min_wakeup_ns: f32,
    pub max_wakeup_ns: f32,
    pub min_migration_cost: f32,
    pub max_migration_cost: f32,
    pub min_walt_init_pct: f32,
    pub max_walt_init_pct: f32,
    pub min_uclamp_min: f32,
    pub max_uclamp_min: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CpuMathConfig {
    pub latency_gran_ratio: f32,
    pub decay_coeff: f32,
    pub uclamp_k: f32,
    pub uclamp_mid: f32,
    pub response_gain: f32,
    pub stability_ratio: f32,
    pub stability_margin: f32,
    pub gain_scheduling_alpha: f32,
    pub sigmoid_k: f32,
    pub sigmoid_mid: f32,
    pub lookahead_time: f32,
    pub efficiency_gain: f32,
    pub trend_amplification: f32,
    pub surge_threshold: f32,
    pub surge_gain: f32,
    pub transient_rate_threshold: f32,
    pub transient_diff_threshold: f32,
    pub transient_poll_interval: f32,
    pub nis_threshold: f32,
    pub bat_level_weight: f32,
}

impl Default for CpuMathConfig {
    fn default() -> Self {
        let tier = DeviceTier::get();
        match tier {
            DeviceTier::Flagship => Self {
                latency_gran_ratio: 0.32,
                decay_coeff: 0.014,
                uclamp_k: 0.21,
                uclamp_mid: 7.5,
                response_gain: 36.0,
                stability_ratio: 2.05,
                stability_margin: 4.0,
                gain_scheduling_alpha: 0.982,
                sigmoid_k: 0.085,
                sigmoid_mid: 5.5,
                lookahead_time: 0.14,
                efficiency_gain: 6.2,
                trend_amplification: 0.135,
                surge_threshold: 15.0,
                surge_gain: 0.115,
                transient_rate_threshold: 0.095,
                transient_diff_threshold: 0.45,
                transient_poll_interval: 45.0,
                nis_threshold: 6.5,
                bat_level_weight: 94.0,
            },
            DeviceTier::MidRange => Self {
                latency_gran_ratio: 0.33,
                decay_coeff: 0.017,
                uclamp_k: 0.195,
                uclamp_mid: 8.5,
                response_gain: 33.0,
                stability_ratio: 2.12,
                stability_margin: 3.5,
                gain_scheduling_alpha: 0.978,
                sigmoid_k: 0.078,
                sigmoid_mid: 6.2,
                lookahead_time: 0.155,
                efficiency_gain: 5.85,
                trend_amplification: 0.125,
                surge_threshold: 16.5,
                surge_gain: 0.105,
                transient_rate_threshold: 0.105,
                transient_diff_threshold: 0.52,
                transient_poll_interval: 48.0,
                nis_threshold: 7.2,
                bat_level_weight: 95.5,
            },
            DeviceTier::LowEnd => Self {
                latency_gran_ratio: 0.34,
                decay_coeff: 0.019,
                uclamp_k: 0.185,
                uclamp_mid: 9.5,
                response_gain: 31.0,
                stability_ratio: 2.18,
                stability_margin: 3.1,
                gain_scheduling_alpha: 0.972,
                sigmoid_k: 0.072,
                sigmoid_mid: 6.8,
                lookahead_time: 0.17,
                efficiency_gain: 5.6,
                trend_amplification: 0.115,
                surge_threshold: 17.5,
                surge_gain: 0.095,
                transient_rate_threshold: 0.115,
                transient_diff_threshold: 0.58,
                transient_poll_interval: 52.0,
                nis_threshold: 7.8,
                bat_level_weight: 97.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DemandInput {
    pub target_psi: f32,
    pub psi_velocity: f32,
    pub dt_real: f32,
    pub dt_safe: f32,
    pub thermal_scale: f32,
    pub trend_factor: f32,
    pub integral_total: f32,
    pub integral_dot: f32,
    pub is_structural_break: bool,
}

#[inline(always)]
fn sigmoid_param(val: f32, k: f32, mid: f32) -> f32 {
    let x = k * (val - mid);
    0.5 * (x / (1.0 + x.abs()) + 1.0)
}

#[inline(always)]
fn decay(val: f32, coeff: f32) -> f32 {
    let x = coeff * val;
    if x < 0.0 {
        return 1.0;
    }
    1.0 / (1.0 + x + 0.5 * x * x)
}

#[inline(always)]
fn tanh(x: f32) -> f32 {
    x / (1.0 + x * x).sqrt()
}

pub fn sanitize_dt(secs: f32) -> f32 {
    secs.clamp(0.000001, 0.1)
}

pub fn is_transient(state: &LoadState, target_psi: f32, math_config: &CpuMathConfig) -> bool {
    state.rate.abs() > math_config.transient_rate_threshold
        || (state.psi_value - target_psi).abs() > math_config.transient_diff_threshold
}

pub fn update_integral_params(
    state: &mut LoadState,
    bat_level: f32,
    dt_safe: f32,
    math_config: &CpuMathConfig,
) -> (f32, f32) {
    let depletion = (100.0 - bat_level).max(0.0) / 100.0;
    let cost_heuristic = math_config.bat_level_weight * depletion.powi(3);
    let total_integral = cost_heuristic;
    if state.first_run {
        state.prev_integral = total_integral;
        state.first_run = false;
        return (total_integral, 0.0);
    }
    let integral_dot = if dt_safe > 0.0 {
        (total_integral - state.prev_integral) / dt_safe
    } else {
        0.0
    };
    state.prev_integral = total_integral;
    (total_integral, integral_dot)
}

pub fn calculate_load_demand(
    state: &mut LoadState,
    input: DemandInput,
    math_config: &CpuMathConfig,
) -> f32 {
    if input.is_structural_break {
        state.psi_value = input.target_psi;
        state.rate = 0.0;
    }
    let load_rate = input.psi_velocity;
    if load_rate.abs() > math_config.surge_threshold {
        state.rate += load_rate * math_config.surge_gain;
    }
    let prediction_target = input.target_psi + (load_rate * math_config.lookahead_time);
    let k_base = math_config.response_gain;
    let k_dynamic = k_base * (1.0 + (math_config.gain_scheduling_alpha * input.trend_factor));
    let k_final = k_dynamic * input.thermal_scale.clamp(0.1, 1.0).powi(2);
    let displacement = prediction_target - state.psi_value;
    let prop_term = k_final * displacement;
    let mut limit_term = input.integral_total * state.psi_value;
    let max_possible_response = k_final * 100.0;
    limit_term = limit_term.min(max_possible_response * 1.5);
    let crit_damp = 2.0 * k_final.sqrt();
    let base_damp = crit_damp * math_config.stability_ratio;
    let rate_sq = load_rate.powi(2) + 0.001;
    let stability_damping_req =
        (0.5 * input.integral_dot.abs() * state.psi_value.powi(2)) / rate_sq;
    let c_stability =
        stability_damping_req.clamp(0.0, base_damp * 4.0) * math_config.stability_margin;
    let c_thermal_adjusted = base_damp / input.thermal_scale.clamp(0.1, 1.0).sqrt();
    let c_final = c_thermal_adjusted.max(c_stability);
    let deriv_term = c_final * state.rate;
    let net_correction = prop_term - deriv_term - limit_term;
    let rate_delta = net_correction;
    state.rate += rate_delta * input.dt_safe;
    state.psi_value += state.rate * input.dt_real;
    if state.psi_value < 0.0 {
        state.psi_value = 0.0;
        state.rate = 0.0;
    }
    if state.psi_value > 500.0 {
        state.psi_value = 500.0;
        state.rate = 0.0;
    }
    state.psi_value
}

pub fn calculate_trend_gain(velocity: f32) -> f32 {
    if velocity > 0.0 { tanh(velocity) } else { 0.0 }
}

pub fn calculate_effective_pressure(
    load_demand: f32,
    trend_factor: f32,
    io_psi: f32,
    math_config: &CpuMathConfig,
) -> f32 {
    let p_response = load_demand * (1.0 + trend_factor * math_config.trend_amplification);
    let ratio_stall = io_psi / (load_demand + 1.0);
    let throughput_ratio = 1.0 / (1.0 + (ratio_stall * math_config.efficiency_gain));
    p_response * throughput_ratio
}

pub fn calculate_thermal_latency_limit(thermal_scale: f32, kernel_limits: &CpuKernelLimits) -> f32 {
    let limit_ratio = (1.0 - thermal_scale).clamp(0.0, 1.0);
    kernel_limits.min_latency_ns
        + (kernel_limits.max_latency_ns - kernel_limits.min_latency_ns) * limit_ratio
}

pub fn calculate_latency_and_granularity(
    p_eff: f32,
    load_demand: f32,
    thermal_min_latency_ns: f32,
    math_config: &CpuMathConfig,
    kernel_limits: &CpuKernelLimits,
) -> (f32, f32) {
    let sigmoid_val = sigmoid_param(p_eff, math_config.sigmoid_k, math_config.sigmoid_mid);
    let factor = 1.0 - sigmoid_val;
    let normal_latency = kernel_limits.min_latency_ns
        + ((kernel_limits.max_latency_ns - kernel_limits.min_latency_ns) * factor);
    let latency_range = kernel_limits.max_latency_ns - kernel_limits.min_latency_ns;
    let effective_demand = (load_demand / 100.0).clamp(0.0, 1.0);
    let low_latency_target = kernel_limits.max_latency_ns - (effective_demand * latency_range);
    let ideal_latency = normal_latency.min(low_latency_target);
    let final_latency = ideal_latency.max(thermal_min_latency_ns);
    let adjusted_latency =
        final_latency.clamp(kernel_limits.min_latency_ns, kernel_limits.max_latency_ns);
    let raw_gran = adjusted_latency * math_config.latency_gran_ratio;
    let final_gran = raw_gran
        .clamp(
            kernel_limits.min_granularity_ns,
            kernel_limits.max_granularity_ns,
        )
        .min(adjusted_latency);
    (adjusted_latency, final_gran)
}

pub fn calculate_wakeup_granularity(
    p_eff: f32,
    math_config: &CpuMathConfig,
    kernel_limits: &CpuKernelLimits,
) -> f32 {
    let decay = decay(p_eff, math_config.decay_coeff);
    let raw_wake = kernel_limits.min_wakeup_ns
        + (kernel_limits.max_wakeup_ns - kernel_limits.min_wakeup_ns) * decay;
    raw_wake.clamp(kernel_limits.min_wakeup_ns, kernel_limits.max_wakeup_ns)
}

pub fn calculate_migration_cost(velocity: f32, p_eff: f32, kernel_limits: &CpuKernelLimits) -> f32 {
    let x = (p_eff / 100.0).clamp(0.0, 1.0);
    let raw_mig = kernel_limits.min_migration_cost
        + (kernel_limits.max_migration_cost - kernel_limits.min_migration_cost) * (x * x);
    let volatility_ratio = (velocity.abs() / 25.0).clamp(0.0, 1.0);
    let dynamic_cost = raw_mig * (1.0 - (volatility_ratio * 0.5));
    dynamic_cost.clamp(
        kernel_limits.min_migration_cost,
        kernel_limits.max_migration_cost,
    )
}

pub fn calculate_walt_init(pressure: f32, kernel_limits: &CpuKernelLimits) -> f32 {
    let ratio = pressure / 100.0;
    let load_curve = ratio * ratio;
    let range = kernel_limits.max_walt_init_pct - kernel_limits.min_walt_init_pct;
    let val = kernel_limits.min_walt_init_pct + (range * load_curve);
    val.clamp(
        kernel_limits.min_walt_init_pct,
        kernel_limits.max_walt_init_pct,
    )
}

pub fn calculate_uclamp_min(
    pressure: f32,
    thermal_scale: f32,
    math_config: &CpuMathConfig,
    kernel_limits: &CpuKernelLimits,
) -> f32 {
    let sigmoid_val = sigmoid_param(pressure, math_config.uclamp_k, math_config.uclamp_mid);
    let range = kernel_limits.max_uclamp_min - kernel_limits.min_uclamp_min;
    let ideal_uclamp = kernel_limits.min_uclamp_min + (range * sigmoid_val);
    (ideal_uclamp * thermal_scale).clamp(kernel_limits.min_uclamp_min, kernel_limits.max_uclamp_min)
}
