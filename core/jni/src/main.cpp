/**
 * @brief Entry point.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * */

#include "logging.h"
#include "native_bridge.h"

#include <atomic>
#include <csignal>
#include <unistd.h>
#include <sys/signalfd.h>
#include <cstring>
#include <cstdlib>
#include <sys/system_properties.h>
#include <fstream>
#include <string>

namespace {
    constexpr const char* kAppName = "QoS";
    // Path configuration file (Magisk Module Path)
    constexpr const char* kConfigPath = "/data/adb/modules/sys_qos/config.ini";
}

/**
 * @brief Reads the display_enabled status from config.ini.
 *
 * Defaults to FALSE (Disabled) if the file is missing or the key is not set.
 */
bool get_display_config_state() {
    std::ifstream file(kConfigPath);
    if (!file.is_open()) {
        LOGI("Config file not found at %s.", kConfigPath);
        return false; // Default: Nonaktif
    }

    std::string line;
    while (std::getline(file, line)) {
        // Parser: Trim spaces and check for key
        // 1. Remove leading whitespace
        line.erase(0, line.find_first_not_of(" \t"));
        
        // 2. Skip comments and empty lines
        if (line.empty() || line[0] == '#' || line[0] == ';') continue;

        // 3. Find separator
        size_t delim_pos = line.find('=');
        if (delim_pos != std::string::npos) {
            std::string key = line.substr(0, delim_pos);
            std::string val = line.substr(delim_pos + 1);

            // 4. Trim trailing space from key
            size_t key_end = key.find_last_not_of(" \t");
            if (key_end != std::string::npos) key = key.substr(0, key_end + 1);

            if (key == "display_enabled") {
                // 5. Trim leading/trailing space from val
                val.erase(0, val.find_first_not_of(" \t"));
                size_t val_end = val.find_last_not_of(" \t");
                if (val_end != std::string::npos) val = val.substr(0, val_end + 1);

                // Check for truthy values
                if (val == "true" || val == "1" || val == "on" || val == "enable") {
                    return true;
                }
                return false;
            }
        }
    }
    
    // Key not found in file
    return false;
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

    // 3. Configure Services from config.ini
    bool enable_display = get_display_config_state();
    LOGI("Configuring Display Service: %s (Source: %s)", 
         enable_display ? "ENABLED" : "DISABLED", kConfigPath);
    
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