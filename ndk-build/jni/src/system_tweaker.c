/**
 * Copyright (C) 2025 Seclususs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @file system_tweaker.c
 * @brief Implementation for applying static system tweaks.
 *
 * Applies a set of predefined optimizations to kernel parameters
 * (sysfs) and Android system properties (props).
 */

#include "include/system_tweaker.h"
#include "include/system_utils.h"
#include "include/logging.h"
#include <stdio.h>
#include <unistd.h>

/* Tweak Values */

/** @brief The desired CPU governor (e.g., "schedutil"). */
const char* const TWEAK_VALUES_GOVERNOR = "schedutil";
/** @brief The desired vm/page-cluster value. */
const char* const TWEAK_VALUES_PAGE_CLUSTER = "0";
/** @brief The desired LMK (Low Memory Killer) minfree levels. */
const char* const TWEAK_VALUES_LMK_MINFREE_LEVELS =
    "0:55296,100:80640,200:106200,300:131760,900:197640,999:262144";
/** @brief The desired LMK kill reporting setting. */
const char* const TWEAK_VALUES_LMK_REPORT_KILLS = "false";
/** @brief The desired kernel scheduler latency. */
const char* const TWEAK_VALUES_SCHED_LATENCY_NS = "18000000";
/** @brief The desired kernel scheduler minimum granularity. */
const char* const TWEAK_VALUES_SCHED_MIN_GRANULARITY_NS = "2250000";
/** @brief The desired state for ADB (Android Debug Bridge). */
const char* const TWEAK_VALUES_ADB_ENABLED = "0";
/** @brief The desired state for global debuggable property. */
const char* const TWEAK_VALUES_DEBUGGABLE_ENABLED = "0";

/* System Paths */

/** @brief Base directory for CPU frequency policies. */
const char* const SYSTEM_PATHS_CPU_POLICY_DIR =
    "/sys/devices/system/cpu/cpufreq/policy";
/** @brief Filename for the scaling governor within a policy directory. */
const char* const SYSTEM_PATHS_SCALING_GOVERNOR = "/scaling_governor";
/** @brief Path to the vm/page-cluster kernel parameter. */
const char* const SYSTEM_PATHS_PAGE_CLUSTER = "/proc/sys/vm/page-cluster";
/** @brief Path to the kernel scheduler latency parameter. */
const char* const SYSTEM_PATHS_SCHED_LATENCY_NS =
    "/proc/sys/kernel/sched_latency_ns";
/** @brief Path to the kernel scheduler minimum granularity parameter. */
const char* const SYSTEM_PATHS_SCHED_MIN_GRANULARITY_NS =
    "/proc/sys/kernel/sched_min_granularity_ns";

/**
 * @brief Applies all defined static system tweaks.
 *
 * Iterates through all CPU cores to set the governor, then applies
 * various kernel and system property tweaks for memory, scheduling,
 * and debugging.
 */
void systemTweaker_applyAll(void) {
    LOGI("Applying static system tweaks...");

    long coreCount = sysconf(_SC_NPROCESSORS_ONLN);

    if (coreCount > 0) {
        LOGI("Applying governor tweak for %ld CPU cores.", coreCount);
        for (unsigned int i = 0; i < (unsigned int)coreCount; ++i) {
            char path[PATH_MAX];
            snprintf(path, sizeof(path), "%s%u%s",
                     SYSTEM_PATHS_CPU_POLICY_DIR, i,
                     SYSTEM_PATHS_SCALING_GOVERNOR);
            if (!systemUtils_applyTweak(path, TWEAK_VALUES_GOVERNOR)) {
                LOGE("Failed to set governor for CPU %u.", i);
            }
        }
    } else {
        LOGE("Failed to detect CPU core count, skipping governor tweak.");
    }

    /* VM Tweaks */
    systemUtils_applyTweak(SYSTEM_PATHS_PAGE_CLUSTER,
                           TWEAK_VALUES_PAGE_CLUSTER);
    systemUtils_setSystemProp("lmk.minfree_levels",
                              TWEAK_VALUES_LMK_MINFREE_LEVELS);
    systemUtils_setSystemProp("persist.sys.lmk.reportkills",
                              TWEAK_VALUES_LMK_REPORT_KILLS);

    /* Scheduler Tweaks */
    systemUtils_applyTweak(SYSTEM_PATHS_SCHED_LATENCY_NS,
                           TWEAK_VALUES_SCHED_LATENCY_NS);
    systemUtils_applyTweak(SYSTEM_PATHS_SCHED_MIN_GRANULARITY_NS,
                           TWEAK_VALUES_SCHED_MIN_GRANULARITY_NS);

    /* Security/Debug Tweaks */
    systemUtils_setSystemProp("persist.service.adb.enable",
                              TWEAK_VALUES_ADB_ENABLED);
    systemUtils_setSystemProp("persist.service.debuggable",
                              TWEAK_VALUES_DEBUGGABLE_ENABLED);

    LOGI("Static tweaks applied successfully.");
}