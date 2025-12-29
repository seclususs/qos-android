/**
 * @brief Configuration loader definition.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#pragma once

#include <map>
#include <string>

namespace qos::config {

    /**
     * @brief Loads and parses the configuration file.
     * Reads a key-value INI-style file. Handles comments (#, ;), 
     * whitespace trimming, and boolean parsing (true/1/on).
     * @param path Path to the configuration file.
     * @return std::map<std::string, bool> Map of internal feature keys ("cpu", "mem", etc.) to enabled state.
     */
    std::map<std::string, bool> load(const char* path);

}