#include "daemon/detector.hpp"
#include "daemon/logger.hpp"

#include <sys/statvfs.h>
#include <unistd.h>

namespace qos::system
{
    namespace
    {

        [[nodiscard]] inline bool can_access(const char* path, int mode = R_OK) noexcept
        {
            return ::access(path, mode) == 0;
        }

        [[nodiscard]] bool check_cleaner_env() noexcept
        {
            if (!can_access("/data/data", R_OK | X_OK))
                return false;

            if (!can_access("/proc", R_OK | X_OK))
                return false;

            struct statvfs vfs{};

            return ::statvfs("/data", &vfs) == 0;
        }

    } // namespace

    KernelFeatures Detector::check_features() noexcept
    {
        KernelFeatures features;

        features.cleaner_supported = check_cleaner_env();

        if (features.cleaner_supported)
        {
            LOGI("Detector: Cleaner prerequisites met.");
        }
        else
        {
            LOGE("Detector: Cleaner disabled.");
        }

        if (!can_access("/proc/pressure", R_OK | X_OK))
        {
            LOGE("Detector: /proc/pressure is missing. All PSI metrics disabled.");
            return features;
        }

        features.has_cpu_psi = can_access("/proc/pressure/cpu");

        if (features.has_cpu_psi)
        {
            LOGI("Detector: PSI CPU DETECTED.");
        }
        else
        {
            LOGE("Detector: WARNING - PSI CPU MISSING.");
        }

        features.has_io_psi = can_access("/proc/pressure/io");

        if (features.has_io_psi)
        {
            LOGI("Detector: PSI I/O DETECTED.");
        }
        else
        {
            LOGE("Detector: WARNING - PSI I/O MISSING.");
        }

        return features;
    }

} // namespace qos::system