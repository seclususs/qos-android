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
    constexpr const char* kPageCluster = "1";
    constexpr const char* kVmStatInterval = "2";
    constexpr const char* kOomDumpTasks = "0";
    constexpr const char* kVmWatermarkScale = "50";
    constexpr const char* kVmExtfragThreshold = "750";
    
    // Low Memory Killer
    constexpr const char* kLmkMinfreeLevels = "18432,23040,27648,32256,58880,76800";
    constexpr const char* kLmkReportKills = "false";

    // CPU Scheduler
    constexpr const char* kSchedLatencyNs = "9000000";
    constexpr const char* kSchedMinGranularityNs = "1000000";
    constexpr const char* kSchedMigrationCost = "500000";
    constexpr const char* kSchedChildFirst = "1";
    constexpr const char* kSchedWakeupGranularity = "2000000";
    constexpr const char* kPerfCpuLimit = "10";
    constexpr const char* kKernelPidMax = "65536";
    constexpr const char* kSchedSchedstats = "0";
    constexpr const char* kPerfEventParanoid = "2";

    // I/O & Storage
    constexpr const char* kIoAddRandom = "0";
    constexpr const char* kIoStats = "0";
    constexpr const char* kMmcRqAffinity = "1";
    constexpr const char* kFsLeaseBreak = "10";
    constexpr const char* kMaxUserWatches = "65536";
    constexpr const char* kFileMax = "524288";
    constexpr const char* kProtectedSymlinks = "1";
    constexpr const char* kProtectedHardlinks = "1";
    constexpr const char* kIoScheduler = "deadline";

    // Network Tweaks
    constexpr const char* kTcpNotsentLowat = "16384";
    constexpr const char* kNetDevBacklog = "2500";
    constexpr const char* kTcpSlowStartIdle = "0";
    constexpr const char* kTcpTwReuse = "1";
    constexpr const char* kNetDevBudget = "300";
    constexpr const char* kNetIpDynaddr = "1";
    constexpr const char* kTcpKeepalive = "1800";
    constexpr const char* kTcpSynBacklog = "2048";
    constexpr const char* kRndReseedSecs = "60";
    constexpr const char* kTcpTimestamps = "0";
    constexpr const char* kSomaxconn = "2048";
    constexpr const char* kTcpFinTimeout = "15";
    constexpr const char* kTcpRetries2 = "5";
    constexpr const char* kIpv6UseTempAddr = "2";
    constexpr const char* kRpFilter = "1";
    constexpr const char* kTcpCongestion = "westwood";

    // System & Debugging
    constexpr const char* kAdbEnabled = "0";
    constexpr const char* kDebuggableEnabled = "0";
    constexpr const char* kKernelPrintk = "0 0 0 0";
    constexpr const char* kKernelPrintkMsg = "off";
    constexpr const char* kCorePattern = "/dev/null";
    constexpr const char* kDmesgRestrict = "1";
}

namespace SystemPaths {
    // Memory & VM
    constexpr const char* kPageCluster = "/proc/sys/vm/page-cluster";
    constexpr const char* kVmStatInterval = "/proc/sys/vm/stat_interval";
    constexpr const char* kOomDumpTasks = "/proc/sys/vm/oom_dump_tasks";
    constexpr const char* kVmWatermarkScale = "/proc/sys/vm/watermark_scale_factor";
    constexpr const char* kVmExtfragThreshold = "/proc/sys/vm/extfrag_threshold";

    // CPU Scheduler
    constexpr const char* kSchedLatencyNs = "/proc/sys/kernel/sched_latency_ns";
    constexpr const char* kSchedMinGranularityNs = "/proc/sys/kernel/sched_min_granularity_ns";
    constexpr const char* kSchedMigrationCost = "/proc/sys/kernel/sched_migration_cost_ns";
    constexpr const char* kSchedChildFirst = "/proc/sys/kernel/sched_child_runs_first";
    constexpr const char* kSchedWakeupGranularity = "/proc/sys/kernel/sched_wakeup_granularity_ns";
    constexpr const char* kPerfCpuLimit = "/proc/sys/kernel/perf_cpu_time_max_percent";
    constexpr const char* kKernelPidMax = "/proc/sys/kernel/pid_max";
    constexpr const char* kSchedSchedstats = "/proc/sys/kernel/sched_schedstats";
    constexpr const char* kPerfEventParanoid = "/proc/sys/kernel/perf_event_paranoid";

    // I/O & Storage
    constexpr const char* kIoAddRandom = "/sys/block/mmcblk0/queue/add_random";
    constexpr const char* kIoStats = "/sys/block/mmcblk0/queue/iostats";
    constexpr const char* kMmcRqAffinity = "/sys/block/mmcblk0/queue/rq_affinity";
    constexpr const char* kFsLeaseBreak = "/proc/sys/fs/lease-break-time";
    constexpr const char* kMaxUserWatches = "/proc/sys/fs/inotify/max_user_watches";
    constexpr const char* kFileMax = "/proc/sys/fs/file-max";
    constexpr const char* kProtectedSymlinks = "/proc/sys/fs/protected_symlinks";
    constexpr const char* kProtectedHardlinks = "/proc/sys/fs/protected_hardlinks";
    constexpr const char* kIoScheduler = "/sys/block/mmcblk0/queue/scheduler";

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
    constexpr const char* kSomaxconn = "/proc/sys/net/core/somaxconn";
    constexpr const char* kTcpFinTimeout = "/proc/sys/net/ipv4/tcp_fin_timeout";
    constexpr const char* kTcpRetries2 = "/proc/sys/net/ipv4/tcp_retries2";
    constexpr const char* kIpv6UseTempAddr = "/proc/sys/net/ipv6/conf/all/use_tempaddr";
    constexpr const char* kRpFilter = "/proc/sys/net/ipv4/conf/default/rp_filter";
    constexpr const char* kTcpCongestion = "/proc/sys/net/ipv4/tcp_congestion_control";

    // System & Debugging
    constexpr const char* kKernelPrintk = "/proc/sys/kernel/printk";
    constexpr const char* kKernelPrintkMsg = "/proc/sys/kernel/printk_devkmsg";
    constexpr const char* kCorePattern = "/proc/sys/kernel/core_pattern";
    constexpr const char* kDmesgRestrict = "/proc/sys/kernel/dmesg_restrict";
}

namespace SystemTweaker {
    void applyAll() {
        LOGI("Applying static system tweaks...");

        // Memory & VM
        SystemUtils::applyTweak(SystemPaths::kPageCluster, TweakValues::kPageCluster);
        SystemUtils::applyTweak(SystemPaths::kVmStatInterval, TweakValues::kVmStatInterval);
        SystemUtils::applyTweak(SystemPaths::kOomDumpTasks, TweakValues::kOomDumpTasks);
        SystemUtils::applyTweak(SystemPaths::kVmWatermarkScale, TweakValues::kVmWatermarkScale);
        SystemUtils::applyTweak(SystemPaths::kVmExtfragThreshold, TweakValues::kVmExtfragThreshold);

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
        SystemUtils::applyTweak(SystemPaths::kSchedSchedstats, TweakValues::kSchedSchedstats);
        SystemUtils::applyTweak(SystemPaths::kPerfEventParanoid, TweakValues::kPerfEventParanoid);

        // I/O & Storage
        SystemUtils::applyTweak(SystemPaths::kIoAddRandom, TweakValues::kIoAddRandom);
        SystemUtils::applyTweak(SystemPaths::kIoStats, TweakValues::kIoStats);
        SystemUtils::applyTweak(SystemPaths::kMmcRqAffinity, TweakValues::kMmcRqAffinity);
        SystemUtils::applyTweak(SystemPaths::kFsLeaseBreak, TweakValues::kFsLeaseBreak);
        SystemUtils::applyTweak(SystemPaths::kMaxUserWatches, TweakValues::kMaxUserWatches);
        SystemUtils::applyTweak(SystemPaths::kFileMax, TweakValues::kFileMax);
        SystemUtils::applyTweak(SystemPaths::kProtectedSymlinks, TweakValues::kProtectedSymlinks);
        SystemUtils::applyTweak(SystemPaths::kProtectedHardlinks, TweakValues::kProtectedHardlinks);
        SystemUtils::applyTweak(SystemPaths::kIoScheduler, TweakValues::kIoScheduler);

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
        SystemUtils::applyTweak(SystemPaths::kSomaxconn, TweakValues::kSomaxconn);
        SystemUtils::applyTweak(SystemPaths::kTcpFinTimeout, TweakValues::kTcpFinTimeout);
        SystemUtils::applyTweak(SystemPaths::kTcpRetries2, TweakValues::kTcpRetries2);
        SystemUtils::applyTweak(SystemPaths::kIpv6UseTempAddr, TweakValues::kIpv6UseTempAddr);
        SystemUtils::applyTweak(SystemPaths::kRpFilter, TweakValues::kRpFilter);
        SystemUtils::applyTweak(SystemPaths::kTcpCongestion, TweakValues::kTcpCongestion);

        // System & Debugging
        SystemUtils::setSystemProp("persist.service.adb.enable", TweakValues::kAdbEnabled);
        SystemUtils::setSystemProp("persist.service.debuggable", TweakValues::kDebuggableEnabled);
        SystemUtils::applyTweak(SystemPaths::kKernelPrintk, TweakValues::kKernelPrintk);
        SystemUtils::applyTweak(SystemPaths::kKernelPrintkMsg, TweakValues::kKernelPrintkMsg);
        SystemUtils::applyTweak(SystemPaths::kCorePattern, TweakValues::kCorePattern);
        SystemUtils::applyTweak(SystemPaths::kDmesgRestrict, TweakValues::kDmesgRestrict);

        LOGI("Static tweaks applied successfully.");
    }
}