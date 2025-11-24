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
#include <sys/epoll.h>
#include <linux/input.h>
#include <fcntl.h>
#include <stdlib.h>
#include <stdio.h>

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
    static int mem_fd = -1;
    const char* kPsiMemory = "/proc/pressure/memory";
    
    if (mem_fd < 0) {
        mem_fd = open(kPsiMemory, O_RDONLY | O_CLOEXEC);
        if (mem_fd < 0) {
            LOGE("cpp_get_memory_pressure: Failed to open %s", kPsiMemory);
            return -1.0;
        }
    }

    char buffer[128];
    ssize_t bytes_read = pread(mem_fd, buffer, sizeof(buffer) - 1, 0);

    if (bytes_read > 0) {
        buffer[bytes_read] = '\0';
        char* avg10_ptr = strstr(buffer, "avg10=");
        if (avg10_ptr) {
            char* end_ptr;
            double val = strtod(avg10_ptr + 6, &end_ptr);
            if (avg10_ptr + 6 != end_ptr) {
                return val;
            }
        }
    } else {
        close(mem_fd);
        mem_fd = -1; 
    }

    return -1.0;
}

extern "C" double cpp_get_io_pressure(void) {
    static int io_fd = -1;
    const char* kPsiIo = "/proc/pressure/io";

    if (io_fd < 0) {
        io_fd = open(kPsiIo, O_RDONLY | O_CLOEXEC);
        if (io_fd < 0) {
            LOGE("cpp_get_io_pressure: Failed to open %s", kPsiIo);
            return -1.0;
        }
    }

    char buffer[128];
    ssize_t bytes_read = pread(io_fd, buffer, sizeof(buffer) - 1, 0);

    if (bytes_read > 0) {
        buffer[bytes_read] = '\0';
        char* avg10_ptr = strstr(buffer, "avg10=");
        if (avg10_ptr) {
            char* end_ptr;
            double val = strtod(avg10_ptr + 6, &end_ptr);
            if (avg10_ptr + 6 != end_ptr) {
                return val;
            }
        }
    } else {
        close(io_fd);
        io_fd = -1;
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

extern "C" int cpp_register_psi_trigger(const char* path, int threshold_us, int window_us) {
    if (!path) return -1;

    int fd = open(path, O_RDWR | O_CLOEXEC);
    if (fd < 0) {
        LOGE("Failed to open PSI file for trigger: %s", path);
        return -1;
    }

    char trigger_cmd[128];
    snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d", threshold_us, window_us);
    
    if (write(fd, trigger_cmd, strlen(trigger_cmd) + 1) < 0) {
        LOGE("Failed to write trigger command: %s (errno: %d)", trigger_cmd, errno);
        close(fd);
        return -1;
    }

    int epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    if (epoll_fd < 0) {
        LOGE("Failed to create epoll instance");
        close(fd);
        return -1;
    }

    struct epoll_event ev;
    ev.events = EPOLLPRI;
    ev.data.fd = fd;

    if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, fd, &ev) < 0) {
        LOGE("Failed to add PSI fd to epoll");
        close(epoll_fd);
        close(fd);
        return -1;
    }
    
    return epoll_fd;
}

extern "C" int cpp_wait_for_psi_event(int epoll_fd, int timeout_ms) {
    struct epoll_event events[1];
    int nfds = epoll_wait(epoll_fd, events, 1, timeout_ms);
    
    if (nfds > 0) return 1;
    if (nfds == 0) return 0;
    if (errno == EINTR) return 0;
    
    LOGE("epoll_wait failed (errno: %d)", errno);
    return -1;
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