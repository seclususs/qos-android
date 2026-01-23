//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct LoadState {
    pub psi_value: f32,
    pub rate: f32,
    pub load_history: [f32; 8],
    pub history_idx: usize,
    pub integral_accum: f32,
    pub prev_integral: f32,
    pub smoothed_integral: f32,
    pub first_run: bool,
}

impl Default for LoadState {
    fn default() -> Self {
        Self {
            psi_value: 0.0,
            rate: 0.0,
            load_history: [0.0; 8],
            history_idx: 0,
            integral_accum: 0.0,
            prev_integral: 0.0,
            smoothed_integral: 0.0,
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
    pub alpha_smooth: f32,
    pub sigmoid_k: f32,
    pub sigmoid_mid: f32,
    pub lookahead_time: f32,
    pub variance_sensitivity: f32,
    pub efficiency_gain: f32,
    pub trend_amplification: f32,
    pub surge_threshold: f32,
    pub surge_gain: f32,
    pub transient_rate_threshold: f32,
    pub transient_diff_threshold: f32,
    pub transient_poll_interval: f32,
    pub nis_threshold: f32,
    pub safe_temp_limit: f32,
    pub max_temp_limit: f32,
    pub temp_cost_weight: f32,
    pub bat_temp_weight: f32,
    pub bat_level_weight: f32,
    pub integral_acc_rate: f32,
    pub memory_migration_alpha: f32,
    pub memory_granularity_scaling: f32,
    pub memory_volatility_cost: f32,
}

impl Default for CpuMathConfig {
    fn default() -> Self {
        Self {
            latency_gran_ratio: 0.60,
            decay_coeff: 0.15,
            uclamp_k: 0.18,
            uclamp_mid: 20.0,
            response_gain: 40.0,
            stability_ratio: 1.50,
            stability_margin: 1.5,
            gain_scheduling_alpha: 1.0,
            alpha_smooth: 0.70,
            sigmoid_k: 0.25,
            sigmoid_mid: 35.0,
            lookahead_time: 0.08,
            variance_sensitivity: 0.12,
            efficiency_gain: 3.0,
            trend_amplification: 0.15,
            surge_threshold: 35.0,
            surge_gain: 0.25,
            transient_rate_threshold: 0.30,
            transient_diff_threshold: 2.0,
            transient_poll_interval: 50.0,
            nis_threshold: 8.0,
            safe_temp_limit: 55.0,
            max_temp_limit: 75.0,
            temp_cost_weight: 7.0,
            bat_temp_weight: 5.0,
            bat_level_weight: 70.0,
            integral_acc_rate: 0.15,
            memory_migration_alpha: 1.8,
            memory_granularity_scaling: 1.0,
            memory_volatility_cost: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DemandInput {
    pub target_psi: f32,
    pub dt_sec: f32,
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

fn calculate_regression_slope(state: &LoadState) -> f32 {
    const N: f32 = 8.0;
    const SUM_X: f32 = 28.0;
    const DENOMINATOR: f32 = 336.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    for i in 0..8 {
        let idx = (state.history_idx + i) % 8;
        let y = state.load_history[idx];
        let x = i as f32;
        sum_y += y;
        sum_xy += x * y;
    }
    let numerator = (N * sum_xy) - (SUM_X * sum_y);
    numerator / DENOMINATOR
}

pub fn smooth_delta(current_delta: f32, prev_smooth: f32, math_config: &CpuMathConfig) -> f32 {
    math_config.alpha_smooth * current_delta + (1.0 - math_config.alpha_smooth) * prev_smooth
}

pub fn is_transient(state: &LoadState, target_psi: f32, math_config: &CpuMathConfig) -> bool {
    state.rate.abs() > math_config.transient_rate_threshold
        || (state.psi_value - target_psi).abs() > math_config.transient_diff_threshold
}

pub fn update_integral_params(
    state: &mut LoadState,
    cpu_temp: f32,
    bat_temp: f32,
    bat_level: f32,
    dt_sec: f32,
    math_config: &CpuMathConfig,
) -> (f32, f32) {
    let temp_ratio = (cpu_temp / math_config.max_temp_limit).clamp(0.0, 1.5);
    let term_cpu = math_config.temp_cost_weight * temp_ratio.powi(2);
    let bat_stress = (bat_temp / 45.0).clamp(0.0, 1.0);
    let term_bat_temp = math_config.bat_temp_weight * bat_stress;
    let depletion = (100.0 - bat_level).max(0.0) / 100.0;
    let term_bat_cap = math_config.bat_level_weight * depletion.powi(3);
    let cost_heuristic = term_cpu + term_bat_temp + term_bat_cap;
    let limit_violation = (cpu_temp - math_config.safe_temp_limit).max(0.0);
    let integration_rate = math_config.integral_acc_rate * limit_violation;
    state.integral_accum += integration_rate * dt_sec;
    if limit_violation <= 0.0 {
        state.integral_accum *= 0.98;
    }
    state.integral_accum = state.integral_accum.clamp(0.0, 200.0);
    let total_integral = cost_heuristic + state.integral_accum;
    if state.first_run {
        state.smoothed_integral = total_integral;
        state.prev_integral = total_integral;
        state.first_run = false;
        return (total_integral, 0.0);
    }
    state.smoothed_integral = (state.smoothed_integral * 0.8) + (total_integral * 0.2);
    let integral_dot = if dt_sec > 0.0 {
        (state.smoothed_integral - state.prev_integral) / dt_sec
    } else {
        0.0
    };
    state.prev_integral = state.smoothed_integral;
    (state.smoothed_integral, integral_dot)
}

pub fn calculate_load_demand(
    state: &mut LoadState,
    input: DemandInput,
    math_config: &CpuMathConfig,
) -> f32 {
    if input.is_structural_break {
        for i in 0..8 {
            state.load_history[i] = input.target_psi;
        }
    }
    state.load_history[state.history_idx] = input.target_psi;
    state.history_idx = (state.history_idx + 1) % 8;
    let mut sum = 0.0;
    for val in state.load_history.iter() {
        sum += val;
    }
    let mean = sum / 8.0;
    let mut variance_sum = 0.0;
    for val in state.load_history.iter() {
        variance_sum += (val - mean).powi(2);
    }
    let std_dev = (variance_sum / 8.0).sqrt();
    let deviation_gain = 1.0 + (math_config.variance_sensitivity * std_dev);
    let slope_per_tick = calculate_regression_slope(state);
    let load_rate = slope_per_tick / input.dt_sec.max(0.001);
    if load_rate.abs() > math_config.surge_threshold {
        state.rate += load_rate * math_config.surge_gain;
    }
    let prediction_target = input.target_psi + (load_rate * math_config.lookahead_time);
    let k_base = math_config.response_gain;
    let k_dynamic = k_base * (1.0 + (math_config.gain_scheduling_alpha * input.trend_factor));
    let k_final = k_dynamic * deviation_gain * input.thermal_scale.clamp(0.1, 1.0).powi(2);
    let displacement = prediction_target - state.psi_value;
    let prop_term = k_final * displacement;
    let mut limit_term = input.integral_total * state.psi_value;
    let max_possible_response = k_final * 100.0;
    limit_term = limit_term.min(max_possible_response * 1.5);
    let crit_damp = 2.0 * k_final.sqrt();
    let base_damp = crit_damp * math_config.stability_ratio;
    let rate_sq = state.rate.powi(2) + 0.001;
    let stability_damping_req =
        (0.5 * input.integral_dot.abs() * state.psi_value.powi(2)) / rate_sq;
    let c_stability =
        stability_damping_req.clamp(0.0, base_damp * 4.0) * math_config.stability_margin;
    let c_thermal_adjusted = base_damp / input.thermal_scale.clamp(0.1, 1.0).sqrt();
    let c_final = c_thermal_adjusted.max(c_stability);
    let deriv_term = c_final * state.rate;
    let net_correction = prop_term - deriv_term - limit_term;
    let rate_delta = net_correction;
    state.rate += rate_delta * input.dt_sec;
    state.psi_value += state.rate * input.dt_sec;
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

pub fn calculate_trend_gain(
    avg10: f32,
    avg60: f32,
    memory_psi: f32,
    math_config: &CpuMathConfig,
) -> f32 {
    let delta = avg10 - avg60;
    let base_gain = if delta > 0.0 { tanh(delta) } else { 0.0 };
    let memory_penalty = (memory_psi / 100.0) * math_config.memory_volatility_cost;
    base_gain / (1.0 + memory_penalty)
}

pub fn calculate_effective_pressure(
    load_demand: f32,
    trend_factor: f32,
    memory_psi: f32,
    io_psi: f32,
    math_config: &CpuMathConfig,
) -> f32 {
    let p_response = load_demand * (1.0 + trend_factor * math_config.trend_amplification);
    let ratio_stall = (memory_psi + io_psi) / (load_demand + 1.0);
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
    memory_psi: f32,
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
    let memory_dilation = 1.0 + (math_config.memory_granularity_scaling * (memory_psi / 100.0));
    let adjusted_latency = (final_latency * memory_dilation)
        .clamp(kernel_limits.min_latency_ns, kernel_limits.max_latency_ns);
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

pub fn calculate_migration_cost(
    delta_smooth: f32,
    p_eff: f32,
    memory_psi: f32,
    math_config: &CpuMathConfig,
    kernel_limits: &CpuKernelLimits,
) -> f32 {
    let x = (p_eff / 100.0).clamp(0.0, 1.0);
    let raw_mig = kernel_limits.min_migration_cost
        + (kernel_limits.max_migration_cost - kernel_limits.min_migration_cost) * (x * x);
    let volatility_ratio = (delta_smooth / 50.0).clamp(0.0, 1.0);
    let dynamic_cost = raw_mig * (1.0 - (volatility_ratio * 0.5));
    let pressure_scale = 1.0 + (math_config.memory_migration_alpha * (memory_psi / 100.0));
    (dynamic_cost * pressure_scale).clamp(
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
