/**
 * @brief Implementation of diagnostic checks.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/diagnostics.h"
#include "logging.h"

#include <unistd.h>

namespace qos::runtime {

    KernelFeatures Diagnostics::check_kernel_features() {
        KernelFeatures features = {false, false, false};

        if (access("/proc/pressure/memory", R_OK) == 0) {
            features.has_mem_psi = true;
            LOGI("Diagnostics: PSI Memory DETECTED.");
        } else {
            LOGI("Diagnostics: WARNING - PSI Memory MISSING.");
        }

        if (access("/proc/pressure/cpu", R_OK) == 0) {
            features.has_cpu_psi = true;
            LOGI("Diagnostics: PSI CPU DETECTED.");
        } else {
            LOGI("Diagnostics: WARNING - PSI CPU MISSING.");
        }

        if (access("/proc/pressure/io", R_OK) == 0) {
            features.has_io_psi = true;
            LOGI("Diagnostics: PSI I/O DETECTED.");
        } else {
            LOGI("Diagnostics: WARNING - PSI I/O MISSING.");
        }

        return features;
    }

}