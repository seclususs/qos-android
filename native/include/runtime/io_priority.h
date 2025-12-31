/**
 * @file io_priority.h
 * @brief I/O scheduling priority management.
 *
 * This header defines the interface for manipulating the I/O priority
 * of the daemon process.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Manages block I/O scheduling classes and priorities.
 */
class IoPriority {
public:
  /**
   * @brief Sets the process I/O priority to Best Effort (High).
   *
   * Configures the process to use the 'Best Effort' I/O scheduling class
   * with the highest possible priority (level 0). This minimizes latency
   * when the daemon performs critical writes to eMMC, such as updating
   * swap swappiness or block queue parameters.
   */
  static void set_high_priority();
};

} // namespace qos::runtime