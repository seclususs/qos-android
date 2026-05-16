#pragma once

#include <android/log.h>

#ifndef LOG_TAG
#define LOG_TAG "QoS"
#endif

#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

#if defined(NDEBUG) && !defined(ENABLE_VERBOSE_LOGS)
#define LOGI(...)                                                                                                      \
    do                                                                                                                 \
    {                                                                                                                  \
    } while (0)
#define LOGD(...)                                                                                                      \
    do                                                                                                                 \
    {                                                                                                                  \
    } while (0)
#else
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#endif