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
#pragma once

#include <string>

namespace SystemUtils {

    // Reading a value from a file
    std::string readValueFromFile(const std::string& path);

    // Writing a value to a file
    void applyTweak(const std::string& path, const std::string& value);

    // Executing a shell command and returning its output
    std::string exec(const char* cmd);

    // Setting an Android system property
    void setProp(const std::string& key, const std::string& value);

    // Getting an Android system property
    std::string getProp(const std::string& key);

    // Waiting until the boot process is complete
    void waitForBoot();

} // namespace SystemUtils
