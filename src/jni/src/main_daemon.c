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
 * @file main_daemon.c
 * @brief Main entry point for the adaptive daemon.
 *
 * Initializes the daemon, sets up signal handling, starts all
 * registered manager modules (memory, refresh rate), and waits
 * for a shutdown signal.
 */

#include "include/main_daemon.h"
#include "include/logging.h"
#include "include/system_tweaker.h"
#include "include/adaptive_memory_manager.h"
#include "include/adaptive_refresh_rate_manager.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>

/**
 * @brief Global flag to request daemon shutdown.
 * @see main_daemon.h
 */
volatile sig_atomic_t g_shutdown_requested = 0;

/**
 * @brief The public name of the application, used in logs.
 * @see main_daemon.h
 */
const char* const TWEAK_VALUES_APP_NAME = "AdaptiveDaemon";

/** @brief Global pointer to the AdaptiveMemoryManager instance. */
static AdaptiveMemoryManager* g_memoryManager = NULL;
/** @brief Global pointer to the AdaptiveRefreshRateManager instance. */
static AdaptiveRefreshRateManager* g_refreshRateManager = NULL;

/**
 * @brief Signal handler for clean shutdown.
 *
 * Sets the `g_shutdown_requested` flag to 1 when a termination
 * signal is caught.
 *
 * @param signum The signal number received.
 */
static void signalHandler(int signum) {
    LOGI("Shutdown signal (%d) received. Cleaning up...", signum);
    g_shutdown_requested = 1;
}

/**
 * @brief Main function of the daemon.
 *
 * Sets up signal handlers, applies static system tweaks,
 * initializes and starts service modules, and then enters
 * a sleep loop, waiting for the shutdown flag.
 *
 * @return 0 on successful shutdown, non-zero on error (though
 * it currently always returns 0).
 */
int main() {
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = signalHandler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;

    /* Catch signals that request termination */
    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);
    sigaction(SIGHUP, &sa, NULL); /* Also treat HUP as shutdown */

    LOGI("=== %s Starting ===", TWEAK_VALUES_APP_NAME);
    LOGI("PID: %d", getpid());

    /* Apply one-time static tweaks at startup */
    systemTweaker_applyAll();

    /* Start the adaptive memory manager */
    g_memoryManager = adaptiveMemoryManager_create();
    if (g_memoryManager) {
        adaptiveMemoryManager_start(g_memoryManager);
    } else {
        LOGE("Failed to create MemoryManager. Service will run without it.");
    }

    /* Start the adaptive refresh rate manager */
    g_refreshRateManager = adaptiveRefreshRateManager_create();
    if (g_refreshRateManager) {
        if (!adaptiveRefreshRateManager_start(g_refreshRateManager)) {
            LOGE("Failed to start RefreshRateManager. Feature disabled.");
            adaptiveRefreshRateManager_destroy(g_refreshRateManager);
            g_refreshRateManager = NULL;
        }
    } else {
        LOGE("Failed to create RefreshRateManager. Service will run without it.");
    }

    LOGI("All services started. Use 'logcat -s %s' to view logs.", LOG_TAG);
    LOGI("To stop the service, use: kill -TERM %d", getpid());

    /* Main loop: wait for shutdown signal */
    while (!g_shutdown_requested) {
        sleep(1);
    }

    LOGI("=== Shutdown request received, cleaning up... ===");

    /* Stop and destroy modules in reverse order */
    if (g_refreshRateManager) {
        adaptiveRefreshRateManager_destroy(g_refreshRateManager);
        g_refreshRateManager = NULL;
    }

    if (g_memoryManager) {
        adaptiveMemoryManager_destroy(g_memoryManager);
        g_memoryManager = NULL;
    }

    LOGI("=== %s Shutdown Complete ===", TWEAK_VALUES_APP_NAME);
    return 0;
}