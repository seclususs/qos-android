/**
 * @brief Implementation of memory locking.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/memory.h"
#include "logging.h"

#include <sys/mman.h>
#include <cerrno>

namespace qos::runtime {

    void Memory::lock_all_pages() {
        // MCL_CURRENT: Lock all pages currently mapped.
        // MCL_FUTURE: Lock all pages mapped in the future.
        if (mlockall(MCL_CURRENT | MCL_FUTURE) == -1) {
            LOGE("Memory: Failed to lock pages. Errno: %d", errno);
        } else {
            LOGI("Memory: RAM Locking Active. No swapping allowed.");
        }
    }

}