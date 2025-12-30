/**
 * @brief Implementation of scheduler manipulation.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/scheduler.h"
#include "logging.h"

#include <sched.h>
#include <unistd.h>
#include <cerrno>
#include <sys/prctl.h>
#include <sys/syscall.h>
#include <linux/types.h>
#include <cstring> 

// Definitions for UClamp
#ifndef __NR_sched_setattr
    #if defined(__aarch64__)
        #define __NR_sched_setattr 274
    #else
        #define __NR_sched_setattr 380
    #endif
#endif

// Flags definition
#ifndef SCHED_FLAG_KEEP_POLICY
    #define SCHED_FLAG_KEEP_POLICY 0x08
#endif

#ifndef SCHED_FLAG_UTIL_CLAMP_MAX
    #define SCHED_FLAG_UTIL_CLAMP_MAX 0x40
#endif

// sched_attr structure matching the Linux kernel
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

// Syscall wrapper
static int sched_setattr(pid_t pid, struct sched_attr *attr, unsigned int flags) {
    return syscall(__NR_sched_setattr, pid, attr, flags);
}

namespace qos::runtime {
    
    // Helper to apply affinity to Little Cores
    static int apply_little_core_affinity() {
        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);
        
        // Target Efficiency Cores (0-5 for Helio G88)
        for (int i = 0; i <= 5; ++i) {
            CPU_SET(i, &cpuset);
        }
        
        return sched_setaffinity(0, sizeof(cpu_set_t), &cpuset);
    }

    void Scheduler::set_realtime_policy() {
        struct sched_param param;
        param.sched_priority = 50; // Moderate RT priority
        
        if (sched_setscheduler(0, SCHED_FIFO, &param) == -1) {
            LOGE("Scheduler: Failed to set SCHED_FIFO. Errno: %d", errno);
        } else {
            LOGI("Scheduler: Real-Time Policy (SCHED_FIFO) Active.");
        }
    }

    void Scheduler::enforce_efficiency_mode() {
        if (apply_little_core_affinity() == -1) {
            LOGE("Scheduler: Failed to bind to Little Cores (errno: %d).", errno);
            
            // Fallback: Enable all typical cores (0-7) to ensure we run somewhere.
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
        // 50ms in nanoseconds.
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
        
        // Combine KEEP_POLICY with UTIL_CLAMP_MAX.
        attr.sched_flags = SCHED_FLAG_KEEP_POLICY | SCHED_FLAG_UTIL_CLAMP_MAX;
        
        // Scale 0 - 1024. 
        // 307 ~= 30% CPU Capacity.
        attr.sched_util_max = 307; 

        // PID 0 = current thread
        if (sched_setattr(0, &attr, 0) == -1) {
            LOGE("Scheduler: Failed to activate UClamp. Errno: %d", errno);
        } else {
            LOGI("Scheduler: UClamp Active.");
        }
    }

}