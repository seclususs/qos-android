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
    
    // Helper to apply affinity to Little Cores
    static int apply_little_core_affinity() {
        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);
        
        // Target Efficiency Cores
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

    void Scheduler::bind_to_little_cores() {
        if (apply_little_core_affinity() == -1) {
            LOGE("Scheduler: Failed to bind to Little Cores.");
        } else {
            LOGD("Scheduler: Thread bound to Little Cores.");
        }
    }

    void Scheduler::prepare_for_rust_handover() {
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
            LOGI("Scheduler: Affinity mask set to Little Cores.");
        }
    }

}