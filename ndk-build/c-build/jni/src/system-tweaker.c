#define LOG_TAG "AdaptiveTweaker"
#include "system-tweaker.h"
#include "system-utils.h"

#include <android/log.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifndef LOGI
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#endif

/* Config values (kept as in original) */
static const char *kGovernor = "schedutil";
static const char *kPageCluster = "0";
static const char *kLmkMinfreeLevels = "0:55296,100:80640,200:106200,300:131760,900:197640,999:262144";
static const char *kLmkReportKills = "false";
static const char *kSchedLatencyNs = "18000000";
static const char *kSchedMinGranularityNs = "2250000";
static const char *kAdbEnabled = "0";
static const char *kDebuggableEnabled = "0";

static const char *cpu_policy_dir = "/sys/devices/system/cpu/cpufreq/policy";
static const char *scaling_governor = "/scaling_governor";
static const char *page_cluster_path = "/proc/sys/vm/page-cluster";
static const char *sched_latency_path = "/proc/sys/kernel/sched_latency_ns";
static const char *sched_min_granularity_path = "/proc/sys/kernel/sched_min_granularity_ns";

bool system_tweaker_apply_all(void) {
    LOGI("Applying static system tweaks...");
    /* Apply governor per CPU discovered by scanning policy directory entries like policy0, policy1, ... */
    for (int i = 0; ; ++i) {
        char path[256];
        int n = snprintf(path, sizeof(path), "%s%d%s", cpu_policy_dir, i, scaling_governor);
        if (n <= 0 || (size_t)n >= sizeof(path)) break;
        /* Try to write; if not exists break when first missing index encountered after 0 */
        if (!sys_write_file(path, kGovernor)) {
            /* If file doesn't exist for this index, assume we've reached end */
            if (i == 0) {
                LOGE("system_tweaker_apply_all: failed to set governor for policy0");
            } else {
                /* stop scanning further */
                break;
            }
        }
    }

    if (!sys_write_file(page_cluster_path, kPageCluster)) {
        LOGE("system_tweaker_apply_all: failed to set page-cluster");
    }

    if (!sys_write_file(sched_latency_path, kSchedLatencyNs)) {
        LOGE("system_tweaker_apply_all: failed to set sched_latency_ns");
    }
    if (!sys_write_file(sched_min_granularity_path, kSchedMinGranularityNs)) {
        LOGE("system_tweaker_apply_all: failed to set sched_min_granularity_ns");
    }

    sys_set_property("lmk.minfree_levels", kLmkMinfreeLevels);
    sys_set_property("persist.sys.lmk.reportkills", kLmkReportKills);
    sys_set_property("persist.service.adb.enable", kAdbEnabled);
    sys_set_property("persist.service.debuggable", kDebuggableEnabled);

    LOGI("Static tweaks applied.");
    return true;
}
