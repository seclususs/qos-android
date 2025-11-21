/**
 * @brief Utilities for filesystem and system property interaction.
 * 
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef SYSTEM_UTILS_H
#define SYSTEM_UTILS_H

#include <string>

/**
 * @namespace SystemUtils
 * @brief Helper functions for system modifications.
 */
namespace SystemUtils {
    /**
     * @brief Writes a string payload to a target file.
     *
     * Attempts to perform a robust write operation, handling file opening
     * and data transfer.
     *
     * @param path The absolute path of the destination file.
     * @param value The string content to write.
     * @return true on success; false if the file cannot be opened or written to.
     */
    bool applyTweak(const std::string& path, const std::string& value);

    /**
     * @brief Sets an Android system property.
     *
     * @param key The property identifier.
     * @param value The value to set.
     */
    void setSystemProp(const std::string& key, const std::string& value);

    /**
     * @brief Updates a value in the Android 'system' settings provider.
     *
     * Involves spawning a process to execute the `settings` binary.
     *
     * @param property The setting name to modify.
     * @param value The new value.
     * @return true if the settings command executed successfully.
     */
    bool setAndroidSetting(const std::string& property, const std::string& value);
}

#endif // SYSTEM_UTILS_H