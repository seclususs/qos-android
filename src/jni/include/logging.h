/**
 * @brief Android Logcat integration macros.
 *
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef LOGGING_H
#define LOGGING_H

#include <android/log.h>

/**
 * @brief The tag used to identify logs in `adb logcat`.
 */
#define LOG_TAG "QoS"

/**
 * @brief ERROR log.
 *
 * Writes to ANDROID_LOG_ERROR. Critical errors are always preserved for
 * production diagnostics and cannot be disabled.
 */
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

#if defined(NDEBUG) && !defined(ENABLE_VERBOSE_LOGS)

    /** @brief INFO log (Disabled in Release). Compiles to no-op. */
    #define LOGI(...) do {} while (0)

    /** @brief DEBUG log (Disabled in Release). Compiles to no-op. */
    #define LOGD(...) do {} while (0)

#else

    /** @brief INFO log. Writes to ANDROID_LOG_INFO. */
    #define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)

    /** @brief DEBUG log. Writes to ANDROID_LOG_DEBUG. */
    #define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)

#endif

#endif // LOGGING_H