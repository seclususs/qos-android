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
#include <fcntl.h>

namespace qos::runtime {

    bool Diagnostics::check_kernel_compatibility() {
        // PSI is critical for the congestion controller logic.
        if (access("/proc/pressure/memory", R_OK) != 0) {
            LOGE("Diagnostics: FATAL - Kernel does not support PSI (Pressure Stall Information).");
            return false;
        }

        // CGroup verification is warning-only; some functionality might degrade but run.
        if (access("/dev/memcg", F_OK) != 0 && access("/sys/fs/cgroup/memory", F_OK) != 0) {
            LOGI("Diagnostics: Warning - Memory CGroup not standard, might reduce effectiveness.");
        }

        LOGI("Diagnostics: Kernel features validated.");
        return true;
    }

}