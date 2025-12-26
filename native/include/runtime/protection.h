/**
 * @brief Process self-defense and identity masking.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

#include <string>

namespace qos::runtime {

    /**
     * @brief Tools to protect the daemon from OS termination.
     */
    class Protection {
    public:
        /**
         * @brief Sets OOM (Out of Memory) Score Adjustment.
         * Sets `/proc/self/oom_score_adj` to -1000 to make the process unkillable
         * by the Low Memory Killer Daemon (LMKD) under normal circumstances.
         */
        static void harden_process();
    };

}