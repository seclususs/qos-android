/**
 * @brief Implementation of the robust configuration parser.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "config_loader.h"

#include <fstream>

namespace qos::config {

    std::map<std::string, bool> load(const char* path) {
        std::map<std::string, bool> config;
        
        // Default values
        config["cpu"] = false;
        config["mem"] = false;
        config["io"] = false;
        config["tweaks"] = false;

        std::ifstream file(path);
        if (!file.is_open()) {
            // If config doesn't exist, return defaults
            return config;
        }

        std::string line;
        while (std::getline(file, line)) {
            // 1. Remove leading whitespace
            size_t first_char = line.find_first_not_of(" \t\r");
            if (first_char == std::string::npos) {
                continue; // Line is empty or contains only whitespace
            }
            line.erase(0, first_char);

            // Skip comments and empty lines
            if (line.empty() || line[0] == '#' || line[0] == ';') {
                continue;
            }

            // Find separator
            size_t delim_pos = line.find('=');
            if (delim_pos != std::string::npos) {
                std::string key = line.substr(0, delim_pos);
                std::string val = line.substr(delim_pos + 1);

                // Trim key (trailing whitespace)
                size_t key_end = key.find_last_not_of(" \t\r");
                if (key_end != std::string::npos) {
                    key = key.substr(0, key_end + 1);
                }

                // Trim value (leading and trailing whitespace)
                size_t val_start = val.find_first_not_of(" \t\r");
                if (val_start != std::string::npos) {
                    val.erase(0, val_start);
                    size_t val_end = val.find_last_not_of(" \t\r");
                    if (val_end != std::string::npos) {
                        val = val.substr(0, val_end + 1);
                    }
                } else {
                    val = ""; // Value is all whitespace
                }

                // Check boolean values
                bool bool_val = false;
                if (val == "true" || val == "1" || val == "True") {
                    bool_val = true;
                } else if (val == "false" || val == "0" || val == "False") {
                    bool_val = false;
                } else {
                    // Invalid value, skip or keep default logic? 
                    // Let's just continue parsing.
                    continue; 
                }

                // Map external keys (file) to internal keys (daemon)
                if (key == "cpu_enabled") config["cpu"] = bool_val;
                else if (key == "memory_enabled") config["mem"] = bool_val;
                else if (key == "storage_enabled") config["io"] = bool_val;
                else if (key == "tweaks_enabled") config["tweaks"] = bool_val;
            }
        }
        
        return config;
    }

}