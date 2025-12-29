//! Author: [Seclususs](https://github.com/seclususs)

pub const MIN_LATENCY_NS: u64 = 8_000_000;
pub const MAX_LATENCY_NS: u64 = 16_000_000;
pub const MIN_GRANULARITY_NS: u64 = 6_000_000;
pub const MAX_GRANULARITY_NS: u64 = 12_000_000;
pub const MIN_WAKEUP_NS: u64 = 3_000_000;
pub const MAX_WAKEUP_NS: u64 = 6_000_000;
pub const MIN_MIGRATION_COST: u64 = 200_000;
pub const MAX_MIGRATION_COST: u64 = 400_000;

pub const MIN_SWAPPINESS: u64 = 20;
pub const MAX_SWAPPINESS: u64 = 60;
pub const MIN_VFS: u64 = 80;
pub const MAX_VFS: u64 = 200;
pub const MIN_DIRTY: u64 = 10;
pub const MAX_DIRTY: u64 = 20;
pub const MIN_DIRTY_BG: u64 = 5;
pub const MAX_DIRTY_BG: u64 = 10;
pub const MIN_DIRTY_EXPIRE: u64 = 1000;
pub const MAX_DIRTY_EXPIRE: u64 = 2000;
pub const MIN_STAT_INTERVAL: u64 = 1;
pub const MAX_STAT_INTERVAL: u64 = 5;
pub const MIN_WATERMARK_SCALE: u64 = 8;
pub const MAX_WATERMARK_SCALE: u64 = 15;
pub const MIN_EXTFRAG_THRESHOLD: u64 = 400;
pub const MAX_EXTFRAG_THRESHOLD: u64 = 600;
pub const MIN_DIRTY_WRITEBACK: u64 = 300;
pub const MAX_DIRTY_WRITEBACK: u64 = 1000;
pub const MIN_PAGE_CLUSTER: u64 = 0;
pub const MAX_PAGE_CLUSTER: u64 = 1;

pub const MAX_READ_AHEAD: u64 = 256;
pub const MIN_READ_AHEAD: u64 = 128;
pub const MAX_NR_REQUESTS: u64 = 256;
pub const MIN_NR_REQUESTS: u64 = 128; 
pub const MIN_FIFO_BATCH: u64 = 8;
pub const MAX_FIFO_BATCH: u64 = 16;