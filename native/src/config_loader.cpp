// Author: [Seclususs](https://github.com/seclususs)

#include "config_loader.h"

#include <fstream>

namespace qos::config {

std::map<std::string, bool> load(const char *path) {
  std::map<std::string, bool> config;

  // Initialize defaults to ensure deterministic behavior if the file is missing
  // or empty.
  config["cpu"] = false;
  config["mem"] = false;
  config["io"] = false;
  config["display"] = false;
  config["cleaner"] = false;
  config["tweaks"] = false;

  std::ifstream file(path);
  if (!file.is_open()) {
    return config;
  }

  std::string line;
  while (std::getline(file, line)) {
    // Trim leading whitespace to handle indented configs.
    size_t first_char = line.find_first_not_of(" \t\r");
    if (first_char == std::string::npos) {
      continue;
    }
    line.erase(0, first_char);

    // Ignore comments (#, ;) and empty lines.
    if (line.empty() || line[0] == '#' || line[0] == ';') {
      continue;
    }

    size_t delim_pos = line.find('=');
    if (delim_pos != std::string::npos) {
      std::string key = line.substr(0, delim_pos);
      std::string val = line.substr(delim_pos + 1);

      // Trim trailing whitespace from the key.
      size_t key_end = key.find_last_not_of(" \t\r");
      if (key_end != std::string::npos) {
        key = key.substr(0, key_end + 1);
      }

      // Trim leading/trailing whitespace from the value.
      size_t val_start = val.find_first_not_of(" \t\r");
      if (val_start != std::string::npos) {
        val.erase(0, val_start);
        size_t val_end = val.find_last_not_of(" \t\r");
        if (val_end != std::string::npos) {
          val = val.substr(0, val_end + 1);
        }
      } else {
        val = "";
      }

      // Normalize boolean string representations.
      bool bool_val = false;
      if (val == "true" || val == "1" || val == "True") {
        bool_val = true;
      } else if (val == "false" || val == "0" || val == "False") {
        bool_val = false;
      } else {
        // Malformed boolean values are ignored; the default remains.
        continue;
      }

      // Map the configuration file keys to the internal identifiers used by the
      // Daemon.
      if (key == "cpu_enabled")
        config["cpu"] = bool_val;
      else if (key == "memory_enabled")
        config["mem"] = bool_val;
      else if (key == "storage_enabled")
        config["io"] = bool_val;
      else if (key == "display_enabled")
        config["display"] = bool_val;
      else if (key == "cleaner_enabled")
        config["cleaner"] = bool_val;
      else if (key == "tweaks_enabled")
        config["tweaks"] = bool_val;
    }
  }

  return config;
}

} // namespace qos::config