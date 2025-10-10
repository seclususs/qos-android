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

#include "cpu_manager.h"
#include <thread>
#include <mutex>
#include <condition_variable>
#include <chrono>

class BoostManager {
public:

    BoostManager(CpuManager& cpu_manager);
    ~BoostManager();

    BoostManager(const BoostManager&) = delete;
    BoostManager& operator=(const BoostManager&) = delete;
    void requestBoost(BoostLevel level, int duration_ms);

private:
    void workerThread();

    CpuManager& cpu_manager_;
    std::thread worker_;
    std::mutex mutex_;
    std::condition_variable cv_;
    bool stop_thread_ = false;

    std::chrono::steady_clock::time_point boost_end_time_;
};
