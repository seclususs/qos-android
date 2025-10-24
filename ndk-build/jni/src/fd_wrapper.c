/**
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

/**
 * @file fd_wrapper.c
 * @brief Implementation of the file descriptor wrapper.
 */

#include "include/fd_wrapper.h"
#include "include/logging.h"
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <string.h>
#include <stdio.h>

/**
 * @brief Initializes a wrapper with an existing file descriptor.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 * @param fd The file descriptor to wrap.
 */
void fdWrapper_init(FdWrapper* wrapper, int fd) {
    wrapper->fd = fd;
}

/**
 * @brief Initializes a wrapper by opening a file path.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 * @param path The file path to open.
 * @param flags The flags to pass to `open()` (e.g., O_RDONLY).
 * @return true if the file was opened successfully, false otherwise.
 */
bool fdWrapper_init_path(FdWrapper* wrapper, const char* path, int flags) {
    wrapper->fd = open(path, flags);
    if (wrapper->fd < 0) {
        LOGD("FdWrapper: Failed to open %s (errno: %d - %s)",
             path, errno, strerror(errno));
        return false;
    }
    return true;
}

/**
 * @brief Closes the wrapped file descriptor if it is valid.
 *
 * Sets the internal file descriptor to -1 after closing.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 */
void fdWrapper_destroy(FdWrapper* wrapper) {
    if (wrapper && wrapper->fd >= 0) {
        close(wrapper->fd);
        wrapper->fd = -1;
    }
}

/**
 * @brief Gets the raw file descriptor value.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 * @return The file descriptor, or -1 if invalid.
 */
int fdWrapper_get(const FdWrapper* wrapper) {
    return wrapper->fd;
}

/**
 * @brief Checks if the wrapped file descriptor is valid.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 * @return true if fd is >= 0, false otherwise.
 */
bool fdWrapper_isValid(const FdWrapper* wrapper) {
    return wrapper->fd >= 0;
}

/**
 * @brief Writes data to the file descriptor.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 * @param buf A pointer to the data buffer to write.
 * @param count The number of bytes to write.
 * @return The number of bytes written, or -1 on error.
 */
ssize_t fdWrapper_write(const FdWrapper* wrapper, const void* buf, size_t count) {
    return write(wrapper->fd, buf, count);
}

/**
 * @brief Reads data from the file descriptor.
 *
 * @param wrapper A pointer to the FdWrapper instance.
 * @param buf A pointer to the buffer to store read data.
 * @param count The maximum number of bytes to read.
 * @return The number of bytes read, 0 on EOF, or -1 on error.
 */
ssize_t fdWrapper_read(const FdWrapper* wrapper, void* buf, size_t count) {
    return read(wrapper->fd, buf, count);
}