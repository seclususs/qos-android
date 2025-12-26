/**
 * @brief I/O Scheduling priority management.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

namespace qos::runtime {

    /**
     * @brief Wrapper for the ioprio_set syscall.
     */
    class IoPriority {
    public:
        /**
         * @brief Boosts the process I/O priority to Best Effort (High).
         * Attempts to set the class to IOPRIO_CLASS_BE with level 0 (highest).
         */
        static void set_high_priority();
    };

}