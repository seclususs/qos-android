/**
 * @brief Implementation of process protection mechanisms.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/protection.h"
#include "logging.h"

#include <fstream>

namespace qos::runtime {

    void Protection::harden_process() {
        const char* path = "/proc/self/oom_score_adj";
        std::ofstream file(path);
        
        if (file.is_open()) {
            // -1000 is the lowest possible score (OOM Shield).
            file << "-1000";
            
            if (file.fail()) {
                LOGE("Protection: Failed to write OOM score.");
            } else {
                LOGI("Protection: OOM Shield Activated.");
            }
        } else {
            LOGE("Protection: Cannot open OOM adjustment file.");
        }
    }

}