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
 * @file system_tweaker.h
 * @brief Public interface for applying static system tweaks.
 *
 * Defines constants for system paths and tweak values, and provides
 * a function to apply all static (one-time) optimizations at startup.
 */

#ifndef SYSTEM_TWEAKER_H
#define SYSTEM_TWEAKER_H

#include <limits.h> /* For PATH_MAX */

/* Tweak Values */

/** @brief The desired CPU governor (e.g., "schedutil"). */
extern const char* const TWEAK_VALUES_GOVERNOR;
/** @brief The desired vm/page-cluster value. */
extern const char* const TWEAK_VALUES_PAGE_CLUSTER;
/** @brief The desired LMK (Low Memory Killer) minfree levels. */
extern const char* const TWEAK_VALUES_LMK_MINFREE_LEVELS;
/** @brief The desired LMK kill reporting setting. */
extern const char* const TWEAK_VALUES_LMK_REPORT_KILLS;
/** @brief The desired kernel scheduler latency. */
extern const char* const TWEAK_VALUES_SCHED_LATENCY_NS;
/** @brief The desired kernel scheduler minimum granularity. */
extern const char* const TWEAK_VALUES_SCHED_MIN_GRANULARITY_NS;
/** @brief The desired state for ADB (Android Debug Bridge). */
extern const char* const TWEAK_VALUES_ADB_ENABLED;
/** @brief The desired state for global debuggable property. */
extern const char* const TWEAK_VALUES_DEBUGGABLE_ENABLED;

/* System Paths */

/** @brief Base directory for CPU frequency policies. */
extern const char* const SYSTEM_PATHS_CPU_POLICY_DIR;
/** @brief Filename for the scaling governor within a policy directory. */
extern const char* const SYSTEM_PATHS_SCALING_GOVERNOR;
/** @brief Path to the vm/page-cluster kernel parameter. */
extern const char* const SYSTEM_PATHS_PAGE_CLUSTER;
/** @brief Path to the kernel scheduler latency parameter. */
extern const char* const SYSTEM_PATHS_SCHED_LATENCY_NS;
/** @brief Path to the kernel scheduler minimum granularity parameter. */
extern const char* const SYSTEM_PATHS_SCHED_MIN_GRANULARITY_NS;

/**
 * @brief Applies all defined static system tweaks.
 *
 * This function is intended to be called once at daemon startup
 * to set persistent or semi-persistent system parameters.
 */
void systemTweaker_applyAll(void);

#endif // SYSTEM_TWEAKER_H