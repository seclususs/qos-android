/**
 * @file logging.h
 * @brief Defines logging macros for the application.
 *
 * Uses the Android logging library (`android/log.h`) to provide
 * standard logging macros (LOGI, LOGE, LOGD). Can be configured
 * to disable certain log levels at compile time.
 */

#ifndef LOGGING_H
#define LOGGING_H

#include <android/log.h>

/** @brief The log tag to use for all log messages. */
#define LOG_TAG "AdaptiveDaemon"

/**
 * @def DISABLE_INFO_ERROR_LOGS
 * @brief If defined, disables LOGI and LOGE macros.
 *
 * This is a release build optimization to reduce log spam.
 * LOGD is separately controlled by the DEBUG macro.
 */
#define DISABLE_INFO_ERROR_LOGS

#ifdef DISABLE_INFO_ERROR_LOGS
    /** @brief Log an info message (disabled in this build). */
    #define LOGI(...) do {} while (0)
    /** @brief Log an error message (disabled in this build). */
    #define LOGE(...) do {} while (0)
#else
    /**
     * @brief Log an info message.
     * @param ... The format string and arguments, like printf.
     */
    #define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
    /**
     * @brief Log an error message.
     * @param ... The format string and arguments, like printf.
     */
    #define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#endif

#ifdef DEBUG
    /**
     * @brief Log a debug message (only enabled in DEBUG builds).
     * @param ... The format string and arguments, like printf.
     */
    #define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#else
    /** @brief Log a debug message (disabled in this build). */
    #define LOGD(...) do {} while (0)
#endif

#endif // LOGGING_H