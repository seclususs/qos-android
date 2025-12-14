/**
 * @author Seclususs
 * https://github.com/seclususs
 */

#include "logging.h"
#include "daemon_interface.h"

#include <atomic>
#include <csignal>
#include <thread>
#include <chrono>
#include <unistd.h>

namespace {
    std::atomic<bool> g_shutdown_requested{false};
    constexpr const char* kAppName = "QoS";
}

void signalHandler(int signum) {
    LOGI("Shutdown signal (%d) received. Cleaning up...", signum);
    g_shutdown_requested.store(true, std::memory_order_release);
}

int main() {
    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    sigaddset(&mask, SIGHUP);
    
    if (sigprocmask(SIG_BLOCK, &mask, nullptr) < 0) {
        LOGE("Failed to set signal mask");
        return 1;
    }

    struct sigaction sa;
    sa.sa_handler = signalHandler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;

    sigaction(SIGINT, &sa, nullptr);
    sigaction(SIGTERM, &sa, nullptr);
    sigaction(SIGHUP, &sa, nullptr);

    LOGI("=== %s Starting ===", kAppName);
    LOGI("PID: %d", getpid());

    LOGI("Starting Rust services...");
    rust_start_services();

    LOGI("All services started successfully. Waiting for signals...", LOG_TAG);
    
    sigset_t suspend_mask;
    sigemptyset(&suspend_mask);

    while (!g_shutdown_requested.load(std::memory_order_acquire)) {
        sigsuspend(&suspend_mask);
    }

    LOGI("=== Shutdown request received, cleaning up... ===");
    rust_stop_services();
    LOGI("=== %s Shutdown Complete ===", kAppName);

    return 0;
}