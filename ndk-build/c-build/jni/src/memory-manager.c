#define LOG_TAG "AdaptiveTweaker"
#include "memory-manager.h"
#include "system-utils.h"

#include <android/log.h>
#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifndef LOGD
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#endif

/* thresholds and values ported from C++ version */
static const char *kSwappinessLow = "20";
static const char *kVfsCachePressureLow = "50";
static const char *kSwappinessMid = "100";
static const char *kVfsCachePressureMid = "100";
static const char *kSwappinessHigh = "150";
static const char *kVfsCachePressureHigh = "200";

static const int kGoToHighThreshold = 20;
static const int kGoToLowThreshold = 45;
static const int kReturnToMidFromLowThreshold = 40;
static const int kReturnToMidFromHighThreshold = 25;

static const char *meminfo_path = "/proc/meminfo";
static const char *swappiness_path = "/proc/sys/vm/swappiness";
static const char *vfs_cache_pressure_path = "/proc/sys/vm/vfs_cache_pressure";

struct memory_manager {
    pthread_t thread;
    atomic_int running;
    int state; /* 0 unknown, 1 low, 2 mid, 3 high */
};

/* helpers */
static int get_free_ram_percent(void) {
    FILE *f = fopen(meminfo_path, "r");
    if (!f) {
        LOGE("get_free_ram_percent: fopen failed (%d)", errno);
        return -1;
    }
    char line[256];
    long memTotal = -1, memAvailable = -1;
    while (fgets(line, sizeof(line), f)) {
        if (strncmp(line, "MemTotal:", 9) == 0) {
            sscanf(line+9, "%ld", &memTotal);
        } else if (strncmp(line, "MemAvailable:", 13) == 0) {
            sscanf(line+13, "%ld", &memAvailable);
        }
        if (memTotal != -1 && memAvailable != -1) break;
    }
    fclose(f);
    if (memTotal > 0 && memAvailable >= 0) {
        double pct = ((double)memAvailable / (double)memTotal) * 100.0;
        return (int)(pct + 0.5);
    }
    return -1;
}

static void apply_memory_tweaks(int new_state) {
    /* 1 low, 2 mid, 3 high */
    if (new_state == 1) {
        LOGI("MemoryManager: Applying LOW tweaks swappiness=%s vfs=%s", kSwappinessLow, kVfsCachePressureLow);
        sys_write_file(swappiness_path, kSwappinessLow);
        sys_write_file(vfs_cache_pressure_path, kVfsCachePressureLow);
    } else if (new_state == 2) {
        LOGI("MemoryManager: Applying MID tweaks swappiness=%s vfs=%s", kSwappinessMid, kVfsCachePressureMid);
        sys_write_file(swappiness_path, kSwappinessMid);
        sys_write_file(vfs_cache_pressure_path, kVfsCachePressureMid);
    } else if (new_state == 3) {
        LOGI("MemoryManager: Applying HIGH tweaks swappiness=%s vfs=%s", kSwappinessHigh, kVfsCachePressureHigh);
        sys_write_file(swappiness_path, kSwappinessHigh);
        sys_write_file(vfs_cache_pressure_path, kVfsCachePressureHigh);
    }
}

static void *memory_thread_fn(void *arg) {
    memory_manager_t *mgr = (memory_manager_t*)arg;
    const unsigned int interval_sec = 5;
    int current_state = 0; /* unknown */

    while (atomic_load(&mgr->running)) {
        int freePct = get_free_ram_percent();
        if (freePct >= 0) {
            LOGD("MemoryManager: free percent = %d", freePct);
            int new_state = current_state;
            if (current_state == 0) {
                if (freePct < kGoToHighThreshold) new_state = 3;
                else if (freePct > kGoToLowThreshold) new_state = 1;
                else new_state = 2;
            } else if (current_state == 3) { /* HIGH */
                if (freePct >= kReturnToMidFromHighThreshold) new_state = 2;
            } else if (current_state == 2) { /* MID */
                if (freePct < kGoToHighThreshold) new_state = 3;
                else if (freePct > kGoToLowThreshold) new_state = 1;
            } else if (current_state == 1) { /* LOW */
                if (freePct < kReturnToMidFromLowThreshold) new_state = 2;
            }

            if (new_state != current_state) {
                apply_memory_tweaks(new_state);
                current_state = new_state;
            }
        }
        for (unsigned int i = 0; i < interval_sec && atomic_load(&mgr->running); ++i) {
            sleep(1);
        }
    }
    return NULL;
}

/* public API */
memory_manager_t *memory_manager_create(void) {
    memory_manager_t *mgr = (memory_manager_t*)calloc(1, sizeof(memory_manager_t));
    if (mgr) {
        atomic_init(&mgr->running, 0);
        mgr->state = 0;
    }
    return mgr;
}

void memory_manager_start(memory_manager_t *mgr) {
    if (!mgr) return;
    atomic_store(&mgr->running, 1);
    if (pthread_create(&mgr->thread, NULL, memory_thread_fn, mgr) != 0) {
        LOGE("memory_manager_start: pthread_create failed");
        atomic_store(&mgr->running, 0);
    } else {
        LOGI("MemoryManager: started");
    }
}

void memory_manager_stop(memory_manager_t *mgr) {
    if (!mgr) return;
    atomic_store(&mgr->running, 0);
    pthread_join(mgr->thread, NULL);
    LOGI("MemoryManager: stopped");
}

void memory_manager_destroy(memory_manager_t *mgr) {
    if (!mgr) return;
    free(mgr);
}
