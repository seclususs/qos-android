//! Author: [Seclususs](https://github.com/seclususs)

pub struct FileTweak {
    pub path: &'static str,
    pub value: &'static str,
}

pub struct PropTweak {
    pub key: &'static str,
    pub value: &'static str,
}

pub fn get_file_tweaks() -> Vec<FileTweak> {
    vec![
        FileTweak {
            path: "/proc/sys/vm/oom_dump_tasks",
            value: "0",
        },
        FileTweak {
            path: "/proc/sys/kernel/printk",
            value: "0 0 0 0",
        },
        FileTweak {
            path: "/proc/sys/kernel/printk_devkmsg",
            value: "off",
        },
        FileTweak {
            path: "/proc/sys/kernel/core_pattern",
            value: "/dev/null",
        },
        FileTweak {
            path: "/proc/sys/kernel/dmesg_restrict",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/kernel/sched_child_runs_first",
            value: "0",
        },
        FileTweak {
            path: "/proc/sys/kernel/sched_tunable_scaling",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/kernel/pid_max",
            value: "65536",
        },
        FileTweak {
            path: "/proc/sys/kernel/sched_schedstats",
            value: "0",
        },
        FileTweak {
            path: "/proc/sys/kernel/perf_event_paranoid",
            value: "2",
        },
        FileTweak {
            path: "/proc/sys/kernel/perf_cpu_time_max_percent",
            value: "1",
        },
        FileTweak {
            path: "/sys/block/mmcblk0/queue/add_random",
            value: "0",
        },
        FileTweak {
            path: "/sys/block/mmcblk0/queue/iostats",
            value: "1",
        },
        FileTweak {
            path: "/sys/block/mmcblk0/queue/rq_affinity",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/fs/lease-break-time",
            value: "10",
        },
        FileTweak {
            path: "/proc/sys/fs/inotify/max_user_watches",
            value: "65536",
        },
        FileTweak {
            path: "/proc/sys/fs/file-max",
            value: "524288",
        },
        FileTweak {
            path: "/proc/sys/fs/protected_symlinks",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/fs/protected_hardlinks",
            value: "1",
        },
        FileTweak {
            path: "/sys/block/mmcblk0/queue/scheduler",
            value: "deadline",
        },
        FileTweak {
            path: "/sys/block/zram0/max_comp_streams",
            value: "8",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_notsent_lowat",
            value: "16384",
        },
        FileTweak {
            path: "/proc/sys/net/core/netdev_max_backlog",
            value: "2000",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_slow_start_after_idle",
            value: "0",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_tw_reuse",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/net/core/netdev_budget",
            value: "300",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/ip_dynaddr",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_keepalive_time",
            value: "1800",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_max_syn_backlog",
            value: "2048",
        },
        FileTweak {
            path: "/proc/sys/kernel/random/urandom_min_reseed_secs",
            value: "60",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_timestamps",
            value: "0",
        },
        FileTweak {
            path: "/proc/sys/net/core/somaxconn",
            value: "2048",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_fin_timeout",
            value: "15",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_retries2",
            value: "5",
        },
        FileTweak {
            path: "/proc/sys/net/ipv6/conf/all/use_tempaddr",
            value: "2",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/conf/default/rp_filter",
            value: "1",
        },
        FileTweak {
            path: "/proc/sys/net/ipv4/tcp_congestion_control",
            value: "westwood",
        },
        FileTweak {
            path: "/proc/sys/debug/exception-trace",
            value: "0",
        },
    ]
}

pub fn get_prop_tweaks() -> Vec<PropTweak> {
    vec![
        PropTweak {
            key: "persist.sys.lmk.reportkills",
            value: "false",
        },
        PropTweak {
            key: "persist.service.adb.enable",
            value: "0",
        },
        PropTweak {
            key: "persist.service.debuggable",
            value: "0",
        },
    ]
}