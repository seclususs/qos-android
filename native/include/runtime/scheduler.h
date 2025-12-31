/**
 * @file scheduler.h
 * @brief CPU scheduling and affinity management.
 *
 * This header defines the interface for controlling the execution context
 * of the daemon, including CPU core affinity, scheduling policies, and
 * utilization limits.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Manages thread scheduling attributes.
 */
class Scheduler {
public:
  /**
   * @brief Sets the scheduling policy to Real-Time FIFO.
   *
   * Configures the process to use `SCHED_FIFO` with a moderate priority.
   * This ensures the daemon preempts standard background tasks, minimizing
   * reaction time to pressure events.
   */
  static void set_realtime_policy();

  /**
   * @brief Restricts execution to Efficiency Cores.
   *
   * Sets the CPU affinity mask to cores 0-5. On the Helio G88 SoC, these
   * correspond to the Cortex-A55 efficiency cores. This reduces power
   * consumption and thermal impact, reserving the Performance cores
   * (Cortex-A75) for user-facing applications.
   */
  static void enforce_efficiency_mode();

  /**
   * @brief Configures timer slack for wakeup coalescing.
   *
   * Increases the allowed jitter for timer expiration (to 50ms). This allows
   * the kernel to group wakeups, reducing CPU active time and improving
   * power efficiency.
   */
  static void maximize_timer_slack();

  /**
   * @brief Limits CPU utilization via UClamp.
   *
   * Applies a utilization clamp (approx 15%) to the process. This prevents
   * the scheduler from ramping up the frequency for this daemon, ensuring
   * it remains on low-power operating points even during activity bursts.
   */
  static void limit_cpu_utilization();
};

} // namespace qos::runtime