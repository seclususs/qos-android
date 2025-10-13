/*
 * Copyright (C) 2025 Seclususs
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#include "hardware-interface.h"
#include "logger.h"
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <sys/select.h>
#include <sys/wait.h>

int write_to_file(const char* path, const char* value) {
    int fd = open(path, O_WRONLY | O_TRUNC);
    if (fd < 0) {
        LOGE("Failed to open '%s' for writing: %s", path, strerror(errno));
        return -1;
    }

    ssize_t written = write(fd, value, strlen(value));
    close(fd);

    if (written < 0) {
        LOGE("Failed to write to file '%s': %s", path, strerror(errno));
        return -1;
    }
    return 0;
}

int read_mem_info(long* mem_total, long* mem_available) {
    FILE* file = fopen("/proc/meminfo", "r");
    if (!file) {
        LOGE("Failed to open /proc/meminfo: %s", strerror(errno));
        return -1;
    }
    
    char line[128];
    int found_total = 0;
    int found_available = 0;

    while (fgets(line, sizeof(line), file)) {
        if (sscanf(line, "MemTotal: %ld kB", mem_total) == 1) {
            found_total = 1;
        } else if (sscanf(line, "MemAvailable: %ld kB", mem_available) == 1) {
            found_available = 1;
        }
        if (found_total && found_available) {
            break;
        }
    }

    fclose(file);
    return (found_total && found_available) ? 0 : -1;
}


int wait_for_input(const char* device_path, int timeout_ms) {
    int fd = open(device_path, O_RDONLY | O_NONBLOCK);
    if (fd < 0) {
        LOGE("Failed to open input device '%s': %s", device_path, strerror(errno));
        return -1;
    }

    fd_set read_fds;
    FD_ZERO(&read_fds);
    FD_SET(fd, &read_fds);

    struct timeval tv;
    struct timeval* timeout_ptr = NULL;

    if (timeout_ms >= 0) {
        tv.tv_sec = timeout_ms / 1000;
        tv.tv_usec = (timeout_ms % 1000) * 1000;
        timeout_ptr = &tv;
    }

    int result = select(fd + 1, &read_fds, NULL, NULL, timeout_ptr);
    
    if (result > 0) {
        // Drain any pending events to prevent re-triggering
        char buffer[256];
        while (read(fd, buffer, sizeof(buffer)) > 0);
    }

    close(fd);

    if (result > 0) return 1; // Input available
    if (result == 0) return 0; // Timeout
    return -1; // Error
}

int execute_command(const char* cmd, char* result_buffer, size_t buffer_size) {
    
    if (!result_buffer || buffer_size == 0) {
        return -1;
    }
    result_buffer[0] = '\0';

    // Redirect stderr to stdout to capture all output
    char cmd_with_stderr[512];
    snprintf(cmd_with_stderr, sizeof(cmd_with_stderr), "%s 2>&1", cmd);

    FILE* pipe = popen(cmd_with_stderr, "r");
    if (!pipe) {
        LOGE("popen() failed for command '%s': %s", cmd, strerror(errno));
        return -1;
    }

    char line[256];
    size_t current_len = 0;
    while (fgets(line, sizeof(line), pipe) != NULL) {
        size_t line_len = strlen(line);
        if (current_len + line_len < buffer_size) {
            strcat(result_buffer, line);
            current_len += line_len;
        }
    }

    int status = pclose(pipe);
    return WIFEXITED(status) ? WEXITSTATUS(status) : -1;
}