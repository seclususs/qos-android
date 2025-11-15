#include "daemon_interface.h"
#include "system_utils.h"
#include "fd_wrapper.h"
#include "logging.h"

#include <string>
#include <fstream>
#include <sstream>
#include <cstdio>
#include <sys/poll.h>
#include <sys/socket.h>
#include <linux/netlink.h>
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

extern "C" int cpp_get_free_ram_percentage(void) {
    const char* kMemInfo = "/proc/meminfo";
    std::ifstream file(kMemInfo);
    if (!file) {
        LOGE("cpp_get_free_ram_percentage: Failed to open %s", kMemInfo);
        return -1;
    }
    long memTotal = -1, memAvailable = -1;
    std::string line;
    line.reserve(128);
    while (std::getline(file, line)) {
        if (line.rfind("MemTotal:", 0) == 0) {
            std::sscanf(line.c_str(), "MemTotal: %ld kB", &memTotal);
        } else if (line.rfind("MemAvailable:", 0) == 0) {
            std::sscanf(line.c_str(), "MemAvailable: %ld kB", &memAvailable);
        }
        if (memTotal != -1 && memAvailable != -1) break;
    }
    if (memTotal > 0 && memAvailable >= 0) {
        return static_cast<int>((static_cast<double>(memAvailable) / memTotal) * 100.0);
    }
    LOGD("cpp_get_free_ram_percentage: Incomplete data");
    return -1;
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
        LOGE("cpp_open_touch_device: Failed to open %s (errno: %d - %s)", path, errno, strerror(errno));
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

extern "C" int cpp_create_netlink_socket(void) {
    struct sockaddr_nl sa;
    std::memset(&sa, 0, sizeof(sa));
    sa.nl_family = AF_NETLINK;
    sa.nl_groups = 1;
    sa.nl_pid = getpid();
    int fd = socket(AF_NETLINK, SOCK_DGRAM | SOCK_CLOEXEC, NETLINK_KOBJECT_UEVENT);
    if (fd < 0) {
        LOGE("cpp_create_netlink_socket: socket() failed (errno: %d - %s)", errno, strerror(errno));
        return -1;
    }
    if (bind(fd, (struct sockaddr*)&sa, sizeof(sa)) < 0) {
        LOGE("cpp_create_netlink_socket: bind() failed (errno: %d - %s)", errno, strerror(errno));
        close(fd);
        return -1;
    }
    LOGI("Netlink socket created successfully.");
    return fd;
}

extern "C" int cpp_read_netlink_event(int fd, char* buffer, int buffer_size) {
    if (fd < 0 || !buffer || buffer_size <= 0) return -1;
    ssize_t len = recv(fd, buffer, buffer_size - 1, 0);
    if (len < 0) {
        if (errno == EINTR) return 0;
        LOGE("cpp_read_netlink_event: recv() failed (errno: %d - %s)", errno, strerror(errno));
        return -1;
    }
    buffer[len] = '\0';
    return static_cast<int>(len);
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