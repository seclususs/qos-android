LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)
LOCAL_C_INCLUDES := $(LOCAL_PATH)/include
LOCAL_MODULE := enhancer_binary
LOCAL_SRC_FILES := \
    src/main.cpp \
    src/cpu_manager.cpp \
    src/memory_manager.cpp \
    src/touch_monitor.cpp \
    src/system_utils.cpp \
    src/boost_manager.cpp
LOCAL_CFLAGS += -pie #-DDEBUG
LOCAL_LDFLAGS += -pie
LOCAL_LDLIBS := -landroid -llog
include $(BUILD_EXECUTABLE)