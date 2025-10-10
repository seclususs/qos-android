#define LOG_TAG "AdaptiveTweaker"
#include "system-utils.h"

#include <android/log.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/system_properties.h>
#include <unistd.h>

#ifndef LOGI
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#endif

bool sys_write_file(const char *path, const char *value) {
    if (!path || !value) return false;
    FILE *f = fopen(path, "w");
    if (!f) {
        LOGE("sys_write_file: fopen failed for %s (errno=%d)", path, errno);
        return false;
    }
    size_t len = strlen(value);
    if (fwrite(value, 1, len, f) != len) {
        LOGE("sys_write_file: fwrite failed for %s (errno=%d)", path, errno);
        fclose(f);
        return false;
    }
    if (fflush(f) != 0) {
        LOGE("sys_write_file: fflush failed for %s (errno=%d)", path, errno);
        fclose(f);
        return false;
    }
    if (fclose(f) != 0) {
        LOGE("sys_write_file: fclose failed for %s (errno=%d)", path, errno);
        return false;
    }
    return true;
}

void sys_set_property(const char *key, const char *value) {
    if (!key || !value) return;
    if (__system_property_set(key, value) < 0) {
        LOGE("sys_set_property: __system_property_set failed for %s", key);
    }
}

bool sys_set_refresh_rate_cmd(const char *rate_str) {
    if (!rate_str) return false;
    /* Use "settings" shell command like original. Caller must ensure process has permission. */
    size_t cmd_len = 64 + strlen(rate_str);
    char *cmd = (char*)malloc(cmd_len);
    if (!cmd) return false;
    snprintf(cmd, cmd_len, "settings put system min_refresh_rate %s", rate_str);
    int rc = system(cmd);
    free(cmd);
    if (rc != 0) {
        /* If rc != 0, extract exit status if possible */
        int exitcode = WEXITSTATUS(rc);
        LOGE("sys_set_refresh_rate_cmd: command failed, exit=%d", exitcode);
        return false;
    }
    return true;
}
