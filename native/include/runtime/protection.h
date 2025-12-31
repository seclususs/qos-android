/**
 * @file protection.h
 * @brief Process self-defense and OOM adjustment.
 *
 * This header defines methods to protect the daemon from being killed
 * by the Android Low Memory Killer (LMK).
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Manages process priority and killability.
 */
class Protection {
public:
  /**
   * @brief Applies the OOM Shield to the process.
   *
   * Writes the minimum possible value (-1000) to `/proc/self/oom_score_adj`.
   * This instructs the kernel to treat this process as critical infrastructure,
   * making it effectively immune to OOM kills under normal operating
   * conditions.
   */
  static void harden_process();
};

} // namespace qos::runtime