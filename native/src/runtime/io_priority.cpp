// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/io_priority.h"
#include "logging.h"

#include <sys/syscall.h>
#include <unistd.h>

// Definition for the ioprio_set syscall, which may be missing in
// some Android NDK headers.
#ifndef __NR_ioprio_set
#if defined(__aarch64__)
#define __NR_ioprio_set 30
#else
#define __NR_ioprio_set 251
#endif
#endif

// Constants for the Linux I/O scheduler.
#define IOPRIO_WHO_PROCESS 1
#define IOPRIO_CLASS_BE 2

namespace qos::runtime {

static inline int ioprio_set(int which, int who, int ioprio) {
  return static_cast<int>(syscall(__NR_ioprio_set, which, who, ioprio));
}

void IoPriority::set_high_priority() {
  // The I/O priority value is constructed by shifting the class
  // into the upper bits and ORing the priority level (0-7).
  // Class BE (2) << 13 | Level 0 (Highest)
  int ioprio_val = (IOPRIO_CLASS_BE << 13) | 0;

  if (ioprio_set(IOPRIO_WHO_PROCESS, 0, ioprio_val) == -1) {
    LOGE("IoPriority: Failed to set I/O priority.");
  } else {
    LOGI("IoPriority: I/O Priority boosted.");
  }
}

} // namespace qos::runtime