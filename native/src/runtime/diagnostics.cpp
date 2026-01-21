// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/diagnostics.h"
#include "device_compat.h"
#include "logging.h"

#include <sys/statvfs.h>
#include <unistd.h>

namespace qos::runtime {

KernelFeatures Diagnostics::check_kernel_features() {
  KernelFeatures features = {false, false, false};

  // Check for Memory Pressure support
  if (access("/proc/pressure/memory", R_OK) == 0) {
    features.has_mem_psi = true;
    LOGI("Diagnostics: PSI Memory DETECTED.");
  } else {
    LOGI("Diagnostics: WARNING - PSI Memory MISSING.");
  }

  // Check for CPU Pressure support
  if (access("/proc/pressure/cpu", R_OK) == 0) {
    features.has_cpu_psi = true;
    LOGI("Diagnostics: PSI CPU DETECTED.");
  } else {
    LOGI("Diagnostics: WARNING - PSI CPU MISSING.");
  }

  // Check for I/O Pressure support
  if (access("/proc/pressure/io", R_OK) == 0) {
    features.has_io_psi = true;
    LOGI("Diagnostics: PSI I/O DETECTED.");
  } else {
    LOGI("Diagnostics: WARNING - PSI I/O MISSING.");
  }

  // Check for Display Compatibility
  if (qos::compat::DeviceCompat::should_force_disable_display()) {
    features.display_supported = false;
    LOGI("Diagnostics: Display disabled (incompatible device).");
  } else {
    features.display_supported = true;
    LOGI("Diagnostics: Display supported.");
  }

  struct statvfs vfs_buf;
  bool has_data = access("/data/data", R_OK | X_OK) == 0;
  bool has_proc = access("/proc", R_OK | X_OK) == 0;
  bool has_stat = statvfs("/data", &vfs_buf) == 0;

  if (has_data && has_proc && has_stat) {
    features.cleaner_supported = true;
    LOGI("Diagnostics: Cleaner prerequisites met.");
  } else {
    features.cleaner_supported = false;
    LOGI("Diagnostics: Cleaner disabled (Environment mismatch).");
  }

  return features;
}

} // namespace qos::runtime