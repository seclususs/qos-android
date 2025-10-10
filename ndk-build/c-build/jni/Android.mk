LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)
LOCAL_MODULE := enhancer_binary
LOCAL_SRC_FILES := \
    src/main.c \
    src/system-utils.c \
    src/system-tweaker.c \
    src/memory-manager.c \
    src/refresh-manager.c
LOCAL_C_INCLUDES := \
    $(LOCAL_PATH)/include
LOCAL_LDLIBS := -llog
LOCAL_CFLAGS := -O2 -std=c11 -Wall -Wextra -Werror \
    -DANDROID -fPIC -fvisibility=hidden
include $(BUILD_EXECUTABLE)