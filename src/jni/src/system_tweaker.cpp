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
    constexpr const char* kPageCluster = "0";
    constexpr const char* kVmStatInterval = "5";
    constexpr const char* kDirtyRatio = "30";
    constexpr const char* kDirtyBackgroundRatio = "10";
    constexpr const char* kDirtyExpire = "1500";
    constexpr const char* kDirtyWriteback = "1500";
    constexpr const char* kOomDumpTasks = "0";
    constexpr const char* kVmWatermarkScale = "100";
    constexpr const char* kMinFreeKbytes = "65536";
    
    // Low Memory Killer
    constexpr const char* kLmkMinfreeLevels = "0:55296,100:80640,200:106200,300:131760,900:197640,999:262144";
    constexpr const char* kLmkReportKills = "false";

    // CPU Scheduler
    constexpr const char* kSchedLatencyNs = "12000000";
    constexpr const char* kSchedMinGranularityNs = "2000000";
    constexpr const char* kSchedMigrationCost = "500000";

    // I/O & Storage
    constexpr const char* kIoAddRandom = "0";
    constexpr const char* kIoStats = "0";

    // Network Tweaks
    constexpr const char* kTcpNotsentLowat = "16384";
    constexpr const char* kNetDevBacklog = "5000";

    // System & Debugging
    constexpr const char* kAdbEnabled = "0";
    constexpr const char* kDebuggableEnabled = "0";
    constexpr const char* kKernelPrintk = "0 0 0 0";
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

    // CPU Scheduler
    constexpr const char* kSchedLatencyNs = "/proc/sys/kernel/sched_latency_ns";
    constexpr const char* kSchedMinGranularityNs = "/proc/sys/kernel/sched_min_granularity_ns";
    constexpr const char* kSchedMigrationCost = "/proc/sys/kernel/sched_migration_cost_ns";

    // I/O & Storage
    constexpr const char* kIoAddRandom = "/sys/block/mmcblk0/queue/add_random";
    constexpr const char* kIoStats = "/sys/block/mmcblk0/queue/iostats";

    // Network
    constexpr const char* kTcpNotsentLowat = "/proc/sys/net/ipv4/tcp_notsent_lowat";
    constexpr const char* kNetDevBacklog = "/proc/sys/net/core/netdev_max_backlog";

    // System
    constexpr const char* kKernelPrintk = "/proc/sys/kernel/printk";
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

        // Low Memory Killer
        SystemUtils::setSystemProp("lmk.minfree_levels", TweakValues::kLmkMinfreeLevels);
        SystemUtils::setSystemProp("persist.sys.lmk.reportkills", TweakValues::kLmkReportKills);

        // CPU Scheduler
        SystemUtils::applyTweak(SystemPaths::kSchedLatencyNs, TweakValues::kSchedLatencyNs);
        SystemUtils::applyTweak(SystemPaths::kSchedMinGranularityNs, TweakValues::kSchedMinGranularityNs);
        SystemUtils::applyTweak(SystemPaths::kSchedMigrationCost, TweakValues::kSchedMigrationCost);

        // I/O & Storage
        SystemUtils::applyTweak(SystemPaths::kIoAddRandom, TweakValues::kIoAddRandom);
        SystemUtils::applyTweak(SystemPaths::kIoStats, TweakValues::kIoStats);

        // Network
        SystemUtils::applyTweak(SystemPaths::kTcpNotsentLowat, TweakValues::kTcpNotsentLowat);
        SystemUtils::applyTweak(SystemPaths::kNetDevBacklog, TweakValues::kNetDevBacklog);

        // System & Debugging
        SystemUtils::setSystemProp("persist.service.adb.enable", TweakValues::kAdbEnabled);
        SystemUtils::setSystemProp("persist.service.debuggable", TweakValues::kDebuggableEnabled);
        SystemUtils::applyTweak(SystemPaths::kKernelPrintk, TweakValues::kKernelPrintk);

        LOGI("Static tweaks applied successfully.");
    }
}