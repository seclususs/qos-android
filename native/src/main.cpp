/**
 * @brief Entry point.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "logging.h"
#include "native_bridge.h"
#include "runtime/protection.h"
#include "runtime/scheduler.h"
#include "runtime/memory.h"
#include "runtime/io_priority.h"
#include "runtime/limits.h"
#include "runtime/diagnostics.h"
#include "runtime/sentinel.h"

#include <string>
#include <unistd.h>
#include <sys/signalfd.h>
#include <csignal>
#include <map>
#include <fstream>
#include <cstdlib>

/**
 * @brief Loads feature flags from the configuration file.
 * Parses a simple key-value config file to determine which
 * QoS services should be enabled upon startup.
 * @param path Path to the configuration file (e.g., config.ini).
 * @return std::map<std::string, bool> Map of feature keys to their enabled state.
 */
static std::map<std::string, bool> load_config(const char* path) {
    std::map<std::string, bool> config;
    // Default values: All services enabled
    config["cpu"] = true;
    config["mem"] = true;
    config["io"] = true;
    config["tweaks"] = true;

    std::ifstream file(path);
    if (file.is_open()) {
        std::string line;
        while (std::getline(file, line)) {
            // Simple substring matching for configuration parsing
            if (line.find("cpu_enabled=false") != std::string::npos) config["cpu"] = false;
            if (line.find("memory_enabled=false") != std::string::npos) config["mem"] = false;
            if (line.find("storage_enabled=false") != std::string::npos) config["io"] = false;
            if (line.find("tweaks_enabled=false") != std::string::npos) config["tweaks"] = false;
        }
    }
    return config;
}

/**
 * @brief Main execution entry point.
 * 1. Hardens the process (OOM shield, scheduling).
 * 2. Checks kernel compatibility.
 * 3. Loads config and passes it to Rust.
 * 4. Hands over control to Rust reactor loop.
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return int Exit code (0 for success, EXIT_FAILURE for errors).
 */
int main(int argc, char* argv[]) {
    LOGI("=== Daemon Starting ===");

    // Make the process resilient against system pressure and termination.
    LOGI("Hardening Environment...");
    qos::runtime::Sentinel::arm();                          // Register crash signal handlers
    qos::runtime::Protection::harden_process();             // Apply OOM Shield (-1000)
    qos::runtime::Limits::expand_resources();               // Increase FD and Stack limits
    qos::runtime::Memory::lock_all_pages();                 // Prevent swapping (mlockall)
    
    // Set initial priority (Little Cores + FIFO) for initialization phase
    qos::runtime::Scheduler::bind_to_little_cores();
    qos::runtime::Scheduler::set_realtime_policy();
    qos::runtime::IoPriority::set_high_priority();
    
    // verify if the kernel supports PSI/cgroups required for logic.
    LOGI("Diagnostics...");
    if (!qos::runtime::Diagnostics::check_kernel_compatibility()) {
        LOGE("System Incompatible. Exiting immediately to save resources.");
        return EXIT_FAILURE; 
    }

    LOGI("Activating Services...");
    
    // Block signals so they can be handled synchronously via a file descriptor (signalfd)
    // inside the Rust event loop.
    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    sigaddset(&mask, SIGHUP);
    sigprocmask(SIG_BLOCK, &mask, nullptr);
    
    int sfd = signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK);
    
    // Configuration Loading
    auto cfg = load_config("/data/adb/modules/sys_qos/config.ini");
    
    // Pass configuration state to the Rust static storage
    rust_set_cpu_service_enabled(cfg["cpu"]);
    rust_set_memory_service_enabled(cfg["mem"]);
    rust_set_storage_service_enabled(cfg["io"]);
    rust_set_tweaks_enabled(cfg["tweaks"]);
    
    LOGI("Handover to Rust Core...");
    
    // Change thread affinity to Big Cores to ensure the Rust reactor runs with max performance.
    qos::runtime::Scheduler::prepare_for_rust_handover();
    
    // Pass the signal FD to Rust. This function blocks until the service stops.
    rust_start_services(sfd);
    
    LOGI("Rust services running. Main thread waiting...");
    
    // Block here until Rust threads finish (usually on SIGTERM).
    rust_join_threads();
    
    LOGI("Shutdown Sequence...");
    
    LOGI("=== Shutdown Cleanly ===");
    return 0;
}