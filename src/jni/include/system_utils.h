/**
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

/**
 * @file system_utils.h
 * @brief Public interface for system utility functions.
 *
 * Provides helper functions for common system interactions, such as
 * writing to kernel (sysfs) files, setting Android system properties,
 * and executing shell commands to change system settings.
 */

#ifndef SYSTEM_UTILS_H
#define SYSTEM_UTILS_H

#include <stdbool.h>
#include <stddef.h>

/**
 * @brief Default buffer size for capturing output from shell commands.
 */
extern const size_t REFRESH_RATE_CONFIG_COMMAND_OUTPUT_BUFFER_SIZE;

/**
 * @brief Writes a string value to a specified file path (e.g., in /proc or /sys).
 *
 * Attempts a direct `write()` first, falling back to `fprintf()`
 * for compatibility.
 *
 * @param path The full filesystem path to write to.
 * @param value The string value to write.
 * @return true on success, false on failure.
 */
bool systemUtils_applyTweak(const char* path, const char* value);

/**
 * @brief Sets an Android system property.
 *
 * Uses the `__system_property_set` function.
 *
 * @param key The name of the property to set.
 * @param value The value to set the property to.
 */
void systemUtils_setSystemProp(const char* key, const char* value);

/**
 * @brief Sets an Android system setting via the `settings` shell command.
 *
 * Executes `/system/bin/settings put system [property] [value]`.
 *
 * @param property The name of the system setting.
 * @param value The value to set the setting to.
 * @return true on success (command exit code 0), false otherwise.
 */
bool systemUtils_setAndroidSetting(const char* property, const char* value);

#endif // SYSTEM_UTILS_H