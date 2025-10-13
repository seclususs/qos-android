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
#ifndef ADAPTIVE_DAEMON_H
#define ADAPTIVE_DAEMON_H

#include "memory-manager.h"
#include "refresh-rate-manager.h"
#include <memory>

/**
 * @class AdaptiveDaemon
 * @brief Manages the lifecycle of all adaptive services.
 *
 * This class encapsulates the primary logic of the daemon. It initializes
 * managers for various subsystems, applies initial system-wide tweaks,
 * and handles the start and stop signals for the entire service.
 */
class AdaptiveDaemon {
public:
    /**
     * @brief Constructs the AdaptiveDaemon and initializes its managers.
     */
    AdaptiveDaemon();

    /**
     * @brief Starts all managed services and applies static tweaks.
     * This function applies one-time system configurations and starts the
     * monitoring loops for all managers (e.g., memory, refresh rate).
     */
    void run();

    /**
     * @brief Stops all managed services gracefully.
     * This function signals all running manager threads to terminate and
     * performs any necessary cleanup.
     */
    void stop();

private:
    /**
     * @brief Applies system tweaks that are set once at startup.
     * This includes setting the CPU governor, kernel scheduler parameters,
     * and other system-level configurations that do not change during runtime.
     */
    void applyStaticTweaks();
    
    /// @brief A unique pointer to the memory manager instance.
    std::unique_ptr<AdaptiveMemoryManager> memoryManager_;
    
    /// @brief A unique pointer to the refresh rate manager instance.
    std::unique_ptr<AdaptiveRefreshRateManager> refreshRateManager_;
};

#endif // ADAPTIVE_DAEMON_H