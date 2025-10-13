/*
 * Copyright (C) 2025 Seclususs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#ifndef HARDWARE_INTERFACE_H
#define HARDWARE_INTERFACE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h> // For size_t

/**
 * @brief Writes a string to a specified file path.
 *
 * This function is primarily intended for interacting with sysfs/procfs
 * device files to change kernel parameters.
 * @param path The absolute path to the target file.
 * @param value The string value to write to the file.
 * @return 0 on success, -1 on failure.
 */
int write_to_file(const char* path, const char* value);

/**
 * @brief Reads total and available memory from /proc/meminfo.
 *
 * @param[out] mem_total Pointer to a long where the total memory (in kB) will be stored.
 * @param[out] mem_available Pointer to a long where the available memory (in kB) will be stored.
 * @return 0 on success, -1 if reading or parsing fails.
 */
int read_mem_info(long* mem_total, long* mem_available);

/**
 * @brief Waits for input activity on a specified input device.
 * * This function uses `select()` to monitor a device file descriptor for readability.
 * It can be used with a timeout to detect periods of inactivity.
 *
 * @param device_path Path to the input device (e.g., /dev/input/event3).
 * @param timeout_ms Timeout duration in milliseconds. A value of -1 indicates an infinite wait.
 * @return 1 if input is available, 0 on timeout, and -1 on error.
 */
int wait_for_input(const char* device_path, int timeout_ms);

/**
 * @brief Executes a shell command and captures its standard output and error.
 *
 * @param cmd The shell command to be executed.
 * @param[out] result_buffer A buffer to store the command's output.
 * @param buffer_size The size of the result_buffer.
 * @return The exit code of the command, or -1 if popen() fails.
 */
int execute_command(const char* cmd, char* result_buffer, size_t buffer_size);

#ifdef __cplusplus
}
#endif

#endif // HARDWARE_INTERFACE_H