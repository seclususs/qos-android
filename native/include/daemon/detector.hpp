#pragma once

namespace qos::system
{

    // System capability support flags.
    struct KernelFeatures
    {
        bool has_cpu_psi{false};
        bool has_io_psi{false};
        bool cleaner_supported{false};
    };

    // System environment.
    class Detector
    {
        public:
        // Scans filesystem to determine available kernel capabilities.
        [[nodiscard]] static KernelFeatures check_features() noexcept;
    };

} // namespace qos::system