/**
 * @brief Defines the Foreign Function Interface (FFI) boundary between C++ and Rust.
 *
 * This header exposes the C-compatible API used for bidirectional communication.
 * It allows the C++ layer to control the Rust daemon lifecycle and
 * permits Rust logic to invoke system-level C++ utilities.
 *
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef DAEMON_INTERFACE_H
#define DAEMON_INTERFACE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Initializes and spawns the background Rust service threads.
 *
 * This function triggers the creation of monitoring threads.
 *
 * @note This function returns immediately after spawning threads.
 * @warning Should be called only once during the application startup phase.
 */
void rust_start_services(void);

/**
 * @brief Signals the Rust services to terminate and joins all threads.
 *
 * This function sets the internal shutdown flag and blocks the calling thread
 * until all Rust background threads have exited gracefully.
 *
 * @post All monitoring threads are destroyed and resources released.
 */
void rust_stop_services(void);

/**
 * @brief Writes a string value to a specified file path.
 *
 * Used primarily for writing to sysfs or procfs nodes (e.g., generic kernel tunables).
 *
 * @param path Absolute path to the target file.
 * @param value Null-terminated string value to write.
 * @return true if the write operation completed successfully; false on failure.
 */
bool cpp_apply_tweak(const char* path, const char* value);

/**
 * @brief Sets an Android system property.
 *
 * Wraps the native property setting mechanism (equivalent to `setprop`).
 *
 * @param key Property key (e.g., "persist.sys.my_prop").
 * @param value Property value.
 */
void cpp_set_system_prop(const char* key, const char* value);

/**
 * @brief Modifies an entry in the Android Settings database.
 *
 * Executes the system `settings` command to update global/system settings.
 *
 * @param property The specific setting key to update.
 * @param value The new value for the setting.
 * @return true if the settings command returned a success exit code; false otherwise.
 */
bool cpp_set_android_setting(const char* property, const char* value);

/**
 * @brief Logs an informational message to the Android system log.
 *
 * @param message Null-terminated string to log.
 */
void cpp_log_info(const char* message);

/**
 * @brief Logs a debug message to the Android system log.
 *
 * @note These logs may be stripped in release builds depending on compilation flags.
 * @param message Null-terminated string to log.
 */
void cpp_log_debug(const char* message);

/**
 * @brief Logs an error message to the Android system log.
 *
 * @param message Null-terminated string to log.
 */
void cpp_log_error(const char* message);

/**
 * @brief Closes a raw file descriptor.
 *
 * @param fd The file descriptor to close. If negative, the call is ignored.
 */
void cpp_close_fd(int fd);

/**
 * @brief Retrieves the current Memory Pressure Stall Information (PSI).
 *
 * Reads the "some" pressure average over the last 10 seconds (avg10).
 *
 * @return The pressure value (0.0 to 100.0), or -1.0 if retrieval fails.
 */
double cpp_get_memory_pressure(void);

/**
 * @brief Retrieves the current I/O Pressure Stall Information (PSI).
 *
 * Reads the "some" pressure average over the last 10 seconds (avg10).
 *
 * @return The pressure value (0.0 to 100.0), or -1.0 if retrieval fails.
 */
double cpp_get_io_pressure(void);

/**
 * @brief Checks a file descriptor for data availability.
 *
 * @param fd The file descriptor to poll.
 * @param timeout_ms Maximum time to wait in milliseconds.
 * @return
 * -  1: Data is available to read.
 * -  0: Timeout occurred.
 * - -1: Error occurred during polling.
 */
int cpp_poll_fd(int fd, int timeout_ms);

/**
 * @brief Opens a touch input device for event reading.
 *
 * Opens the device in non-blocking mode.
 *
 * @param path Path to the input device (e.g., "/dev/input/eventX").
 * @return A valid file descriptor on success, or -1 on failure.
 */
int cpp_open_touch_device(const char* path);

/**
 * @brief Consumes all pending events from the file descriptor.
 *
 * Intended to drain the event buffer to prevent stale data reads.
 *
 * @param fd Valid file descriptor for the input device.
 */
void cpp_read_touch_events(int fd);

#ifdef __cplusplus
}
#endif

#endif // DAEMON_INTERFACE_H