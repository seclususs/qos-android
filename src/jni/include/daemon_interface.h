/**
 * @brief C-style function interface for communication between Rust and C++ components.
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
 * @brief Starts the background Rust services.
 */
void rust_start_services(void);

/**
 * @brief Stops the background Rust services.
 */
void rust_stop_services(void);

/**
 * @brief Applies a system tweak by writing a value to a file path.
 *
 * @param path The file path to write to.
 * @param value The string value to write.
 * @return true if the write was successful, false otherwise.
 */
bool cpp_apply_tweak(const char* path, const char* value);

/**
 * @brief Sets an Android system property.
 *
 * @param key The property key.
 * @param value The value to set for the property.
 */
void cpp_set_system_prop(const char* key, const char* value);

/**
 * @brief Sets an Android system setting.
 *
 * @param property The setting property name.
 * @param value The value to set for the setting.
 * @return true if the setting was set successfully, false otherwise.
 */
bool cpp_set_android_setting(const char* property, const char* value);

/**
 * @brief Logs an informational message.
 *
 * @param message The message string to log.
 */
void cpp_log_info(const char* message);

/**
 * @brief Logs a debug message.
 *
 * @param message The message string to log.
 */
void cpp_log_debug(const char* message);

/**
 * @brief Logs an error message.
 *
 * @param message The message string to log.
 */
void cpp_log_error(const char* message);

/**
 * @brief Closes a file descriptor.
 *
 * @param fd The file descriptor to close.
 */
void cpp_close_fd(int fd);

/**
 * @brief Gets the percentage of free RAM.
 *
 * @return The free RAM percentage (0-100), or -1 on failure.
 */
int cpp_get_free_ram_percentage(void);

/**
 * @brief Creates and binds a netlink socket.
 *
 * @return The file descriptor for the netlink socket, or -1 on failure.
 */
int cpp_create_netlink_socket(void);

/**
 * @brief Polls a file descriptor for readable data.
 *
 * @param fd The file descriptor to poll.
 * @param timeout_ms The maximum time to wait in milliseconds.
 * @return 1 if data is available, 0 on timeout, -1 on error.
 */
int cpp_poll_fd(int fd, int timeout_ms);

/**
 * @brief Reads a netlink event from a socket.
 *
 * @param fd The netlink socket file descriptor.
 * @param buffer A pointer to the destination buffer.
 * @param buffer_size The size of the destination buffer.
 * @return The number of bytes read, 0 if non-blocking read or interrupt, or -1 on error.
 */
int cpp_read_netlink_event(int fd, char* buffer, int buffer_size);

/**
 * @brief Opens a touch input device.
 *
 * @param path The path to the input event device.
 * @return The file descriptor for the device, or -1 on failure.
 */
int cpp_open_touch_device(const char* path);

/**
 * @brief Drains all pending touch events from a file descriptor.
 *
 * @param fd The file descriptor of the touch device.
 */
void cpp_read_touch_events(int fd);


#ifdef __cplusplus
}
#endif

#endif // DAEMON_INTERFACE_H