//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct CpuConfig {
    pub min_latency_ns: u64,
    pub max_latency_ns: u64,
    pub min_granularity_ns: u64,
    pub max_granularity_ns: u64,
    pub min_wakeup_ns: u64,
    pub max_wakeup_ns: u64,
    pub min_migration_cost: u64,
    pub max_migration_cost: u64,
    pub min_walt_init_pct: u64,
    pub max_walt_init_pct: u64,
    pub min_uclamp_min: u64,
    pub max_uclamp_min: u64,
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

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            min_latency_ns: 8_000_000,
            max_latency_ns: 16_000_000,
            min_granularity_ns: 6_000_000,
            max_granularity_ns: 12_000_000,
            min_wakeup_ns: 3_000_000,
            max_wakeup_ns: 6_000_000,
            min_migration_cost: 200_000,
            max_migration_cost: 600_000,
            min_walt_init_pct: 15,
            max_walt_init_pct: 45,
            min_uclamp_min: 0,
            max_uclamp_min: 256,
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
pub struct MemoryConfig {
    pub min_swappiness: u64,
    pub max_swappiness: u64,
    pub min_vfs: u64,
    pub max_vfs: u64,
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

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            min_swappiness: 20,
            max_swappiness: 60,
            min_vfs: 80,
            max_vfs: 200,
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

#[derive(Debug, Clone, Copy)]
pub struct StorageConfig {
    pub max_read_ahead: u64,
    pub min_read_ahead: u64,
    pub max_nr_requests: u64,
    pub min_nr_requests: u64,
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

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_read_ahead: 256,
            min_read_ahead: 128,
            max_nr_requests: 256,
            min_nr_requests: 128,
            min_req_size_kb: 32.0,
            max_req_size_kb: 256.0,
            write_cost_factor: 5.0,
            target_latency_base_ms: 75.0,
            hysteresis_threshold: 0.15,
            critical_threshold_psi: 40.0,
            queue_pressure_low: 1.0,
            queue_pressure_high: 4.0,
            smoothing_factor: 0.25,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalConfig {
    pub cpu: CpuConfig,
    pub memory: MemoryConfig,
    pub storage: StorageConfig,
}
