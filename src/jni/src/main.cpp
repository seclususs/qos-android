/**
 * @brief Entry point.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "logging.h"
#include "native_bridge.h"

#include <atomic>
#include <csignal>
#include <unistd.h>
#include <sys/signalfd.h>
#include <cstring>
#include <cstdlib>
#include <sys/system_properties.h>

namespace {
    constexpr const char* kAppName = "QoS";
    // System property to toggle display features without binary modification.
    constexpr const char* kDisplayFeatureProp = "persist.qos.display_enabled";
}

/**
 * @brief Checks if the display optimization feature is enabled via system props.
 *
 * Defaults to TRUE if the property is missing or empty to ensure
 * out-of-the-box functionality.
 */
bool is_display_feature_enabled() {
    char value[PROP_VALUE_MAX] = {0};
    int len = __system_property_get(kDisplayFeatureProp, value);
    
    // Default safe fallback: Enabled.
    if (len == 0) {
        return true;
    }
    
    // Explicit disable check.
    if (strcmp(value, "false") == 0 || strcmp(value, "0") == 0) {
        return false;
    }
    return true;
}

int main() {
    LOGI("=== %s Starting ===", kAppName);
    LOGI("PID: %d", getpid());

    // 1. Setup Signal Masking
    // We block signals in the main thread so they can be queued and consumed
    // via a file descriptor (signalfd). This converts asynchronous signals
    // into synchronous IO events for the Rust event loop.
    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    sigaddset(&mask, SIGHUP);

    if (sigprocmask(SIG_BLOCK, &mask, nullptr) < 0) {
        LOGE("Failed to set signal mask. Fatal.");
        return 1;
    }

    // 2. Create Signal File Descriptor
    // SFD_CLOEXEC: Prevent fd leak to child processes.
    // SFD_NONBLOCK: Essential for integration with epoll in Rust.
    int sfd = signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK);
    if (sfd < 0) {
        LOGE("Failed to create signalfd. Fatal.");
        return 1;
    }

    // 3. Configure Services
    bool enable_display = is_display_feature_enabled();
    LOGI("Configuring Display Service: %s", enable_display ? "ENABLED" : "DISABLED");
    rust_set_display_service_enabled(enable_display);

    // 4. Handover to Rust
    LOGI("Signalfd created (fd: %d). Passing control to Rust...", sfd);
    
    // This spawns the Rust threads. It does NOT block indefinitely here.
    rust_start_services(sfd); 

    LOGI("Services initialized. Waiting for Rust to finish...");
    
    // 5. Join and Shutdown
    // Block main thread until the Rust logic decides to shut down (e.g., on SIGTERM).
    rust_join_threads();

    LOGI("=== %s Shutdown Complete ===", kAppName);
    return 0;
}