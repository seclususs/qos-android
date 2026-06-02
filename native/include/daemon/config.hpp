/**
 * @file config.hpp
 *
 * @brief Configuration state and management.
 */

#pragma once

#include "qos/bridge.hpp"
#include <cstdint>
#include <string_view>

namespace qos::config
{
    constexpr uint64_t EMPTY_U64 = static_cast<uint64_t>(-1);

    /**
     * @brief Operational configuration state for daemon services.
     */
    struct ServiceState
    {
        bool cpu{false};     ///< CPU service status.
        bool io{false};      ///< Storage I/O service status.
        bool cleaner{false}; ///< Cleaner service status.
        bool tweaks{false};  ///< System tweaks status.
        bool blocker{false}; ///< Blocker service status.

        bool has_cpu_limits{false}; ///< CPU limits availability flag.
        FfiCpuLimits cpu_limits{EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64,
                                EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64};

        bool has_storage_limits{false}; ///< Storage limits availability flag.
        FfiStorageLimits storage_limits{EMPTY_U64, EMPTY_U64, EMPTY_U64, EMPTY_U64};
    };

    /**
     * @brief Loads and validates system configuration.
     */
    class Loader
    {
        public:
        /**
         * @brief Parses and validates configuration from the provided path.
         *
         * @param path Path to the configuration file.
         *
         * @return A validated ServiceState structure.
         *
         * @note Fallback: Missing, empty, or unreadable entries will automatically fall
         *                 back to their safe baseline defaults.
         *
         * @note Sanitization: Successfully parsed values are strictly sanitized against
         *                     absolute system boundaries. Values may be dynamically
         *                     adjusted or clamped to ensure kernel safety.
         */
        [[nodiscard]] static ServiceState load_from_file(std::string_view path);
    };

} // namespace qos::config