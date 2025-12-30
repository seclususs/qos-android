// This file is part of QoS-Android.
// Licensed under the GNU GPL v3 or later.

#include "logging.h"
#include "native_bridge.h"
#include "config_loader.h"
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
#include <cstdlib>
#include <malloc.h>

// Android Bionic Constants
#ifndef M_DECAY_TIME
#define M_DECAY_TIME -100
#endif

#ifndef M_PURGE
#define M_PURGE -101
#endif

/**
 * @brief Main execution entry point.
 * 1. Hardens the process (OOM shield, scheduling, memory trim).
 * 2. Checks kernel compatibility.
 * 3. Loads config and passes it to Rust.
 * 4. Hands over control to Rust reactor loop.
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return int Exit code (0 for success, EXIT_FAILURE for errors).
 */
int main(int argc, char* argv[]) {
    mallopt(M_DECAY_TIME, 0);
    LOGI("=== Daemon Starting ===");

    // Make the process resilient against system pressure and termination.
    LOGI("Hardening Environment...");
    qos::runtime::Sentinel::arm();                          // Register crash signal handlers
    qos::runtime::Protection::harden_process();             // Apply OOM Shield (-1000)
    qos::runtime::Limits::expand_resources();               // Increase FD and Stack limits
    qos::runtime::Memory::lock_all_pages();                 // Prevent swapping (mlockall)
    
    // Combined Scheduler Initialization
    // 1. Lock to Little Cores (Efficiency)
    // 2. Set RT Priority (Responsiveness)
    // 3. Set Timer Slack (Power Saving)
    // 4. UClamp (Limit Max Freq to ~30%)
    qos::runtime::Scheduler::enforce_efficiency_mode(); 
    qos::runtime::Scheduler::set_realtime_policy();
    qos::runtime::Scheduler::maximize_timer_slack();
    qos::runtime::Scheduler::limit_cpu_utilization();
    
    // 5. Set I/O Priority (Disk Access)
    qos::runtime::IoPriority::set_high_priority();
    
    LOGI("Checking Hardware Support...");
    auto features = qos::runtime::Diagnostics::check_kernel_features();

    LOGI("Loading Configuration...");
    auto cfg = qos::config::load("/data/adb/modules/sys_qos/config.ini");

    bool final_cpu = cfg["cpu"] && features.has_cpu_psi;
    bool final_mem = cfg["mem"] && features.has_mem_psi;
    bool final_io  = cfg["io"] && features.has_io_psi;
    bool final_tweaks = cfg["tweaks"];

    if (!final_cpu && !final_mem && !final_io && !final_tweaks) {
        LOGE("Daemon shutting down to save resources.");
        return EXIT_FAILURE;
    }

    LOGI("Activating Services...");
    rust_set_cpu_service_enabled(final_cpu);
    rust_set_memory_service_enabled(final_mem);
    rust_set_storage_service_enabled(final_io);
    rust_set_tweaks_enabled(final_tweaks);
    
    // Block signals so they can be handled synchronously via a file descriptor (signalfd)
    // inside the Rust event loop.
    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    sigaddset(&mask, SIGHUP);
    sigprocmask(SIG_BLOCK, &mask, nullptr);
    
    int sfd = signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK);
    
    LOGI("Handover to Rust Core...");
    
    // Pass the signal FD to Rust. This function blocks until the service stops.
    int rust_status = rust_start_services(sfd);
    if (rust_status != 0) {
        LOGE("Fatal: Rust services failed to start (Error: %d).", rust_status);
        return EXIT_FAILURE;
    }
    
    LOGI("Rust services running. Main thread waiting...");
    
    // Block here until Rust threads finish (usually on SIGTERM).
    rust_join_threads();
    
    LOGI("Shutdown Sequence...");
    
    LOGI("=== Shutdown Cleanly ===");
    return 0;
}