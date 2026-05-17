#pragma once

#include "qos/bridge.hpp"
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

        bool has_cpu_limits{false};
        FfiCpuLimits cpu_limits{8000000, 20000000, 2500000, 6500000, 1500000, 6500000, 200000, 600000, 10, 40, 0, 384};

        bool has_storage_limits{false};
        FfiStorageLimits storage_limits{128, 1024, 64, 256};
    };

    class Loader
    {
        public:
        [[nodiscard]] static ServiceState load_from_file(std::string_view path);
    };

} // namespace qos::config