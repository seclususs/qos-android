/**
 * @file parser.hpp
 *
 * @brief Configuration file parser.
 */

#pragma once

#include <map>
#include <string>
#include <string_view>

namespace qos::parser
{

    /**
     * @brief Utility for extracting configuration key-value pairs.
     */
    class Ini
    {
        public:
        /**
         * @brief Parses the specified configuration file.
         *
         * @param path Filesystem path to the configuration file.
         *
         * @return A map of extracted key-value pairs. Returns an empty map if file access fails.
         *
         * @note Parsing Behavior: Malformed lines, comments, or entries without a valid
         *                         delimiter ('=') are silently ignored to ensure parsing
         *                         continuity.
         *
         * @note Duplicate Keys: If multiple identical keys exist within the configuration file,
         *                       only the FIRST occurrence is stored, subsequent duplicates are
         *                       silently discarded.
         */
        [[nodiscard]] static std::map<std::string, std::string> parse_file(std::string_view path);
    };

} // namespace qos::parser