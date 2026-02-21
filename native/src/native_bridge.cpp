// This file is part of QoS-Android.
// Licensed under the GNU GPL v3 or later.
// Author: [Seclususs](https://github.com/seclususs)

#include "native_bridge.h"
#include "logging.h"

#include <cerrno>
#include <cstdio>
#include <fcntl.h>
#include <linux/input.h>
#include <string>
#include <sys/system_properties.h>
#include <sys/wait.h>
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

extern "C" int cpp_set_refresh_rate(int refresh_rate_mode) {
  // Select the pre-defined argument string based on the mode.
  // Using static literals avoids runtime allocations and ensures
  // address space safety during the critical vfork window.
  const char *val_str = (refresh_rate_mode != 0) ? "1" : "0";

  // Argument vector for:
  //   /system/bin/service call SurfaceFlinger 1035 i32 <value>
  // Explicit casting is used to satisfy execve signature requirements.
  char *const argv[] = {(char *)"/system/bin/service",
                        (char *)"call",
                        (char *)"SurfaceFlinger",
                        (char *)"1035",
                        (char *)"i32",
                        (char *)val_str,
                        nullptr};

  // Use vfork() to avoid page table duplication.
  // The parent process is blocked until the child either calls execve()
  // or terminates via _exit().

  // NOLINTNEXTLINE
  pid_t pid = vfork();

  if (pid < 0) {
    LOGE("SurfaceFlinger: vfork failed (errno: %d)", errno);
    return -1;
  } else if (pid == 0) {
    // CRITICAL:
    // Do not allocate memory, acquire locks, or modify global state here.
    // The child shares the parent's address space until execve().

    // Redirect stdout and stderr to /dev/null to avoid log spam
    // from the service binary.
    int dev_null = open("/dev/null", O_RDWR);
    if (dev_null >= 0) {
      dup2(dev_null, STDOUT_FILENO);
      dup2(dev_null, STDERR_FILENO);
      if (dev_null > STDERR_FILENO) {
        close(dev_null);
      }
    } else {
      close(STDOUT_FILENO);
      close(STDERR_FILENO);
    }

    // Execute the service binary directly without shell involvement
    // to minimize overhead and ensure deterministic behavior.
    execve(argv[0], argv, environ);

    // execve() only returns on failure (e.g. binary not found).
    // Use _exit() instead of exit() to avoid flushing shared stdio buffers.
    _exit(127);
  }

  // Wait synchronously for the child process to complete the transaction.
  // The loop handles EINTR to ensure the wait is not aborted by system signals.
  int status;
  int ret;
  do {
    ret = waitpid(pid, &status, 0);
  } while (ret == -1 && errno == EINTR);

  if (ret == -1) {
    LOGE("SurfaceFlinger: waitpid failed (errno: %d)", errno);
    return -1;
  }

  // Treat exit code 0 as a successful SurfaceFlinger transaction.
  if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
    return 0;
  }

  LOGE("SurfaceFlinger: Transaction failed (code: %d)", WEXITSTATUS(status));
  errno = EPROTO;
  return -1;
}

extern "C" int cpp_touch_monitor_open(const char *path) {
  if (!path) {
    errno = EINVAL;
    return -1;
  }

  // Open the input device in non-blocking mode to integrate with the
  // event loop mechanism (e.g., epoll or Rust reactor).
  // O_CLOEXEC ensures the file descriptor is not leaked to child processes.
  int fd = open(path, O_RDONLY | O_NONBLOCK | O_CLOEXEC);

  if (fd < 0) {
    LOGE("TouchMonitor: Failed to open device %s (errno: %d)", path, errno);
    return -1;
  }

  return fd;
}

extern "C" int cpp_touch_monitor_check(int fd) {
  // Use a stack-allocated buffer to batch-read input events.
  // High-frequency touch devices emit bursts of events (coordinates, pressure,
  // sync). Reading in chunks minimizes system call overhead (context switches)
  // and ensures the thread keeps pace with the hardware interrupt rate.
  const size_t BATCH_SIZE = 64;
  struct input_event ev_batch[BATCH_SIZE];

  int touch_state = -1; // Default: No state change detected in this batch.
  ssize_t bytes_read;

  // Drain the kernel input buffer until empty.
  // If the buffer contains more events than BATCH_SIZE, the loop continues.
  while ((bytes_read = read(fd, ev_batch, sizeof(ev_batch))) > 0) {
    // Calculate the number of complete events received in this chunk.
    size_t count = bytes_read / sizeof(struct input_event);

    // Process the batch in userspace to find the latest touch state.
    for (size_t i = 0; i < count; ++i) {
      if (ev_batch[i].type == EV_KEY) {
        // Monitor both BTN_TOUCH and BTN_TOOL_FINGER.
        // While BTN_TOUCH is the standard for contact, some drivers/protocols
        // rely on BTN_TOOL_FINGER to indicate active finger presence.
        // Checking both ensures compatibility across different kernel versions.
        if (ev_batch[i].code == BTN_TOUCH ||
            ev_batch[i].code == BTN_TOOL_FINGER) {
          touch_state = ev_batch[i].value;
        }
      }
    }
  }

  return touch_state;
}