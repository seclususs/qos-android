/**
 * @brief Virtual Memory management tools.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

namespace qos::runtime {

    /**
     * @brief Handles memory locking and paging behavior.
     */
    class Memory {
    public:
        /**
         * @brief Locks the process memory into RAM.
         * Prevents the daemon's pages from being swapped out to ZRAM/Storage,
         * ensuring consistent latency for real-time operations.
         */
        static void lock_all_pages();
    };

}