/**
 * @author Seclususs
 * https://github.com/seclususs
 */

#include "daemon_interface.h"
#include "logging.h"

#include <unistd.h>
#include <fcntl.h>
#include <cstdio>
#include <cstring>
#include <cerrno>
#include <cstdlib>

extern "C" void cpp_log_info(const char* message) {
    if (message) LOGI("%s", message);
}

extern "C" void cpp_log_debug(const char* message) {
    if (message) LOGD("%s", message);
}

extern "C" void cpp_log_error(const char* message) {
    if (message) LOGE("%s", message);
}

extern "C" void cpp_notify_service_death(const char* context) {
    const char* reason = context ? context : "Unknown Reason";
    
    LOGE("!!! FATAL: RUST SERVICE DIED !!!");
    LOGE("Reason: %s", reason);
    LOGE("Triggering emergency exit to force service restart...");
    std::exit(EXIT_FAILURE);
}

extern "C" int cpp_open_touch_device(const char* path) {
    if (!path) return -1;

    return open(path, O_RDONLY | O_NONBLOCK | O_CLOEXEC);
}

extern "C" void cpp_read_touch_events(int fd) {
    if (fd < 0) return;

    char buffer[1024]; 
    while (read(fd, buffer, sizeof(buffer)) > 0);
}

extern "C" int cpp_register_psi_trigger(const char* path, int threshold_us, int window_us) {
    if (!path) return -1;

    int fd = open(path, O_RDWR | O_CLOEXEC | O_NONBLOCK);
    if (fd < 0) {
        LOGE("Failed to open PSI file for trigger: %s (errno: %d)", path, errno);
        return -1;
    }

    char trigger_cmd[128];
    snprintf(trigger_cmd, sizeof(trigger_cmd), "some %d %d", threshold_us, window_us);
    
    if (write(fd, trigger_cmd, strlen(trigger_cmd) + 1) < 0) {
        LOGE("Failed to write trigger command: %s (errno: %d)", trigger_cmd, errno);
        close(fd);
        return -1;
    }
    
    return fd;
}