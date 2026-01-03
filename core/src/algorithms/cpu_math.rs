//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct PhysicsState {
    pub pos: f64,
    pub vel: f64,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self { pos: 0.0, vel: 0.0 }
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
}

pub fn calculate_physics_urgency(
    state: &mut PhysicsState,
    target_psi: f64,
    dt_sec: f64,
    damping_factor: f64,
    stress_gain: f64,
    tunables: &CpuTunables,
) -> f64 {
    let k_base = tunables.spring_stiffness;
    let k_dynamic = k_base * (1.0 + (tunables.gain_scheduling_alpha * stress_gain));
    let thermal_health = damping_factor.clamp(0.1, 1.0);
    let k_final = k_dynamic * thermal_health.powi(2);
    let c_critical = 2.0 * k_final.sqrt();
    let c_base = c_critical * tunables.damping_ratio;
    let c_final = c_base / thermal_health.sqrt();
    let displacement = target_psi - state.pos;
    let spring_force = k_final * displacement;
    let damping_force = -c_final * state.vel;
    let total_force = spring_force + damping_force;
    let acceleration = total_force;
    state.vel += acceleration * dt_sec;
    state.pos += state.vel * dt_sec;
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
    tunables: &CpuTunables,
) -> f64 {
    physics_urgency * (1.0 + trend_gain * tunables.trend_boost_intensity)
}

pub fn is_animating(state: &PhysicsState, target_psi: f64, tunables: &CpuTunables) -> bool {
    state.vel.abs() > tunables.animation_vel_threshold
        || (state.pos - target_psi).abs() > tunables.animation_pos_threshold
}