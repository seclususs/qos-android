// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/memory.h"
#include "logging.h"

#include <cerrno>
#include <sys/mman.h>

namespace qos::runtime {

void Memory::lock_all_pages() {
  // MCL_CURRENT: Lock all pages currently mapped into the address space.
  // MCL_FUTURE: Lock all pages that will be mapped in the future.
  // This ensures that neither the executable code nor the dynamically
  // allocated memory (heap/stack) will trigger page faults due to swapping.
  if (mlockall(MCL_CURRENT | MCL_FUTURE) == -1) {
    LOGE("Memory: Failed to lock pages. Errno: %d", errno);
  } else {
    LOGI("Memory: RAM Locking Active. No swapping allowed.");
  }
}

} // namespace qos::runtime