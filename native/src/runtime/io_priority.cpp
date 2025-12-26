/**
 * @brief Implementation of I/O priority adjustments.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/io_priority.h"
#include "logging.h"

#include <unistd.h>
#include <sys/syscall.h>

// Android NDK headers often miss these definitions for raw syscall usage.
#ifndef __NR_ioprio_set
    #if defined(__aarch64__)
        #define __NR_ioprio_set 30
    #else
        #define __NR_ioprio_set 251
    #endif
#endif

#define IOPRIO_WHO_PROCESS 1
#define IOPRIO_CLASS_BE    2
#define IOPRIO_CLASS_RT    1

namespace qos::runtime {

    /**
     * @brief Direct syscall wrapper for ioprio_set.
     */
    static inline int ioprio_set(int which, int who, int ioprio) {
        return syscall(__NR_ioprio_set, which, who, ioprio);
    }

    void IoPriority::set_high_priority() {
        // Construct the I/O priority value: Class BE (2) shifted 13 bits | Priority 0.
        int ioprio_val = (IOPRIO_CLASS_BE << 13) | 0;

        if (ioprio_set(IOPRIO_WHO_PROCESS, 0, ioprio_val) == -1) {
            LOGE("IoPriority: Failed to set I/O priority.");
        } else {
            LOGI("IoPriority: I/O Priority boosted.");
        }
    }

}