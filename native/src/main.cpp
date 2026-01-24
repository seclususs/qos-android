// This file is part of QoS-Android.
// Licensed under the GNU GPL v3 or later.
// Author: [Seclususs](https://github.com/seclususs)

#include "config_loader.h"
#include "logging.h"
#include "native_bridge.h"
#include "runtime/diagnostics.h"
#include "runtime/io_priority.h"
#include "runtime/limits.h"
#include "runtime/memory.h"
#include "runtime/protection.h"
#include "runtime/scheduler.h"
#include "runtime/sentinel.h"

#include <csignal>
#include <cstdlib>
#include <malloc.h>
#include <string>
#include <sys/signalfd.h>

// Bionic libc definitions for malloc tuning.
#ifndef M_DECAY_TIME
#define M_DECAY_TIME -100
#endif

#ifndef M_PURGE
#define M_PURGE -101
#endif

int main(int argc, char *argv[]) {
  // Disable delayed free to keep memory footprint deterministic.
  mallopt(M_DECAY_TIME, 0);
  LOGI("=== Daemon Starting ===");

  // Phase 1: Environmental Hardening
  // Lock down the process against OOM kills, swapping, and resource exhaustion.
  LOGI("Hardening Environment...");
  qos::runtime::Sentinel::arm();
  qos::runtime::Protection::harden_process();
  qos::runtime::Limits::expand_resources();
  qos::runtime::Memory::lock_all_pages();

  // Phase 2: Scheduling Optimization
  // Bind to Efficiency Cores (Helio G88 A55 cluster), set RT priority,
  // and clamp utilization to prevent thermal throttling.
  qos::runtime::Scheduler::enforce_efficiency_mode();
  qos::runtime::Scheduler::set_realtime_policy();
  qos::runtime::Scheduler::maximize_timer_slack();
  qos::runtime::Scheduler::limit_cpu_utilization();

  // Set I/O priority to High (Best Effort) to minimize eMMC latency.
  qos::runtime::IoPriority::set_high_priority();

  // Phase 3: Capability Detection
  LOGI("Checking Hardware Support...");
  auto features = qos::runtime::Diagnostics::check_kernel_features();

  // Phase 4: Configuration
  LOGI("Loading Configuration...");
  auto cfg = qos::config::load("/data/adb/modules/sys_qos/config.ini");

  // Reconcile configuration with available kernel features.
  bool final_cpu = cfg["cpu"] && features.has_cpu_psi && features.has_mem_psi;
  bool final_io = cfg["io"] && features.has_io_psi;
  bool final_display = cfg["display"] && features.display_supported;
  bool final_cleaner = cfg["cleaner"] && features.cleaner_supported &&
                       features.has_cpu_psi && features.has_io_psi;
  bool final_tweaks = cfg["tweaks"];

  if (!final_cpu && !final_io && !final_tweaks && !final_display &&
      !final_cleaner) {
    LOGE("Daemon shutting down to save resources (No services enabled).");
    return EXIT_FAILURE;
  }

  // Phase 5: Service Activation
  LOGI("Activating Services...");
  rust_set_cpu_service_enabled(final_cpu);
  rust_set_storage_service_enabled(final_io);
  rust_set_display_service_enabled(final_display);
  rust_set_cleaner_service_enabled(final_cleaner);
  rust_set_tweaks_enabled(final_tweaks);

  // Prepare signal handling for the event loop.
  // We block standard signals here so they can be consumed via a file
  // descriptor inside the reactor loop.
  sigset_t mask;
  sigemptyset(&mask);
  sigaddset(&mask, SIGINT);
  sigaddset(&mask, SIGTERM);
  sigaddset(&mask, SIGHUP);
  sigprocmask(SIG_BLOCK, &mask, nullptr);

  int sfd = signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK);

  LOGI("Handover to Core Logic...");

  // Pass control to the Core Library (Rust). This call blocks.
  int rust_status = rust_start_services(sfd);
  if (rust_status != 0) {
    LOGE("Fatal: Core services failed to start (Error: %d).", rust_status);
    return EXIT_FAILURE;
  }

  LOGI("Core services running. Main thread waiting...");

  // Wait for the Core Library threads to shut down cleanly.
  rust_join_threads();

  LOGI("=== Shutdown Cleanly ===");
  return 0;
}