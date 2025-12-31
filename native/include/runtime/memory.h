/**
 * @file memory.h
 * @brief Virtual memory locking and management.
 *
 * This header defines the interface for controlling the physical memory
 * residency of the daemon process.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::runtime {

/**
 * @brief Manages memory locking policies.
 */
class Memory {
public:
  /**
   * @brief Locks the process address space into RAM.
   *
   * Invokes `mlockall` to prevent any part of the daemon (code, data, stack)
   * from being paged out to storage/ZRAM. This is critical for maintaining
   * deterministic latency when responding to PSI triggers.
   *
   * @warning This increases the persistent memory footprint of the process.
   */
  static void lock_all_pages();
};

} // namespace qos::runtime