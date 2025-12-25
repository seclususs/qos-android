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
    pub min_perf_cpu_percent: f64,
    pub max_perf_cpu_percent: f64,
    pub trend_factor: f64,
    pub critical_threshold: f64,
    pub alpha_smooth: f64,
    pub burst_threshold: f64,
    pub sigmoid_k: f64,
    pub sigmoid_mid: f64,
    pub decay_coeff: f64,
    pub latency_gran_ratio: f64,
}

pub fn calculate_trend_gain(avg10: f64, avg60: f64, tunables: &CpuTunables) -> f64 {
    let delta = avg10 - avg60;
    1.0 + tunables.trend_factor * delta.tanh()
}

pub fn calculate_thermal_floor(sustained_load: f64, tunables: &CpuTunables) -> f64 {
    let fatigue = (sustained_load / 100.0).clamp(0.0, 1.0);
    tunables.min_latency_ns + (tunables.max_latency_ns - tunables.min_latency_ns) * fatigue
}

pub fn calculate_perf_limit(avg10: f64, tunables: &CpuTunables) -> f64 {
    let saturation = (avg10 / tunables.critical_threshold).clamp(0.0, 1.0);
    let min_limit = tunables.min_perf_cpu_percent;
    let max_limit = tunables.max_perf_cpu_percent;
    let limit = min_limit + (max_limit - min_limit) * (1.0 - saturation);
    limit.clamp(min_limit, max_limit)
}

pub fn calculate_migration_cost(delta_smooth: f64, p_eff: f64, tunables: &CpuTunables) -> f64 {
    if delta_smooth > tunables.burst_threshold {
        tunables.min_migration_cost
    } else {
        let x = (p_eff / 100.0).clamp(0.0, 1.0);
        let raw_mig = tunables.min_migration_cost + (tunables.max_migration_cost - tunables.min_migration_cost) * (x * x);
        raw_mig.clamp(tunables.min_migration_cost, tunables.max_migration_cost)
    }
}

pub fn calculate_latency_and_granularity(p_eff: f64, thermal_floor: f64, tunables: &CpuTunables) -> (f64, f64) {
    let denom = 1.0 + (tunables.sigmoid_k * (p_eff - tunables.sigmoid_mid)).exp();
    let raw_latency = thermal_floor + ((tunables.max_latency_ns - thermal_floor) / denom);
    let target_latency = raw_latency.clamp(thermal_floor, tunables.max_latency_ns);
    let raw_gran = target_latency * tunables.latency_gran_ratio;
    let target_min_gran = raw_gran.clamp(tunables.min_granularity_ns, tunables.max_granularity_ns);
    (target_latency, target_min_gran)
}

pub fn calculate_wakeup_granularity(p_eff: f64, tunables: &CpuTunables) -> f64 {
    let decay = (-tunables.decay_coeff * p_eff).exp();
    let raw_wake = tunables.min_wakeup_ns + (tunables.max_wakeup_ns - tunables.min_wakeup_ns) * decay;
    raw_wake.clamp(tunables.min_wakeup_ns, tunables.max_wakeup_ns)
}

pub fn smooth_delta(current_delta: f64, prev_smooth: f64, tunables: &CpuTunables) -> f64 {
    tunables.alpha_smooth * current_delta + (1.0 - tunables.alpha_smooth) * prev_smooth
}