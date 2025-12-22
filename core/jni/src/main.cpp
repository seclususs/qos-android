/**
 * @brief Entry point.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
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
#include <map>

namespace {
    constexpr const char* kAppName = "QoS";
    // Path configuration file (Magisk Module Path)
    constexpr const char* kConfigPath = "/data/adb/modules/sys_qos/config.ini";
}

/**
 * @brief Parses config.ini and returns a map of key-value pairs.
 */
std::map<std::string, bool> load_config() {
    std::map<std::string, bool> config;
    
    // Default values
    config["cpu_enabled"] = true;
    config["memory_enabled"] = true;
    config["storage_enabled"] = true;
    config["tweaks_enabled"] = true;

    std::ifstream file(kConfigPath);
    if (!file.is_open()) {
        LOGI("Config file not found at %s. Using defaults.", kConfigPath);
        return config;
    }

    std::string line;
    while (std::getline(file, line)) {
        // 1. Remove leading whitespace
        line.erase(0, line.find_first_not_of(" \t\r"));
        
        // 2. Skip comments and empty lines
        if (line.empty() || line[0] == '#' || line[0] == ';') continue;

        // 3. Find separator
        size_t delim_pos = line.find('=');
        if (delim_pos != std::string::npos) {
            std::string key = line.substr(0, delim_pos);
            std::string val = line.substr(delim_pos + 1);

            // 4. Trim key
            size_t key_end = key.find_last_not_of(" \t\r");
            if (key_end != std::string::npos) key = key.substr(0, key_end + 1);

            // 5. Trim value
            val.erase(0, val.find_first_not_of(" \t\r"));
            size_t val_end = val.find_last_not_of(" \t\r");
            if (val_end != std::string::npos) val = val.substr(0, val_end + 1);

            // Check boolean
            bool bool_val = (val == "true" || val == "1" || val == "on" || val == "enable");
            config[key] = bool_val;
        }
    }
    return config;
}

int main() {
    LOGI("=== %s Starting ===", kAppName);
    LOGI("PID: %d", getpid());

    // 1. Setup Signal Masking
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
    int sfd = signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK);
    if (sfd < 0) {
        LOGE("Failed to create signalfd. Fatal.");
        return 1;
    }

    // 3. Configure Services from config.ini
    auto config = load_config();
    
    LOGI("--- Configuration Loaded ---");
    LOGI("CPU     : %s", config["cpu_enabled"] ? "ENABLED" : "DISABLED");
    LOGI("Memory  : %s", config["memory_enabled"] ? "ENABLED" : "DISABLED");
    LOGI("Storage : %s", config["storage_enabled"] ? "ENABLED" : "DISABLED");
    LOGI("Tweaks  : %s", config["tweaks_enabled"] ? "ENABLED" : "DISABLED");
    LOGI("--------------------------");

    rust_set_cpu_service_enabled(config["cpu_enabled"]);
    rust_set_memory_service_enabled(config["memory_enabled"]);
    rust_set_storage_service_enabled(config["storage_enabled"]);
    rust_set_tweaks_enabled(config["tweaks_enabled"]);

    // 4. Handover to Rust
    LOGI("Signalfd created (fd: %d). Passing control to Rust...", sfd);
    rust_start_services(sfd); 

    LOGI("Services initialized. Waiting for Rust to finish...");
    
    // 5. Join and Shutdown
    rust_join_threads();

    LOGI("=== %s Shutdown Complete ===", kAppName);
    return 0;
}