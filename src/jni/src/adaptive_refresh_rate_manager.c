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
 * @file adaptive_refresh_rate_manager.c
 * @brief Implementation of the Adaptive Refresh Rate Manager.
 *
 * Monitors a specific input device for touch events. When touch events
 * are detected, it switches the display to a high refresh rate. After a
 * period of inactivity, it reverts to a lower, power-saving refresh rate.
 */

#include "include/adaptive_refresh_rate_manager.h"
#include "include/system_utils.h"
#include "include/fd_wrapper.h"
#include "include/logging.h"
#include "include/main_daemon.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/input.h>
#include <sys/select.h>
#include <time.h>

/** @brief Filesystem path to the touch input event device. */
const char* const REFRESH_RATE_CONFIG_TOUCH_DEVICE_PATH = "/dev/input/event3";
/** @brief The refresh rate (in Hz) to use when idle. */
const float REFRESH_RATE_CONFIG_LOW_REFRESH_RATE = 60.0f;
/** @brief The refresh rate (in Hz) to use when active (touching). */
const float REFRESH_RATE_CONFIG_HIGH_REFRESH_RATE = 90.0f;
/** @brief The Android system setting property key for refresh rate. */
const char* const REFRESH_RATE_CONFIG_REFRESH_RATE_PROPERTY = "min_refresh_rate";
/** @brief Duration of inactivity (in seconds) to wait before switching to low RR. */
const long REFRESH_RATE_CONFIG_IDLE_TIMEOUT_SEC = 4;
/** @brief Max number of consecutive read/select errors before stopping. */
const int REFRESH_RATE_CONFIG_MAX_CONSECUTIVE_ERRORS = 10;
/** @brief Size of the buffer for reading raw input events (holds 64 events). */
const size_t REFRESH_RATE_CONFIG_INPUT_EVENT_BUFFER_SIZE = 64;

/**
 * @brief Sets the system's minimum refresh rate.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 * @param newMode The target refresh rate mode (RR_LOW or RR_HIGH).
 */
static void adaptiveRefreshRateManager_setRefreshRate(
    AdaptiveRefreshRateManager* this_ptr,
    enum RefreshRateMode newMode
);

/**
 * @brief Calculates the difference in seconds between two timespecs.
 *
 * @param start The start time.
 * @param end The end time.
 * @return The difference (end - start) in seconds, as a double.
 */
static double get_timespec_diff_sec(
    struct timespec* start,
    struct timespec* end
);

/**
 * @brief The main function for the input monitor thread.
 *
 * Uses `select()` to wait for events on the touch device. Manages
 * transitions between RR_LOW and RR_HIGH modes based on touch
 * activity and idle timeouts.
 *
 * @param arg A void pointer to the AdaptiveRefreshRateManager instance.
 * @return Always returns NULL.
 */
static void* adaptiveRefreshRateManager_monitor(void* arg);

static double get_timespec_diff_sec(
    struct timespec* start,
    struct timespec* end
) {
    return (double)(end->tv_sec - start->tv_sec) +
           (double)(end->tv_nsec - start->tv_nsec) / 1000000000.0;
}

static void* adaptiveRefreshRateManager_monitor(void* arg) {
    AdaptiveRefreshRateManager* this_ptr = (AdaptiveRefreshRateManager*) arg;
    bool running = true;

    FdWrapper fd;
    if (!fdWrapper_init_path(&fd, REFRESH_RATE_CONFIG_TOUCH_DEVICE_PATH,
                             O_RDONLY | O_NONBLOCK)) {
        LOGE("RefreshManager: Failed to open %s (errno: %d - %s). Exiting.",
             REFRESH_RATE_CONFIG_TOUCH_DEVICE_PATH, errno, strerror(errno));
        return NULL;
    }

    /* Start in low power mode */
    adaptiveRefreshRateManager_setRefreshRate(this_ptr, RR_LOW);
    clock_gettime(CLOCK_MONOTONIC, &this_ptr->lastTouchTime);

    int consecutiveErrors = 0;
    char buffer[sizeof(struct input_event) *
                REFRESH_RATE_CONFIG_INPUT_EVENT_BUFFER_SIZE];

    struct timespec error_sleep_time = {1, 0}; /* 1 second */

    while (running && !g_shutdown_requested) {
        fd_set readFds;
        FD_ZERO(&readFds);
        FD_SET(fdWrapper_get(&fd), &readFds);

        /* Short timeout to check for idle state */
        struct timeval timeout_val;
        timeout_val.tv_sec = 0;
        timeout_val.tv_usec = 100000; /* 100ms */

        int selectResult = select(fdWrapper_get(&fd) + 1, &readFds,
                                  NULL, NULL, &timeout_val);

        if (selectResult > 0) {
            /* Input event detected */
            consecutiveErrors = 0;

            /* Drain all pending events */
            while (fdWrapper_read(&fd, buffer, sizeof(buffer)) > 0);

            clock_gettime(CLOCK_MONOTONIC, &this_ptr->lastTouchTime);
            if (this_ptr->currentMode != RR_HIGH) {
                LOGI("Touch detected -> Switching to %.1fHz.",
                     REFRESH_RATE_CONFIG_HIGH_REFRESH_RATE);
                adaptiveRefreshRateManager_setRefreshRate(this_ptr, RR_HIGH);
            }
        } else if (selectResult == 0) {
            /* Timeout - no input */
            struct timespec now;
            clock_gettime(CLOCK_MONOTONIC, &now);
            double idleDuration =
                get_timespec_diff_sec(&this_ptr->lastTouchTime, &now);

            if (idleDuration >= REFRESH_RATE_CONFIG_IDLE_TIMEOUT_SEC &&
                this_ptr->currentMode == RR_HIGH) {
                LOGI("No activity -> Reverting to %.1fHz.",
                     REFRESH_RATE_CONFIG_LOW_REFRESH_RATE);
                adaptiveRefreshRateManager_setRefreshRate(this_ptr, RR_LOW);
            }
        } else {
            /* select() error */
            if (errno == EINTR) {
                continue;
            }

            consecutiveErrors++;
            LOGE("RefreshManager: select() error (errno: %d - %s), attempt %d/%d",
                 errno, strerror(errno), consecutiveErrors,
                 REFRESH_RATE_CONFIG_MAX_CONSECUTIVE_ERRORS);

            if (consecutiveErrors >= REFRESH_RATE_CONFIG_MAX_CONSECUTIVE_ERRORS) {
                LOGE("RefreshManager: Too many errors, stopping monitoring.");
                break;
            }
            nanosleep(&error_sleep_time, NULL);
        }

        pthread_mutex_lock(&this_ptr->mutex);
        running = this_ptr->isRunning;
        pthread_mutex_unlock(&this_ptr->mutex);
    }

    fdWrapper_destroy(&fd);
    LOGD("RefreshManager: Monitor thread exited.");
    return NULL;
}

static void adaptiveRefreshRateManager_setRefreshRate(
    AdaptiveRefreshRateManager* this_ptr,
    enum RefreshRateMode newMode
) {
    if (newMode == this_ptr->currentMode) {
        return;
    }

    float rate = (newMode == RR_HIGH)
                     ? REFRESH_RATE_CONFIG_HIGH_REFRESH_RATE
                     : REFRESH_RATE_CONFIG_LOW_REFRESH_RATE;

    LOGD("RefreshManager: Requesting switch to %s mode (%.1fHz)",
         (newMode == RR_HIGH ? "HIGH" : "LOW"), rate);

    char rateString[16];
    snprintf(rateString, sizeof(rateString), "%.1f", rate);

    if (systemUtils_setAndroidSetting(REFRESH_RATE_CONFIG_REFRESH_RATE_PROPERTY,
                                      rateString)) {
        this_ptr->currentMode = newMode;
    }
}

/**
 * @brief Creates and initializes a new AdaptiveRefreshRateManager instance.
 *
 * Allocates memory for the manager struct and initializes its components,
 * including the mutex and default state.
 *
 * @return A pointer to the newly created AdaptiveRefreshRateManager, or
 * NULL if memory allocation fails.
 */
AdaptiveRefreshRateManager* adaptiveRefreshRateManager_create() {
    AdaptiveRefreshRateManager* this_ptr =
        (AdaptiveRefreshRateManager*) malloc(
            sizeof(AdaptiveRefreshRateManager));
    if (!this_ptr) {
        LOGE("RefreshManager: Failed to allocate memory");
        return NULL;
    }

    pthread_mutex_init(&this_ptr->mutex, NULL);
    this_ptr->isRunning = true;
    this_ptr->currentMode = RR_UNKNOWN;
    this_ptr->monitorThread = (pthread_t)0;
    clock_gettime(CLOCK_MONOTONIC, &this_ptr->lastTouchTime);

    return this_ptr;
}

/**
 * @brief Starts the input monitoring thread.
 *
 * Creates and launches the background thread that listens for touch
 * input events and manages the refresh rate state.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 * @return true if the thread was started successfully, false otherwise.
 */
bool adaptiveRefreshRateManager_start(AdaptiveRefreshRateManager* this_ptr) {
    LOGI("RefreshManager: Starting monitoring on: %s",
         REFRESH_RATE_CONFIG_TOUCH_DEVICE_PATH);
    LOGI("RefreshManager: LOW mode: %.1fHz, HIGH mode: %.1fHz",
         REFRESH_RATE_CONFIG_LOW_REFRESH_RATE,
         REFRESH_RATE_CONFIG_HIGH_REFRESH_RATE);

    if (pthread_create(&this_ptr->monitorThread, NULL,
                       adaptiveRefreshRateManager_monitor, this_ptr) != 0) {
        LOGE("RefreshManager: Failed to create monitor thread (errno: %d - %s)",
             errno, strerror(errno));
        this_ptr->monitorThread = (pthread_t)0;
        return false;
    }
    return true;
}

/**
 * @brief Signals the input monitoring thread to stop and waits for it.
 *
 * Sets the `isRunning` flag to false and joins the monitor thread
 * to ensure a clean shutdown. Also reverts the refresh rate to
 * the low-power mode.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 */
void adaptiveRefreshRateManager_stop(AdaptiveRefreshRateManager* this_ptr) {
    if (!this_ptr) {
        return;
    }

    pthread_mutex_lock(&this_ptr->mutex);
    this_ptr->isRunning = false;
    pthread_mutex_unlock(&this_ptr->mutex);

    if (this_ptr->monitorThread != (pthread_t)0) {
        pthread_join(this_ptr->monitorThread, NULL);
        this_ptr->monitorThread = (pthread_t)0;
    }

    LOGI("RefreshManager: Monitoring stopped. Reverting to power-saving mode.");
    adaptiveRefreshRateManager_setRefreshRate(this_ptr, RR_LOW);
}

/**
 * @brief Stops the manager and frees all associated resources.
 *
 * Calls `adaptiveRefreshRateManager_stop` to terminate the thread, then
 * destroys the mutex and frees the manager struct itself.
 *
 * @param this_ptr A pointer to the AdaptiveRefreshRateManager instance.
 */
void adaptiveRefreshRateManager_destroy(AdaptiveRefreshRateManager* this_ptr) {
    if (!this_ptr) {
        return;
    }
    adaptiveRefreshRateManager_stop(this_ptr);
    pthread_mutex_destroy(&this_ptr->mutex);
    free(this_ptr);
}