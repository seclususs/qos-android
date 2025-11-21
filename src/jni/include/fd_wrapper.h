/**
 * @brief RAII wrapper for managing POSIX file descriptor ownership.
 * 
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef FD_WRAPPER_H
#define FD_WRAPPER_H

#include "logging.h"
#include <unistd.h>
#include <fcntl.h>
#include <cerrno>
#include <cstring>

/**
 * @class FdWrapper
 * @brief Encapsulates a file descriptor with strict ownership rules.
 */
class FdWrapper {
    
public:
    /**
     * @brief Wraps an existing raw file descriptor.
     *
     * @param fd The raw descriptor to manage. Defaults to -1 (invalid).
     * The FdWrapper takes ownership of this fd.
     */
    explicit FdWrapper(int fd = -1) : fd_(fd) {}

    /**
     * @brief Opens a file and manages the resulting descriptor.
     *
     * @param path Filesystem path to open.
     * @param flags Open flags.
     * @post Check isValid() to verify if the file was opened successfully.
     */
    FdWrapper(const char* path, int flags) : fd_(open(path, flags)) {
        if (!isValid()) {
            LOGD("FdWrapper: Failed to open %s (errno: %d - %s)", path, errno, strerror(errno));
        }
    }

    /**
     * @brief Destructor.
     *
     * Automatically closes the managed file descriptor if it is valid.
     */
    ~FdWrapper() {
        if (isValid()) {
            close(fd_);
        }
    }

    /**
     * @brief Deleted Copy Constructor.
     *
     * Prevent copying to ensure unique ownership of the descriptor.
     */
    FdWrapper(const FdWrapper&) = delete;

    /**
     * @brief Deleted Copy Assignment.
     *
     * Prevent copying to ensure unique ownership of the descriptor.
     */
    FdWrapper& operator=(const FdWrapper&) = delete;

    /**
     * @brief Move Constructor.
     *
     * Transfers ownership from another FdWrapper.
     * @param other The source object. It will be invalidated (fd set to -1).
     */
    FdWrapper(FdWrapper&& other) noexcept : fd_(other.fd_) { other.fd_ = -1; }

    /**
     * @brief Move Assignment.
     *
     * Closes the current descriptor (if valid) and takes ownership from the source.
     * @param other The source object. It will be invalidated.
     * @return Reference to this object.
     */
    FdWrapper& operator=(FdWrapper&& other) noexcept {
        if (this != &other) {
            if (isValid()) close(fd_);
            fd_ = other.fd_;
            other.fd_ = -1;
        }
        return *this;
    }

    /**
     * @brief Returns the raw file descriptor without transferring ownership.
     *
     * @warning Do not manually close the returned fd.
     * @return The raw file descriptor.
     */
    int get() const { return fd_; }

    /**
     * @brief Checks if the managed descriptor is valid.
     *
     * @return true if fd >= 0; false otherwise.
     */
    bool isValid() const { return fd_ >= 0; }

    /**
     * @brief Writes data to the file descriptor.
     *
     * @param buf Buffer containing data to write.
     * @param count Number of bytes to write.
     * @return Number of bytes written, or -1 on error.
     */
    ssize_t write(const void* buf, size_t count) const {
        return ::write(fd_, buf, count);
    }

    /**
     * @brief Reads data from the file descriptor.
     *
     * @param buf Buffer to store read data.
     * @param count Maximum bytes to read.
     * @return Number of bytes read, 0 on EOF, or -1 on error.
     */
    ssize_t read(void* buf, size_t count) const {
        return ::read(fd_, buf, count);
    }

private:
    /** @brief The underlying raw file descriptor. */
    int fd_;
};

#endif // FD_WRAPPER_H