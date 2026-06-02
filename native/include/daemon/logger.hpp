/**
 * @file logger.hpp
 *
 * @brief Android logging macros for the daemon.
 */

#pragma once

#include <android/log.h>

#ifndef LOG_TAG
#define LOG_TAG "QoS"
#endif

/** @brief Logs an error message to logcat. */
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

/** @brief Logs a warning message to logcat. */
#define LOGW(...) __android_log_print(ANDROID_LOG_WARN, LOG_TAG, __VA_ARGS__)

/** @brief Logs an informational message to logcat. */
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)

#if defined(NDEBUG) && !defined(ENABLE_VERBOSE_LOGS)
/** @brief Logs a debug message. Compiled out in release builds unless ENABLE_VERBOSE_LOGS is defined. */
#define LOGD(...)                                                                                                      \
    do                                                                                                                 \
    {                                                                                                                  \
    } while (0)
#else
/** @brief Logs a debug message to logcat. */
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#endif