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
 * @brief Updates the global state of the display service in the Rust layer.
 * @param enabled True to enable display optimization logic, false to bypass.
 */
void rust_set_display_service_enabled(bool enabled);

/**
 * @brief Initializes and starts the Rust event loop.
 *
 * This function passes control to the Rust runtime. It typically spawns
 * worker threads and prepares the reactor.
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