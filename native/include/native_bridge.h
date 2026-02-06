/**
 * @file native_bridge.h
 * @brief ABI boundary declaration for the Native-to-Core interface.
 *
 * This header defines the C linkage functions used to interoperate between
 * the C++ runtime environment and the core logic library. It strictly uses
 * standard C types to maintain ABI compatibility across the language boundary.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#ifndef NATIVE_BRIDGE_H
#define NATIVE_BRIDGE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// -----------------------------------------------------------------------------
// Core Library Entry Points (Downcalls: C++ calls Rust)
// -----------------------------------------------------------------------------

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
void rust_set_blocker_service_enabled(bool enabled);

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
void rust_set_cleaner_service_enabled(bool enabled);

/**
 * @brief Configures the enabled state of the CPU Controller service.
 *
 * Updates the configuration state for the CPU pressure monitor.
 * This operation is thread-safe and the new state takes effect immediately
 * for the next polling cycle.
 *
 * @param[in] enabled True to enable the service, false to disable.
 */
void rust_set_cpu_service_enabled(bool enabled);

/**
 * @brief Configures the enabled state of the Storage Controller service.
 *
 * Updates the configuration state for the Storage/IO pressure monitor.
 * This operation is thread-safe and the new state takes effect immediately
 * for the next polling cycle.
 *
 * @param[in] enabled True to enable the service, false to disable.
 */
void rust_set_storage_service_enabled(bool enabled);

/**
 * @brief Configures the enabled state of the Display Controller service.
 *
 * Updates the configuration state for the Adaptive Display monitor.
 * This operation is thread-safe and the new state takes effect immediately
 * for the next polling cycle.
 *
 * @param[in] enabled True to enable the service, false to disable.
 */
void rust_set_display_service_enabled(bool enabled);

/**
 * @brief Configures the enabled state of the System Tweaks module.
 *
 * Determines whether boot-time optimizations (sysctl/prop) should be applied.
 *
 * @note This configuration is read only once during the service startup
 *       sequence. Changes made after `rust_start_services` is called may 
 *       have no effect.
 *
 * @param[in] enabled True to apply tweaks, false to skip.
 */
void rust_set_tweaks_enabled(bool enabled);

/**
 * @brief Initializes and starts the core service reactor in a background
 * thread.
 *
 * This function initializes the logging subsystem and spawns the main event
 * loop thread. It blocks only until initialization is complete (handshake
 * received). Use rust_join_threads() to wait for the service to terminate.
 *
 * @param[in] signal_fd A valid file descriptor (created via signalfd) used to
 *                      receive asynchronous POSIX signals within the event loop. 
 *                      Must be a valid, readable file descriptor.
 *
 * @return 0 on successful initialization, non-zero on failure or timeout.
 */
int rust_start_services(int signal_fd);

/**
 * @brief Waits for the core service threads to terminate.
 *
 * This function blocks the calling thread until the main thread of the
 * core library has joined. It ensures that the process does not exit
 * prematurely while services are cleaning up.
 */
void rust_join_threads(void);

// -----------------------------------------------------------------------------
// Native Runtime Callbacks (Upcalls: Rust calls C++)
// -----------------------------------------------------------------------------

/**
 * @brief Reports a critical service failure to the native runtime.
 *
 * This callback allows the core library to log fatal errors via the
 * Android logging system before initiating a shutdown.
 *
 * @param[in] context A null-terminated C string describing the error context.
 *                    If NULL, a default "Unknown Reason" message is used.
 */
void cpp_notify_service_death(const char *context);

/**
 * @brief Registers a Pressure Stall Information (PSI) trigger with the kernel.
 *
 * This function handles the low-level file I/O required to register a
 * pollable trigger with the Linux kernel's PSI interface.
 *
 * @param[in] path         The filesystem path to the PSI resource (e.g.,
 *                         "/proc/pressure/cpu"). Must not be NULL.
 * @param[in] threshold_us The stall threshold in microseconds.
 * @param[in] window_us    The monitoring window size in microseconds.
 *
 * @return A valid file descriptor (>= 0) on success.
 * @return -1 on failure. In this case, `errno` is set to indicate the specific
 *         error (e.g., `EINVAL` if path is null, `EACCES` if permission denied).
 *
 * @note The returned file descriptor ownership is transferred to the caller
 *       and must be managed (closed) by the caller.
 */
int cpp_register_psi_trigger(const char *path, int threshold_us, int window_us);

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
 *         this wrapper sets `errno` to `EACCES` by default.
 */
int cpp_set_system_property(const char *key, const char *value);

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
int cpp_get_system_property(const char *key, char *value, size_t max_len);

/**
 * @brief Sets the display refresh rate via a direct SurfaceFlinger transaction.
 *
 * Issues a low-level Binder transaction to the SurfaceFlinger service by
 * invoking `/system/bin/service` through `execve()`, bypassing the shell
 * environment entirely. This reduces overhead and avoids dependency on
 * shell state or PATH resolution.
 *
 * @note This function relies on a device-specific SurfaceFlinger transaction
 *       code (e.g. `1035`) that is **not part of the public Android API**.
 *       The transaction ID, accepted parameters, and behavior are vendor-
 *       and version-specific and may differ across devices, ROMs, or
 *       Android releases.
 *
 * @warning This function is intended only for devices known to support the
 *          targeted SurfaceFlinger transaction. Calling it on unsupported
 *          devices may result in a no-op, transaction failure, or undefined
 *          behavior.
 *
 * @param[in] refresh_rate_mode Vendor-defined mode identifier
 *                              (commonly 0 = 60Hz, 1 = 90Hz/120Hz).
 *
 * @return 0 on successful transaction execution.
 * @return -1 on failure. Errors may indicate missing service support,
 *         execution failure, or an unsupported transaction code.
 */
int cpp_set_refresh_rate(int refresh_rate_mode);

#ifdef __cplusplus
}
#endif

#endif // NATIVE_BRIDGE_H