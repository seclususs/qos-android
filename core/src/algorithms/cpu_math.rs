//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct PhysicsState {
    pub pos: f64,
    pub vel: f64,
    pub last_psi: f64,
    pub psi_history: [f64; 8],
    pub history_idx: usize,
    pub lambda_accum: f64,
    pub prev_lambda: f64,
    pub smoothed_lambda: f64,
    pub first_run: bool,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            pos: 0.0,
            vel: 0.0,
            last_psi: 0.0,
            psi_history: [0.0; 8],
            history_idx: 0,
            lambda_accum: 0.0,
            prev_lambda: 0.0,
            smoothed_lambda: 0.0,
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
    pub trend_factor: f64,
    pub alpha_smooth: f64,
    pub spring_stiffness: f64,
    pub damping_ratio: f64,
    pub gain_scheduling_alpha: f64,
    pub sigmoid_k: f64,
    pub sigmoid_mid: f64,
    pub decay_coeff: f64,
    pub latency_gran_ratio: f64,
    pub memory_migration_alpha: f64,
    pub memory_granularity_scaling: f64,
    pub memory_burst_penalty: f64,
    pub trend_boost_intensity: f64,
    pub animation_vel_threshold: f64,
    pub animation_pos_threshold: f64,
    pub animation_poll_interval: f64,
    pub impulse_threshold: f64,
    pub impulse_factor: f64,
    pub variance_sensitivity: f64,
    pub lookahead_time: f64,
    pub efficiency_gain: f64,
    pub energy_cost_alpha: f64,
    pub energy_cost_beta: f64,
    pub energy_cost_gamma: f64,
    pub constraint_learning_rate: f64,
    pub safe_temp_limit: f64,
    pub max_temp_limit: f64,
    pub lyapunov_margin: f64,
    pub nis_threshold: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct UrgencyInput {
    pub target_psi: f64,
    pub dt_sec: f64,
    pub damping_factor: f64,
    pub stress_gain: f64,
    pub lambda_total: f64,
    pub lambda_dot: f64,
    pub is_structural_break: bool,
}

fn calculate_regression_slope(state: &PhysicsState) -> f64 {
    const N: f64 = 8.0;
    const SUM_X: f64 = 28.0;
    const DENOMINATOR: f64 = 336.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    for i in 0..8 {
        let idx = (state.history_idx + i) % 8;
        let y = state.psi_history[idx];
        let x = i as f64;
        sum_y += y;
        sum_xy += x * y;
    }
    let numerator = (N * sum_xy) - (SUM_X * sum_y);
    numerator / DENOMINATOR
}

pub fn update_hamiltonian_params(
    state: &mut PhysicsState,
    cpu_temp: f64,
    bat_temp: f64,
    bat_level: f64,
    dt_sec: f64,
    tunables: &CpuTunables,
) -> (f64, f64) {
    let temp_ratio = (cpu_temp / tunables.max_temp_limit).clamp(0.0, 1.5);
    let term_cpu = tunables.energy_cost_alpha * temp_ratio.powi(2);
    let bat_stress = (bat_temp / 45.0).clamp(0.0, 1.0);
    let term_bat_temp = tunables.energy_cost_beta * bat_stress;
    let depletion = (100.0 - bat_level).max(0.0) / 100.0;
    let term_bat_cap = tunables.energy_cost_gamma * depletion.powi(3);
    let lambda_heuristic = term_cpu + term_bat_temp + term_bat_cap;
    let constraint_violation = (cpu_temp - tunables.safe_temp_limit).max(0.0);
    let lambda_rate = tunables.constraint_learning_rate * constraint_violation;
    state.lambda_accum += lambda_rate * dt_sec;
    if constraint_violation <= 0.0 {
        state.lambda_accum *= 0.98;
    }
    state.lambda_accum = state.lambda_accum.clamp(0.0, 200.0);
    let lambda_raw = lambda_heuristic + state.lambda_accum;
    if state.first_run {
        state.smoothed_lambda = lambda_raw;
        state.prev_lambda = lambda_raw;
        state.first_run = false;
        return (lambda_raw, 0.0);
    }
    state.smoothed_lambda = (state.smoothed_lambda * 0.8) + (lambda_raw * 0.2);
    let lambda_dot = if dt_sec > 0.0 {
        (state.smoothed_lambda - state.prev_lambda) / dt_sec
    } else {
        0.0
    };
    state.prev_lambda = state.smoothed_lambda;
    (state.smoothed_lambda, lambda_dot)
}

pub fn calculate_physics_urgency(
    state: &mut PhysicsState,
    input: UrgencyInput,
    tunables: &CpuTunables,
) -> f64 {
    if input.is_structural_break {
        for i in 0..8 {
            state.psi_history[i] = input.target_psi;
        }
    }
    state.psi_history[state.history_idx] = input.target_psi;
    state.history_idx = (state.history_idx + 1) % 8;
    let mut sum = 0.0;
    for val in state.psi_history.iter() {
        sum += val;
    }
    let mean = sum / 8.0;
    let mut variance_sum = 0.0;
    for val in state.psi_history.iter() {
        variance_sum += (val - mean).powi(2);
    }
    let std_dev = (variance_sum / 8.0).sqrt();
    let stiffness_mod = 1.0 + (tunables.variance_sensitivity * std_dev);
    let slope_per_tick = calculate_regression_slope(state);
    let v_load = slope_per_tick / input.dt_sec.max(0.001);
    if v_load.abs() > tunables.impulse_threshold {
        state.vel += v_load * tunables.impulse_factor;
    }
    let ghost_target = input.target_psi + (v_load * tunables.lookahead_time);
    let k_base = tunables.spring_stiffness;
    let k_dynamic = k_base * (1.0 + (tunables.gain_scheduling_alpha * input.stress_gain));
    let k_final = k_dynamic * stiffness_mod * input.damping_factor.clamp(0.1, 1.0).powi(2);
    let displacement = ghost_target - state.pos;
    let spring_force = k_final * displacement;
    let mut hamiltonian_force = input.lambda_total * state.pos;
    let max_possible_spring = k_final * 100.0;
    hamiltonian_force = hamiltonian_force.min(max_possible_spring * 1.5);
    let c_critical = 2.0 * k_final.sqrt();
    let c_base = c_critical * tunables.damping_ratio;
    let velocity_sq = state.vel.powi(2) + 0.001;
    let lyapunov_damping_req = (0.5 * input.lambda_dot.abs() * state.pos.powi(2)) / velocity_sq;
    let c_lyapunov = lyapunov_damping_req.clamp(0.0, c_base * 4.0) * tunables.lyapunov_margin;
    let c_thermal_adjusted = c_base / input.damping_factor.clamp(0.1, 1.0).sqrt();
    let c_final = c_thermal_adjusted.max(c_lyapunov);
    let damping_force = c_final * state.vel;
    let total_force = spring_force - damping_force - hamiltonian_force;
    let acceleration = total_force;
    state.vel += acceleration * input.dt_sec;
    state.pos += state.vel * input.dt_sec;
    state.last_psi = input.target_psi;
    if state.pos < 0.0 {
        state.pos = 0.0;
        state.vel = 0.0;
    }
    if state.pos > 500.0 {
        state.pos = 500.0;
        state.vel = 0.0;
    }
    state.pos
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

pub fn calculate_thermal_floor(damping_factor: f64, tunables: &CpuTunables) -> f64 {
    let fatigue = (1.0 - damping_factor).clamp(0.0, 1.0);
    tunables.min_latency_ns + (tunables.max_latency_ns - tunables.min_latency_ns) * fatigue
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
    let kinetic_mig = raw_mig * (1.0 - (burst_factor * 0.5));
    let inertia_factor = 1.0 + (tunables.memory_migration_alpha * (memory_psi / 100.0));
    (kinetic_mig * inertia_factor).clamp(
        tunables.min_migration_cost,
        tunables.max_migration_cost * 3.0,
    )
}

pub fn calculate_latency_and_granularity(
    p_eff: f64,
    physics_urgency: f64,
    thermal_floor_ns: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> (f64, f64) {
    let denom = 1.0 + (tunables.sigmoid_k * (p_eff - tunables.sigmoid_mid)).exp();
    let normal_latency =
        tunables.min_latency_ns + ((tunables.max_latency_ns - tunables.min_latency_ns) / denom);
    let latency_range = tunables.max_latency_ns - tunables.min_latency_ns;
    let effective_urgency = (physics_urgency / 100.0).clamp(0.0, 1.0);
    let burst_latency = tunables.max_latency_ns - (effective_urgency * latency_range);
    let ideal_latency = normal_latency.min(burst_latency);
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
    let inertia_curve = ratio * ratio;
    let range = tunables.max_walt_init_pct - tunables.min_walt_init_pct;
    tunables.min_walt_init_pct + (range * inertia_curve)
}

pub fn calculate_uclamp_min(pressure: f64, damping_factor: f64, tunables: &CpuTunables) -> f64 {
    let exponent = -tunables.uclamp_k * (pressure - tunables.uclamp_mid);
    let denominator = 1.0 + exponent.exp();
    let range = tunables.max_uclamp_min - tunables.min_uclamp_min;
    let ideal_uclamp = tunables.min_uclamp_min + (range / denominator);
    ideal_uclamp * damping_factor
}

pub fn smooth_delta(current_delta: f64, prev_smooth: f64, tunables: &CpuTunables) -> f64 {
    tunables.alpha_smooth * current_delta + (1.0 - tunables.alpha_smooth) * prev_smooth
}

pub fn calculate_effective_pressure(
    physics_urgency: f64,
    trend_gain: f64,
    memory_psi: f64,
    io_psi: f64,
    tunables: &CpuTunables,
) -> f64 {
    let p_spring = physics_urgency * (1.0 + trend_gain * tunables.trend_boost_intensity);
    let ratio_stall = (memory_psi + io_psi) / (physics_urgency + 1.0);
    let eta_cpu = 1.0 / (1.0 + (ratio_stall * tunables.efficiency_gain));
    p_spring * eta_cpu
}

pub fn is_animating(state: &PhysicsState, target_psi: f64, tunables: &CpuTunables) -> bool {
    state.vel.abs() > tunables.animation_vel_threshold
        || (state.pos - target_psi).abs() > tunables.animation_pos_threshold
}