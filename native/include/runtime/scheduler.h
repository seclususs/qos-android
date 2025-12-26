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
         * @brief Binds the current thread to Little/Efficiency Cores.
         * Typically cores 0-5 on standard octa-core mobile SoCs.
         * Used for low-power monitoring threads.
         */
        static void bind_to_little_cores();
        
        /**
         * @brief Prepares CPU affinity for the Rust heavy-lifting threads.
         * Sets affinity to Big/Performance cores (e.g., 6-7) before handing over control.
         */
        static void prepare_for_rust_handover();
    };

}