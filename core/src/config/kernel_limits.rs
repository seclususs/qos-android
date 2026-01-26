//! Author: [Seclususs](https://github.com/seclususs)

use crate::utils::tier::DeviceTier;

#[derive(Debug, Clone, Copy)]
pub struct CpuKernelLimitsConfig {
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
}

impl Default for CpuKernelLimitsConfig {
    fn default() -> Self {
        let tier = DeviceTier::get();
        match tier {
            DeviceTier::Flagship => Self {
                min_latency_ns: 8_000_000,
                max_latency_ns: 18_000_000,
                min_granularity_ns: 2_500_000,
                max_granularity_ns: 6_000_000,
                min_wakeup_ns: 1_500_000,
                max_wakeup_ns: 6_000_000,
                min_migration_cost: 200_000,
                max_migration_cost: 600_000,
                min_walt_init_pct: 10,
                max_walt_init_pct: 40,
                min_uclamp_min: 0,
                max_uclamp_min: 384,
            },
            DeviceTier::MidRange => Self {
                min_latency_ns: 9_000_000,
                max_latency_ns: 16_000_000,
                min_granularity_ns: 2_750_000,
                max_granularity_ns: 5_500_000,
                min_wakeup_ns: 1_750_000,
                max_wakeup_ns: 5_500_000,
                min_migration_cost: 225_000,
                max_migration_cost: 550_000,
                min_walt_init_pct: 12,
                max_walt_init_pct: 37,
                min_uclamp_min: 0,
                max_uclamp_min: 320,
            },
            DeviceTier::LowEnd => Self {
                min_latency_ns: 10_000_000,
                max_latency_ns: 15_000_000,
                min_granularity_ns: 3_000_000,
                max_granularity_ns: 5_000_000,
                min_wakeup_ns: 2_000_000,
                max_wakeup_ns: 5_000_000,
                min_migration_cost: 250_000,
                max_migration_cost: 500_000,
                min_walt_init_pct: 15,
                max_walt_init_pct: 35,
                min_uclamp_min: 0,
                max_uclamp_min: 256,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StorageKernelLimitsConfig {
    pub max_read_ahead: u64,
    pub min_read_ahead: u64,
    pub max_nr_requests: u64,
    pub min_nr_requests: u64,
}

impl Default for StorageKernelLimitsConfig {
    fn default() -> Self {
        let tier = DeviceTier::get();
        match tier {
            DeviceTier::Flagship => Self {
                max_read_ahead: 1024,
                min_read_ahead: 128,
                max_nr_requests: 256,
                min_nr_requests: 64,
            },
            DeviceTier::MidRange => Self {
                max_read_ahead: 768,
                min_read_ahead: 128,
                max_nr_requests: 192,
                min_nr_requests: 64,
            },
            DeviceTier::LowEnd => Self {
                max_read_ahead: 512,
                min_read_ahead: 128,
                max_nr_requests: 128,
                min_nr_requests: 64,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalConfig {
    pub cpu_config: CpuKernelLimitsConfig,
    pub storage_config: StorageKernelLimitsConfig,
}
