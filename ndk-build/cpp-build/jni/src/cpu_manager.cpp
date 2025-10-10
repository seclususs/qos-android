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
#include "include/cpu_manager.h"
#include "include/system_utils.h"
#include "include/log_utils.h"
#include <dirent.h>
#include <unistd.h>

CpuManager::CpuManager() : current_boost_level_(BoostLevel::NONE) {}

void CpuManager::initialize() {
    findThermalPath();
    initializeFrequencies();
    LOGI("CpuManager initialized.");
}

void CpuManager::findThermalPath() {
    DIR* dir = opendir("/sys/class/thermal/");
    if (dir == nullptr) {
        LOGE("Unable to open /sys/class/thermal/");
        return;
    }

    struct dirent* entry;
    while ((entry = readdir(dir)) != nullptr) {
        if (std::string(entry->d_name).rfind("thermal_zone", 0) == 0) {
            std::string type_path = "/sys/class/thermal/" + std::string(entry->d_name) + "/type";
            std::string temp_path = "/sys/class/thermal/" + std::string(entry->d_name) + "/temp";

            if (access(type_path.c_str(), R_OK) == 0) {
                std::string type = SystemUtils::readValueFromFile(type_path);
                if (type.find("cpu") != std::string::npos || type.find("cluster") != std::string::npos || type.find("soc") != std::string::npos) {
                    if (access(temp_path.c_str(), R_OK) == 0) {
                        thermal_path_ = temp_path;
                        LOGI("CPU temperature path found: %s", thermal_path_.c_str());
                        closedir(dir);
                        return;
                    }
                }
            }
        }
    }
    closedir(dir);
    LOGE("Unable to find a valid CPU temperature path.");
}

int CpuManager::getTemperature() {
    if (thermal_path_.empty()) {
        return -1;
    }
    std::string temp_str = SystemUtils::readValueFromFile(thermal_path_);
    if (!temp_str.empty()) {
        try {
            return std::stoi(temp_str);
        } catch (...) {
            LOGE("Failed to parse temperature string: %s", temp_str.c_str());
            return -1;
        }
    }
    return -1;
}

void CpuManager::initializeFrequencies() {
    long highest_max_freq = 0;
    std::vector<std::string> all_max_freqs(NUM_CPU_CORES);

    for (int i = 0; i < NUM_CPU_CORES; ++i) {
        std::string min_freq_path = "/sys/devices/system/cpu/cpu" + std::to_string(i) + "/cpufreq/scaling_min_freq";
        std::string max_freq_path = "/sys/devices/system/cpu/cpu" + std::to_string(i) + "/cpufreq/cpuinfo_max_freq";
        
        if (access(min_freq_path.c_str(), F_OK) == 0 && access(max_freq_path.c_str(), F_OK) == 0) {
            original_min_freqs_.push_back(SystemUtils::readValueFromFile(min_freq_path));
            std::string max_freq_str = SystemUtils::readValueFromFile(max_freq_path);
            full_boost_min_freqs_.push_back(max_freq_str);
            all_max_freqs[i] = max_freq_str;
            
            if (!max_freq_str.empty()) {
                try {
                    long current_max_freq = std::stol(max_freq_str);
                    medium_boost_min_freqs_.push_back(std::to_string(current_max_freq / 2));
                    if (current_max_freq > highest_max_freq) highest_max_freq = current_max_freq;
                } catch(...) { medium_boost_min_freqs_.push_back(""); }
            } else { medium_boost_min_freqs_.push_back(""); }
        } else {
            original_min_freqs_.push_back("");
            full_boost_min_freqs_.push_back("");
            medium_boost_min_freqs_.push_back("");
        }
    }

    if (highest_max_freq > 0) {
        big_core_indices_.clear();
        for (int i = 0; i < NUM_CPU_CORES; ++i) {
            if (!all_max_freqs[i].empty()) {
                try {
                    if (std::stol(all_max_freqs[i]) == highest_max_freq) {
                        big_core_indices_.push_back(i);
                    }
                } catch(...) {}
            }
        }
    }
    LOGI("CPU frequencies for tiered boosting initialized. %zu big cores detected.", big_core_indices_.size());
}

void CpuManager::applyPerformanceBoost(BoostLevel level) {
    int current_temp = getTemperature();
    BoostLevel effective_level = level;

    if (current_temp != -1) {
        if (current_temp >= CRITICAL_TEMP) {
            LOGD("Thermal Guard: CPU temperature (%d °C) CRITICAL. Forcing boost to NONE.", current_temp / 1000);
            effective_level = BoostLevel::NONE;
        } else if (current_temp >= WARNING_TEMP && level > BoostLevel::LIGHT) {
             LOGD("Thermal Guard: CPU temperature (%d °C) WARNING. Limiting boost to LIGHT.", current_temp / 1000);
             effective_level = BoostLevel::LIGHT;
        }
    }

    switch (effective_level) {
        case BoostLevel::NONE:
            LOGD("Applying Boost: NONE");
            SystemUtils::applyTweak("/dev/stune/top-app/schedtune.boost", "0");
            SystemUtils::applyTweak("/dev/stune/foreground/schedtune.boost", "5");
            for (size_t i = 0; i < original_min_freqs_.size(); ++i) {
                 if (!original_min_freqs_[i].empty()) {
                    SystemUtils::applyTweak("/sys/devices/system/cpu/cpu" + std::to_string(i) + "/cpufreq/scaling_min_freq", original_min_freqs_[i]);
                }
            }
            break;
        case BoostLevel::LIGHT:
            LOGD("Applying Boost: LIGHT");
            SystemUtils::applyTweak("/dev/stune/foreground/schedtune.boost", "10");
            break;
        case BoostLevel::MEDIUM:
            LOGD("Applying Boost: MEDIUM");
            SystemUtils::applyTweak("/dev/stune/top-app/schedtune.boost", "15");
            for (int core_index : big_core_indices_) {
                if (core_index < medium_boost_min_freqs_.size() && !medium_boost_min_freqs_[core_index].empty()) {
                    SystemUtils::applyTweak("/sys/devices/system/cpu/cpu" + std::to_string(core_index) + "/cpufreq/scaling_min_freq", medium_boost_min_freqs_[core_index]);
                }
            }
            break;
        case BoostLevel::FULL:
            LOGD("Applying Boost: FULL");
            SystemUtils::applyTweak("/dev/stune/top-app/schedtune.boost", "20");
            for (int core_index : big_core_indices_) {
                if (core_index < full_boost_min_freqs_.size() && !full_boost_min_freqs_[core_index].empty()) {
                    SystemUtils::applyTweak("/sys/devices/system/cpu/cpu" + std::to_string(core_index) + "/cpufreq/scaling_min_freq", full_boost_min_freqs_[core_index]);
                }
            }
            break;
    }
    current_boost_level_ = effective_level;
}

void CpuManager::restoreDefaults() {
    LOGI("Restoring default CPU settings...");
    for (size_t i = 0; i < original_min_freqs_.size(); ++i) {
         if (!original_min_freqs_[i].empty()) {
            SystemUtils::applyTweak("/sys/devices/system/cpu/cpu" + std::to_string(i) + "/cpufreq/scaling_min_freq", original_min_freqs_[i]);
        }
    }
    SystemUtils::applyTweak("/dev/stune/top-app/schedtune.boost", "0");
    SystemUtils::applyTweak("/dev/stune/foreground/schedtune.boost", "5");
    LOGI("Default CPU settings restored.");
}

BoostLevel CpuManager::getCurrentBoostLevel() {
    return current_boost_level_.load();
}
