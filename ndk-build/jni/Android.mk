LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)
LOCAL_C_INCLUDES := $(LOCAL_PATH)/include
LOCAL_MODULE := enhancer_binary
LOCAL_SRC_FILES := \
    src/main.cpp \
    src/adaptive-daemon.cpp \
    src/memory-manager.cpp \
    src/refresh-rate-manager.cpp \
    src/hardware-interface.c
LOCAL_CFLAGS += -DDISABLE_LOGGING
LOCAL_CPPFLAGS += -DDISABLE_LOGGING
LOCAL_CPPFLAGS += -std=c++17 -pthread
LOCAL_CFLAGS += -std=c11
LOCAL_LDFLAGS += -pie
LOCAL_LDLIBS := -llog
include $(BUILD_EXECUTABLE)