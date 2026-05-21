#include "daemon/tuner.hpp"
#include "daemon/logger.hpp"

#include <cerrno>
#include <fstream>
#include <limits>
#include <string>
#include <vector>

#include <linux/types.h>
#include <sched.h>
#include <sys/mman.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/syscall.h>
#include <unistd.h>

#ifndef MCL_ONFAULT
#define MCL_ONFAULT 4
#endif

#ifndef __NR_ioprio_set
#if defined(__aarch64__)
#define __NR_ioprio_set 30
#else
#define __NR_ioprio_set 251
#endif
#endif

#ifndef __NR_sched_setattr
#if defined(__aarch64__)
#define __NR_sched_setattr 274
#else
#define __NR_sched_setattr 380
#endif
#endif

#ifndef SCHED_FLAG_KEEP_POLICY
#define SCHED_FLAG_KEEP_POLICY 0x08
#endif

#ifndef SCHED_FLAG_UTIL_CLAMP_MAX
#define SCHED_FLAG_UTIL_CLAMP_MAX 0x40
#endif

namespace qos::system
{
    namespace
    {

        struct sched_attr
        {
            __u32 size;
            __u32 sched_policy;
            __u64 sched_flags;
            __s32 sched_nice;
            __u32 sched_priority;
            __u64 sched_runtime;
            __u64 sched_deadline;
            __u64 sched_period;
            __u32 sched_util_min;
            __u32 sched_util_max;
        };

        [[nodiscard]] long read_sysfs_long(const std::string& path) noexcept
        {
            std::ifstream file(path);

            if (!file.is_open())
                return -1;

            long value = -1;
            file >> value;
            return value;
        }

        [[nodiscard]] int apply_little_core_affinity() noexcept
        {
            const int num_cores = static_cast<int>(::sysconf(_SC_NPROCESSORS_CONF));

            if (num_cores <= 0)
                return -1;

            std::vector<int> little_cores;
            long min_val = std::numeric_limits<long>::max();
            bool has_topology = false;

            for (int i = 0; i < num_cores; ++i)
            {
                long cap = read_sysfs_long("/sys/devices/system/cpu/cpu" + std::to_string(i) + "/cpu_capacity");

                if (cap <= 0)
                    continue;

                has_topology = true;

                if (cap < min_val)
                {
                    min_val = cap;
                    little_cores.assign({i});
                }

                else if (cap == min_val)
                {
                    little_cores.push_back(i);
                }
            }

            if (!has_topology)
            {
                min_val = std::numeric_limits<long>::max();
                for (int i = 0; i < num_cores; ++i)
                {
                    long freq = read_sysfs_long("/sys/devices/system/cpu/cpu" + std::to_string(i) +
                                                "/cpufreq/cpuinfo_max_freq");

                    if (freq <= 0)
                        continue;

                    has_topology = true;

                    if (freq < min_val)
                    {
                        min_val = freq;
                        little_cores.assign({i});
                    }

                    else if (freq == min_val)
                    {
                        little_cores.push_back(i);
                    }
                }
            }

            cpu_set_t cpuset;
            CPU_ZERO(&cpuset);

            if (!has_topology || little_cores.empty())
            {
                LOGW("Topology detection failed. Binding to ALL cores.");

                for (int i = 0; i < num_cores; ++i)
                {
                    CPU_SET(i, &cpuset);
                }
            }
            else
            {
                LOGD("Topology detected. Found %zu Little cores.", little_cores.size());

                for (int core_id : little_cores)
                {
                    CPU_SET(core_id, &cpuset);
                }
            }

            return ::sched_setaffinity(0, sizeof(cpu_set_t), &cpuset);
        }

    } // namespace

    void Tuner::expand_resources() noexcept
    {
        struct rlimit rl_fd{};

        if (::getrlimit(RLIMIT_NOFILE, &rl_fd) == 0)
        {
            rl_fd.rlim_cur = rl_fd.rlim_max;

            if (::setrlimit(RLIMIT_NOFILE, &rl_fd) == 0)
            {
                LOGD("FD limit expanded to %lu", rl_fd.rlim_cur);
            }
            else
            {
                LOGW("Failed to maximize FD limit.");
            }
        }

        struct rlimit rl_stack{};

        if (::getrlimit(RLIMIT_STACK, &rl_stack) == 0)
        {
            rlim_t target = 512UL * 1024UL;

            if (rl_stack.rlim_max != RLIM_INFINITY && target > rl_stack.rlim_max)
            {
                target = rl_stack.rlim_max;
            }

            rl_stack.rlim_cur = target;

            if (::setrlimit(RLIMIT_STACK, &rl_stack) == 0)
            {
                LOGD("Stack expanded to %lu bytes", rl_stack.rlim_cur);
            }
            else
            {
                LOGW("Failed to expand Stack.");
            }
        }
    }

    void Tuner::lock_memory() noexcept
    {
        if (::mlockall(MCL_CURRENT | MCL_FUTURE | MCL_ONFAULT) == 0)
        {
            LOGD("Smart RAM Locking Active.");
            return;
        }

        LOGW("MCL_ONFAULT failed. Retrying with MCL_CURRENT...");

        if (::mlockall(MCL_CURRENT) == 0)
        {
            LOGD("RAM Locking Active.");
            return;
        }

        LOGE("Failed to lock pages. Errno: %d", errno);
    }

    void Tuner::harden_process() noexcept
    {
        std::ofstream file("/proc/self/oom_score_adj");

        if (!file.is_open())
        {
            LOGE("Cannot open OOM adjustment file.");
            return;
        }

        file << "-1000";

        if (file.fail())
        {
            LOGE("Failed to write OOM score.");
            return;
        }

        LOGD("OOM Shield Activated.");
    }

    void Tuner::set_high_io_priority() noexcept
    {
        constexpr int IOPRIO_CLASS_BE = 2;
        constexpr int IOPRIO_WHO_PROCESS = 1;
        const int ioprio_val = (IOPRIO_CLASS_BE << 13) | 0;

        if (::syscall(__NR_ioprio_set, IOPRIO_WHO_PROCESS, 0, ioprio_val) == -1)
        {
            LOGE("Failed to set I/O priority.");
            return;
        }

        LOGD("I/O Priority boosted.");
    }

    void Tuner::set_realtime_policy() noexcept
    {
        struct sched_param param{};
        param.sched_priority = 50;

        if (::sched_setscheduler(0, SCHED_FIFO, &param) == -1)
        {
            LOGE("Failed to set SCHED_FIFO. Errno: %d", errno);
            return;
        }

        LOGD("Real-Time Policy (SCHED_FIFO) Active.");
    }

    void Tuner::enforce_efficiency_mode() noexcept
    {
        if (apply_little_core_affinity() == 0)
        {
            LOGD("Affinity mask locked to Little Cores.");
            return;
        }

        LOGW("Failed to bind to Little Cores (errno: %d).", errno);

        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);

        const int num_cores = static_cast<int>(::sysconf(_SC_NPROCESSORS_CONF));

        for (int i = 0; i < num_cores; ++i)
        {
            CPU_SET(i, &cpuset);
        }

        if (::sched_setaffinity(0, sizeof(cpu_set_t), &cpuset) == -1)
        {
            LOGE("CRITICAL - Failed to reset affinity.");
            return;
        }

        LOGD("Fallback successful. Affinity reset to default.");
    }

    void Tuner::maximize_timer_slack() noexcept
    {
        constexpr unsigned long slack_ns = 50UL * 1000UL * 1000UL;

        if (::prctl(PR_SET_TIMERSLACK, slack_ns) == -1)
        {
            LOGE("Failed to set Timer Slack. Errno: %d", errno);
            return;
        }

        LOGD("Wakeup Coalescing Active.");
    }

    void Tuner::limit_cpu_utilization() noexcept
    {
        sched_attr attr{};
        attr.size = sizeof(attr);
        attr.sched_flags = SCHED_FLAG_KEEP_POLICY | SCHED_FLAG_UTIL_CLAMP_MAX;
        attr.sched_util_max = 102;

        if (::syscall(__NR_sched_setattr, 0, &attr, 0) == -1)
        {
            LOGE("Failed to activate UClamp. Errno: %d", errno);
            return;
        }

        LOGD("UClamp Active.");
    }

} // namespace qos::system