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
#include "include/system_utils.h"
#include "include/log_utils.h"
#include <fstream>
#include <memory>
#include <array>
#include <unistd.h>
#include <sys/system_properties.h>
#include <thread>
#include <chrono>

namespace SystemUtils {

std::string readValueFromFile(const std::string& path) {
    std::ifstream file(path);
    std::string value;
    if (file.is_open()) {
        std::getline(file, value);
        file.close();
    }
    return value;
}

void applyTweak(const std::string& path, const std::string& value) {
    if (access(path.c_str(), W_OK) == 0) {
        std::ofstream outfile(path);
        if (outfile.is_open()) {
            outfile << value;
            outfile.close();
        } else {
            LOGE("Failed to open for writing: %s", path.c_str());
        }
    }
}

std::string exec(const char* cmd) {
    std::array<char, 128> buffer;
    std::string result;
    std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(cmd, "r"), pclose);
    if (!pipe) {
        LOGE("popen() failed for command: %s", cmd);
        return "";
    }
    while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
        result += buffer.data();
    }
    if (!result.empty() && result.back() == '\n') {
        result.pop_back();
    }
    return result;
}

void setProp(const std::string& key, const std::string& value) {
    if (__system_property_set(key.c_str(), value.c_str()) < 0) {
        LOGE("Failed to set property: %s", key.c_str());
    }
}

std::string getProp(const std::string& key) {
    char value[PROP_VALUE_MAX] = {0};
    __system_property_get(key.c_str(), value);
    return std::string(value);
}

void waitForBoot() {
    while (getProp("sys.boot_completed") != "1") {
        std::this_thread::sleep_for(std::chrono::seconds(1));
    }
    LOGI("Boot completed.");
}

} // namespace SystemUtils
