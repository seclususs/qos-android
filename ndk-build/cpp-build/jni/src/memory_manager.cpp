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
#include "include/memory_manager.h"
#include "include/system_utils.h"
#include "include/log_utils.h"
#include "include/config.h"
#include <fstream>
#include <unistd.h>

MemoryManager::MemoryManager() : vfs_pressure_boosted_(false) {}

void MemoryManager::initialize() {
    original_read_ahead_kb_ = SystemUtils::readValueFromFile(STORAGE_READ_AHEAD_PATH);
    if (original_read_ahead_kb_.empty()) {
        original_read_ahead_kb_ = "128";
    }
    LOGI("MemoryManager initialized. Original read_ahead_kb: %s", original_read_ahead_kb_.c_str());
}

long MemoryManager::getMemInfo(const std::string& key) {
    std::ifstream meminfo("/proc/meminfo");
    std::string line;
    long value = 0;
    if (meminfo.is_open()) {
        while (std::getline(meminfo, line)) {
            if (line.rfind(key, 0) == 0) {
                try {
                    value = std::stol(line.substr(line.find(":") + 1));
                } catch (...) { value = 0; }
                break;
            }
        }
        meminfo.close();
    }
    return value;
}

void MemoryManager::manage() {
    long mem_total_kb = getMemInfo("MemTotal:");
    if (mem_total_kb <= 0) return;

    long mem_available_kb = getMemInfo("MemAvailable:");
    long critical_threshold_kb = mem_total_kb / 10;
    long low_threshold_kb = mem_total_kb / 4;

    if (mem_available_kb < critical_threshold_kb) {
        LOGD("Critical memory pressure (Available: %ld KB). Performing aggressive cleanup.", mem_available_kb);
        if (!vfs_pressure_boosted_) {
            SystemUtils::applyTweak("/proc/sys/vm/vfs_cache_pressure", "200");
            vfs_pressure_boosted_ = true;
        }
        SystemUtils::applyTweak("/proc/sys/vm/compact_memory", "1");
    } else if (mem_available_kb < low_threshold_kb) {
        LOGD("Low memory pressure (Available: %ld KB). Reclaiming cache.", mem_available_kb);
        if (!vfs_pressure_boosted_) {
            SystemUtils::applyTweak("/proc/sys/vm/vfs_cache_pressure", "200");
            vfs_pressure_boosted_ = true;
        }
        SystemUtils::applyTweak("/proc/sys/vm/compact_memory", "1");
    } else if (vfs_pressure_boosted_) {
        LOGD("Memory pressure relieved (Available: %ld KB). Restoring vfs_cache_pressure.", mem_available_kb);
        SystemUtils::applyTweak("/proc/sys/vm/vfs_cache_pressure", "100");
        vfs_pressure_boosted_ = false;
    }
}

void MemoryManager::restoreDefaults() {
    LOGI("Restoring default memory settings...");
    if (!original_read_ahead_kb_.empty()) {
        SystemUtils::applyTweak(STORAGE_READ_AHEAD_PATH, original_read_ahead_kb_);
    }
    SystemUtils::applyTweak("/proc/sys/vm/vfs_cache_pressure", "100");
    LOGI("Default memory settings restored.");
}
