//! Author: [Seclususs](https://github.com/seclususs)

pub struct CpuTunables {
    pub min_latency_ns: f64,
    pub max_latency_ns: f64,
    pub min_granularity_ns: f64,
    pub max_granularity_ns: f64,
    pub min_wakeup_ns: f64,
    pub max_wakeup_ns: f64,
    pub min_migration_cost: f64,
    pub max_migration_cost: f64,
    pub trend_factor: f64,
    pub alpha_smooth: f64,
    pub burst_threshold: f64,
    pub sigmoid_k: f64,
    pub sigmoid_mid: f64,
    pub decay_coeff: f64,
    pub latency_gran_ratio: f64,
    pub memory_migration_alpha: f64,
    pub memory_granularity_scaling: f64,
    pub memory_burst_penalty: f64,
}

pub fn calculate_trend_gain(
    avg10: f64,
    avg60: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> f64 {
    let delta = avg10 - avg60;
    let base_gain = 1.0 + tunables.trend_factor * delta.tanh();
    let memory_penalty = (memory_psi / 100.0) * tunables.memory_burst_penalty;
    base_gain / (1.0 + memory_penalty)
}

pub fn calculate_thermal_floor(avg60: f64, tunables: &CpuTunables) -> f64 {
    let fatigue = (avg60 / 100.0).clamp(0.0, 1.0);
    tunables.min_latency_ns + (tunables.max_latency_ns - tunables.min_latency_ns) * fatigue
}

pub fn calculate_migration_cost(
    delta_smooth: f64,
    p_eff: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> f64 {
    let base_cost = if delta_smooth > tunables.burst_threshold {
        tunables.min_migration_cost
    } else {
        let x = (p_eff / 100.0).clamp(0.0, 1.0);
        let raw_mig = tunables.min_migration_cost
            + (tunables.max_migration_cost - tunables.min_migration_cost) * (x * x);
        raw_mig.clamp(tunables.min_migration_cost, tunables.max_migration_cost)
    };
    let inertia_factor = 1.0 + (tunables.memory_migration_alpha * (memory_psi / 100.0));
    (base_cost * inertia_factor).min(tunables.max_migration_cost * 3.0)
}

pub fn calculate_latency_and_granularity(
    p_eff: f64,
    avg10: f64,
    avg300: f64,
    thermal_floor: f64,
    memory_psi: f64,
    tunables: &CpuTunables,
) -> (f64, f64) {
    let load_delta = avg10 - avg300;
    if load_delta > tunables.burst_threshold && memory_psi < 20.0 {
        let burst_latency = tunables.min_latency_ns;
        let burst_gran = tunables.min_granularity_ns;
        return (burst_latency, burst_gran);
    }
    if load_delta.abs() < 5.0 && avg10 > 40.0 {
        let sustained_latency = (thermal_floor * 1.5).min(tunables.max_latency_ns);
        let sustained_gran = (sustained_latency * tunables.latency_gran_ratio)
            .clamp(tunables.min_granularity_ns, tunables.max_granularity_ns);
        return (sustained_latency, sustained_gran);
    }
    let denom = 1.0 + (tunables.sigmoid_k * (p_eff - tunables.sigmoid_mid)).exp();
    let raw_latency = thermal_floor + ((tunables.max_latency_ns - thermal_floor) / denom);
    let memory_time_dilation = 1.0 + (tunables.memory_granularity_scaling * (memory_psi / 100.0));
    let adjusted_latency =
        (raw_latency * memory_time_dilation).clamp(thermal_floor, tunables.max_latency_ns * 1.5);
    let raw_gran = adjusted_latency * tunables.latency_gran_ratio;
    let target_min_gran = raw_gran.clamp(
        tunables.min_granularity_ns,
        tunables.max_granularity_ns * 1.5,
    );
    let final_gran = target_min_gran.min(adjusted_latency);
    (adjusted_latency, final_gran)
}

pub fn calculate_wakeup_granularity(p_eff: f64, tunables: &CpuTunables) -> f64 {
    let decay = (-tunables.decay_coeff * p_eff).exp();
    let raw_wake =
        tunables.min_wakeup_ns + (tunables.max_wakeup_ns - tunables.min_wakeup_ns) * decay;
    raw_wake.clamp(tunables.min_wakeup_ns, tunables.max_wakeup_ns)
}

pub fn smooth_delta(current_delta: f64, prev_smooth: f64, tunables: &CpuTunables) -> f64 {
    tunables.alpha_smooth * current_delta + (1.0 - tunables.alpha_smooth) * prev_smooth
}