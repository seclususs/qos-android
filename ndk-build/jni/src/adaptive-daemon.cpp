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
#include "adaptive-daemon.h"
#include "logger.h"
#include "config.h"
#include "hardware-interface.h"
#include <thread>

AdaptiveDaemon::AdaptiveDaemon() {
    memoryManager_ = std::make_unique<AdaptiveMemoryManager>();
    refreshRateManager_ = std::make_unique<AdaptiveRefreshRateManager>();
}

void AdaptiveDaemon::applyStaticTweaks() {
    LOGI("Applying static system tweaks...");
    
    // Tweak CPU Governor
    const unsigned int coreCount = std::thread::hardware_concurrency();
    if (coreCount > 0) {
        for (unsigned int i = 0; i < coreCount; ++i) {
            std::string path = std::string(SystemPaths::kCpuPolicyDir) + std::to_string(i) + SystemPaths::kScalingGovernor;
            if (write_to_file(path.c_str(), TweakValues::kGovernor) != 0) {
                 LOGE("Failed to set governor for CPU %u.", i);
            }
        }
    } else {
        LOGE("Failed to detect CPU core count.");
    }
    
    // Other Tweaks
    write_to_file(SystemPaths::kPageCluster, TweakValues::kPageCluster);
    write_to_file(SystemPaths::kSchedLatencyNs, TweakValues::kSchedLatencyNs);
    write_to_file(SystemPaths::kSchedMinGranularityNs, TweakValues::kSchedMinGranularityNs);

    LOGI("Static tweaks applied successfully.");
}


void AdaptiveDaemon::run() {
    applyStaticTweaks();
    
    memoryManager_->start();
    refreshRateManager_->start();
    
    LOGI("All services started successfully.");
}

void AdaptiveDaemon::stop() {
    if (refreshRateManager_) {
        refreshRateManager_->stop();
    }
    if (memoryManager_) {
        memoryManager_->stop();
    }
    LOGI("All services stopped successfully.");
}