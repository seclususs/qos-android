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
#include "include/cpu_manager.h"
#include "include/memory_manager.h"
#include "include/touch_monitor.h"
#include "include/system_utils.h"
#include "include/log_utils.h"
#include "include/boost_manager.h"
#include <unistd.h>
#include <thread>
#include <chrono>
#include <csignal>
#include <memory>
#include <functional>
#include <array>

std::unique_ptr<CpuManager> cpu_manager;
std::unique_ptr<MemoryManager> memory_manager;
std::unique_ptr<BoostManager> boost_manager;

void trigger_boost(BoostLevel level, int duration_ms) {
    if (boost_manager) {
        boost_manager->requestBoost(level, duration_ms);
    }
}

void applyOptimizerTweaks() {
    for (int i = 0; i < NUM_CPU_CORES; ++i) {
        SystemUtils::applyTweak("/sys/devices/system/cpu/cpufreq/policy" + std::to_string(i) + "/scaling_governor", "schedutil");
    }
    SystemUtils::applyTweak("/proc/sys/vm/swappiness", "100");
    SystemUtils::applyTweak("/proc/sys/vm/vfs_cache_pressure", "100");
    SystemUtils::applyTweak("/proc/sys/vm/page-cluster", "0");
    SystemUtils::setProp("lmk.minfree_levels", "0:55296,100:80640,200:106200,300:131760,900:197640,999:262144");
    if (access("/sys/block/mmcblk0/queue/nr_requests", F_OK) == 0) {
        SystemUtils::applyTweak("/sys/block/mmcblk0/queue/nr_requests", "256");
        SystemUtils::applyTweak(STORAGE_READ_AHEAD_PATH, "256");
    }
    SystemUtils::applyTweak("/proc/sys/kernel/sched_latency_ns", "18000000");
    SystemUtils::applyTweak("/proc/sys/kernel/sched_min_granularity_ns", "2250000");
    SystemUtils::applyTweak("/dev/stune/foreground/schedtune.boost", "5");
    SystemUtils::applyTweak("/dev/stune/top-app/schedtune.boost", "0");
    SystemUtils::setProp("persist.sys.lmk.reportkills", "false");
    LOGI("Tweak optimizer applied.");
}

void logcatMonitorTask() {
    system("logcat -c");
    std::string last_focused_app = "";
    const char* logcat_cmd = "logcat -b system -s ActivityManager:I *:S";

    std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(logcat_cmd, "r"), pclose);
    if (!pipe) {
        LOGE("LogcatMonitor: popen() failed!");
        return;
    }

    LOGI("Logcat monitoring started.");
    std::array<char, 512> buffer;
    while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
        std::string line(buffer.data());
        size_t pos = line.find("Displayed ");
        if (pos != std::string::npos) {
            std::string component_str = line.substr(pos + 10);
            size_t end_pos = component_str.find(':');
            if (end_pos != std::string::npos) {
                std::string current_app = component_str.substr(0, end_pos);
                if (!current_app.empty() && current_app != last_focused_app) {
                    LOGD("Application switch detected: %s", current_app.c_str());
                    trigger_boost(BoostLevel::FULL, 2500);
                    last_focused_app = current_app;
                }
            }
        }
    }
}

void memoryMonitorTask() {
    while(true) {
        std::this_thread::sleep_for(std::chrono::minutes(3));
        memory_manager->manage();
    }
}

void cleanup() {
    LOGI("Cleaning up and restoring default settings...");
    if (cpu_manager) cpu_manager->restoreDefaults();
    if (memory_manager) memory_manager->restoreDefaults();
    system("settings put system min_refresh_rate 60.0");
    LOGI("Cleanup completed.");
}

void signalHandler(int signum) {
    LOGI("Interrupt signal (%d) received. Cleaning up...", signum);
    cleanup();
    exit(signum);
}

int main() {
    SystemUtils::waitForBoot();

    cpu_manager = std::make_unique<CpuManager>();
    memory_manager = std::make_unique<MemoryManager>();
    boost_manager = std::make_unique<BoostManager>(*cpu_manager);
    
    cpu_manager->initialize();
    memory_manager->initialize();
    
    applyOptimizerTweaks();

    signal(SIGTERM, signalHandler);
    signal(SIGINT, signalHandler);
    
    auto touch_monitor = std::make_unique<TouchMonitor>([](BoostLevel level, int duration) {
        trigger_boost(level, duration);
    });
    touch_monitor->start();

    std::thread(logcatMonitorTask).detach();
    std::thread(memoryMonitorTask).detach();

    while (true) {
        std::this_thread::sleep_for(std::chrono::hours(24));
    }

    return 0;
}
