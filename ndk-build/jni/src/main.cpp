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
#include "logger.h"
#include "config.h"
#include "adaptive-daemon.h"
#include <csignal>
#include <unistd.h>
#include <atomic>
#include <memory>
#include <chrono>

// Global variables to control the main loop and daemon instance
static std::atomic<bool> g_shutdown_requested{false};
static std::unique_ptr<AdaptiveDaemon> g_daemon;

/**
 * @brief Handles termination signals to allow for graceful shutdown.
 * @param signum The signal number received.
 */
void signalHandler(int signum) {
    LOGI("Shutdown signal (%d) received. Cleaning up...", signum);
    g_shutdown_requested.store(true, std::memory_order_release);
}

/**
 * @brief Sets up signal handlers for SIGINT, SIGTERM, and SIGHUP.
 */
void setupSignalHandlers() {
    struct sigaction sa;
    sa.sa_handler = signalHandler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;
    
    sigaction(SIGINT, &sa, nullptr);
    sigaction(SIGTERM, &sa, nullptr);
    sigaction(SIGHUP, &sa, nullptr);
}

/**
 * @brief The main entry point for the adaptive daemon.
 * @return 0 on successful shutdown, non-zero otherwise.
 */
int main() {
    setupSignalHandlers();
    
    LOGI("=== %s Starting ===", TweakValues::kAppName);
    LOGI("PID: %d", getpid());

    g_daemon = std::make_unique<AdaptiveDaemon>();
    g_daemon->run();
    
    LOGI("To stop the service, use: kill -TERM %d", getpid());

    // Main loop to keep the program running
    while (!g_shutdown_requested.load(std::memory_order_acquire)) {
        std::this_thread::sleep_for(std::chrono::seconds(1));
    }
    
    LOGI("=== Shutdown requested, cleaning up... ===");
    
    if (g_daemon) {
        g_daemon->stop();
    }
    
    LOGI("=== %s Shutdown Complete ===", TweakValues::kAppName);
    
    return 0;
}