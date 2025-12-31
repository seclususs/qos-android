/**
 * @file limits.h
 * @brief Resource limit (RLIMIT) management.
 *
 * This header defines methods to adjust the POSIX resource limits
 * for the running process, ensuring sufficient resources for stable operation.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Adjusts process resource limits.
 */
class Limits {
public:
  /**
   * @brief Expands File Descriptor and Stack limits.
   *
   * Increases `RLIMIT_NOFILE` to the hard limit to accommodate the
   * various file handles used for PSI triggers and sysfs nodes.
   * Increases `RLIMIT_STACK` to prevent stack overflows during
   * complex initialization sequences.
   */
  static void expand_resources();
};

} // namespace qos::runtime