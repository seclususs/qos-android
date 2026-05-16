#pragma once

namespace qos::system
{

    class Tuner
    {
        public:
        // Expands process resource limits.
        static void expand_resources() noexcept;

        // Locks process memory into RAM to prevent swapping.
        static void lock_memory() noexcept;

        // Applies OOM shield to exempt process from LMK kills.
        static void harden_process() noexcept;

        // Sets process I/O priority to Best Effort.
        static void set_high_io_priority() noexcept;

        // Sets CPU scheduling policy to Real-Time.
        static void set_realtime_policy() noexcept;

        // Binds process execution strictly to Efficiency (Little) Cores.
        static void enforce_efficiency_mode() noexcept;

        // Maximizes timer slack for better wakeup coalescing and efficiency.
        static void maximize_timer_slack() noexcept;

        // Clamps CPU utilization to prevent frequency ramping.
        static void limit_cpu_utilization() noexcept;
    };

} // namespace qos::system