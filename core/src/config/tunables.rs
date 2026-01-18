//! Author: [Seclususs](https://github.com/seclususs)

pub const MIN_LATENCY_NS: u64 = 8_000_000;
pub const MAX_LATENCY_NS: u64 = 16_000_000;
pub const MIN_GRANULARITY_NS: u64 = 6_000_000;
pub const MAX_GRANULARITY_NS: u64 = 12_000_000;
pub const MIN_WAKEUP_NS: u64 = 3_000_000;
pub const MAX_WAKEUP_NS: u64 = 6_000_000;
pub const MIN_MIGRATION_COST: u64 = 200_000;
pub const MAX_MIGRATION_COST: u64 = 600_000;
pub const MIN_WALT_INIT_PCT: u64 = 15;
pub const MAX_WALT_INIT_PCT: u64 = 45;
pub const MIN_UCLAMP_MIN: u64 = 0;
pub const MAX_UCLAMP_MIN: u64 = 256;

pub const MIN_SWAPPINESS: u64 = 20;
pub const MAX_SWAPPINESS: u64 = 60;
pub const MIN_VFS: u64 = 80;
pub const MAX_VFS: u64 = 200;

pub const MAX_READ_AHEAD: u64 = 256;
pub const MIN_READ_AHEAD: u64 = 128;
pub const MAX_NR_REQUESTS: u64 = 256;
pub const MIN_NR_REQUESTS: u64 = 128;