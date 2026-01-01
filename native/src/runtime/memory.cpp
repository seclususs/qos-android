// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/memory.h"
#include "logging.h"

#include <cerrno>
#include <sys/mman.h>

#ifndef MCL_ONFAULT
#define MCL_ONFAULT 4
#endif

namespace qos::runtime {

void Memory::lock_all_pages() {
  // MCL_CURRENT: Lock all pages currently mapped into the address space.
  // MCL_FUTURE: Lock all pages that will be mapped in the future.
  // MCL_ONFAULT: Lock pages only when they are populated.
  // This prevents swapping while avoiding locking empty virtual memory.
  if (mlockall(MCL_CURRENT | MCL_FUTURE | MCL_ONFAULT) == -1) {
    LOGE("Memory: MCL_ONFAULT failed. Retrying with MCL_CURRENT...");

    if (mlockall(MCL_CURRENT) == -1) {
      LOGE("Memory: Failed to lock pages. Errno: %d", errno);
    } else {
      LOGI("Memory: RAM Locking Active.");
    }
  } else {
    LOGI("Memory: Smart RAM Locking Active.");
  }
}

} // namespace qos::runtime