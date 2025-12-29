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

namespace qos::runtime {
    
    void Scheduler::set_realtime_policy() {
        struct sched_param param;
        param.sched_priority = 50; // Moderate RT priority
        
        if (sched_setscheduler(0, SCHED_FIFO, &param) == -1) {
            LOGE("Scheduler: Failed to set SCHED_FIFO. Errno: %d", errno);
        } else {
            LOGI("Scheduler: Real-Time Policy (SCHED_FIFO) Active.");
        }
    }

    void Scheduler::bind_to_little_cores() {
        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);
        
        // Hardcoded assumption for typical 6+2 or 4+4 setups.
        // TODO: Dynamically detect topology via /sys/devices/system/cpu.
        for (int i = 0; i <= 5; ++i) {
            CPU_SET(i, &cpuset);
        }
        
        if (sched_setaffinity(0, sizeof(cpu_set_t), &cpuset) == -1) {
            LOGE("Scheduler: Failed to bind to Little Cores.");
        } else {
            LOGD("Scheduler: Thread bound to Little Cores.");
        }
    }

    void Scheduler::prepare_for_rust_handover() {
        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);
        
        // Target Big Cores for intensive Rust logic.
        // On standard Helio/Snapdragon, these are typically the last 2 cores.
        CPU_SET(6, &cpuset);
        CPU_SET(7, &cpuset);
        
        if (sched_setaffinity(0, sizeof(cpu_set_t), &cpuset) == -1) {
            // If Big Cores are offline (thermal throttling/hotplug), setaffinity fails.
            // We MUST NOT leave the thread bound to Little Cores (from previous call).
            LOGE("Scheduler: Failed to bind to Big Cores (errno: %d).", errno);
            
            CPU_ZERO(&cpuset);
            // Fallback: Enable all typical cores (0-7) to ensure we run somewhere.
            for (int i = 0; i <= 7; ++i) {
                CPU_SET(i, &cpuset);
            }
            
            if (sched_setaffinity(0, sizeof(cpu_set_t), &cpuset) == -1) {
                LOGE("Scheduler: CRITICAL - Failed to reset affinity.");
            } else {
                LOGI("Scheduler: Fallback successful. Affinity reset to default.");
            }
        } else {
            LOGI("Scheduler: Affinity mask set to Big Cores.");
        }
    }

}