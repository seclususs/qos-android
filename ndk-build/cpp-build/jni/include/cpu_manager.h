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
#pragma once

#include "config.h"
#include <string>
#include <vector>
#include <atomic>

class CpuManager {
public:
    CpuManager();
    void initialize();
    void applyPerformanceBoost(BoostLevel level);
    void restoreDefaults();
    int getTemperature();
    BoostLevel getCurrentBoostLevel();

private:
    void findThermalPath();
    void initializeFrequencies();

    std::string thermal_path_;
    std::vector<std::string> original_min_freqs_;
    std::vector<std::string> medium_boost_min_freqs_;
    std::vector<std::string> full_boost_min_freqs_;
    std::vector<int> big_core_indices_;
    std::atomic<BoostLevel> current_boost_level_;
};
