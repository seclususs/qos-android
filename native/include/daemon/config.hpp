#pragma once

#include <string_view>

namespace qos::config
{

    struct ServiceState
    {
        bool cpu{false};
        bool io{false};
        bool cleaner{false};
        bool tweaks{false};
        bool blocker{false};
    };

    class Loader
    {
        public:
        [[nodiscard]] static ServiceState load_from_file(std::string_view path);
    };

} // namespace qos::config