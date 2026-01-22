//! Author: [Seclususs](https://github.com/seclususs)

pub use crate::resources::discovery::{
    get_cpu_temp_path, get_diskstats_path, get_nr_requests_path, get_read_ahead_path,
};

pub const K_PSI_CPU_PATH: &str = "/proc/pressure/cpu";
pub const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
pub const K_PSI_IO_PATH: &str = "/proc/pressure/io";

pub const K_SCHED_LATENCY_NS: &str = "/proc/sys/kernel/sched_latency_ns";
pub const K_SCHED_MIN_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_min_granularity_ns";
pub const K_SCHED_WAKEUP_GRANULARITY_NS: &str = "/proc/sys/kernel/sched_wakeup_granularity_ns";
pub const K_SCHED_MIGRATION_COST_NS: &str = "/proc/sys/kernel/sched_migration_cost_ns";
pub const K_SCHED_WALT_INIT_TASK_LOAD_PCT: &str = "/proc/sys/kernel/sched_walt_init_task_load_pct";
pub const K_SCHED_UCLAMP_UTIL_MIN: &str = "/proc/sys/kernel/sched_uclamp_util_min";

pub const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
pub const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";

pub const K_BATTERY_TEMP_PATH: &str = "/sys/class/power_supply/battery/temp";
pub const K_BATTERY_CAPACITY_PATH: &str = "/sys/class/power_supply/battery/capacity";
pub const K_VMSTAT_PATH: &str = "/proc/vmstat";

pub const K_TOUCH_DEVICE_PATH: &str = "/dev/input/event3";
