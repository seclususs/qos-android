/**
 * @author Seclususs
 * https://github.com/seclususs
 */

#include "system_tweaker.h"
#include "system_utils.h"
#include "logging.h"

#include <string>
#include <thread>

namespace TweakValues {
    // Memory & VM
    constexpr const char* kPageCluster = "3";
    constexpr const char* kVmStatInterval = "2";
    constexpr const char* kDirtyRatio = "20";
    constexpr const char* kDirtyBackgroundRatio = "10";
    constexpr const char* kDirtyExpire = "2000";
    constexpr const char* kDirtyWriteback = "2000";
    constexpr const char* kOomDumpTasks = "0";
    constexpr const char* kVmWatermarkScale = "100";
    constexpr const char* kMinFreeKbytes = "65536";
    constexpr const char* kVmLaptopMode = "5";
    constexpr const char* kVmExtfragThreshold = "600";
    constexpr const char* kVmOomKillAlloc = "1";
    
    // Low Memory Killer
    constexpr const char* kLmkMinfreeLevels = "0:18432,100:23040,200:27648,300:51200,800:102400,999:192000";
    constexpr const char* kLmkReportKills = "false";

    // CPU Scheduler
    constexpr const char* kSchedLatencyNs = "12000000";
    constexpr const char* kSchedMinGranularityNs = "2000000";
    constexpr const char* kSchedMigrationCost = "2000000";
    constexpr const char* kSchedChildFirst = "1";
    constexpr const char* kSchedWakeupGranularity = "2500000";
    constexpr const char* kPerfCpuLimit = "10";
    constexpr const char* kKernelPidMax = "65536";

    // I/O & Storage
    constexpr const char* kIoAddRandom = "0";
    constexpr const char* kIoStats = "0";
    constexpr const char* kMmcRqAffinity = "2";
    constexpr const char* kFsLeaseBreak = "15";

    // Network Tweaks
    constexpr const char* kTcpNotsentLowat = "4096";
    constexpr const char* kNetDevBacklog = "5000";
    constexpr const char* kTcpSlowStartIdle = "0";
    constexpr const char* kTcpTwReuse = "1";
    constexpr const char* kNetDevBudget = "600";
    constexpr const char* kNetIpDynaddr = "1";
    constexpr const char* kTcpKeepalive = "1800";
    constexpr const char* kTcpSynBacklog = "1024";
    constexpr const char* kRndReseedSecs = "120";
    constexpr const char* kTcpTimestamps = "1";

    // System & Debugging
    constexpr const char* kAdbEnabled = "0";
    constexpr const char* kDebuggableEnabled = "0";
    constexpr const char* kKernelPrintk = "0 0 0 0";
    constexpr const char* kKernelPrintkMsg = "off";
}

namespace SystemPaths {
    // Memory & VM
    constexpr const char* kPageCluster = "/proc/sys/vm/page-cluster";
    constexpr const char* kVmStatInterval = "/proc/sys/vm/stat_interval";
    constexpr const char* kDirtyRatio = "/proc/sys/vm/dirty_ratio";
    constexpr const char* kDirtyBackgroundRatio = "/proc/sys/vm/dirty_background_ratio";
    constexpr const char* kDirtyExpire = "/proc/sys/vm/dirty_expire_centisecs";
    constexpr const char* kDirtyWriteback = "/proc/sys/vm/dirty_writeback_centisecs";
    constexpr const char* kOomDumpTasks = "/proc/sys/vm/oom_dump_tasks";
    constexpr const char* kVmWatermarkScale = "/proc/sys/vm/watermark_scale_factor";
    constexpr const char* kMinFreeKbytes = "/proc/sys/vm/min_free_kbytes";
    constexpr const char* kVmLaptopMode = "/proc/sys/vm/laptop_mode";
    constexpr const char* kVmExtfragThreshold = "/proc/sys/vm/extfrag_threshold";
    constexpr const char* kVmOomKillAlloc = "/proc/sys/vm/oom_kill_allocating_task";

    // CPU Scheduler
    constexpr const char* kSchedLatencyNs = "/proc/sys/kernel/sched_latency_ns";
    constexpr const char* kSchedMinGranularityNs = "/proc/sys/kernel/sched_min_granularity_ns";
    constexpr const char* kSchedMigrationCost = "/proc/sys/kernel/sched_migration_cost_ns";
    constexpr const char* kSchedChildFirst = "/proc/sys/kernel/sched_child_runs_first";
    constexpr const char* kSchedWakeupGranularity = "/proc/sys/kernel/sched_wakeup_granularity_ns";
    constexpr const char* kPerfCpuLimit = "/proc/sys/kernel/perf_cpu_time_max_percent";
    constexpr const char* kKernelPidMax = "/proc/sys/kernel/pid_max";

    // I/O & Storage
    constexpr const char* kIoAddRandom = "/sys/block/mmcblk0/queue/add_random";
    constexpr const char* kIoStats = "/sys/block/mmcblk0/queue/iostats";
    constexpr const char* kMmcRqAffinity = "/sys/block/mmcblk0/queue/rq_affinity";
    constexpr const char* kFsLeaseBreak = "/proc/sys/fs/lease-break-time";

    // Network
    constexpr const char* kTcpNotsentLowat = "/proc/sys/net/ipv4/tcp_notsent_lowat";
    constexpr const char* kNetDevBacklog = "/proc/sys/net/core/netdev_max_backlog";
    constexpr const char* kTcpSlowStartIdle = "/proc/sys/net/ipv4/tcp_slow_start_after_idle";
    constexpr const char* kTcpTwReuse = "/proc/sys/net/ipv4/tcp_tw_reuse";
    constexpr const char* kNetDevBudget = "/proc/sys/net/core/netdev_budget";
    constexpr const char* kNetIpDynaddr = "/proc/sys/net/ipv4/ip_dynaddr";
    constexpr const char* kTcpKeepalive = "/proc/sys/net/ipv4/tcp_keepalive_time";
    constexpr const char* kTcpSynBacklog = "/proc/sys/net/ipv4/tcp_max_syn_backlog";
    constexpr const char* kRndReseedSecs = "/proc/sys/kernel/random/urandom_min_reseed_secs";
    constexpr const char* kTcpTimestamps = "/proc/sys/net/ipv4/tcp_timestamps";

    // System
    constexpr const char* kKernelPrintk = "/proc/sys/kernel/printk";
    constexpr const char* kKernelPrintkMsg = "/proc/sys/kernel/printk_devkmsg";
}

namespace SystemTweaker {
    void applyAll() {
        LOGI("Applying static system tweaks...");

        // Memory & VM
        SystemUtils::applyTweak(SystemPaths::kPageCluster, TweakValues::kPageCluster);
        SystemUtils::applyTweak(SystemPaths::kVmStatInterval, TweakValues::kVmStatInterval);
        SystemUtils::applyTweak(SystemPaths::kDirtyRatio, TweakValues::kDirtyRatio);
        SystemUtils::applyTweak(SystemPaths::kDirtyBackgroundRatio, TweakValues::kDirtyBackgroundRatio);
        SystemUtils::applyTweak(SystemPaths::kDirtyExpire, TweakValues::kDirtyExpire);
        SystemUtils::applyTweak(SystemPaths::kDirtyWriteback, TweakValues::kDirtyWriteback);
        SystemUtils::applyTweak(SystemPaths::kOomDumpTasks, TweakValues::kOomDumpTasks);
        SystemUtils::applyTweak(SystemPaths::kVmWatermarkScale, TweakValues::kVmWatermarkScale);
        SystemUtils::applyTweak(SystemPaths::kMinFreeKbytes, TweakValues::kMinFreeKbytes);
        SystemUtils::applyTweak(SystemPaths::kVmLaptopMode, TweakValues::kVmLaptopMode);
        SystemUtils::applyTweak(SystemPaths::kVmExtfragThreshold, TweakValues::kVmExtfragThreshold);
        SystemUtils::applyTweak(SystemPaths::kVmOomKillAlloc, TweakValues::kVmOomKillAlloc);

        // Low Memory Killer
        SystemUtils::setSystemProp("lmk.minfree_levels", TweakValues::kLmkMinfreeLevels);
        SystemUtils::setSystemProp("persist.sys.lmk.reportkills", TweakValues::kLmkReportKills);

        // CPU Scheduler
        SystemUtils::applyTweak(SystemPaths::kSchedLatencyNs, TweakValues::kSchedLatencyNs);
        SystemUtils::applyTweak(SystemPaths::kSchedMinGranularityNs, TweakValues::kSchedMinGranularityNs);
        SystemUtils::applyTweak(SystemPaths::kSchedMigrationCost, TweakValues::kSchedMigrationCost);
        SystemUtils::applyTweak(SystemPaths::kSchedChildFirst, TweakValues::kSchedChildFirst);
        SystemUtils::applyTweak(SystemPaths::kSchedWakeupGranularity, TweakValues::kSchedWakeupGranularity);
        SystemUtils::applyTweak(SystemPaths::kPerfCpuLimit, TweakValues::kPerfCpuLimit);
        SystemUtils::applyTweak(SystemPaths::kKernelPidMax, TweakValues::kKernelPidMax);

        // I/O & Storage
        SystemUtils::applyTweak(SystemPaths::kIoAddRandom, TweakValues::kIoAddRandom);
        SystemUtils::applyTweak(SystemPaths::kIoStats, TweakValues::kIoStats);
        SystemUtils::applyTweak(SystemPaths::kMmcRqAffinity, TweakValues::kMmcRqAffinity);
        SystemUtils::applyTweak(SystemPaths::kFsLeaseBreak, TweakValues::kFsLeaseBreak);

        // Network
        SystemUtils::applyTweak(SystemPaths::kTcpNotsentLowat, TweakValues::kTcpNotsentLowat);
        SystemUtils::applyTweak(SystemPaths::kNetDevBacklog, TweakValues::kNetDevBacklog);
        SystemUtils::applyTweak(SystemPaths::kTcpSlowStartIdle, TweakValues::kTcpSlowStartIdle);
        SystemUtils::applyTweak(SystemPaths::kTcpTwReuse, TweakValues::kTcpTwReuse);
        SystemUtils::applyTweak(SystemPaths::kNetDevBudget, TweakValues::kNetDevBudget);
        SystemUtils::applyTweak(SystemPaths::kNetIpDynaddr, TweakValues::kNetIpDynaddr);
        SystemUtils::applyTweak(SystemPaths::kTcpKeepalive, TweakValues::kTcpKeepalive);
        SystemUtils::applyTweak(SystemPaths::kTcpSynBacklog, TweakValues::kTcpSynBacklog);
        SystemUtils::applyTweak(SystemPaths::kRndReseedSecs, TweakValues::kRndReseedSecs);
        SystemUtils::applyTweak(SystemPaths::kTcpTimestamps, TweakValues::kTcpTimestamps);

        // System & Debugging
        SystemUtils::setSystemProp("persist.service.adb.enable", TweakValues::kAdbEnabled);
        SystemUtils::setSystemProp("persist.service.debuggable", TweakValues::kDebuggableEnabled);
        SystemUtils::applyTweak(SystemPaths::kKernelPrintk, TweakValues::kKernelPrintk);
        SystemUtils::applyTweak(SystemPaths::kKernelPrintkMsg, TweakValues::kKernelPrintkMsg);

        LOGI("Static tweaks applied successfully.");
    }
}