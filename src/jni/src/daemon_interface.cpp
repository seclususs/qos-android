/**
 * @author Seclususs
 * https://github.com/seclususs
 */

#include "daemon_interface.h"
#include "system_utils.h"
#include "fd_wrapper.h"
#include "logging.h"

#include <string>
#include <fstream>
#include <unistd.h>
#include <cerrno>
#include <cstring>
#include <sys/poll.h>
#include <linux/input.h>

extern "C" bool cpp_apply_tweak(const char* path, const char* value) {
    if (!path || !value) return false;
    return SystemUtils::applyTweak(path, value);
}

extern "C" void cpp_set_system_prop(const char* key, const char* value) {
    if (!key || !value) return;
    SystemUtils::setSystemProp(key, value);
}

extern "C" bool cpp_set_android_setting(const char* property, const char* value) {
    if (!property || !value) return false;
    return SystemUtils::setAndroidSetting(property, value);
}

extern "C" double cpp_get_memory_pressure(void) {
    const char* kPsiMemory = "/proc/pressure/memory";
    std::ifstream file(kPsiMemory);

    if (!file) {
        LOGE("cpp_get_memory_pressure: Failed to open %s", kPsiMemory);
        return -1.0;
    }

    std::string line;
    while (std::getline(file, line)) {
        if (line.rfind("some", 0) == 0) {
            double avg10 = 0.0;
            size_t pos = line.find("avg10=");
            if (pos != std::string::npos) {
                try {
                    avg10 = std::stod(line.substr(pos + 6));
                    return avg10;
                } catch (...) {
                    LOGE("cpp_get_memory_pressure: Failed to parse avg10");
                }
            }
        }
    }

    return -1.0;
}

extern "C" double cpp_get_io_pressure(void) {
    const char* kPsiIo = "/proc/pressure/io";
    std::ifstream file(kPsiIo);

    if (!file) {
        LOGE("cpp_get_io_pressure: Failed to open %s", kPsiIo);
        return -1.0;
    }

    std::string line;
    while (std::getline(file, line)) {
        if (line.rfind("some", 0) == 0) {
            double avg10 = 0.0;
            size_t pos = line.find("avg10=");
            if (pos != std::string::npos) {
                try {
                    avg10 = std::stod(line.substr(pos + 6));
                    return avg10;
                } catch (...) {
                    LOGE("cpp_get_io_pressure: Failed to parse avg10");
                }
            }
        }
    }

    return -1.0;
}

extern "C" void cpp_close_fd(int fd) {
    if (fd >= 0) {
        close(fd);
    }
}

extern "C" int cpp_open_touch_device(const char* path) {
    if (!path) return -1;

    FdWrapper fd(path, O_RDONLY | O_NONBLOCK);
    if (!fd.isValid()) {
        LOGE("cpp_open_touch_device: Failed to open %s (errno: %d - %s)",
             path, errno, strerror(errno));
        return -1;
    }

    int raw_fd = fd.get();
    new FdWrapper(std::move(fd));
    return raw_fd;
}

extern "C" void cpp_read_touch_events(int fd) {
    if (fd < 0) return;

    char buffer[sizeof(struct input_event) * 64];
    while (read(fd, buffer, sizeof(buffer)) > 0);
}

extern "C" int cpp_poll_fd(int fd, int timeout_ms) {
    if (fd < 0) return -1;

    struct pollfd pfd;
    pfd.fd = fd;
    pfd.events = POLLIN;

    int result = poll(&pfd, 1, timeout_ms);
    if (result > 0) {
        if (pfd.revents & POLLIN) {
            return 1;
        } else {
            return -1;
        }
    } else if (result == 0) {
        return 0;
    } else {
        if (errno == EINTR) return 0;
        LOGE("cpp_poll_fd: poll() error (errno: %d - %s)", errno, strerror(errno));
        return -1;
    }
}

extern "C" void cpp_log_info(const char* message) {
    if (message) LOGI("%s", message);
}

extern "C" void cpp_log_debug(const char* message) {
    if (message) LOGD("%s", message);
}

extern "C" void cpp_log_error(const char* message) {
    if (message) LOGE("%s", message);
}