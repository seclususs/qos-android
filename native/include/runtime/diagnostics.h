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
     * @brief Structure to report available kernel features.
     */
    struct KernelFeatures {
        bool has_cpu_psi;   ///< Is /proc/pressure/cpu available?
        bool has_mem_psi;   ///< Is /proc/pressure/memory available?
        bool has_io_psi;    ///< Is /proc/pressure/io available?
    };

    /**
     * @brief Handling system compatibility checks.
     */
    class Diagnostics {
    public:
        /**
         * @brief Scans the kernel environment for supported features.
         * Instead of returning a simple boolean, this provides a detailed
         * report of which Pressure Stall Information (PSI) interfaces 
         * are actually usable.
         * @return KernelFeatures A struct containing flags for each feature.
         */
        static KernelFeatures check_kernel_features();
    };

}