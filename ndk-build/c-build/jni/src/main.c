#define LOG_TAG "AdaptiveTweaker"
#include "system-tweaker.h"
#include "memory-manager.h"
#include "refresh-manager.h"

#include <android/log.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static volatile sig_atomic_t g_shutdown = 0;

static void handle_signal(int sig) {
    (void)sig;
    g_shutdown = 1;
}

int main(int argc, char **argv) {
    (void)argc; (void)argv;
    signal(SIGINT, handle_signal);
    signal(SIGTERM, handle_signal);
    signal(SIGHUP, handle_signal);

    __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "=== AdaptiveTweaker Starting ===");

    if (!system_tweaker_apply_all()) {
        __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Some static tweaks failed.");
    }

    memory_manager_t *mm = memory_manager_create();
    refresh_manager_t *rm = refresh_manager_create("/dev/input/event3"); /* original default */

    if (rm) refresh_manager_start(rm);
    if (mm) memory_manager_start(mm);

    __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "All services started. Use 'logcat -s AdaptiveTweaker' to view logs.");

    while (!g_shutdown) {
        sleep(1);
    }

    __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "Shutdown requested, cleaning up...");

    if (rm) {
        refresh_manager_stop(rm);
        refresh_manager_destroy(rm);
    }
    if (mm) {
        memory_manager_stop(mm);
        memory_manager_destroy(mm);
    }

    __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "=== AdaptiveTweaker shutdown complete ===");
    return 0;
}
