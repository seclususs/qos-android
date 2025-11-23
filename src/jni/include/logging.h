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
 * @brief Feature flag to suppress INFO and ERROR logs.
 *
 * Define this to compile out standard logging calls.
 */
#define DISABLE_INFO_ERROR_LOGS

#ifdef DISABLE_INFO_ERROR_LOGS
    /** @brief INFO log (Disabled). */
    #define LOGI(...) do {} while (0)
    /** @brief ERROR log (Disabled). */
    #define LOGE(...) do {} while (0)
#else
    /** @brief INFO log (Enabled). Writes to ANDROID_LOG_INFO. */
    #define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
    /** @brief ERROR log (Enabled). Writes to ANDROID_LOG_ERROR. */
    #define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#endif

#ifdef DEBUG
    /** @brief DEBUG log (Enabled). Writes to ANDROID_LOG_DEBUG only if DEBUG is defined. */
    #define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#else
    /** @brief DEBUG log (Disabled). Stripped from the binary. */
    #define LOGD(...) do {} while (0)
#endif

#endif // LOGGING_H