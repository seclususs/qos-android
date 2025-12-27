//! Author: [Seclususs](https://github.com/seclususs)

pub const K_PSI_CPU_PATH: &str = "/proc/pressure/cpu";
pub const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
pub const K_PSI_IO_PATH: &str = "/proc/pressure/io";

pub const K_SCHED_LATENCY_NS: &str = "/proc/sys/kernel/sched_latency_ns";
pub const K_SCHED_MIN_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_min_granularity_ns";
pub const K_SCHED_WAKEUP_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_wakeup_granularity_ns";
pub const K_SCHED_MIGRATION_COST_NS: &str = "/proc/sys/kernel/sched_migration_cost_ns";
pub const K_PERF_CPU_TIME_MAX_PERCENT: &str = "/proc/sys/kernel/perf_cpu_time_max_percent";

pub const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
pub const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";
pub const K_DIRTY_RATIO: &str = "/proc/sys/vm/dirty_ratio";
pub const K_DIRTY_BG_RATIO: &str = "/proc/sys/vm/dirty_background_ratio";
pub const K_DIRTY_EXPIRE_CENTISECS: &str = "/proc/sys/vm/dirty_expire_centisecs";
pub const K_STAT_INTERVAL: &str = "/proc/sys/vm/stat_interval";
pub const K_WATERMARK_SCALE_FACTOR: &str = "/proc/sys/vm/watermark_scale_factor";
pub const K_EXTFRAG_THRESHOLD: &str = "/proc/sys/vm/extfrag_threshold";
pub const K_DIRTY_WRITEBACK_CENTISECS: &str = "/proc/sys/vm/dirty_writeback_centisecs";
pub const K_PAGE_CLUSTER: &str = "/proc/sys/vm/page-cluster";

pub const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
pub const K_NR_REQUESTS_PATH: &str = "/sys/block/mmcblk0/queue/nr_requests";
pub const K_FIFO_BATCH_PATH: &str = "/sys/block/mmcblk0/queue/iosched/fifo_batch";

pub const K_BATTERY_TEMP_PATH: &str = "/sys/class/power_supply/battery/temp";
pub const K_THERMAL_ZONE0_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";