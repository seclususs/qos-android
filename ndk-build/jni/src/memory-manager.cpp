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
#include "memory-manager.h"
#include "logger.h"
#include "config.h"
#include "hardware-interface.h"
#include <chrono>

AdaptiveMemoryManager::AdaptiveMemoryManager()
    : isRunning_(false), currentState_(MemoryState::UNKNOWN) {}

void AdaptiveMemoryManager::start() {
    isRunning_.store(true, std::memory_order_release);
    monitorThread_ = std::thread(&AdaptiveMemoryManager::monitorLoop, this);
    LOGI("MemoryManager: Memory monitoring started.");
}

void AdaptiveMemoryManager::stop() {
    isRunning_.store(false, std::memory_order_release);
    if (monitorThread_.joinable()) {
        monitorThread_.join();
    }
    LOGI("MemoryManager: Monitoring stopped.");
}

int AdaptiveMemoryManager::getFreeRamPercentage() {
    long memTotal = -1, memAvailable = -1;
    if (read_mem_info(&memTotal, &memAvailable) != 0) {
        LOGE("MemoryManager: Failed to read memory info from C layer.");
        return -1;
    }

    if (memTotal > 0 && memAvailable >= 0) {
        return static_cast<int>((static_cast<double>(memAvailable) / memTotal) * 100.0);
    }
    return -1;
}

void AdaptiveMemoryManager::applyMemoryTweaks(MemoryState newState) {
    if (newState == currentState_) return;

    const char* swappiness = nullptr;
    const char* vfs_cache = nullptr;
    const char* profile_name = "UNKNOWN";

    switch (newState) {
        case MemoryState::LOW:
            profile_name = "LOW";
            swappiness = MemoryTweakValues::kSwappinessLow;
            vfs_cache = MemoryTweakValues::kVFSCachePressureLow;
            break;
        case MemoryState::MID:
            profile_name = "MID";
            swappiness = MemoryTweakValues::kSwappinessMid;
            vfs_cache = MemoryTweakValues::kVFSCachePressureMid;
            break;
        case MemoryState::HIGH:
            profile_name = "HIGH";
            swappiness = MemoryTweakValues::kSwappinessHigh;
            vfs_cache = MemoryTweakValues::kVFSCachePressureHigh;
            break;
        default: return;
    }
    
    LOGI("MemoryManager: Switching to %s memory profile.", profile_name);
    write_to_file(SystemPaths::kSwappiness, swappiness);
    write_to_file(SystemPaths::kVFSCachePressure, vfs_cache);

    currentState_ = newState;
}

void AdaptiveMemoryManager::monitorLoop() {
    // Apply initial state based on current memory
    applyMemoryTweaks(MemoryState::MID);

    while (isRunning_.load(std::memory_order_acquire)) {
        int freeRamPercent = getFreeRamPercentage();
        if (freeRamPercent >= 0) {
            LOGD("MemoryManager: Free RAM percentage: %d%%", freeRamPercent);
            MemoryState newState = currentState_;
            switch (currentState_) {
                case MemoryState::UNKNOWN: // Fallthrough to MID logic for initialization
                case MemoryState::MID:
                    if (freeRamPercent < MemoryTweakValues::kGoToHighThreshold) newState = MemoryState::HIGH;
                    else if (freeRamPercent > MemoryTweakValues::kGoToLowThreshold) newState = MemoryState::LOW;
                    break;
                case MemoryState::HIGH:
                    if (freeRamPercent >= MemoryTweakValues::kReturnToMidFromHighThreshold) newState = MemoryState::MID;
                    break;
                case MemoryState::LOW:
                    if (freeRamPercent <= MemoryTweakValues::kReturnToMidFromLowThreshold) newState = MemoryState::MID;
                    break;
            }
            applyMemoryTweaks(newState);
        }
        std::this_thread::sleep_for(std::chrono::seconds(5));
    }
}