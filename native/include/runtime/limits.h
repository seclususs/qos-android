/**
 * @brief Resource limit (RLIMIT) adjustments.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

namespace qos::runtime {

    /**
     * @brief Manages POSIX resource limits for the daemon.
     */
    class Limits {
    public:
        /**
         * @brief Increases File Descriptor and Stack limits.
         * Maximizes RLIMIT_NOFILE to avoid "Too many open files" during high load
         * and increases RLIMIT_STACK to prevent overflows in deep recursion.
         */
        static void expand_resources();
    };

}