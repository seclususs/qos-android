/**
 * @file tuner.hpp
 *
 * @brief System environment and process tuning utilities.
 */

#pragma once

namespace qos::system
{

    /**
     * @brief Utility class for applying system-level process configurations.
     */
    class Tuner
    {
        public:
        /**
         * @brief Optimizes critical process resource limits for background execution.
         *
         * @note Execution: This function maximizes File Descriptor limits to prevent I/O
         *                  bottlenecks, while strictly constraining the process Stack Size
         *                  (reducing it below OS defaults) to minimize the daemon's
         *                  memory footprint.
         */
        static void expand_resources() noexcept;

        /**
         * @brief Locks process memory into RAM to prevent swapping.
         */
        static void lock_memory() noexcept;

        /**
         * @brief Exempts the process from system-level low memory termination.
         */
        static void harden_process() noexcept;

        /**
         * @brief Elevates process I/O scheduling to the highest Best-Effort priority class.
         */
        static void set_high_io_priority() noexcept;

        /**
         * @brief Enforces real-time CPU scheduling policies.
         */
        static void set_realtime_policy() noexcept;

        /**
         * @brief Attempts to restrict process execution to efficiency-focused CPU cores.
         *
         * @note Fallback: If the underlying hardware topology cannot be safely determined,
         *                 this function falls back to allowing execution on all available
         *                 cores to prevent thread starvation.
         */
        static void enforce_efficiency_mode() noexcept;

        /**
         * @brief Adjusts timer slack to improve power efficiency via wakeup coalescing.
         */
        static void maximize_timer_slack() noexcept;

        /**
         * @brief Constrains CPU performance states to prevent aggressive frequency ramping.
         */
        static void limit_cpu_utilization() noexcept;
    };

} // namespace qos::system