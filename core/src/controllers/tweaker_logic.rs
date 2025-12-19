//! Author: [Seclususs](https://github.com/seclususs)

use crate::common::fs::{write_to_file, set_system_prop};
use std::sync::OnceLock;

struct StaticTweak {
    path: &'static str,
    value: &'static str,
}

static STATIC_TWEAKS: OnceLock<Vec<StaticTweak>> = OnceLock::new();

pub struct SystemTweaker;

impl SystemTweaker {
    pub fn apply_all() {
        log::info!("Rust: Preparing system tweaks...");
        let tweaks = STATIC_TWEAKS.get_or_init(|| {
            vec![
                StaticTweak { path: "/proc/sys/vm/page-cluster", value: "1" },
                StaticTweak { path: "/proc/sys/vm/stat_interval", value: "3" },
                StaticTweak { path: "/proc/sys/vm/oom_dump_tasks", value: "0" },
                StaticTweak { path: "/proc/sys/vm/watermark_scale_factor", value: "15" },
                StaticTweak { path: "/proc/sys/vm/extfrag_threshold", value: "550" },
                StaticTweak { path: "/proc/sys/kernel/printk", value: "0 0 0 0" },
                StaticTweak { path: "/proc/sys/kernel/printk_devkmsg", value: "off" },
                StaticTweak { path: "/proc/sys/kernel/core_pattern", value: "/dev/null" },
                StaticTweak { path: "/proc/sys/kernel/dmesg_restrict", value: "1" },
                StaticTweak { path: "/proc/sys/kernel/sched_migration_cost_ns", value: "600000" },
                StaticTweak { path: "/proc/sys/kernel/sched_child_runs_first", value: "1" },
                StaticTweak { path: "/proc/sys/kernel/perf_cpu_time_max_percent", value: "15" },
                StaticTweak { path: "/proc/sys/kernel/pid_max", value: "65536" },
                StaticTweak { path: "/proc/sys/kernel/sched_schedstats", value: "0" },
                StaticTweak { path: "/proc/sys/kernel/perf_event_paranoid", value: "2" },
                StaticTweak { path: "/sys/block/mmcblk0/queue/add_random", value: "0" },
                StaticTweak { path: "/sys/block/mmcblk0/queue/iostats", value: "0" },
                StaticTweak { path: "/sys/block/mmcblk0/queue/rq_affinity", value: "1" },
                StaticTweak { path: "/proc/sys/fs/lease-break-time", value: "10" },
                StaticTweak { path: "/proc/sys/fs/inotify/max_user_watches", value: "65536" },
                StaticTweak { path: "/proc/sys/fs/file-max", value: "524288" },
                StaticTweak { path: "/proc/sys/fs/protected_symlinks", value: "1" },
                StaticTweak { path: "/proc/sys/fs/protected_hardlinks", value: "1" },
                StaticTweak { path: "/sys/block/mmcblk0/queue/scheduler", value: "deadline" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_notsent_lowat", value: "16384" },
                StaticTweak { path: "/proc/sys/net/core/netdev_max_backlog", value: "2000" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_slow_start_after_idle", value: "0" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_tw_reuse", value: "1" },
                StaticTweak { path: "/proc/sys/net/core/netdev_budget", value: "300" },
                StaticTweak { path: "/proc/sys/net/ipv4/ip_dynaddr", value: "1" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_keepalive_time", value: "1800" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_max_syn_backlog", value: "2048" },
                StaticTweak { path: "/proc/sys/kernel/random/urandom_min_reseed_secs", value: "60" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_timestamps", value: "0" },
                StaticTweak { path: "/proc/sys/net/core/somaxconn", value: "2048" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_fin_timeout", value: "15" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_retries2", value: "5" },
                StaticTweak { path: "/proc/sys/net/ipv6/conf/all/use_tempaddr", value: "2" },
                StaticTweak { path: "/proc/sys/net/ipv4/conf/default/rp_filter", value: "1" },
                StaticTweak { path: "/proc/sys/net/ipv4/tcp_congestion_control", value: "westwood" },
            ]
        });
        let mut success_count = 0;
        let total_count = tweaks.len();
        for tweak in tweaks.iter() {
            match write_to_file(tweak.path, tweak.value) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    log::debug!("Failed to apply tweak {}: {}", tweak.path, e);
                }
            }
        }
        log::info!("Rust: Applied {}/{} tweaks successfully.", success_count, total_count);
        set_system_prop("persist.sys.lmk.reportkills", "false");
        set_system_prop("persist.service.adb.enable", "0");
        set_system_prop("persist.service.debuggable", "0");
        log::info!("Rust: Tweaks process finished.");
    }
}