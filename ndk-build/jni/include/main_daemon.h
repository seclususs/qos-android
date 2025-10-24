/**
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

/**
 * @file main_daemon.h
 * @brief Global definitions for the main daemon.
 *
 * Declares global variables and constants shared across the daemon,
 * particularly for signal handling and application identity.
 */

#ifndef MAIN_DAEMON_H
#define MAIN_DAEMON_H

#include <signal.h>

/**
 * @brief Global flag to request daemon shutdown.
 *
 * This flag is set by the signal handler when a termination signal
 * (SIGINT, SIGTERM) is received. Main loops should check this
 * flag to exit gracefully.
 */
extern volatile sig_atomic_t g_shutdown_requested;

/** @brief The public name of the application, used in logs. */
extern const char* const TWEAK_VALUES_APP_NAME;

#endif // MAIN_DAEMON_H