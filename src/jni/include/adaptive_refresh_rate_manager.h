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
 * @file adaptive_refresh_rate_manager.h
 * @brief Public interface for the Adaptive Refresh Rate Manager.
 *
 * Defines structures, constants, and functions to monitor touch input
 * and automatically switch the display's refresh rate between a low-power
 * (idle) mode and a high-performance (active) mode.
 */

#ifndef ADAPTIVE_REFRESH_RATE_MANAGER_H
#define ADAPTIVE_REFRESH_RATE_MANAGER_H

#include <pthread.h>
#include <stdbool.h>
#include <time.h>
#include <stddef.h>

/** @brief Filesystem path to the touch input event device. */
extern const char* const REFRESH_RATE_CONFIG_TOUCH_DEVICE_PATH;
/** @brief The refresh rate (in Hz) to use when idle. */
extern const float REFRESH_RATE_CONFIG_LOW_REFRESH_RATE;
/** @brief The refresh rate (in Hz) to use when active (touching). */
extern const float REFRESH_RATE_CONFIG_HIGH_REFRESH_RATE;
/** @brief The Android system setting property key for refresh rate. */
extern const char* const REFRESH_RATE_CONFIG_REFRESH_RATE_PROPERTY;
/** @brief Duration of inactivity (in seconds) to wait before switching to low RR. */
extern const long REFRESH_RATE_CONFIG_IDLE_TIMEOUT_SEC;
/** @brief Max number of consecutive read/select errors before stopping. */
extern const int REFRESH_RATE_CONFIG_MAX_CONSECUTIVE_ERRORS;
/** @brief Size of the buffer for reading raw input events. */
extern const size_t REFRESH_RATE_CONFIG_INPUT_EVENT_BUFFER_SIZE;

/**
 * @enum RefreshRateMode
 * @brief Represents the target refresh rate state.
 */
enum RefreshRateMode {
    RR_LOW,     /**< Low refresh rate (idle) mode. */
    RR_HIGH,    /**< High refresh rate (active) mode. */
    RR_UNKNOWN  /**< Initial state before first setting. */
};

/**
 * @struct AdaptiveRefreshRateManager
 * @brief Manages the state and resources for adaptive refresh rate control.
 *
 * Holds the state for the input monitoring thread, including its running
 * status, synchronization mutex, current refresh rate mode, and the
 * timestamp of the last detected touch event.
 */
typedef struct {
    pthread_t monitorThread;      /**< Handle for the monitor thread. */
    pthread_mutex_t mutex;        /**< Mutex for synchronizing access to state. */
    bool isRunning;               /**< Flag to control the monitor thread loop. */
    enum RefreshRateMode currentMode; /**< The current refresh rate mode. */
    struct timespec lastTouchTime;/**< Timestamp of the last touch event. */
} AdaptiveRefreshRateManager;

/**
 * @brief Creates and initializes a new AdaptiveRefreshRateManager instance.
 *
 * Allocates memory for the manager struct and initializes its components,
 * including the mutex and default state.
 *
 * @return A pointer to the newly created AdaptiveRefreshRateManager, or
 * NULL if memory allocation fails.
 */
AdaptiveRefreshRateManager* adaptiveRefreshRateManager_create(void);

/**
 * @brief Starts the input monitoring thread.
 *
 * Creates and launches the background thread that listens for touch
 * input events and manages the refresh rate state.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 * @return true if the thread was started successfully, false otherwise.
 */
bool adaptiveRefreshRateManager_start(AdaptiveRefreshRateManager* this_ptr);

/**
 * @brief Signals the input monitoring thread to stop and waits for it.
 *
 * Sets the `isRunning` flag to false and joins the monitor thread
 * to ensure a clean shutdown. Also reverts the refresh rate to
 * the low-power mode.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 */
void adaptiveRefreshRateManager_stop(AdaptiveRefreshRateManager* this_ptr);

/**
 * @brief Stops the manager and frees all associated resources.
 *
 * Calls `adaptiveRefreshRateManager_stop` to terminate the thread, then
 * destroys the mutex and frees the manager struct itself.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 */
void adaptiveRefreshRateManager_destroy(AdaptiveRefreshRateManager* this_ptr);

#endif // ADAPTIVE_REFRESH_RATE_MANAGER_H