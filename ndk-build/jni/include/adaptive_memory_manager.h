/**
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

/**
 * @file adaptive_memory_manager.h
 * @brief Public interface for the Adaptive Memory Manager.
 *
 * Defines the structures, constants, and functions used to monitor system
 * memory and apply different kernel tunables based on memory pressure.
 */

#ifndef ADAPTIVE_MEMORY_MANAGER_H
#define ADAPTIVE_MEMORY_MANAGER_H

#include <pthread.h>
#include <stdbool.h>

/** @brief Path to the meminfo file in procfs. */
extern const char* const SYSTEM_PATHS_MEM_INFO;
/** @brief Path to the swappiness kernel parameter. */
extern const char* const SYSTEM_PATHS_SWAPPINESS;
/** @brief Path to the vfs_cache_pressure kernel parameter. */
extern const char* const SYSTEM_PATHS_VFS_CACHE_PRESSURE;

/** @brief Swappiness value for the LOW memory pressure state. */
extern const char* const MEMORY_TWEAK_VALUES_SWAPPINESS_LOW;
/** @brief VFS cache pressure value for the LOW memory pressure state. */
extern const char* const MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_LOW;
/** @brief Swappiness value for the MID memory pressure state. */
extern const char* const MEMORY_TWEAK_VALUES_SWAPPINESS_MID;
/** @brief VFS cache pressure value for the MID memory pressure state. */
extern const char* const MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_MID;
/** @brief Swappiness value for the HIGH memory pressure state. */
extern const char* const MEMORY_TWEAK_VALUES_SWAPPINESS_HIGH;
/** @brief VFS cache pressure value for the HIGH memory pressure state. */
extern const char* const MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_HIGH;

/** @brief Free RAM percentage threshold to transition to the HIGH state. */
extern const int MEMORY_TWEAK_VALUES_GO_TO_HIGH_THRESHOLD;
/** @brief Free RAM percentage threshold to transition to the LOW state. */
extern const int MEMORY_TWEAK_VALUES_GO_TO_LOW_THRESHOLD;
/** @brief Free RAM percentage threshold to return to MID from LOW state. */
extern const int MEMORY_TWEAK_VALUES_RETURN_TO_MID_FROM_LOW_THRESHOLD;
/** @brief Free RAM percentage threshold to return to MID from HIGH state. */
extern const int MEMORY_TWEAK_VALUES_RETURN_TO_MID_FROM_HIGH_THRESHOLD;

/**
 * @enum MemoryState
 * @brief Represents the perceived system memory pressure.
 */
enum MemoryState {
    MEM_LOW,     /**< Ample free memory. */
    MEM_MID,     /**< Moderate free memory. */
    MEM_HIGH,    /**< Low free memory (high pressure). */
    MEM_UNKNOWN  /**< Initial state before first check. */
};

/**
 * @struct AdaptiveMemoryManager
 * @brief Manages the state and resources for adaptive memory monitoring.
 *
 * This structure holds the state for the memory monitoring thread,
 * including its running status, synchronization mutex, and the current
 * memory pressure state.
 */
typedef struct {
    pthread_t monitorThread;      /**< Handle for the monitor thread. */
    pthread_mutex_t mutex;        /**< Mutex for synchronizing access to state. */
    bool isRunning;               /**< Flag to control the monitor thread loop. */
    enum MemoryState currentState;/**< The current memory pressure state. */
} AdaptiveMemoryManager;

/**
 * @brief Creates and initializes a new AdaptiveMemoryManager instance.
 *
 * Allocates memory for the manager struct and initializes its components,
 * including the mutex and default state.
 *
 * @return A pointer to the newly created AdaptiveMemoryManager, or NULL
 * if memory allocation fails.
 */
AdaptiveMemoryManager* adaptiveMemoryManager_create(void);

/**
 * @brief Starts the memory monitoring thread.
 *
 * Creates and launches the background thread that periodically checks
 * memory usage and applies tunables.
 *
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance.
 */
void adaptiveMemoryManager_start(AdaptiveMemoryManager* this_ptr);

/**
 * @brief Signals the memory monitoring thread to stop and waits for it.
 *
 * Sets the `isRunning` flag to false and joins the monitor thread
 * to ensure a clean shutdown.
 *
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance.
 */
void adaptiveMemoryManager_stop(AdaptiveMemoryManager* this_ptr);

/**
 * @brief Stops the manager and frees all associated resources.
 *
 * Calls `adaptiveMemoryManager_stop` to terminate the thread, then
 * destroys the mutex and frees the manager struct itself.
 *
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance to destroy.
 */
void adaptiveMemoryManager_destroy(AdaptiveMemoryManager* this_ptr);

#endif // ADAPTIVE_MEMORY_MANAGER_H