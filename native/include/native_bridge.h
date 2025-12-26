/**
 * @brief C ABI boundary between the C++ Daemon/JNI layer and the Rust logic library.
 *
 * This file defines the contract for FFI (Foreign Function Interface).
 * It uses standard C types to ensure ABI compatibility.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#ifndef NATIVE_BRIDGE_H
#define NATIVE_BRIDGE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// -----------------------------------------------------------------------------
// Rust -> C++ Calls (Downcalls)
// -----------------------------------------------------------------------------

/**
 * @brief Controls the lifecycle of the CPU Congestion Controller.
 *
 * Updates the atomic state in the Rust layer to enable or disable 
 * dynamic CPU scheduling adjustments based on Pressure Stall Information (PSI).
 *
 * @param enabled Set to true to activate the controller, false to pause/disable it.
 */
void rust_set_cpu_service_enabled(bool enabled);

/**
 * @brief Controls the lifecycle of the Memory Management Controller.
 *
 * Updates the atomic state in the Rust layer to enable or disable 
 * dynamic virtual memory tuning (swappiness, cache pressure, etc.).
 *
 * @param enabled Set to true to activate the controller, false to pause/disable it.
 */
void rust_set_memory_service_enabled(bool enabled);

/**
 * @brief Controls the lifecycle of the Storage I/O Controller.
 *
 * Updates the atomic state in the Rust layer to enable or disable 
 * dynamic I/O scheduler tuning (read-ahead, request queues) based on I/O pressure.
 *
 * @param enabled Set to true to activate the controller, false to pause/disable it.
 */
void rust_set_storage_service_enabled(bool enabled);

/**
 * @brief Toggles the application of static System Tweaks.
 *
 * Updates the atomic configuration in the Rust layer. If enabled, static
 * sysctl and property tweaks will be applied upon service startup.
 *
 * @param enabled Set to true to apply tweaks on start, false to skip.
 */
void rust_set_tweaks_enabled(bool enabled);

/**
 * @brief Initializes and starts the Rust event loop.
 *
 * This function passes control to the Rust runtime. It typically spawns
 * worker threads and prepares the reactor for event handling.
 *
 * @param signal_fd A file descriptor created via signalfd() to handle Unix signals.
 * @return 0 on success, non-zero error code on failure.
 */
int rust_start_services(int signal_fd);

/**
 * @brief Blocks the calling thread until all Rust services have shut down.
 *
 * Designed to be called from main() to prevent the process from exiting
 * while background threads are still active.
 */
void rust_join_threads(void);

// -----------------------------------------------------------------------------
// C++ -> Rust Callbacks (Upcalls)
// -----------------------------------------------------------------------------

/**
 * @brief Notifies the native layer that a service has encountered a fatal error.
 * @param context Null-terminated string describing the failure reason.
 */
void cpp_notify_service_death(const char* context);

/**
 * @brief Registers a Pressure Stall Information (PSI) trigger with the kernel.
 *
 * @param path Path to the PSI file (e.g., "/proc/pressure/memory").
 * @param threshold_us Stall threshold in microseconds.
 * @param window_us Window size in microseconds.
 * @return A valid file descriptor on success, or -1 on failure with errno set.
 *
 * @note The returned FD must be polled (epoll/select) for readable events.
 */
int cpp_register_psi_trigger(const char* path, int threshold_us, int window_us);

/**
 * @brief Sets an Android system property.
 * @return 0 on success, -1 on failure.
 */
int cpp_set_system_property(const char* key, const char* value);

/**
 * @brief Retrieves an Android system property.
 * @param max_len Size of the output buffer.
 * @return Length of the string copied, or -1 on error.
 */
int cpp_get_system_property(const char* key, char* value, size_t max_len);

#ifdef __cplusplus
}
#endif

#endif // NATIVE_BRIDGE_H