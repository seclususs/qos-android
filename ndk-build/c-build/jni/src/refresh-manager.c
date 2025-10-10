#define LOG_TAG "AdaptiveTweaker"
#include "refresh-manager.h"
#include "system-utils.h"

#include <android/log.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/select.h>
#include <sys/time.h>
#include <sys/types.h>
#include <unistd.h>
#include <linux/input.h>

#ifndef LOGI
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#endif

struct refresh_manager {
    char *touch_path;
    pthread_t thread;
    atomic_int running;
    int current_rate; /* 60 or 90 */
};

static void set_refresh_rate_int(int *current_rate, int new_rate) {
    if (*current_rate == new_rate) return;
    const char *s = (new_rate == 90) ? "90.0" : "60.0";
    if (sys_set_refresh_rate_cmd(s)) {
        *current_rate = new_rate;
        LOGD("RefreshManager: set rate to %s", s);
    } else {
        LOGE("RefreshManager: failed to set rate %s", s);
    }
}

static void *refresh_thread_fn(void *arg) {
    refresh_manager_t *mgr = (refresh_manager_t*)arg;
    const int idle_timeout_sec = 4;
    const int check_interval_ms = 100;

    int fd = open(mgr->touch_path, O_RDONLY | O_NONBLOCK);
    if (fd < 0) {
        LOGE("RefreshManager: cannot open %s (errno=%d)", mgr->touch_path, errno);
        return NULL;
    }

    set_refresh_rate_int(&mgr->current_rate, 60);
    struct timeval last_touch;
    gettimeofday(&last_touch, NULL);

    while (atomic_load(&mgr->running)) {
        fd_set rfds;
        FD_ZERO(&rfds);
        FD_SET(fd, &rfds);
        struct timeval tv;
        tv.tv_sec = 0;
        tv.tv_usec = check_interval_ms * 1000;

        int sel = select(fd + 1, &rfds, NULL, NULL, &tv);
        if (sel > 0) {
            struct input_event ev;
            /* read and drain events */
            while (1) {
                ssize_t r = read(fd, &ev, sizeof(ev));
                if (r <= 0) break;
                /* update last touch */
                gettimeofday(&last_touch, NULL);
            }
            /* set to high */
            set_refresh_rate_int(&mgr->current_rate, 90);
        } else if (sel == 0) {
            /* check idle */
            struct timeval now;
            gettimeofday(&now, NULL);
            long diff = now.tv_sec - last_touch.tv_sec;
            if (diff >= idle_timeout_sec && mgr->current_rate == 90) {
                set_refresh_rate_int(&mgr->current_rate, 60);
            }
        } else {
            if (errno == EINTR) continue;
            LOGE("RefreshManager: select error (errno=%d)", errno);
            /* attempt to reopen occasionally */
            close(fd);
            sleep(1);
            fd = open(mgr->touch_path, O_RDONLY | O_NONBLOCK);
            if (fd < 0) {
                LOGE("RefreshManager: reopen failed (errno=%d), will retry", errno);
                sleep(5);
            }
        }
    }

    close(fd);
    /* restore 60 */
    set_refresh_rate_int(&mgr->current_rate, 60);
    return NULL;
}

/* public API */
refresh_manager_t *refresh_manager_create(const char *touch_dev_path) {
    if (!touch_dev_path) return NULL;
    refresh_manager_t *mgr = (refresh_manager_t*)calloc(1, sizeof(refresh_manager_t));
    if (!mgr) return NULL;
    mgr->touch_path = strdup(touch_dev_path);
    mgr->current_rate = 0;
    atomic_init(&mgr->running, 0);
    return mgr;
}

void refresh_manager_start(refresh_manager_t *mgr) {
    if (!mgr) return;
    atomic_store(&mgr->running, 1);
    if (pthread_create(&mgr->thread, NULL, refresh_thread_fn, mgr) != 0) {
        LOGE("refresh_manager_start: pthread_create failed");
        atomic_store(&mgr->running, 0);
    } else {
        LOGI("RefreshManager: started on %s", mgr->touch_path);
    }
}

void refresh_manager_stop(refresh_manager_t *mgr) {
    if (!mgr) return;
    atomic_store(&mgr->running, 0);
    pthread_join(mgr->thread, NULL);
    LOGI("RefreshManager: stopped");
}

void refresh_manager_destroy(refresh_manager_t *mgr) {
    if (!mgr) return;
    free(mgr->touch_path);
    free(mgr);
}
