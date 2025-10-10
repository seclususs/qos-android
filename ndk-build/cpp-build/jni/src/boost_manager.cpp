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
#include "include/boost_manager.h"
#include "include/log_utils.h"

BoostManager::BoostManager(CpuManager& cpu_manager) : cpu_manager_(cpu_manager) {
    worker_ = std::thread(&BoostManager::workerThread, this);
}

BoostManager::~BoostManager() {
    {
        std::unique_lock<std::mutex> lock(mutex_);
        stop_thread_ = true;
    }
    cv_.notify_one();
    if (worker_.joinable()) {
        worker_.join();
    }
}

void BoostManager::requestBoost(BoostLevel level, int duration_ms) {
    std::unique_lock<std::mutex> lock(mutex_);
    
    auto new_end_time = std::chrono::steady_clock::now() + std::chrono::milliseconds(duration_ms);

    if (level > cpu_manager_.getCurrentBoostLevel()) {
        cpu_manager_.applyPerformanceBoost(level);
        boost_end_time_ = new_end_time;
    } else if (new_end_time > boost_end_time_) {
        boost_end_time_ = new_end_time;
    }
    
    cv_.notify_one();
}

void BoostManager::workerThread() {
    LOGI("BoostManager worker thread started.");
    std::unique_lock<std::mutex> lock(mutex_);

    while (!stop_thread_) {
        if (cpu_manager_.getCurrentBoostLevel() == BoostLevel::NONE) {
            cv_.wait(lock, [this]{ return stop_thread_ || cpu_manager_.getCurrentBoostLevel() != BoostLevel::NONE; });
        } else {
            auto status = cv_.wait_until(lock, boost_end_time_);

            if (status == std::cv_status::timeout && std::chrono::steady_clock::now() >= boost_end_time_) {
                cpu_manager_.applyPerformanceBoost(BoostLevel::NONE);
            }
        }
    }
    LOGI("BoostManager worker thread stopped.");
}
