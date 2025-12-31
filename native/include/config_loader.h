/**
 * @file config_loader.h
 * @brief Configuration file parser for the QoS Android.
 *
 * This header defines the interface for loading and parsing the runtime
 * configuration. It is designed to read a simple key-value pair format
 * (INI-style) to determine which service modules should be enabled at startup.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

#include <map>
#include <string>

namespace qos::config {

/**
 * @brief Loads and parses a configuration file from the specified path.
 *
 * This function reads the file line-by-line, stripping whitespace and comments,
 * and maps supported configuration keys to boolean states.
 *
 * @param path The filesystem path to the configuration file (e.g.,
 * "/data/adb/...").
 * @return A map associating internal feature identifiers (e.g., "cpu", "mem")
 * with their enabled state. Keys absent from the file will default to false in
 * the map unless handled by the caller.
 *
 * @note I/O operations performed by this function are blocking.
 */
std::map<std::string, bool> load(const char *path);

} // namespace qos::config