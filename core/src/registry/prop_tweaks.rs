//! Author: [Seclususs](https://github.com/seclususs)

pub struct PropTweak {
    pub key: &'static str,
    pub value: &'static str,
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
