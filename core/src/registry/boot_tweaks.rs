//! Author: [Seclususs](https://github.com/seclususs)

pub struct FileTweak {
    pub path: &'static str,
    pub value: &'static str,
}

pub struct PropTweak {
    pub key: &'static str,
    pub value: &'static str,
}

pub fn get_file_tweaks() -> &'static [FileTweak] {
    &[
        FileTweak {
            path: "/proc/sys/vm/oom_dump_tasks",
            value: "0",
        },
        FileTweak {
            path: "/proc/sys/vm/stat_interval",
            value: "2",
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
            path: "/sys/block/mmcblk0/queue/iosched/fifo_batch",
            value: "16",
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

pub fn get_prop_tweaks() -> &'static [PropTweak] {
    &[
        PropTweak {
            key: "persist.sys.lmk.reportkills",
            value: "false",
        },
        PropTweak {
            key: "persist.service.debuggable",
            value: "0",
        },
        PropTweak {
            key: "sys.wifitracing.started",
            value: "0",
        },
        PropTweak {
            key: "persist.device_config.runtime_native.metrics.write-to-statsd",
            value: "false",
        },
        PropTweak {
            key: "debug.als.logs",
            value: "0",
        },
        PropTweak {
            key: "debug.svi.logs",
            value: "0",
        },
        PropTweak {
            key: "persist.vendor.sys.pq.mdp.color.dbg",
            value: "0",
        },
        PropTweak {
            key: "persist.vendor.log.tel_log_ctrl",
            value: "0",
        },
        PropTweak {
            key: "sys.lmk.reportkills",
            value: "0",
        },
        PropTweak {
            key: "dalvik.vm.minidebuginfo",
            value: "false",
        },
        PropTweak {
            key: "dalvik.vm.dex2oat-minidebuginfo",
            value: "false",
        },
        PropTweak {
            key: "persist.vendor.connsys.dedicated.log",
            value: "0",
        },
        PropTweak {
            key: "persist.vendor.dpm.loglevel",
            value: "0",
        },
        PropTweak {
            key: "persist.vendor.ims.disableADBLogs",
            value: "1",
        },
        PropTweak {
            key: "persist.vendor.ims.disableDebugLogs",
            value: "1",
        },
        PropTweak {
            key: "persist.vendor.ims.disableQXDMLogs",
            value: "1",
        },
        PropTweak {
            key: "persist.device_config.storage_native_boot.smart_idle_maint_enabled",
            value: "true",
        },
        PropTweak {
            key: "persist.device_config.netd_native.parallel_lookup",
            value: "true",
        },
        PropTweak {
            key: "persist.device_config.runtime_native_boot.disable_lock_profiling",
            value: "true",
        },
        PropTweak {
            key: "profiler.force_disable_err_rpt",
            value: "1",
        },
        PropTweak {
            key: "profiler.force_disable_ulog",
            value: "1",
        },
        PropTweak {
            key: "logd.logpersistd.enable",
            value: "false",
        },
        PropTweak {
            key: "log.tag.stats_log",
            value: "0",
        },
        PropTweak {
            key: "log.tag.APM_AudioPolicyManager",
            value: "0",
        },
        PropTweak {
            key: "persist.log.tag.snet_event_log",
            value: "0",
        },
    ]
}
