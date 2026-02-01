// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/scheduler.h"
#include "logging.h"

#include <cerrno>
#include <cstring>
#include <fstream>
#include <limits>
#include <linux/types.h>
#include <sched.h>
#include <string>
#include <sys/prctl.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <vector>

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

// Helper to read a numeric value from a sysfs node.
// Returns -1 if the node is missing, unreadable, or reports an invalid value.
static long read_sysfs_long(const std::string &path) {
  std::ifstream file(path);
  long value = -1;
  if (file.is_open()) {
    file >> value;
  }
  return value;
}

// Detects and binds the current thread to the Little CPU cores.
// The Little cores are inferred from kernel-exposed topology data so this
// logic remains compatible across different SoCs and kernel configurations.
static int apply_little_core_affinity() {
  cpu_set_t cpuset;
  CPU_ZERO(&cpuset);

  int num_cores = sysconf(_SC_NPROCESSORS_CONF);
  if (num_cores <= 0) {
    LOGE("Scheduler: Invalid core count detected.");
    return -1;
  }

  std::vector<int> little_cores;
  bool topology_found = false;

  // The kernel-provided CPU capacity reflects relative compute capability.
  // Lower capacity values typically correspond to Little cores.
  long min_capacity = std::numeric_limits<long>::max();
  bool has_capacity_interface = false;

  for (int i = 0; i < num_cores; ++i) {
    std::string path =
        "/sys/devices/system/cpu/cpu" + std::to_string(i) + "/cpu_capacity";
    long cap = read_sysfs_long(path);

    if (cap > 0) {
      has_capacity_interface = true;
      if (cap < min_capacity) {
        min_capacity = cap;
        little_cores.clear();
        little_cores.push_back(i);
      } else if (cap == min_capacity) {
        little_cores.push_back(i);
      }
    }
  }

  if (has_capacity_interface && !little_cores.empty()) {
    topology_found = true;
    LOGI("Scheduler: Topology detected via EAS Capacity. Found %zu Little "
         "cores.",
         little_cores.size());
  }

  // If CPU capacity is unavailable, maximum frequency is used as a heuristic.
  // Cores with lower peak frequency are treated as Little cores.
  if (!topology_found) {
    LOGD("Scheduler: EAS Capacity missing. Fallback to Frequency detection.");

    long min_freq = std::numeric_limits<long>::max();
    little_cores.clear();

    for (int i = 0; i < num_cores; ++i) {
      std::string path = "/sys/devices/system/cpu/cpu" + std::to_string(i) +
                         "/cpufreq/cpuinfo_max_freq";
      long freq = read_sysfs_long(path);

      if (freq <= 0)
        continue; // Skip offline or invalid cores

      if (freq < min_freq) {
        min_freq = freq;
        little_cores.clear();
        little_cores.push_back(i);
      } else if (freq == min_freq) {
        little_cores.push_back(i);
      }
    }

    if (!little_cores.empty()) {
      topology_found = true;
      LOGI(
          "Scheduler: Topology detected via Frequency. Found %zu Little cores.",
          little_cores.size());
    }
  }

  if (!topology_found || little_cores.empty()) {
    LOGE("Scheduler: Failed to detect topology. Binding to ALL cores.");

    // Fallback: If detection fails, allow execution on all cores
    // to avoid leaving the task in an indeterminate affinity state.
    for (int i = 0; i < num_cores; ++i) {
      CPU_SET(i, &cpuset);
    }
  } else {

    // Bind the thread affinity mask strictly to the detected Little cores.
    for (int core_id : little_cores) {
      CPU_SET(core_id, &cpuset);
    }
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
    int num_cores = sysconf(_SC_NPROCESSORS_CONF);
    for (int i = 0; i < num_cores; ++i) {
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