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
 * @file system_utils.c
 * @brief Implementation of system utility functions.
 */

#include "include/system_utils.h"
#include "include/fd_wrapper.h"
#include "include/logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/system_properties.h>
#include <sys/wait.h>
#include <sys/types.h>

/**
 * @brief Default buffer size for capturing output from shell commands.
 */
const size_t REFRESH_RATE_CONFIG_COMMAND_OUTPUT_BUFFER_SIZE = 128;

/**
 * @brief Writes a string value to a specified file path.
 *
 * Attempts a direct `write()` first for efficiency, falling back
 * to `fprintf()` which may work better for some pseudo-files.
 *
 * @param path The full filesystem path to write to.
 * @param value The string value to write.
 * @return true on success, false on failure.
 */
bool systemUtils_applyTweak(const char* path, const char* value) {
    FdWrapper fd;
    size_t value_len = strlen(value);

    /* Try fast path first */
    if (fdWrapper_init_path(&fd, path, O_WRONLY | O_TRUNC)) {
        ssize_t written = fdWrapper_write(&fd, value, value_len);
        fdWrapper_destroy(&fd);
        if (written == (ssize_t)value_len) {
            return true;
        }
        /* Log partial write, but fall through to FILE* method */
        LOGD("Partial write to %s: %zd/%zu bytes", path, written, value_len);
    }

    /* Fallback to FILE* method */
    FILE* outfile = fopen(path, "w");
    if (!outfile) {
        LOGE("Failed to open for writing: %s (errno: %d - %s)",
             path, errno, strerror(errno));
        return false;
    }

    int result = fprintf(outfile, "%s", value);
    if (result < 0) {
        LOGE("Failed to write '%s' to: %s", value, path);
        fclose(outfile);
        return false;
    }

    fclose(outfile);
    return true;
}

/**
 * @brief Sets an Android system property.
 *
 * @param key The name of the property to set.
 * @param value The value to set the property to.
 */
void systemUtils_setSystemProp(const char* key, const char* value) {
    if (__system_property_set(key, value) < 0) {
        LOGE("Failed to set system property: %s (errno: %d - %s)",
             key, errno, strerror(errno));
    }
}

/**
 * @brief Sets an Android system setting via the `settings` shell command.
 *
 * Forks and executes `/system/bin/settings` to modify the system
 * settings database. Captures and logs stdout/stderr from the command.
 *
 * @param property The name of the system setting.
 * @param value The value to set the setting to.
 * @return true on success (command exit code 0), false otherwise.
 */
bool systemUtils_setAndroidSetting(const char* property, const char* value) {
    char* argv[6];
    bool strdup_failed = false;

    /* Prepare arguments for execv */
    argv[0] = strdup("/system/bin/settings");
    argv[1] = strdup("put");
    argv[2] = strdup("system");
    argv[3] = strdup(property);
    argv[4] = strdup(value);
    argv[5] = NULL;

    /* Check for allocation failures */
    for (int i = 0; i < 5; ++i) {
        if (argv[i] == NULL) {
            strdup_failed = true;
            for (int j = 0; j < i; ++j) { /* Free already allocated args */
                free(argv[j]);
            }
            break;
        }
    }

    if (strdup_failed) {
        LOGE("setAndroidSetting: strdup() failed (out of memory?)");
        return false;
    }

    /* Create a pipe to capture child's stdout/stderr */
    int pipefd[2];
    if (pipe(pipefd) == -1) {
        LOGE("setAndroidSetting: pipe() failed (errno: %d - %s)",
             errno, strerror(errno));
        for (int i = 0; i < 5; ++i) free(argv[i]);
        return false;
    }

    pid_t pid = fork();
    if (pid == -1) {
        /* Fork failed */
        LOGE("setAndroidSetting: fork() failed (errno: %d - %s)",
             errno, strerror(errno));
        close(pipefd[0]);
        close(pipefd[1]);
        for (int i = 0; i < 5; ++i) free(argv[i]);
        return false;
    }

    if (pid == 0) {
        /* --- Child Process --- */
        close(pipefd[0]); /* Close read end */

        /* Redirect stdout and stderr to the pipe */
        while ((dup2(pipefd[1], STDOUT_FILENO) == -1) && (errno == EINTR)) {}
        while ((dup2(pipefd[1], STDERR_FILENO) == -1) && (errno == EINTR)) {}

        close(pipefd[1]); /* Close original write end */

        execv(argv[0], argv);

        /* execv only returns on error */
        fprintf(stderr, "execv failed: %s\n", strerror(errno));
        _exit(127); /* Exit with 127 to indicate execv failure */

    } else {
        /* --- Parent Process --- */
        close(pipefd[1]); /* Close write end */

        char output[REFRESH_RATE_CONFIG_COMMAND_OUTPUT_BUFFER_SIZE * 2];
        char buffer[REFRESH_RATE_CONFIG_COMMAND_OUTPUT_BUFFER_SIZE];
        ssize_t count;
        size_t total_read = 0;

        /* Read all output from the child */
        while ((count = read(pipefd[0], buffer, sizeof(buffer) - 1)) > 0) {
            if (total_read + count < sizeof(output)) {
                memcpy(output + total_read, buffer, count);
                total_read += count;
            } else {
                /* Buffer full, stop reading to avoid overflow */
            }
        }
        output[total_read] = '\0'; /* Null-terminate */

        if (count == -1 && errno != 0) {
            LOGE("setAndroidSetting: read() from pipe failed (errno: %d - %s)",
                 errno, strerror(errno));
        }

        close(pipefd[0]); /* Close read end */

        /* Wait for child to exit */
        int status;
        waitpid(pid, &status, 0);

        /* Free allocated arguments */
        for (int i = 0; i < 5; ++i) {
            free(argv[i]);
        }

        int exit_code = -1;
        if (WIFEXITED(status)) {
            exit_code = WEXITSTATUS(status);
        }

        if (exit_code == 0) {
            LOGI("Successfully set '%s' to %s", property, value);
            return true;
        }

        /* Clean up trailing newline from output for logging */
        if (total_read > 0 && output[total_read - 1] == '\n') {
            output[total_read - 1] = '\0';
        }

        LOGE("Failed to set '%s' to %s. Code: %d, Output: %s",
             property, value, exit_code, output);
        return false;
    }
}