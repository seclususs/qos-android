/**
 * @brief Implementation of the C ABI exposed to Rust.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "native_bridge.h"
#include "logging.h"

#include <unistd.h>
#include <fcntl.h>
#include <cstdio>
#include <cstring>
#include <cerrno>
#include <sys/system_properties.h>

/**
 * @brief Simple RAII wrapper for file descriptors.
 * Ensures strict ownership and prevents resource leaks on early returns.
 */
class ScopedFd {
public:
    explicit ScopedFd(int fd) : fd_(fd) {}
    
    ~ScopedFd() {
        if (fd_ >= 0) {
            close(fd_);
        }
    }

    // Prevent copying to avoid double-close issues.
    ScopedFd(const ScopedFd&) = delete;
    ScopedFd& operator=(const ScopedFd&) = delete;

    bool isValid() const { return fd_ >= 0; }
    int get() const { return fd_; }

    /** Releases ownership of the fd to the caller. */
    int release() {
        int temp = fd_;
        fd_ = -1;
        return temp;
    }
private:
    int fd_;
};

// -----------------------------------------------------------------------------
// System Helpers
// -----------------------------------------------------------------------------

extern "C" void cpp_notify_service_death(const char* context) {
    const char* reason = context ? context : "Unknown Reason";
    LOGE("!!! SERVICE CRITICAL: %s !!!", reason);
    LOGE("Requesting graceful shutdown from Rust layer...");
}

extern "C" int cpp_register_psi_trigger(const char* path, int threshold_us, int window_us) {
    if (!path) {
        errno = EINVAL;
        return -1;
    }

    // Open PSI file for writing to register the trigger.
    // O_RDWR is required because we might need to read/poll events later.
    // O_NONBLOCK is critical for event loops (epoll).
    int raw_fd = open(path, O_RDWR | O_CLOEXEC | O_NONBLOCK);
    ScopedFd fd(raw_fd);
    
    if (!fd.isValid()) {
        LOGE("Failed to open PSI file: %s (errno: %d)", path, errno);
        return -1;
    }

    // Format strictly required by Linux PSI interface: "some <threshold> <window>"
    // threshold: max stall time allowed within the window.
    // window: size of the sliding window.
    char trigger_cmd[128];
    
    // We strictly use snprintf to ensure buffer safety.
    int len = snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d", threshold_us, window_us);
    
    if (len < 0 || (size_t)len >= sizeof(trigger_cmd)) {
        errno = EOVERFLOW;
        return -1;
    }

    // Include the null terminator ('\0') in the write buffer to ensure correct parsing 
    // by the kernel's sscanf implementation.
    if (write(fd.get(), trigger_cmd, len + 1) < 0) {
        LOGE("Failed to write trigger: %s (errno: %d). Retrying with newline...", trigger_cmd, errno);
        
        // Fallback strategy for kernels that strictly require a newline delimiter.
        int len_nl = snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d\n", threshold_us, window_us);
        if (write(fd.get(), trigger_cmd, len_nl) < 0) {
            LOGE("Retry failed. Fatal trigger write error: %s (errno: %d)", trigger_cmd, errno);
            return -1;
        }
    }

    LOGD("Successfully registered PSI trigger: %s on fd %d", trigger_cmd, fd.get());

    // Transfer ownership to Rust; Rust will add this FD to epoll.
    return fd.release();
}

extern "C" int cpp_set_system_property(const char* key, const char* value) {
    if (!key || !value) {
        errno = EINVAL;
        return -1;
    }
    
    if (__system_property_set(key, value) == 0) {
        return 0;
    } else {
        // __system_property_set doesn't always set errno, force EACCES if ambiguous.
        if (errno == 0) errno = EACCES; 
        return -1;
    }
}

extern "C" int cpp_get_system_property(const char* key, char* value, size_t max_len) {
    if (!key || !value || max_len == 0) {
        errno = EINVAL;
        return -1;
    }
    return __system_property_get(key, value);
}