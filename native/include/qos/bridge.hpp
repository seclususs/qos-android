/**
 * @file bridge.hpp
 * @brief ABI boundary declaration for the Native-to-Core interface.
 *
 * This header defines the C linkage functions used to interoperate between
 * the C++ runtime environment and the core logic library. It strictly uses
 * standard C types to maintain ABI compatibility across the language boundary.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

#include <cstddef>
#include <cstdint>

/**
 * @brief Foreign Function Interface (FFI) layout for CPU kernel limits.
 *
 * This structure defines the absolute upper and lower scheduling boundaries.
 * The core service dynamically adjusts the actual system parameters within
 * these configured bounds based on real-time hardware conditions.
 */
struct FfiCpuLimits
{
    /**
     * @brief Lower bound for CFS scheduler latency.
     *
     * Defines the minimum allowable latency to ensure that context switching
     * remains highly responsive during extreme computational workloads.
     * Unit: Nanoseconds (ns).
     */
    uint64_t min_latency_ns;

    /**
     * @brief Upper bound for CFS scheduler latency.
     *
     * Defines the maximum allowable latency used to conserve power during
     * idle periods or to mitigate hardware thermal constraints.
     * Unit: Nanoseconds (ns).
     */
    uint64_t max_latency_ns;

    /**
     * @brief Lower bound for task execution timeslice.
     *
     * Guarantees a strict minimum execution window for active tasks to
     * prevent cache thrashing and excessive context switch overhead.
     * Unit: Nanoseconds (ns).
     */
    uint64_t min_granularity_ns;

    /**
     * @brief Upper bound for task execution timeslice.
     *
     * Sets a ceiling on task execution time to prevent heavy background
     * processes from monopolizing CPU cycles and degrading responsiveness.
     * Unit: Nanoseconds (ns).
     */
    uint64_t max_granularity_ns;

    /**
     * @brief Lower bound for preemption penalty.
     *
     * Defines the minimum penalty to allow newly awakened tasks (e.g., UI
     * threads) to interrupt currently running tasks as quickly as possible.
     * Unit: Nanoseconds (ns).
     */
    uint64_t min_wakeup_ns;

    /**
     * @brief Upper bound for preemption penalty.
     *
     * Defines the maximum penalty to protect the throughput of active tasks
     * by preventing unnecessary interruptions from minor background events.
     * Unit: Nanoseconds (ns).
     */
    uint64_t max_wakeup_ns;

    /**
     * @brief Lower bound for inter-core migration penalty.
     *
     * The minimum cost threshold, allowing the system to aggressively
     * offload tasks to other cores during sudden spikes in workload.
     * Unit: Nanoseconds (ns).
     */
    uint64_t min_migration_cost;

    /**
     * @brief Upper bound for inter-core migration penalty.
     *
     * The maximum cost threshold to prevent tasks from bouncing between
     * cores, forcing them to utilize warm cache data during stable loads.
     * Unit: Nanoseconds (ns).
     */
    uint64_t max_migration_cost;

    /**
     * @brief Lower bound for new task weight prediction.
     *
     * The minimum initial load assumption for newly spawned tasks,
     * preventing them from unnecessarily waking up high-performance cores.
     * Range: 1 to 100.
     */
    uint64_t min_walt_init_pct;

    /**
     * @brief Upper bound for new task weight prediction.
     *
     * The maximum initial load assumption, ensuring that new tasks are
     * immediately allocated adequate performance during heavy system loads.
     * Range: 1 to 100.
     */
    uint64_t max_walt_init_pct;

    /**
     * @brief Lower bound for utilization clamp floor.
     *
     * The absolute minimum base frequency boost limit, allowing maximum
     * power savings when the device is completely idle. Range: 0 to 1024.
     */
    uint64_t min_uclamp_min;

    /**
     * @brief Upper bound for utilization clamp floor.
     *
     * The maximum allowable base frequency boost limit. Range: 0 to 1024.
     */
    uint64_t max_uclamp_min;
};

/**
 * @brief Foreign Function Interface (FFI) layout for block storage limits.
 *
 * This structure defines the operational boundaries for I/O queue management.
 * The core service operates within these limits to optimize throughput and
 * latency based on real-time storage characteristics.
 */
struct FfiStorageLimits
{
    /**
     * @brief Lower bound for read-ahead buffer size.
     *
     * Targeted when processing small, random files to minimize RAM usage
     * and queue latency. Unit: Kilobytes (KB).
     */
    uint64_t min_read_ahead;

    /**
     * @brief Upper bound for read-ahead buffer size.
     *
     * Targeted when loading large, sequential files to maximize hardware
     * transfer speeds. Unit: Kilobytes (KB).
     */
    uint64_t max_read_ahead;

    /**
     * @brief Lower bound for disk queue depth.
     *
     * The strict minimum queue length enforced during critical I/O
     * congestion to prevent severe bufferbloat. Unit: Request count.
     */
    uint64_t min_nr_requests;

    /**
     * @brief Upper bound for disk queue depth.
     *
     * The maximum allowed queue expansion point to fully utilize hardware
     * capabilities during healthy latency periods. Unit: Request count.
     */
    uint64_t max_nr_requests;
};

#ifdef __cplusplus
extern "C"
{
#endif

    /**
     * @brief Injects custom boundaries for the CPU scheduling service.
     *
     * Provides the core backend with custom operational limits. The service
     * will strictly adhere to these boundaries for its tuning operations.
     *
     * @param[in] limits Pointer to the FfiCpuLimits configuration. If NULL,
     * the backend safely falls back to its internal defaults.
     */
    void set_cpu_limits(const FfiCpuLimits* limits);

    /**
     * @brief Injects custom boundaries for the storage queue service.
     *
     * Provides the core backend with custom I/O operational limits.
     *
     * @param[in] limits Pointer to the FfiStorageLimits configuration.
     * If NULL, the backend safely falls back to its internal defaults.
     */
    void set_storage_limits(const FfiStorageLimits* limits);

    /**
     * @brief Configures the enabled state of the Blocker Controller service.
     *
     * Updates the configuration state for the component blocker service.
     * When enabled, this service enforces the disabled state of targeted
     * background components (such as specific GMS analytics and ad services)
     * to reduce unnecessary resource consumption and wakeups.
     *
     * This operation is thread-safe and the new state takes effect immediately.
     *
     * @param[in] enabled True to enable the service, false to disable.
     */
    void set_blocker_service(bool enabled);

    /**
     * @brief Configures the enabled state of the Cleaner Controller service.
     *
     * Updates the configuration state for the background cache and stale data
     * cleanup service. When enabled, the cleaner operates opportunistically
     * based on system load, thermal conditions, and storage pressure.
     *
     * This operation is thread-safe and the new state takes effect immediately
     * for the next maintenance cycle.
     *
     * @param[in] enabled True to enable the service, false to disable.
     */
    void set_cleaner_service(bool enabled);

    /**
     * @brief Configures the enabled state of the CPU Controller service.
     *
     * Updates the configuration state for the CPU pressure monitor.
     * This operation is thread-safe and the new state takes effect immediately
     * for the next polling cycle.
     *
     * @param[in] enabled True to enable the service, false to disable.
     */
    void set_cpu_service(bool enabled);

    /**
     * @brief Configures the enabled state of the Storage Controller service.
     *
     * Updates the configuration state for the Storage/IO pressure monitor.
     * This operation is thread-safe and the new state takes effect immediately
     * for the next polling cycle.
     *
     * @param[in] enabled True to enable the service, false to disable.
     */
    void set_storage_service(bool enabled);

    /**
     * @brief Configures the enabled state of the System Tweaks module.
     *
     * Determines whether boot-time optimizations (sysctl/prop) should be applied.
     *
     * @note This configuration is read only once during the service startup
     * sequence. Changes made after `rust_start_services` is called may
     * have no effect.
     *
     * @param[in] enabled True to apply tweaks, false to skip.
     */
    void set_tweaks(bool enabled);

    /**
     * @brief Initializes and starts the core service reactor in a background
     * thread.
     *
     * This function initializes the logging subsystem and spawns the main event
     * loop thread. It blocks only until initialization is complete (handshake
     * received). Use rust_join_threads() to wait for the service to terminate.
     *
     * @param[in] signal_fd A valid file descriptor (created via signalfd) used to
     * receive asynchronous POSIX signals within the event loop.
     * Must be a valid, readable file descriptor.
     *
     * @return 0 on successful initialization, non-zero on failure or timeout.
     */
    int start_services(int signal_fd);

    /**
     * @brief Waits for the core service threads to terminate.
     *
     * This function blocks the calling thread until the main thread of the
     * core library has joined. It ensures that the process does not exit
     * prematurely while services are cleaning up.
     */
    void join_threads(void);

    /**
     * @brief Reports a critical service failure to the native runtime.
     *
     * This callback allows the core library to log fatal errors via the
     * Android logging system before initiating a shutdown.
     *
     * @param[in] context A null-terminated C string describing the error context.
     * If NULL, a default "Unknown Reason" message is used.
     */
    void notify_service_death(const char* context);

    /**
     * @brief Registers a Pressure Stall Information (PSI) trigger with the kernel.
     *
     * This function handles the low-level file I/O required to register a
     * pollable trigger with the Linux kernel's PSI interface.
     *
     * @param[in] path         The filesystem path to the PSI resource (e.g.,
     * "/proc/pressure/cpu"). Must not be NULL.
     * @param[in] threshold_us The stall threshold in microseconds.
     * @param[in] window_us    The monitoring window size in microseconds.
     *
     * @return A valid file descriptor (>= 0) on success.
     * @return -1 on failure. In this case, `errno` is set to indicate the specific
     * error (e.g., `EINVAL` if path is null, `EACCES` if permission denied).
     *
     * @note The returned file descriptor ownership is transferred to the caller
     * and must be managed (closed) by the caller.
     */
    int register_psi_trigger(const char* path, int threshold_us, int window_us);

    /**
     * @brief Sets an Android system property.
     *
     * Wrapper around the Android system property API.
     *
     * @param[in] key   The property key string. Must not be NULL.
     * @param[in] value The property value string. Must not be NULL.
     *
     * @return 0 on success.
     * @return -1 on failure. If the underlying API fails without setting `errno`,
     * this wrapper sets `errno` to `EACCES` by default.
     */
    int set_system_property(const char* key, const char* value);

    /**
     * @brief Retrieves an Android system property.
     *
     * Wrapper around the Android system property API.
     *
     * @param[in]  key     The property key string. Must not be NULL.
     * @param[out] value   Buffer to store the retrieved value. Must not be NULL.
     * @param[in]  max_len Size of the buffer in bytes.
     *
     * @return The length of the retrieved value on success.
     * @return -1 on failure (e.g., if inputs are invalid).
     */
    int get_system_property(const char* key, char* value, size_t max_len);

#ifdef __cplusplus
}
#endif