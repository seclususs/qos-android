/**
 * @file diagnostics.h
 * @brief System capability analysis tools.
 *
 * This header defines structures and classes used to introspect the
 * kernel and runtime environment to determine feature availability.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Represents the availability of system capabilities and hardware
 * support.
 */
struct KernelFeatures {
  bool has_cpu_psi;       ///< True if /proc/pressure/cpu is readable.
  bool has_io_psi;        ///< True if /proc/pressure/io is readable.
  bool display_supported; ///< True if the device is compatible.
  bool cleaner_supported; ///< True if environment supports cleaning ops.
};

/**
 * @brief Provides static methods for environment verification.
 */
class Diagnostics {
public:
  /**
   * @brief Scans the filesystem to detect supported kernel features.
   *
   * Checks for the existence and accessibility of PSI interfaces. This
   * allows the daemon to degrade gracefully on kernels that do not support
   * specific pressure metrics.
   *
   * @return A filled KernelFeatures structure indicating available subsystems.
   */
  static KernelFeatures check_kernel_features();
};

} // namespace qos::runtime