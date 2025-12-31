/**
 * @file sentinel.h
 * @brief Crash handling and signal monitoring.
 *
 * This header defines the interface for the crash reporting mechanism,
 * ensuring that fatal signals result in a logged reason before termination.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Installs emergency signal handlers.
 */
class Sentinel {
public:
  /**
   * @brief Registers signal handlers for fatal events.
   *
   * Traps signals such as SIGSEGV, SIGFPE, and SIGABRT. The handler
   * attempts to write a failure message to stderr/logcat before
   * reraising the signal to allow core dumping or default termination.
   */
  static void arm();
};

} // namespace qos::runtime