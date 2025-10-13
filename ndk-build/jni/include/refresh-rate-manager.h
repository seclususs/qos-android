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
#ifndef REFRESH_RATE_MANAGER_H
#define REFRESH_RATE_MANAGER_H

#include <thread>
#include <atomic>
#include <string>

/**
 * @class AdaptiveRefreshRateManager
 * @brief Switches refresh rate based on touch activity.
 *
 * Runs a background thread that listens for events on a specified touch input
 * device. When touch input is detected, it switches to a high refresh rate.
 * After a period of inactivity, it reverts to a low refresh rate.
 */
class AdaptiveRefreshRateManager {
public:
    /**
     * @brief Constructs the AdaptiveRefreshRateManager.
     */
    AdaptiveRefreshRateManager();

    /**
     * @brief Starts the touch input monitoring thread.
     */
    void start();

    /**
     * @brief Stops the monitoring thread and sets refresh rate to low.
     */
    void stop();

private:
    /**
     * @enum RefreshRateMode
     * @brief Represents the two possible refresh rate states.
     */
    enum class RefreshRateMode { 
        LOW,      ///< Low power, lower refresh rate (e.g., 60Hz).
        HIGH,     ///< High performance, higher refresh rate (e.g., 90Hz).
        UNKNOWN   ///< Initial state before the first setting is applied.
    };
    
    /**
     * @brief The main loop for the monitoring thread.
     * Waits for touch input or timeout and calls setRefreshRate accordingly.
     */
    void monitorLoop();

    /**
     * @brief Sets the system's display refresh rate.
     * @param newMode The refresh rate mode to switch to.
     */
    void setRefreshRate(RefreshRateMode newMode);

    /**
     * @brief Executes a shell command to change an Android system setting.
     * @param property The name of the system property to change.
     * @param value The new value for the property.
     * @return True on success, false on failure.
     */
    bool setAndroidSetting(const std::string& property, const std::string& value);

    /// @brief The background thread for monitoring touch input.
    std::thread monitorThread_;
    
    /// @brief Atomic flag to control the running state of the monitor thread.
    std::atomic<bool> isRunning_;
    
    /// @brief The current refresh rate mode.
    RefreshRateMode currentMode_;
};

#endif // REFRESH_RATE_MANAGER_H