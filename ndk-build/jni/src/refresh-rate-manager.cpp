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
#include "refresh-rate-manager.h"
#include "logger.h"
#include "config.h"
#include "hardware-interface.h"
#include <string>
#include <sstream>
#include <iomanip>
#include <chrono>

AdaptiveRefreshRateManager::AdaptiveRefreshRateManager()
    : isRunning_(false), currentMode_(RefreshRateMode::UNKNOWN) {}

void AdaptiveRefreshRateManager::start() {
    isRunning_.store(true, std::memory_order_release);
    monitorThread_ = std::thread(&AdaptiveRefreshRateManager::monitorLoop, this);
    LOGI("RefreshManager: Touch monitoring started.");
}

void AdaptiveRefreshRateManager::stop() {
    isRunning_.store(false, std::memory_order_release);
    if (monitorThread_.joinable()) {
        monitorThread_.join();
    }
    LOGI("RefreshManager: Monitoring stopped.");
    // Ensure we leave the system in a power-saving state
    setRefreshRate(RefreshRateMode::LOW);
}

bool AdaptiveRefreshRateManager::setAndroidSetting(const std::string& property, const std::string& value) {
    std::string cmd = "settings put system " + property + " " + value;
    char output_buffer[128];
    int exit_code = execute_command(cmd.c_str(), output_buffer, sizeof(output_buffer));

    if (exit_code == 0) {
        LOGI("Successfully set '%s' to %s", property.c_str(), value.c_str());
        return true;
    }
    
    LOGE("Failed to set '%s' to %s. Code: %d, Output: %s", property.c_str(), value.c_str(), exit_code, output_buffer);
    return false;
}

void AdaptiveRefreshRateManager::setRefreshRate(RefreshRateMode newMode) {
    if (newMode == currentMode_) return;

    std::stringstream rateStream;
    rateStream << std::fixed << std::setprecision(1);

    if (newMode == RefreshRateMode::HIGH) {
        LOGI("Touch detected -> Switching to %.1fHz.", RefreshRateConfig::kHighRefreshRate);
        rateStream << RefreshRateConfig::kHighRefreshRate;
    } else { // LOW
        LOGI("No activity -> Reverting to %.1fHz.", RefreshRateConfig::kLowRefreshRate);
        rateStream << RefreshRateConfig::kLowRefreshRate;
    }
    
    if (setAndroidSetting(RefreshRateConfig::kRefreshRateProperty, rateStream.str())) {
        currentMode_ = newMode;
    }
}

void AdaptiveRefreshRateManager::monitorLoop() {
    // Start in a power-saving mode
    setRefreshRate(RefreshRateMode::LOW);
    
    while (isRunning_.load(std::memory_order_acquire)) {
        // In low power mode, wait indefinitely for touch.
        // In high power mode, wait for the idle timeout.
        int timeout_ms = -1; // Wait forever by default (LOW mode)
        if (currentMode_ == RefreshRateMode::HIGH) {
            timeout_ms = static_cast<int>(std::chrono::duration_cast<std::chrono::milliseconds>(RefreshRateConfig::kIdleTimeout).count());
        }

        int result = wait_for_input(RefreshRateConfig::kTouchDevicePath, timeout_ms);

        if (!isRunning_.load(std::memory_order_acquire)) break;

        if (result == 1) { // Input event occurred
            setRefreshRate(RefreshRateMode::HIGH);
        } else if (result == 0) { // Timeout occurred
            setRefreshRate(RefreshRateMode::LOW);
        } else { // Error
            LOGE("RefreshManager: Error while monitoring input. Pausing.");
            std::this_thread::sleep_for(std::chrono::seconds(2)); // Avoid fast error loops
        }
    }
}