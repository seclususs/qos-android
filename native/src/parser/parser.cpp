#include "daemon/parser.hpp"

#include <fstream>

namespace qos::parser
{
    namespace
    {

        [[nodiscard]] constexpr std::string_view trim(std::string_view sv) noexcept
        {
            const auto start = sv.find_first_not_of(" \t\r\n");

            if (start == std::string_view::npos)
                return {};

            const auto end = sv.find_last_not_of(" \t\r\n");

            return sv.substr(start, end - start + 1);
        }

    } // namespace

    std::map<std::string, std::string> Ini::parse_file(std::string_view path)
    {
        std::map<std::string, std::string> config;

        std::ifstream file{std::string(path)};

        if (!file.is_open())
            return config;

        std::string line;

        while (std::getline(file, line))
        {
            const auto sv = trim(line);

            if (sv.empty() || sv.starts_with('#') || sv.starts_with(';'))
                continue;

            const auto delim = sv.find('=');

            if (delim == std::string_view::npos)
                continue;

            const auto key = trim(sv.substr(0, delim));

            if (key.empty())
                continue;

            const auto val = trim(sv.substr(delim + 1));

            config.emplace(key, val);
        }

        return config;
    }

} // namespace qos::parser