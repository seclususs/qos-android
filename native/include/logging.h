/**
 * @file logging.h
 * @brief Macros for Android system logging.
 *
 * Provides a standardized interface for writing to the Android Logcat system.
 * It handles log tagging and conditional compilation for debug builds.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#ifndef LOGGING_H
#define LOGGING_H

#include <android/log.h>

/**
 * @brief The log tag used to identify this process in Logcat.
 */
#define LOG_TAG "QoS"

/**
 * @brief Logs a message at the ERROR priority.
 *
 * This macro is always active, regardless of build configuration, to ensure
 * critical runtime failures are recorded for diagnostics.
 *
 * @param ... Format string and arguments (printf-style).
 */
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

#if defined(NDEBUG) && !defined(ENABLE_VERBOSE_LOGS)

/**
 * @brief Logs a message at the INFO priority.
 * @note Disabled in Release builds to reduce overhead and log noise.
 */
#define LOGI(...)                                                              \
  do {                                                                         \
  } while (0)

/**
 * @brief Logs a message at the DEBUG priority.
 * @note Disabled in Release builds.
 */
#define LOGD(...)                                                              \
  do {                                                                         \
  } while (0)

#else

/**
 * @brief Logs a message at the INFO priority.
 * @param ... Format string and arguments.
 */
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)

/**
 * @brief Logs a message at the DEBUG priority.
 * @param ... Format string and arguments.
 */
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)

#endif

#endif // LOGGING_H