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
    constexpr const char* kPageCluster = "0";
    constexpr const char* kLmkMinfreeLevels = "0:55296,100:80640,200:106200,300:131760,900:197640,999:262144";
    constexpr const char* kLmkReportKills = "false";
    constexpr const char* kSchedLatencyNs = "18000000";
    constexpr const char* kSchedMinGranularityNs = "2250000";
    constexpr const char* kAdbEnabled = "0";
    constexpr const char* kDebuggableEnabled = "0";
}

namespace SystemPaths {
    constexpr const char* kPageCluster = "/proc/sys/vm/page-cluster";
    constexpr const char* kSchedLatencyNs = "/proc/sys/kernel/sched_latency_ns";
    constexpr const char* kSchedMinGranularityNs = "/proc/sys/kernel/sched_min_granularity_ns";
}

namespace SystemTweaker {
    void applyAll() {
        LOGI("Applying static system tweaks...");
        SystemUtils::applyTweak(SystemPaths::kPageCluster, TweakValues::kPageCluster);
        SystemUtils::setSystemProp("lmk.minfree_levels", TweakValues::kLmkMinfreeLevels);
        SystemUtils::setSystemProp("persist.sys.lmk.reportkills", TweakValues::kLmkReportKills);
        SystemUtils::applyTweak(SystemPaths::kSchedLatencyNs, TweakValues::kSchedLatencyNs);
        SystemUtils::applyTweak(SystemPaths::kSchedMinGranularityNs, TweakValues::kSchedMinGranularityNs);
        SystemUtils::setSystemProp("persist.service.adb.enable", TweakValues::kAdbEnabled);
        SystemUtils::setSystemProp("persist.service.debuggable", TweakValues::kDebuggableEnabled);
        LOGI("Static tweaks applied successfully.");
    }
}