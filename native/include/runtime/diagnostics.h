/**
 * @brief Diagnostic tools for kernel feature verification.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

namespace qos::runtime {

    /**
     * @brief Handling system compatibility checks.
     */
    class Diagnostics {
    public:
        /**
         * @brief Verifies if the kernel supports required features.
         * Checks for the existence of Pressure Stall Information (PSI)
         * and Memory Cgroups (memcg).
         * @return true If critical features (PSI) are available.
         * @return false If the environment is unsupported.
         */
        static bool check_kernel_compatibility();
    };

}