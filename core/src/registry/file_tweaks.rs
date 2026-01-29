//! Author: [Seclususs](https://github.com/seclususs)

use std::borrow;

#[derive(Clone)]
pub struct FileTweak {
    pub path: borrow::Cow<'static, str>,
    pub value: &'static str,
}

impl FileTweak {
    pub const fn new_static(path: &'static str, value: &'static str) -> Self {
        Self {
            path: borrow::Cow::Borrowed(path),
            value,
        }
    }
    pub fn new_dynamic(path: String, value: &'static str) -> Self {
        Self {
            path: borrow::Cow::Owned(path),
            value,
        }
    }
}

pub fn generate_file_tweaks() -> Vec<FileTweak> {
    let mut tweaks = Vec::with_capacity(40);
    tweaks.extend_from_slice(&[
        FileTweak::new_static("/proc/sys/vm/oom_dump_tasks", "0"),
        FileTweak::new_static("/proc/sys/vm/swappiness", "40"),
        FileTweak::new_static("/proc/sys/vm/vfs_cache_pressure", "100"),
        FileTweak::new_static("/proc/sys/kernel/printk", "0 0 0 0"),
        FileTweak::new_static("/proc/sys/kernel/printk_devkmsg", "off"),
        FileTweak::new_static("/proc/sys/kernel/dmesg_restrict", "1"),
        FileTweak::new_static("/proc/sys/kernel/sched_child_runs_first", "0"),
        FileTweak::new_static("/proc/sys/kernel/sched_tunable_scaling", "1"),
        FileTweak::new_static("/proc/sys/kernel/pid_max", "65536"),
        FileTweak::new_static("/proc/sys/kernel/sched_schedstats", "0"),
        FileTweak::new_static("/proc/sys/kernel/perf_event_paranoid", "2"),
        FileTweak::new_static("/proc/sys/kernel/perf_cpu_time_max_percent", "1"),
        FileTweak::new_static("/proc/sys/kernel/sched_stune_task_threshold", "0"),
        FileTweak::new_static("/proc/sys/fs/lease-break-time", "10"),
        FileTweak::new_static("/proc/sys/fs/file-max", "524288"),
        FileTweak::new_static("/proc/sys/fs/protected_symlinks", "1"),
        FileTweak::new_static("/proc/sys/fs/protected_hardlinks", "1"),
        FileTweak::new_static("/proc/sys/net/ipv4/tcp_notsent_lowat", "16384"),
        FileTweak::new_static("/proc/sys/net/core/netdev_max_backlog", "2000"),
        FileTweak::new_static("/proc/sys/net/ipv4/tcp_slow_start_after_idle", "0"),
        FileTweak::new_static("/proc/sys/net/ipv4/tcp_tw_reuse", "1"),
        FileTweak::new_static("/proc/sys/net/core/netdev_budget", "300"),
        FileTweak::new_static("/proc/sys/net/ipv4/ip_dynaddr", "1"),
        FileTweak::new_static("/proc/sys/net/ipv4/tcp_keepalive_time", "1800"),
        FileTweak::new_static("/proc/sys/net/ipv4/tcp_max_syn_backlog", "2048"),
        FileTweak::new_static("/proc/sys/kernel/random/urandom_min_reseed_secs", "60"),
        FileTweak::new_static("/proc/sys/net/core/somaxconn", "2048"),
        FileTweak::new_static("/proc/sys/net/ipv4/tcp_fin_timeout", "15"),
        FileTweak::new_static("/proc/sys/net/ipv6/conf/all/use_tempaddr", "2"),
        FileTweak::new_static("/proc/sys/net/ipv4/conf/default/rp_filter", "1"),
        FileTweak::new_static("/proc/sys/debug/exception-trace", "0"),
        // FileTweak::new_static("/proc/sys/net/ipv4/tcp_congestion_control", "westwood"),
    ]);
    tweaks.extend(super::scheduler_io::generate_scheduler_tweaks());
    tweaks
}
