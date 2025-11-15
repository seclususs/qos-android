/**
 * @brief A C++ RAII (Resource Acquisition Is Initialization) wrapper for file descriptors.
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
 * @brief A RAII wrapper for managing file descriptors.
 */
class FdWrapper {
public:
    /**
     * @brief Constructs an FdWrapper, optionally taking ownership of an existing fd.
     *
     * @param fd The file descriptor to manage. Defaults to -1 (invalid).
     */
    explicit FdWrapper(int fd = -1) : fd_(fd) {}

    /**
     * @brief Constructs an FdWrapper by opening a file.
     *
     * @param path The file path to open.
     * @param flags The flags to use when opening (e.g., O_RDONLY).
     */
    FdWrapper(const char* path, int flags) : fd_(open(path, flags)) {
        if (!isValid()) {
            LOGD("FdWrapper: Failed to open %s (errno: %d - %s)", path, errno, strerror(errno));
        }
    }

    /**
     * @brief Destructor. Closes the managed file descriptor if it is valid.
     */
    ~FdWrapper() {
        if (isValid()) {
            close(fd_);
        }
    }

    /**
     * @brief Deleted copy constructor.
     */
    FdWrapper(const FdWrapper&) = delete;

    /**
     * @brief Deleted copy assignment operator.
     */
    FdWrapper& operator=(const FdWrapper&) = delete;

    /**
     * @brief Move constructor.
     *
     * @param other The FdWrapper to move from, which will be invalidated.
     */
    FdWrapper(FdWrapper&& other) noexcept : fd_(other.fd_) { other.fd_ = -1; }

    /**
     * @brief Move assignment operator.
     *
     * @param other The FdWrapper to move from, which will be invalidated.
     * @return A reference to this object.
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
     * @brief Gets the raw file descriptor value.
     *
     * @return The managed file descriptor.
     */
    int get() const { return fd_; }

    /**
     * @brief Checks if the managed file descriptor is valid.
     *
     * @return true if fd_ is >= 0, false otherwise.
     */
    bool isValid() const { return fd_ >= 0; }

    /**
     * @brief Writes data to the file descriptor.
     *
     * @param buf Pointer to the data buffer.
     * @param count Number of bytes to write.
     * @return The number of bytes written, or -1 on error.
     */
    ssize_t write(const void* buf, size_t count) const {
        return ::write(fd_, buf, count);
    }

    /**
     * @brief Reads data from the file descriptor.
     *
     * @param buf Pointer to the destination buffer.
     * @param count Number of bytes to read.
     * @return The number of bytes read, 0 on EOF, or -1 on error.
     */
    ssize_t read(void* buf, size_t count) const {
        return ::read(fd_, buf, count);
    }
private:
    /**
     * @brief The managed file descriptor.
     */
    int fd_;
};

#endif // FD_WRAPPER_H