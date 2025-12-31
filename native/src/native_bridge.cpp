// This file is part of QoS-Android.
// Licensed under the GNU GPL v3 or later.
// Author: [Seclususs](https://github.com/seclususs)

#include "native_bridge.h"
#include "logging.h"

#include <cerrno>
#include <cstdio>
#include <fcntl.h>
#include <sys/system_properties.h>
#include <unistd.h>

/**
 * RAII wrapper for file descriptors to ensure resources are released
 * if initialization logic fails early.
 */
class ScopedFd {
public:
  explicit ScopedFd(int fd) : fd_(fd) {}

  ~ScopedFd() {
    if (fd_ >= 0) {
      close(fd_);
    }
  }

  ScopedFd(const ScopedFd &) = delete;
  ScopedFd &operator=(const ScopedFd &) = delete;

  bool isValid() const { return fd_ >= 0; }
  int get() const { return fd_; }

  int release() {
    int temp = fd_;
    fd_ = -1;
    return temp;
  }

private:
  int fd_;
};

extern "C" void cpp_notify_service_death(const char *context) {
  const char *reason = context ? context : "Unknown Reason";
  LOGE("!!! SERVICE CRITICAL: %s !!!", reason);
  LOGE("Requesting graceful shutdown from Logic layer...");
}

extern "C" int cpp_register_psi_trigger(const char *path, int threshold_us,
                                        int window_us) {
  if (!path) {
    errno = EINVAL;
    return -1;
  }

  // Open PSI file for writing to register the trigger.
  // O_NONBLOCK is critical: The core library uses epoll, and the FD must be
  // non-blocking. O_CLOEXEC ensures the FD is not leaked to child processes.
  int raw_fd = open(path, O_RDWR | O_CLOEXEC | O_NONBLOCK);
  ScopedFd fd(raw_fd);

  if (!fd.isValid()) {
    LOGE("Failed to open PSI file: %s (errno: %d)", path, errno);
    return -1;
  }

  // Construct the trigger command string: "some <threshold> <window>"
  char trigger_cmd[128];

  // snprintf protects against buffer overflows if the inputs are unexpectedly
  // large.
  int len = snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d",
                     threshold_us, window_us);

  if (len < 0 || (size_t)len >= sizeof(trigger_cmd)) {
    errno = EOVERFLOW;
    return -1;
  }

  // Write the trigger command. We include the null terminator in the first
  // attempt as some kernel parsers are sensitive to buffer boundaries.
  if (write(fd.get(), trigger_cmd, len + 1) < 0) {
    LOGE("Failed to write trigger: %s (errno: %d). Retrying with newline...",
         trigger_cmd, errno);

    // Fallback: Retry with an explicit newline, which is required by some
    // stricter PSI implementations or specific kernel versions.
    int len_nl = snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d\n",
                          threshold_us, window_us);
    if (write(fd.get(), trigger_cmd, len_nl) < 0) {
      LOGE("Retry failed. Fatal trigger write error: %s (errno: %d)",
           trigger_cmd, errno);
      return -1;
    }
  }

  LOGD("Successfully registered PSI trigger: %s on fd %d", trigger_cmd,
       fd.get());

  // Transfer ownership of the valid FD to the caller (Core Library).
  return fd.release();
}

extern "C" int cpp_set_system_property(const char *key, const char *value) {
  if (!key || !value) {
    errno = EINVAL;
    return -1;
  }

  if (__system_property_set(key, value) == 0) {
    return 0;
  } else {
    // Ensure errno is set if the system call fails without setting it.
    if (errno == 0)
      errno = EACCES;
    return -1;
  }
}

extern "C" int cpp_get_system_property(const char *key, char *value,
                                       size_t max_len) {
  if (!key || !value || max_len == 0) {
    errno = EINVAL;
    return -1;
  }
  return __system_property_get(key, value);
}