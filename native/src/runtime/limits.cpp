// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/limits.h"
#include "logging.h"

#include <sys/resource.h>

namespace qos::runtime {

void Limits::expand_resources() {
  struct rlimit rl;

  // 1. Maximize File Descriptors
  // We set the soft limit equal to the hard limit to ensure we can
  // open all required PSI monitors and control files without hitting
  // the default (often low) limit.
  if (getrlimit(RLIMIT_NOFILE, &rl) == 0) {
    rl.rlim_cur = rl.rlim_max;
    if (setrlimit(RLIMIT_NOFILE, &rl) != 0) {
      LOGE("Limits: Failed to maximize FD limit.");
    } else {
      LOGD("Limits: FD limit expanded to %lu", rl.rlim_cur);
    }
  }

  // 2. Expand Stack Size
  // We aim for a 2MB stack to provide ample headroom.
  if (getrlimit(RLIMIT_STACK, &rl) == 0) {
    rlim_t target = 2 * 1024 * 1024; // 2MB

    // Respect the hard limit if it's lower than our target.
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

} // namespace qos::runtime