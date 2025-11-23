/**
 * @author Seclususs
 * https://github.com/seclususs
 */

#include "logging.h"
#include "system_tweaker.h"
#include "daemon_interface.h"

#include <atomic>
#include <csignal>
#include <thread>
#include <chrono>
#include <unistd.h>

using namespace std::chrono_literals;

namespace {
    std::atomic<bool> g_shutdown_requested{false};
    constexpr const char* kAppName = "QoS";
}

void signalHandler(int signum) {
    LOGI("Shutdown signal (%d) received. Cleaning up...", signum);
    g_shutdown_requested.store(true, std::memory_order_release);
}

int main() {
    struct sigaction sa;
    sa.sa_handler = signalHandler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;

    sigaction(SIGINT, &sa, nullptr);
    sigaction(SIGTERM, &sa, nullptr);
    sigaction(SIGHUP, &sa, nullptr);

    LOGI("=== %s Starting ===", kAppName);
    LOGI("PID: %d", getpid());

    SystemTweaker::applyAll();

    LOGI("Starting Rust services...");
    rust_start_services();

    LOGI("All services started successfully. Use 'logcat -s %s' to view the logs.", LOG_TAG);
    LOGI("To stop the service, use: kill -TERM %d", getpid());

    while (!g_shutdown_requested.load(std::memory_order_acquire)) {
        std::this_thread::sleep_for(1s);
    }

    LOGI("=== Shutdown request received, cleaning up... ===");
    rust_stop_services();
    LOGI("=== %s Shutdown Complete ===", kAppName);

    return 0;
}