#pragma once

namespace qos::core
{

    // Manages emergency signal handling for process crashes.
    class Handler
    {
        public:
        // Registers traps for fatal signals (SIGSEGV, SIGFPE, SIGABRT, SIGILL).
        static void arm() noexcept;
    };

} // namespace qos::core