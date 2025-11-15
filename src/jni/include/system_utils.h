/**
 * @brief Provides utility functions for interacting with the Android system.
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef SYSTEM_UTILS_H
#define SYSTEM_UTILS_H

#include <string>

/**
 * @brief Namespace for general system utility functions.
 */
namespace SystemUtils {
    /**
     * @brief Applies a tweak by writing a string value to a file.
     *
     * @param path The absolute file path to write to.
     * @param value The string value to write.
     * @return true on successful write, false otherwise.
     */
    bool applyTweak(const std::string& path, const std::string& value);

    /**
     * @brief Sets an Android system property.
     *
     * @param key The property key.
     * @param value The property value.
     */
    void setSystemProp(const std::string& key, const std::string& value);

    /**
     * @brief Sets an Android system setting by executing the `settings` binary.
     *
     * @param property The setting name.
     * @param value The value to set.
     * @return true on successful execution, false otherwise.
     */
    bool setAndroidSetting(const std::string& property, const std::string& value);
}

#endif // SYSTEM_UTILS_H