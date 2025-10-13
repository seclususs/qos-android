/*
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
#ifndef CONFIG_H
#define CONFIG_H

#include <chrono>

using namespace std::chrono_literals;

namespace TweakValues {
    constexpr const char* kAppName = "AdaptiveDaemon";
    constexpr const char* kGovernor = "schedutil";
    constexpr const char* kPageCluster = "0";
    constexpr const char* kSchedLatencyNs = "18000000";
    constexpr const char* kSchedMinGranularityNs = "2250000";
}

namespace SystemPaths {
    constexpr const char* kCpuPolicyDir = "/sys/devices/system/cpu/cpufreq/policy";
    constexpr const char* kScalingGovernor = "/scaling_governor";
    constexpr const char* kPageCluster = "/proc/sys/vm/page-cluster";
    constexpr const char* kSchedLatencyNs = "/proc/sys/kernel/sched_latency_ns";
    constexpr const char* kSchedMinGranularityNs = "/proc/sys/kernel/sched_min_granularity_ns";
    constexpr const char* kSwappiness = "/proc/sys/vm/swappiness";
    constexpr const char* kVFSCachePressure = "/proc/sys/vm/vfs_cache_pressure";
}

namespace MemoryTweakValues {
    constexpr const char* kSwappinessLow = "20";
    constexpr const char* kVFSCachePressureLow = "50";
    constexpr const char* kSwappinessMid = "100";
    constexpr const char* kVFSCachePressureMid = "100";
    constexpr const char* kSwappinessHigh = "150";
    constexpr const char* kVFSCachePressureHigh = "200";

    constexpr int kGoToHighThreshold = 20;
    constexpr int kGoToLowThreshold = 45;
    constexpr int kReturnToMidFromLowThreshold = 40;
    constexpr int kReturnToMidFromHighThreshold = 25;
}

namespace RefreshRateConfig {
    constexpr const char* kTouchDevicePath = "/dev/input/event3";
    constexpr float kLowRefreshRate = 60.0f;
    constexpr float kHighRefreshRate = 90.0f;
    constexpr const char* kRefreshRateProperty = "min_refresh_rate";
    constexpr auto kIdleTimeout = 4s;
}

#endif // CONFIG_H