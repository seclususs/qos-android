/**
 * @brief CPU Scheduling and Affinity management.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

namespace qos::runtime {

    /**
     * @brief Manages thread priority and CPU core binding.
     */
    class Scheduler {
    public:
        /**
         * @brief Sets the scheduling policy to Real-Time (FIFO).
         * Uses SCHED_FIFO with a high priority to preempt standard background tasks.
         */
        static void set_realtime_policy();
        
        /**
         * @brief Enforces Little/Efficiency Core affinity (0-5).
         * This function applies the affinity mask immediately.
         * Includes a fallback mechanism: if locking to 0-5 fails, it defaults to all cores (0-7)
         * to prevent the process from crashing or hanging.
         * Should be called once at startup. Child threads will inherit this.
         */
        static void enforce_efficiency_mode();

        /**
         * @brief Sets the Timer Slack for Wakeup Coalescing.
         * Target: 50ms slack.
         */
        static void maximize_timer_slack();

        /**
         * @brief Activates Utilization Clamping (UClamp).
         * Sets a hard cap on the daemon's CPU utilization to prevent it from 
         * triggering high frequencies (e.g., above 30%).
         * Crucial for 12nm SoCs (Helio G88) to prevent unnecessary thermal buildup.
         * @note This function ensures the SCHED_FIFO policy is preserved using SCHED_FLAG_KEEP_POLICY.
         */
        static void limit_cpu_utilization();
    };

}