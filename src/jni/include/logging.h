/**
 * @brief Macro-based logging definitions for Android.
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef LOGGING_H
#define LOGGING_H

#include <android/log.h>

/**
 * @brief The log tag to use for all Android logcat messages.
 */
#define LOG_TAG "AdaptiveDaemon"

/**
 * @brief Compile-time flag to disable INFO and ERROR logs.
 */
#define DISABLE_INFO_ERROR_LOGS

#ifdef DISABLE_INFO_ERROR_LOGS
    /**
     * @brief Logs an informational message. (Disabled)
     */
    #define LOGI(...) do {} while (0)
    /**
     * @brief Logs an error message. (Disabled)
     */
    #define LOGE(...) do {} while (0)
#else
    /**
     * @brief Logs an informational message. (Enabled)
     */
    #define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
    /**
     * @brief Logs an error message. (Enabled)
     */
    #define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#endif

#ifdef DEBUG
    /**
     * @brief Logs a debug message. (Enabled in DEBUG builds)
     */
    #define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#else
    /**
     * @brief Logs a debug message. (Disabled in non-DEBUG builds)
     */
    #define LOGD(...) do {} while (0)
#endif

#endif // LOGGING_H