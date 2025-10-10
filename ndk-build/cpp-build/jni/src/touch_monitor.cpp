/*
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
#include "include/touch_monitor.h"
#include "include/log_utils.h"
#include "include/system_utils.h"
#include <fcntl.h>
#include <unistd.h>
#include <dirent.h>
#include <linux/input.h>
#include <sys/ioctl.h>
#include <thread>
#include <chrono>

TouchMonitor::TouchMonitor(BoostCallback callback) : boost_callback_(callback) {}

void TouchMonitor::start() {
    touch_device_path_ = findTouchDevice();
    if (touch_device_path_.empty()) {
        LOGE("TouchMonitor: Unable to find touch device, exiting task.");
        return;
    }
    std::thread(&TouchMonitor::monitor, this).detach();
}

std::string TouchMonitor::findTouchDevice() {
    DIR *dir = opendir("/dev/input");
    if (dir == NULL) return "";
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (std::string(entry->d_name).rfind("event", 0) == 0) {
            std::string path = "/dev/input/" + std::string(entry->d_name);
            int fd = open(path.c_str(), O_RDONLY);
            if (fd < 0) continue;

            unsigned long ev_bit = 0;
            ioctl(fd, EVIOCGBIT(0, sizeof(ev_bit)), &ev_bit);
            if (ev_bit & (1 << EV_ABS)) {
                unsigned long abs_bit = 0;
                ioctl(fd, EVIOCGBIT(EV_ABS, sizeof(abs_bit)), &abs_bit);
                if ((abs_bit & (1ULL << ABS_MT_POSITION_X)) && (abs_bit & (1ULL << ABS_MT_POSITION_Y))) {
                    close(fd);
                    closedir(dir);
                    LOGI("Perangkat sentuh ditemukan: %s", path.c_str());
                    return path;
                }
            }
            close(fd);
        }
    }
    closedir(dir);
    LOGE("Unable to find touch device.");
    return "";
}

void TouchMonitor::setRefreshRate(const std::string& rate) {
    std::string current_rate = SystemUtils::exec("settings get system min_refresh_rate");
    if (current_rate.find(rate) == std::string::npos) {
        LOGD("Setting refresh rate to: %s", rate.c_str());
        std::string cmd = "settings put system min_refresh_rate " + rate;
        system(cmd.c_str());
    }
}

void TouchMonitor::monitor() {
    const std::string RATE_90HZ = "90.0";
    const std::string RATE_60HZ = "60.0";
    const int REFRESH_RATE_IDLE_SEC = 4;

    int fd = open(touch_device_path_.c_str(), O_RDONLY | O_NONBLOCK);
    if (fd < 0) {
        LOGE("TouchMonitor: Failed to open touch device, exiting task.");
        return;
    }
    setRefreshRate(RATE_60HZ);

    long long last_event_time = 0;
    int last_y = 0;
    bool touching = false;
    
    while (true) {
        fd_set read_fds;
        FD_ZERO(&read_fds);
        FD_SET(fd, &read_fds);

        struct timeval tv = { REFRESH_RATE_IDLE_SEC, 0 };
        int ret = select(fd + 1, &read_fds, nullptr, nullptr, &tv);

        if (ret > 0) {
            setRefreshRate(RATE_90HZ);
            
            struct input_event ev[64];
            int bytes_read = read(fd, ev, sizeof(ev));
            
            for (int i = 0; i < bytes_read / sizeof(struct input_event); ++i) {
                if (ev[i].type == EV_ABS && ev[i].code == ABS_MT_POSITION_Y) {
                    long long current_time = std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now().time_since_epoch()).count();
                    if (touching) {
                        int delta_y = std::abs(ev[i].value - last_y);
                        if ((current_time - last_event_time) < 30) {
                            if (delta_y > 20) boost_callback_(BoostLevel::MEDIUM, 1000);
                            else if (delta_y > 5) boost_callback_(BoostLevel::LIGHT, 500);
                        }
                    }
                    last_y = ev[i].value;
                    last_event_time = current_time;
                } else if (ev[i].type == EV_KEY && (ev[i].code == BTN_TOUCH || ev[i].code == BTN_TOOL_FINGER)) {
                    touching = (ev[i].value == 1);
                    if (touching) boost_callback_(BoostLevel::LIGHT, 300);
                }
            }
        } else if (ret == 0) {
            setRefreshRate(RATE_60HZ);
        } else {
            LOGE("TouchMonitor: select() error. Reopening device.");
            close(fd);
            std::this_thread::sleep_for(std::chrono::seconds(30));
            fd = open(touch_device_path_.c_str(), O_RDONLY | O_NONBLOCK);
            if (fd < 0) {
                 LOGE("Failed to reopen, exiting task.");
                 return;
            }
        }
    }
    close(fd);
}
