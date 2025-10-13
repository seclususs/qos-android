/*
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
#ifndef LOGGER_H
#define LOGGER_H

#include <android/log.h>

#define LOG_TAG "AdaptiveDaemon"

#ifdef DISABLE_LOGGING
    #define LOGI(...) do {} while (0)
    #define LOGE(...) do {} while (0)
    #define LOGD(...) do {} while (0)
#else
    #ifdef DEBUG
        #define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
    #else
        #define LOGD(...) do {} while (0)
    #endif

    #define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
    #define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#endif // DISABLE_LOGGING

#endif // LOGGER_H