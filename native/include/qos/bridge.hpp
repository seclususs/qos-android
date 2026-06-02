/**
 * @file bridge.hpp
 *
 * @brief Foreign Function Interface (FFI) bindings for system and daemon runtime integration.
 *
 * This file defines the structures and functions used to bridge the system-level C++ implementation
 * with the background runtime services.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

#include <cstddef>
#include <cstdint>

/**
 * @brief C-compatible structure defining CPU limits configuration.
 */
struct FfiCpuLimits
{
    uint64_t min_latency_ns;     ///< Minimum scheduler latency in nanoseconds.
    uint64_t max_latency_ns;     ///< Maximum scheduler latency in nanoseconds.
    uint64_t min_granularity_ns; ///< Minimum scheduling granularity in nanoseconds.
    uint64_t max_granularity_ns; ///< Maximum scheduling granularity in nanoseconds.
    uint64_t min_wakeup_ns;      ///< Minimum wakeup granularity in nanoseconds.
    uint64_t max_wakeup_ns;      ///< Maximum wakeup granularity in nanoseconds.
    uint64_t min_migration_cost; ///< Minimum task migration cost in nanoseconds.
    uint64_t max_migration_cost; ///< Maximum task migration cost in nanoseconds.
    uint64_t min_walt_init_pct;  ///< Minimum WALT initial task load percentage.
    uint64_t max_walt_init_pct;  ///< Maximum WALT initial task load percentage.
    uint64_t min_uclamp_min;     ///< Minimum utilization clamp value.
    uint64_t max_uclamp_min;     ///< Maximum utilization clamp value.
};

/**
 * @brief C-compatible structure defining storage I/O limits configuration.
 */
struct FfiStorageLimits
{
    uint64_t min_read_ahead;  ///< Minimum read-ahead limit.
    uint64_t max_read_ahead;  ///< Maximum read-ahead limit.
    uint64_t min_nr_requests; ///< Minimum number of I/O requests.
    uint64_t max_nr_requests; ///< Maximum number of I/O requests.
};

#ifdef __cplusplus
extern "C"
{
#endif

    /**
     * @brief Initializes the CPU limits override configuration.
     *
     * @param[in] limits Pointer to the CPU limits configuration structure. If null,
     *                   the operation is safely ignored.
     *
     * @note Execution Limit: This configuration utilizes a write-once mechanism. It must only be
     *                        called once during startup. Subsequent calls will be safely ignored
     *                        without modifying the active state.
     *
     * @note Ownership: The memory pointed to by limits remains owned by the caller. The data is
     *                  converted and copied internally during the call.
     */
    void set_cpu_limits(const FfiCpuLimits* limits);

    /**
     * @brief Initializes the storage I/O limits override configuration.
     *
     * @param[in] limits Pointer to the storage limits configuration structure. If null,
     *                   the operation is safely ignored.
     *
     * @note Execution Limit: This configuration utilizes a write-once mechanism. It must only be
     *                        called once during startup. Subsequent calls will be safely ignored
     *                        without modifying the active state.
     *
     * @note Ownership: The memory pointed to by limits remains owned by the caller. The data is
     *                  converted and copied internally during the call.
     */
    void set_storage_limits(const FfiStorageLimits* limits);

    /**
     * @brief Sets the enablement state for the Blocker service.
     *
     * @param[in] enabled True to enable the service, false to disable.
     *
     * @note Execution Limit: This is an initialization parameter. It MUST be set prior
     *                        to calling start_services(). Any state changes made after the
     *                        runtime has started will not affect the running service.
     *
     * @note Default State: Defaults to false if not explicitly set.
     */
    void set_blocker_service(bool enabled);

    /**
     * @brief Sets the enablement state for the Cleaner service.
     *
     * @param[in] enabled True to enable the service, false to disable.
     *
     * @note Execution Limit: This is an initialization parameter. It MUST be set prior
     *                        to calling start_services(). Any state changes made after the
     *                        runtime has started will not affect the running service.
     *
     * @note Default State: Defaults to false if not explicitly set.
     */
    void set_cleaner_service(bool enabled);

    /**
     * @brief Sets the enablement state for the CPU service.
     *
     * @param[in] enabled True to enable the service, false to disable.
     *
     * @note Execution Limit: This is an initialization parameter. It MUST be set prior
     *                        to calling start_services(). Any state changes made after the
     *                        runtime has started will not affect the running service.
     *
     * @note Default State: Defaults to false if not explicitly set.
     */
    void set_cpu_service(bool enabled);

    /**
     * @brief Sets the enablement state for the Storage service.
     *
     * @param[in] enabled True to enable the service, false to disable.
     *
     * @note Execution Limit: This is an initialization parameter. It MUST be set prior
     *                        to calling start_services(). Any state changes made after the
     *                        runtime has started will not affect the running service.
     *
     * @note Default State: Defaults to false if not explicitly set.
     */
    void set_storage_service(bool enabled);

    /**
     * @brief Sets the enablement state for system tweaks application.
     *
     * @param[in] enabled True to enable system tweaks, false to disable.
     *
     * @note Execution Limit: This is an initialization parameter. It MUST be set prior
     *                        to calling start_services(). Any state changes made after the
     *                        runtime has started will not affect the running service.
     *
     * @note Default State: Defaults to false if not explicitly set.
     */
    void set_tweaks(bool enabled);

    /**
     * @brief Initializes the runtime and starts background services and event loops.
     *
     * @param[in] signal_fd A raw file descriptor used for signal handling.
     *
     * @retval  0 On successful runtime initialization and handshake.
     *
     * @retval -1 On initialization failure or timeout.
     *
     * @note Execution: This is a synchronous blocking call. It will block the calling thread
     *                  to perform a handshake with the background runtime until it succeeds
     *                  or a timeout occurs.
     *
     * @note Ownership: Absolute ownership of signal_fd is transferred to the runtime upon
     *                  calling this function.
     *
     * @warning Undefined Behavior: The caller must not close, read, or write to signal_fd
     *                              after passing it to this function, as the runtime manages
     *                              its lifecycle and closes it upon shutdown or failure.
     */
    int start_services(int signal_fd);

    /**
     * @brief Blocks the current thread until the main service threads terminate.
     *
     * @note Idempotency: This function is idempotent. Subsequent calls after the thread
     *                    has been joined are safe and will return immediately.
     *
     * @warning Deadlock: This function must not be called from within the background service
     *                    threads themselves.
     */
    void join_threads(void);

    /**
     * @brief Logs a critical service death notification and triggers a graceful shutdown sequence.
     *
     * @param[in] context Null-terminated string indicating the context or cause of failure. If null,
     *                    a generic unknown reason string is assumed.
     *
     * @note Execution: This function initiates a graceful shutdown sequence for the application,
     *                  allowing background event loops to clean up resources before exiting.
     */
    void notify_service_death(const char* context);

    /**
     * @brief Registers a Pressure Stall Information (PSI) trigger on a given path.
     *
     * @param[in] path         Null-terminated string specifying the path to the PSI monitor file.
     *
     * @param[in] threshold_us Threshold value in microseconds.
     *
     * @param[in] window_us    Window value in microseconds.
     *
     * @return Returns a valid file descriptor on success, or -1 on failure.
     *
     * @note Error Handling: Sets errno to EINVAL if parameters are invalid, or an appropriate
     *                       error code if internal parameter bounds are exceeded or underlying
     *                       system operations fail.
     *
     * @note Ownership: The caller assumes ownership of the returned file descriptor and is
     *                  responsible for closing it.
     */
    int register_psi_trigger(const char* path, int threshold_us, int window_us);

    /**
     * @brief Sets an Android system property key to a specified value.
     *
     * @param[in] key   Null-terminated string representing the system property key.
     *
     * @param[in] value Null-terminated string representing the value to assign.
     *
     * @retval  0 On successful property assignment.
     *
     * @retval -1 On failure.
     *
     * @note Error Handling: Sets errno to EINVAL if parameters are invalid, or an appropriate
     *                       error code if the underlying system property operation fails.
     */
    int set_system_property(const char* key, const char* value);

    /**
     * @brief Retrieves the value of an Android system property.
     *
     * @param[in]  key     Null-terminated string representing the system property key to query.
     *
     * @param[out] value   Buffer where the retrieved property value will be stored.
     *
     * @param[in]  max_len Maximum capacity of the value buffer. The output will be safely
     *                     truncated and null-terminated if it exceeds this length.
     *
     * @return Returns the length of the copied property value on success. Returns 0 if the
     *         property does not exist or is empty. Returns -1 on fatal parameter failure.
     *
     * @note Error Handling: Sets errno to EINVAL if key is null, value is null, or max_len is 0.
     */
    int get_system_property(const char* key, char* value, size_t max_len);

#ifdef __cplusplus
}
#endif