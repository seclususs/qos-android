//! Author: [Seclususs](https://github.com/seclususs)

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
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryKernelLimitsConfig {
    pub min_swappiness: u64,
    pub max_swappiness: u64,
    pub min_vfs: u64,
    pub max_vfs: u64,
}

impl Default for MemoryKernelLimitsConfig {
    fn default() -> Self {
        Self {
            min_swappiness: 20,
            max_swappiness: 60,
            min_vfs: 80,
            max_vfs: 200,
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
        Self {
            max_read_ahead: 256,
            min_read_ahead: 128,
            max_nr_requests: 256,
            min_nr_requests: 128,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalConfig {
    pub cpu_config: CpuKernelLimitsConfig,
    pub memory_config: MemoryKernelLimitsConfig,
    pub storage_config: StorageKernelLimitsConfig,
}
