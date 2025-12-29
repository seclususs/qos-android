/**
 * @brief Implementation of resource expansion logic.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/limits.h"
#include "logging.h"

#include <sys/resource.h>

namespace qos::runtime {

    void Limits::expand_resources() {
        struct rlimit rl;

        // 1. Expand File Descriptors (FDs)
        if (getrlimit(RLIMIT_NOFILE, &rl) == 0) {
            rl.rlim_cur = rl.rlim_max; // Set soft limit to hard limit
            if (setrlimit(RLIMIT_NOFILE, &rl) != 0) {
                LOGE("Limits: Failed to maximize FD limit.");
            } else {
                LOGD("Limits: FD limit expanded to %lu", rl.rlim_cur);
            }
        }

        // 2. Expand Stack Size
        if (getrlimit(RLIMIT_STACK, &rl) == 0) {
            // Target 16MB stack, ensuring we don't exceed the hard limit.
            rlim_t target = 16 * 1024 * 1024;
            if (rl.rlim_max != RLIM_INFINITY && target > rl.rlim_max) {
                target = rl.rlim_max;
            }
            rl.rlim_cur = target;
            if (setrlimit(RLIMIT_STACK, &rl) != 0) {
                LOGE("Limits: Failed to expand Stack.");
            } else {
                LOGD("Limits: Stack expanded to %lu bytes", rl.rlim_cur);
            }
        }
    }

}