//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct LoadState {
    pub psi_value: f64,
    pub rate: f64,
    pub load_history: [f64; 8],
    pub history_idx: usize,
    pub integral_accum: f64,
    pub prev_integral: f64,
    pub smoothed_integral: f64,
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

pub struct CpuTunables {
    pub min_latency_ns: f64,
    pub max_latency_ns: f64,
    pub min_granularity_ns: f64,
    pub max_granularity_ns: f64,
    pub min_wakeup_ns: f64,
    pub max_wakeup_ns: f64,
    pub min_migration_cost: f64,
    pub max_migration_cost: f64,
    pub min_nr_migrate: f64,
    pub max_nr_migrate: f64,
    pub nr_migrate_k: f64,
    pub min_walt_init_pct: f64,
    pub max_walt_init_pct: f64,
    pub min_uclamp_min: f64,
    pub max_uclamp_min: f64,
    pub uclamp_k: f64,
    pub uclamp_mid: f64,
    pub alpha_smooth: f64,
    pub response_gain: f64,
    pub stability_ratio: f64,
    pub gain_scheduling_alpha: f64,
    pub sigmoid_k: f64,
    pub sigmoid_mid: f64,
    pub decay_coeff: f64,
    pub latency_gran_ratio: f64,
    pub memory_migration_alpha: f64,
    pub memory_granularity_scaling: f64,
    pub memory_burst_penalty: f64,
    pub trend_boost_intensity: f64,
    pub transient_rate_threshold: f64,
    pub transient_diff_threshold: f64,
    pub transient_poll_interval: f64,
    pub spike_threshold: f64,
    pub spike_gain: f64,
    pub variance_sensitivity: f64,
    pub lookahead_time: f64,
    pub efficiency_gain: f64,
    pub temp_cost_weight: f64,
    pub bat_temp_weight: f64,
    pub bat_level_weight: f64,
    pub integral_learning_rate: f64,
    pub safe_temp_limit: f64,
    pub max_temp_limit: f64,
    pub stability_margin: f64,
    pub nis_threshold: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct DemandInput {
    pub target_psi: f64,
    pub dt_sec: f64,
    pub thermal_scale: f64,
    pub trend_factor: f64,
    pub integral_total: f64,
    pub integral_dot: f64,
    pub is_structural_break: bool,
}

fn calculate_regression_slope(state: &LoadState) -> f64 {
    const N: f64 = 8.0;
    const SUM_X: f64 = 28.0;
    const DENOMINATOR: f64 = 336.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    for i in 0..8 {
        let idx = (state.history_idx + i) % 8;
        let y = state.load_history[idx];
        let x = i as f64;
        sum_y += y;
        sum_xy += x * y;
    }
    let numerator = (N * sum_xy) - (SUM_X * sum_y);
    numerator / DENOMINATOR
}

pub fn update_integral_params(
    state: &mut LoadState,
    cpu_temp: f64,
    bat_temp: f64,
    bat_level: f64,
    dt_sec: f64,
    tunables: &CpuTunables,
) -> (f64, f64) {
    let temp_ratio = (cpu_temp / tunables.max_temp_limit).clamp(0.0, 1.5);
    let term_cpu = tunables.temp_cost_weight * temp_ratio.powi(2);
    let bat_stress = (bat_temp / 45.0).clamp(0.0, 1.0);
    let term_bat_temp = tunables.bat_temp_weight * bat_stress;
    let depletion = (100.0 - bat_level).max(0.0) / 100.0;
    let term_bat_cap = tunables.bat_level_weight * depletion.powi(3);
    let cost_heuristic = term_cpu + term_bat_temp + term_bat_cap;
    let limit_violation = (cpu_temp - tunables.safe_temp_limit).max(0.0);
    let integration_rate = tunables.integral_learning_rate * limit_violation;
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
    tunables: &CpuTunables,
) -> f64 {
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
    let deviation_gain = 1.0 + (tunables.variance_sensitivity * std_dev);
    let slope_per_tick = calculate_regression_slope(state);
    let load_rate = slope_per_tick / input.dt_sec.max(0.001);
    if load_rate.abs() > tunables.spike_threshold {
        state.rate += load_rate * tunables.spike_gain;
    }
    let prediction_target = input.target_psi + (load_rate * tunables.lookahead_time);
    let k_base = tunables.response_gain;
    let k_dynamic = k_base * (1.0 + (tunables.gain_scheduling_alpha * input.trend_factor));
    let k_final = k_dynamic * deviation_gain * input.thermal_scale.clamp(0.1, 1.0).powi(2);
    let displacement = prediction_target - state.psi_value;
    let prop_term = k_final * displacement;
    let mut limit_term = input.integral_total * state.psi_value;
    let max_possible_response = k_final * 100.0;
    limit_term = limit_term.min(max_possible_response * 1.5);
    let crit_damp = 2.0 * k_final.sqrt();
    let base_damp = crit_damp * tunables.stability_ratio;
    let rate_sq = state.rate.powi(2) + 0.001;
    let stability_damping_req =
        (0.5 * input.integral_dot.abs() * state.psi_value.powi(2)) / rate_sq;
    let c_stability = stability_damping_req.clamp(0.0, base_damp * 4.0) * tunables.stability_margin;
    let c_thermal_adjusted = base_damp / input.thermal_scale.clamp(0.1, 1.0).sqrt();
    let c_final = c_thermal_adjusted.max(c_stability);
    let deriv_term = c_final * state.rate;
    let net_drive = prop_term - deriv_term - limit_term;
    let rate_delta = net_drive;
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
    avg10: f64,
    avg60: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> f64 {
    let delta = avg10 - avg60;
    let base_gain = if delta > 0.0 { delta.tanh() } else { 0.0 };
    let memory_penalty = (memory_psi / 100.0) * tunables.memory_burst_penalty;
    base_gain / (1.0 + memory_penalty)
}

pub fn calculate_thermal_floor(thermal_scale: f64, tunables: &CpuTunables) -> f64 {
    let limit_ratio = (1.0 - thermal_scale).clamp(0.0, 1.0);
    tunables.min_latency_ns + (tunables.max_latency_ns - tunables.min_latency_ns) * limit_ratio
}

pub fn calculate_migration_cost(
    delta_smooth: f64,
    p_eff: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> f64 {
    let x = (p_eff / 100.0).clamp(0.0, 1.0);
    let raw_mig = tunables.min_migration_cost
        + (tunables.max_migration_cost - tunables.min_migration_cost) * (x * x);
    let burst_factor = (delta_smooth / 50.0).clamp(0.0, 1.0);
    let dynamic_cost = raw_mig * (1.0 - (burst_factor * 0.5));
    let pressure_scale = 1.0 + (tunables.memory_migration_alpha * (memory_psi / 100.0));
    (dynamic_cost * pressure_scale).clamp(
        tunables.min_migration_cost,
        tunables.max_migration_cost * 3.0,
    )
}

pub fn calculate_latency_and_granularity(
    p_eff: f64,
    load_demand: f64,
    thermal_floor_ns: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> (f64, f64) {
    let denom = 1.0 + (tunables.sigmoid_k * (p_eff - tunables.sigmoid_mid)).exp();
    let normal_latency =
        tunables.min_latency_ns + ((tunables.max_latency_ns - tunables.min_latency_ns) / denom);
    let latency_range = tunables.max_latency_ns - tunables.min_latency_ns;
    let effective_demand = (load_demand / 100.0).clamp(0.0, 1.0);
    let low_latency_target = tunables.max_latency_ns - (effective_demand * latency_range);
    let ideal_latency = normal_latency.min(low_latency_target);
    let final_latency = ideal_latency.max(thermal_floor_ns);
    let memory_dilation = 1.0 + (tunables.memory_granularity_scaling * (memory_psi / 100.0));
    let adjusted_latency = (final_latency * memory_dilation)
        .clamp(tunables.min_latency_ns, tunables.max_latency_ns * 1.5);
    let raw_gran = adjusted_latency * tunables.latency_gran_ratio;
    let final_gran = raw_gran
        .clamp(
            tunables.min_granularity_ns,
            tunables.max_granularity_ns * 1.5,
        )
        .min(adjusted_latency);
    (adjusted_latency, final_gran)
}

pub fn calculate_wakeup_granularity(p_eff: f64, tunables: &CpuTunables) -> f64 {
    let decay = (-tunables.decay_coeff * p_eff).exp();
    let raw_wake =
        tunables.min_wakeup_ns + (tunables.max_wakeup_ns - tunables.min_wakeup_ns) * decay;
    raw_wake.clamp(tunables.min_wakeup_ns, tunables.max_wakeup_ns)
}

pub fn calculate_nr_migrate(pressure: f64, tunables: &CpuTunables) -> f64 {
    let denominator = 1.0 + (tunables.nr_migrate_k * pressure);
    let range = tunables.max_nr_migrate - tunables.min_nr_migrate;
    tunables.min_nr_migrate + (range / denominator)
}

pub fn calculate_walt_init(pressure: f64, tunables: &CpuTunables) -> f64 {
    let ratio = pressure / 100.0;
    let load_curve = ratio * ratio;
    let range = tunables.max_walt_init_pct - tunables.min_walt_init_pct;
    tunables.min_walt_init_pct + (range * load_curve)
}

pub fn calculate_uclamp_min(pressure: f64, thermal_scale: f64, tunables: &CpuTunables) -> f64 {
    let exponent = -tunables.uclamp_k * (pressure - tunables.uclamp_mid);
    let denominator = 1.0 + exponent.exp();
    let range = tunables.max_uclamp_min - tunables.min_uclamp_min;
    let ideal_uclamp = tunables.min_uclamp_min + (range / denominator);
    ideal_uclamp * thermal_scale
}

pub fn smooth_delta(current_delta: f64, prev_smooth: f64, tunables: &CpuTunables) -> f64 {
    tunables.alpha_smooth * current_delta + (1.0 - tunables.alpha_smooth) * prev_smooth
}

pub fn calculate_effective_pressure(
    load_demand: f64,
    trend_factor: f64,
    memory_psi: f64,
    io_psi: f64,
    tunables: &CpuTunables,
) -> f64 {
    let p_response = load_demand * (1.0 + trend_factor * tunables.trend_boost_intensity);
    let ratio_stall = (memory_psi + io_psi) / (load_demand + 1.0);
    let throughput_ratio = 1.0 / (1.0 + (ratio_stall * tunables.efficiency_gain));
    p_response * throughput_ratio
}

pub fn is_transient(state: &LoadState, target_psi: f64, tunables: &CpuTunables) -> bool {
    state.rate.abs() > tunables.transient_rate_threshold
        || (state.psi_value - target_psi).abs() > tunables.transient_diff_threshold
}