LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)
LOCAL_C_INCLUDES := $(LOCAL_PATH)/include
LOCAL_MODULE := enhancer_binary
LOCAL_SRC_FILES := \
    src/main_daemon.c \
    src/system_tweaker.c \
    src/system_utils.c \
    src/adaptive_memory_manager.c \
    src/adaptive_refresh_rate_manager.c \
    src/fd_wrapper.c
LOCAL_CFLAGS += -std=c11 -Wall -Wextra -pthread
LOCAL_LDFLAGS += -pie -pthread
LOCAL_LDLIBS := -llog -lm
include $(BUILD_EXECUTABLE)