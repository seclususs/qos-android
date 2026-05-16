#pragma once

#include <map>
#include <string>
#include <string_view>

namespace qos::parser
{

    class Ini
    {
        public:
        // Extracts key-value pairs from the specified file.
        [[nodiscard]] static std::map<std::string, std::string> parse_file(std::string_view path);
    };

} // namespace qos::parser