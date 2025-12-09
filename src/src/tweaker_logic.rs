//! Author: [Seclususs](https://github.com/seclususs)

use crate::system_utils;

pub struct SystemTweaker;

impl SystemTweaker {
    pub fn apply_all() {
        info!("Rust: Applying static system tweaks...");
        system_utils::write_to_file("/proc/sys/vm/page-cluster", "1");
        system_utils::write_to_file("/proc/sys/vm/stat_interval", "2");
        system_utils::write_to_file("/proc/sys/vm/oom_dump_tasks", "0");
        system_utils::write_to_file("/proc/sys/vm/watermark_scale_factor", "50");
        system_utils::write_to_file("/proc/sys/vm/extfrag_threshold", "750");
        system_utils::set_system_prop("lmk.minfree_levels", "18432,23040,27648,32256,58880,76800");
        system_utils::set_system_prop("persist.sys.lmk.reportkills", "false");
        let sched_tweaks = [
            ("/proc/sys/kernel/sched_latency_ns", "9000000"),
            ("/proc/sys/kernel/sched_min_granularity_ns", "4000000"),
            ("/proc/sys/kernel/sched_migration_cost_ns", "500000"),
            ("/proc/sys/kernel/sched_child_runs_first", "1"),
            ("/proc/sys/kernel/sched_wakeup_granularity_ns", "2000000"),
            ("/proc/sys/kernel/perf_cpu_time_max_percent", "10"),
            ("/proc/sys/kernel/pid_max", "65536"),
            ("/proc/sys/kernel/sched_schedstats", "0"),
            ("/proc/sys/kernel/perf_event_paranoid", "2"),
        ];
        for (path, val) in sched_tweaks {
            system_utils::write_to_file(path, val);
        }
        let io_tweaks = [
            ("/sys/block/mmcblk0/queue/add_random", "0"),
            ("/sys/block/mmcblk0/queue/iostats", "0"),
            ("/sys/block/mmcblk0/queue/rq_affinity", "1"),
            ("/proc/sys/fs/lease-break-time", "10"),
            ("/proc/sys/fs/inotify/max_user_watches", "65536"),
            ("/proc/sys/fs/file-max", "524288"),
            ("/proc/sys/fs/protected_symlinks", "1"),
            ("/proc/sys/fs/protected_hardlinks", "1"),
            ("/sys/block/mmcblk0/queue/scheduler", "deadline"),
        ];
        for (path, val) in io_tweaks {
            system_utils::write_to_file(path, val);
        }
        let net_tweaks = [
            ("/proc/sys/net/ipv4/tcp_notsent_lowat", "16384"),
            ("/proc/sys/net/core/netdev_max_backlog", "2000"),
            ("/proc/sys/net/ipv4/tcp_slow_start_after_idle", "0"),
            ("/proc/sys/net/ipv4/tcp_tw_reuse", "1"),
            ("/proc/sys/net/core/netdev_budget", "300"),
            ("/proc/sys/net/ipv4/ip_dynaddr", "1"),
            ("/proc/sys/net/ipv4/tcp_keepalive_time", "1800"),
            ("/proc/sys/net/ipv4/tcp_max_syn_backlog", "2048"),
            ("/proc/sys/kernel/random/urandom_min_reseed_secs", "60"),
            ("/proc/sys/net/ipv4/tcp_timestamps", "0"),
            ("/proc/sys/net/core/somaxconn", "2048"),
            ("/proc/sys/net/ipv4/tcp_fin_timeout", "15"),
            ("/proc/sys/net/ipv4/tcp_retries2", "5"),
            ("/proc/sys/net/ipv6/conf/all/use_tempaddr", "2"),
            ("/proc/sys/net/ipv4/conf/default/rp_filter", "1"),
            ("/proc/sys/net/ipv4/tcp_congestion_control", "westwood"),
        ];
        for (path, val) in net_tweaks {
            system_utils::write_to_file(path, val);
        }
        system_utils::set_system_prop("persist.service.adb.enable", "0");
        system_utils::set_system_prop("persist.service.debuggable", "0");
        system_utils::write_to_file("/proc/sys/kernel/printk", "0 0 0 0");
        system_utils::write_to_file("/proc/sys/kernel/printk_devkmsg", "off");
        system_utils::write_to_file("/proc/sys/kernel/core_pattern", "/dev/null");
        system_utils::write_to_file("/proc/sys/kernel/dmesg_restrict", "1");
        info!("Rust: Static tweaks applied successfully.");
    }
}