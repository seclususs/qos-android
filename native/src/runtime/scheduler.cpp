// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/scheduler.h"
#include "logging.h"

#include <cerrno>
#include <cstring>
#include <linux/types.h>
#include <sched.h>
#include <sys/prctl.h>
#include <sys/syscall.h>
#include <unistd.h>

// System call numbers for sched_setattr, varying by architecture.
#ifndef __NR_sched_setattr
#if defined(__aarch64__)
#define __NR_sched_setattr 274
#else
#define __NR_sched_setattr 380
#endif
#endif

// Flags for sched_attr.
#ifndef SCHED_FLAG_KEEP_POLICY
#define SCHED_FLAG_KEEP_POLICY 0x08
#endif

#ifndef SCHED_FLAG_UTIL_CLAMP_MAX
#define SCHED_FLAG_UTIL_CLAMP_MAX 0x40
#endif

// Kernel structure for extended scheduling attributes.
struct sched_attr {
  __u32 size;
  __u32 sched_policy;
  __u64 sched_flags;
  __s32 sched_nice;
  __u32 sched_priority;
  __u64 sched_runtime;
  __u64 sched_deadline;
  __u64 sched_period;
  __u32 sched_util_min;
  __u32 sched_util_max;
};

static int sched_setattr(pid_t pid, struct sched_attr *attr,
                         unsigned int flags) {
  return syscall(__NR_sched_setattr, pid, attr, flags);
}

namespace qos::runtime {

// Helper to apply the specific CPU mask for the G88 Efficiency Cluster.
static int apply_little_core_affinity() {
  cpu_set_t cpuset;
  CPU_ZERO(&cpuset);

  // The Helio G88 uses a 2x A75 (cores 6-7) + 6x A55 (cores 0-5) configuration.
  // We bind strictly to the A55 cores (0-5) to minimize power usage.
  for (int i = 0; i <= 5; ++i) {
    CPU_SET(i, &cpuset);
  }

  return sched_setaffinity(0, sizeof(cpu_set_t), &cpuset);
}

void Scheduler::set_realtime_policy() {
  struct sched_param param;
  param.sched_priority = 50; // Moderate RT priority.

  if (sched_setscheduler(0, SCHED_FIFO, &param) == -1) {
    LOGE("Scheduler: Failed to set SCHED_FIFO. Errno: %d", errno);
  } else {
    LOGI("Scheduler: Real-Time Policy (SCHED_FIFO) Active.");
  }
}

void Scheduler::enforce_efficiency_mode() {
  if (apply_little_core_affinity() == -1) {
    LOGE("Scheduler: Failed to bind to Little Cores (errno: %d).", errno);

    // Fallback: If strict binding fails, ensure we can run on any core
    // rather than being left in an indeterminate state.
    cpu_set_t cpuset;
    CPU_ZERO(&cpuset);
    for (int i = 0; i <= 7; ++i) {
      CPU_SET(i, &cpuset);
    }

    if (sched_setaffinity(0, sizeof(cpu_set_t), &cpuset) == -1) {
      LOGE("Scheduler: CRITICAL - Failed to reset affinity.");
    } else {
      LOGI("Scheduler: Fallback successful. Affinity reset to default.");
    }
  } else {
    LOGI("Scheduler: Affinity mask locked to Little Cores.");
  }
}

void Scheduler::maximize_timer_slack() {
  // Set slack to 50ms (in nanoseconds).
  // This allows the kernel to defer wakeups to coalesce them with other events.
  const unsigned long slack_ns = 50 * 1000 * 1000;

  if (prctl(PR_SET_TIMERSLACK, slack_ns) == -1) {
    LOGE("Scheduler: Failed to set Timer Slack. Errno: %d", errno);
  } else {
    LOGI("Scheduler: Wakeup Coalescing Active.");
  }
}

void Scheduler::limit_cpu_utilization() {
  struct sched_attr attr;
  std::memset(&attr, 0, sizeof(attr));

  attr.size = sizeof(attr);

  // We preserve the existing policy (SCHED_FIFO) while adding the clamp.
  attr.sched_flags = SCHED_FLAG_KEEP_POLICY | SCHED_FLAG_UTIL_CLAMP_MAX;

  // UClamp value range is 0 - 1024.
  // 102 corresponds to approx 10% of max capacity.
  // This prevents the scheduler from selecting high-frequency OPPs for this
  // task.
  attr.sched_util_max = 102;

  // Apply to the current thread (PID 0).
  if (sched_setattr(0, &attr, 0) == -1) {
    LOGE("Scheduler: Failed to activate UClamp. Errno: %d", errno);
  } else {
    LOGI("Scheduler: UClamp Active.");
  }
}

} // namespace qos::runtime