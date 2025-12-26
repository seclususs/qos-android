/**
 * @brief Crash handling and signal trapping.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

namespace qos::runtime {
    
    /**
     * @brief The Sentinel watches for fatal signals.
     */
    class Sentinel {
    public:
        /**
         * @brief Registers signal handlers for crashes (SEGV, ABRT, etc).
         * Ensures a log message is printed to stderr before the process dies.
         */
        static void arm();
    };

}