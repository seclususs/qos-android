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
#ifndef MEMORY_MANAGER_H
#define MEMORY_MANAGER_H

#include <thread>
#include <atomic>

/**
 * @class AdaptiveMemoryManager
 * @brief Dynamically adjusts VM parameters based on memory availability.
 *
 * It runs a background thread to periodically check the available system memory
 * and transitions between different states (LOW, MID, HIGH pressure) to apply
 * corresponding kernel tweaks.
 */
class AdaptiveMemoryManager {
public:
    /**
     * @brief Constructs the AdaptiveMemoryManager.
     */
    AdaptiveMemoryManager();

    /**
     * @brief Starts the memory monitoring thread.
     */
    void start();

    /**
     * @brief Stops the memory monitoring thread.
     */
    void stop();

private:
    /**
     * @enum MemoryState
     * @brief Represents the current memory pressure state.
     */
    enum class MemoryState { 
        LOW,        ///< Low memory pressure (plenty of free RAM).
        MID,        ///< Medium memory pressure.
        HIGH,       ///< High memory pressure (low free RAM).
        UNKNOWN     ///< Initial state before the first check.
    };

    /**
     * @brief The main loop for the monitoring thread.
     * Periodically checks RAM and calls applyMemoryTweaks if the state changes.
     */
    void monitorLoop();

    /**
     * @brief Applies kernel VM tweaks based on the new memory state.
     * @param newState The memory state to transition to.
     */
    void applyMemoryTweaks(MemoryState newState);

    /**
     * @brief Calculates the percentage of available RAM.
     * @return The percentage of free RAM, or -1 on error.
     */
    int getFreeRamPercentage();

    /// @brief The background thread for monitoring memory.
    std::thread monitorThread_;
    
    /// @brief Atomic flag to control the running state of the monitor thread.
    std::atomic<bool> isRunning_;
    
    /// @brief The current memory pressure state.
    MemoryState currentState_;
};

#endif // MEMORY_MANAGER_H