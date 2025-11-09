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
 * @file adaptive_memory_manager.c
 * @brief Implementation of the Adaptive Memory Manager.
 *
 * Monitors available system memory and dynamically adjusts kernel VM
 * (Virtual Memory) parameters like swappiness and vfs_cache_pressure
 * to optimize performance based on memory pressure.
 */

#include "include/adaptive_memory_manager.h"
#include "include/system_utils.h"
#include "include/logging.h"
#include "include/main_daemon.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#include <unistd.h>

/** @brief Path to the meminfo file in procfs. */
const char* const SYSTEM_PATHS_MEM_INFO = "/proc/meminfo";
/** @brief Path to the swappiness kernel parameter. */
const char* const SYSTEM_PATHS_SWAPPINESS = "/proc/sys/vm/swappiness";
/** @brief Path to the vfs_cache_pressure kernel parameter. */
const char* const SYSTEM_PATHS_VFS_CACHE_PRESSURE =
    "/proc/sys/vm/vfs_cache_pressure";
/** @brief Path to the page-cluster kernel parameter. */
const char* const SYSTEM_PATHS_PAGE_CLUSTER = "/proc/sys/vm/page-cluster";

/** @brief Swappiness value for the LOW memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_SWAPPINESS_LOW = "20";
/** @brief VFS cache pressure value for the LOW memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_LOW = "50";
/** @brief Page-cluster value for the LOW memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_PAGE_CLUSTER_LOW_MID = "1";
/** @brief Swappiness value for the MID memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_SWAPPINESS_MID = "100";
/** @brief VFS cache pressure value for the MID memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_MID = "100";
/** @brief Swappiness value for the HIGH memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_SWAPPINESS_HIGH = "150";
/** @brief VFS cache pressure value for the HIGH memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_HIGH = "200";
/** @brief Page-cluster value for the HIGH memory pressure state. */
const char* const MEMORY_TWEAK_VALUES_PAGE_CLUSTER_HIGH = "0";

/** @brief Available RAM threshold (in KB) to transition to the HIGH state. */
const long MEMORY_TWEAK_VALUES_GO_TO_HIGH_THRESHOLD_KB = 1200000; /* 1.2 GB */
/** @brief Available RAM threshold (in KB) to transition to the LOW state. */
const long MEMORY_TWEAK_VALUES_GO_TO_LOW_THRESHOLD_KB = 2700000; /* 2.7 GB */
/** @brief Available RAM threshold (in KB) to return to MID from LOW state. */
const long MEMORY_TWEAK_VALUES_RETURN_TO_MID_FROM_LOW_THRESHOLD_KB = 2400000; /* 2.4 GB */
/** @brief Available RAM threshold (in KB) to return to MID from HIGH state. */
const long MEMORY_TWEAK_VALUES_RETURN_TO_MID_FROM_HIGH_THRESHOLD_KB = 1500000; /* 1.5 GB */

/**
 * @brief Applies new memory kernel tunables based on the new state.
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance.
 * @param newState The memory state to transition to.
 */
static void adaptiveMemoryManager_applyMemoryTweaks(
    AdaptiveMemoryManager* this_ptr,
    enum MemoryState newState
);

/**
 * @brief Reads /proc/meminfo to get available RAM in KB.
 *
 * Reads the "MemAvailable" field from /proc/meminfo.
 *
 * @return The available RAM (in KB), or -1 on error.
 */
static long adaptiveMemoryManager_getFreeRamKb(void);

/**
 * @brief The main function for the memory monitor thread.
 *
 * Periodically checks free RAM and transitions between memory states,
 * applying tunables as needed.
 *
 * @param arg A void pointer to the AdaptiveMemoryManager instance.
 * @return Always returns NULL.
 */
static void* adaptiveMemoryManager_monitor(void* arg);

static void* adaptiveMemoryManager_monitor(void* arg) {
    AdaptiveMemoryManager* this_ptr = (AdaptiveMemoryManager*) arg;
    bool running = true;

    struct timespec sleep_time = {5, 0}; /* 5 seconds */

    while (running && !g_shutdown_requested) {
        long freeRamKb = adaptiveMemoryManager_getFreeRamKb();
        if (freeRamKb >= 0) {
            LOGD("MemoryManager: Available RAM: %ld KB", freeRamKb);
            enum MemoryState newState = this_ptr->currentState;

            switch (this_ptr->currentState) {
                case MEM_UNKNOWN:
                    if (freeRamKb < MEMORY_TWEAK_VALUES_GO_TO_HIGH_THRESHOLD_KB) {
                        newState = MEM_HIGH;
                    } else if (freeRamKb > MEMORY_TWEAK_VALUES_GO_TO_LOW_THRESHOLD_KB) {
                        newState = MEM_LOW;
                    } else {
                        newState = MEM_MID;
                    }
                    break;
                case MEM_HIGH:
                    if (freeRamKb >=
                        MEMORY_TWEAK_VALUES_RETURN_TO_MID_FROM_HIGH_THRESHOLD_KB) {
                        newState = MEM_MID;
                    }
                    break;
                case MEM_MID:
                    if (freeRamKb < MEMORY_TWEAK_VALUES_GO_TO_HIGH_THRESHOLD_KB) {
                        newState = MEM_HIGH;
                    } else if (freeRamKb > MEMORY_TWEAK_VALUES_GO_TO_LOW_THRESHOLD_KB) {
                        newState = MEM_LOW;
                    }
                    break;
                case MEM_LOW:
                    if (freeRamKb <
                        MEMORY_TWEAK_VALUES_RETURN_TO_MID_FROM_LOW_THRESHOLD_KB) {
                        newState = MEM_MID;
                    }
                    break;
            }
            adaptiveMemoryManager_applyMemoryTweaks(this_ptr, newState);
        }

        nanosleep(&sleep_time, NULL);

        pthread_mutex_lock(&this_ptr->mutex);
        running = this_ptr->isRunning;
        pthread_mutex_unlock(&this_ptr->mutex);
    }
    LOGD("MemoryManager: Monitor thread exited.");
    return NULL;
}

static long adaptiveMemoryManager_getFreeRamKb(void) {
    FILE* file = fopen(SYSTEM_PATHS_MEM_INFO, "r");
    if (!file) {
        LOGE("MemoryManager: Failed to open %s (errno: %d - %s)",
             SYSTEM_PATHS_MEM_INFO, errno, strerror(errno));
        return -1;
    }

    long memAvailable = -1;
    char line[256];

    while (fgets(line, sizeof(line), file)) {
        if (strncmp(line, "MemAvailable:", 13) == 0) {
            sscanf(line, "MemAvailable: %ld kB", &memAvailable);
            break;
        }
    }

    fclose(file);

    if (memAvailable >= 0) {
        return memAvailable;
    }

    LOGD("MemoryManager: Failed to read MemAvailable");
    return -1;
}

static void adaptiveMemoryManager_applyMemoryTweaks(
    AdaptiveMemoryManager* this_ptr,
    enum MemoryState newState
) {
    if (newState == this_ptr->currentState) {
        return;
    }

    switch (newState) {
        case MEM_LOW:
            LOGI("MemoryManager: RAM ample. Profile: LOW");
            systemUtils_applyTweak(SYSTEM_PATHS_SWAPPINESS,
                                   MEMORY_TWEAK_VALUES_SWAPPINESS_LOW);
            systemUtils_applyTweak(SYSTEM_PATHS_VFS_CACHE_PRESSURE,
                                   MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_LOW);
            systemUtils_applyTweak(SYSTEM_PATHS_PAGE_CLUSTER,
                                   MEMORY_TWEAK_VALUES_PAGE_CLUSTER_LOW_MID);
            break;
        case MEM_MID:
            LOGI("MemoryManager: Moderate RAM usage. Profile: MID");
            systemUtils_applyTGetKey(SYSTEM_PATHS_SWAPPINESS,
                                   MEMORY_TWEAK_VALUES_SWAPPINESS_MID);
            systemUtils_applyTweak(SYSTEM_PATHS_VFS_CACHE_PRESSURE,
                                   MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_MID);
            systemUtils_applyTweak(SYSTEM_PATHS_PAGE_CLUSTER,
                                   MEMORY_TWEAK_VALUES_PAGE_CLUSTER_LOW_MID);
            break;
        case MEM_HIGH:
            LOGI("MemoryManager: RAM nearly full. Profile: HIGH");
            systemUtils_applyTweak(SYSTEM_PATHS_SWAPPINESS,
                                   MEMORY_TWEAK_VALUES_SWAPPINESS_HIGH);
            systemUtils_applyTweak(SYSTEM_PATHS_VFS_CACHE_PRESSURE,
                                   MEMORY_TWEAK_VALUES_VFS_CACHE_PRESSURE_HIGH);
            systemUtils_applyTweak(SYSTEM_PATHS_PAGE_CLUSTER,
                                   MEMORY_TWEAK_VALUES_PAGE_CLUSTER_HIGH);
            break;
        default:
            break;
    }
    this_ptr->currentState = newState;
}

/**
 * @brief Creates and initializes a new AdaptiveMemoryManager instance.
 *
 * Allocates memory for the manager struct and initializes its components,
 * including the mutex and default state.
 *
 * @return A pointer to the newly created AdaptiveMemoryManager, or NULL
 * if memory allocation fails.
 */
AdaptiveMemoryManager* adaptiveMemoryManager_create() {
    AdaptiveMemoryManager* this_ptr =
        (AdaptiveMemoryManager*) malloc(sizeof(AdaptiveMemoryManager));
    if (!this_ptr) {
        LOGE("MemoryManager: Failed to allocate memory");
        return NULL;
    }

    pthread_mutex_init(&this_ptr->mutex, NULL);
    this_ptr->isRunning = true;
    this_ptr->currentState = MEM_UNKNOWN;
    this_ptr->monitorThread = (pthread_t)0;

    return this_ptr;
}

/**
 * @brief Starts the memory monitoring thread.
 *
 * Creates and launches the background thread that periodically checks
 * memory usage and applies tunables.
 *
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance.
 */
void adaptiveMemoryManager_start(AdaptiveMemoryManager* this_ptr) {
    LOGI("MemoryManager: Starting memory monitoring...");
    if (pthread_create(&this_ptr->monitorThread, NULL,
                       adaptiveMemoryManager_monitor, this_ptr) != 0) {
        LOGE("MemoryManager: Failed to create monitor thread (errno: %d - %s)",
             errno, strerror(errno));
        this_ptr->monitorThread = (pthread_t)0;
    }
}

/**
 * @brief Signals the memory monitoring thread to stop and waits for it.
 *
 * Sets the `isRunning` flag to false and joins the monitor thread
 * to ensure a clean shutdown.
 *
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance.
 */
void adaptiveMemoryManager_stop(AdaptiveMemoryManager* this_ptr) {
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
    LOGI("MemoryManager: Monitoring stopped.");
}

/**
 * @brief Stops the manager and frees all associated resources.
 *
 * Calls `adaptiveMemoryManager_stop` to terminate the thread, then
 * destroys the mutex and frees the manager struct itself.
 *
 * @param this_ptr A pointer to the AdaptiveMemoryManager instance to destroy.
 */
void adaptiveMemoryManager_destroy(AdaptiveMemoryManager* this_ptr) {
    if (!this_ptr) {
        return;
    }
    adaptiveMemoryManager_stop(this_ptr);
    pthread_mutex_destroy(&this_ptr->mutex);
    free(this_ptr);
}