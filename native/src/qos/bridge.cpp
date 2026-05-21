#include "qos/bridge.hpp"
#include "daemon/logger.hpp"

#include <cerrno>
#include <cstdio>
#include <fcntl.h>
#include <sys/system_properties.h>
#include <unistd.h>
#include <utility>

namespace
{
    class ScopedFd
    {
        public:
        explicit ScopedFd(int fd) noexcept : fd_(fd) {}
        ~ScopedFd()
        {
            if (fd_ >= 0)
                ::close(fd_);
        }

        ScopedFd(const ScopedFd&) = delete;
        ScopedFd& operator=(const ScopedFd&) = delete;

        [[nodiscard]] bool is_valid() const noexcept
        {
            return fd_ >= 0;
        }

        [[nodiscard]] int get() const noexcept
        {
            return fd_;
        }

        int release() noexcept
        {
            return std::exchange(fd_, -1);
        }

        private:
        int fd_ = -1;
    };
} // namespace

extern "C" void notify_service_death(const char* context)
{
    LOGE("!!! CRITICAL: %s !!!", context ? context : "Unknown Reason");
    LOGE("Requesting graceful shutdown...");
}

extern "C" int register_psi_trigger(const char* path, int threshold_us, int window_us)
{
    if (!path)
        return (errno = EINVAL, -1);

    ScopedFd fd(::open(path, O_RDWR | O_CLOEXEC | O_NONBLOCK));

    if (!fd.is_valid())
    {
        LOGE("Failed to open PSI file: %s (errno: %d)", path, errno);
        return -1;
    }

    char trigger_cmd[128];
    int len_nl = std::snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d\n", threshold_us, window_us);

    if (len_nl < 0 || static_cast<size_t>(len_nl) >= sizeof(trigger_cmd))
    {
        errno = EOVERFLOW;
        return -1;
    }

    if (::write(fd.get(), trigger_cmd, len_nl) >= 0)
    {
        LOGD("Registered PSI trigger: %s on fd %d", trigger_cmd, fd.get());
        return fd.release();
    }

    LOGE("Fatal trigger write error: %s (errno: %d)", trigger_cmd, errno);
    return -1;
}

extern "C" int set_system_property(const char* key, const char* value)
{
    if (!key || !value)
        return (errno = EINVAL, -1);

    if (__system_property_set(key, value) == 0)
        return 0;

    if (errno == 0)
        errno = EACCES;

    return -1;
}

extern "C" int get_system_property(const char* key, char* value, size_t max_len)
{
    if (!key || !value || max_len == 0)
        return (errno = EINVAL, -1);

    return __system_property_get(key, value);
}