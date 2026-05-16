#include "daemon/crash.hpp"

#include <csignal>
#include <unistd.h>

namespace qos::core
{
    namespace
    {
        void signal_handler(int sig, siginfo_t* /*info*/, void* /*context*/) noexcept
        {
            constexpr char msg[] = "!!! Crash Signal Received !!!\n";

            [[maybe_unused]] auto res = ::write(STDERR_FILENO, msg, sizeof(msg) - 1);

            ::signal(sig, SIG_DFL);
            ::raise(sig);
        }

    } // namespace

    void Handler::arm() noexcept
    {
        struct sigaction sa{};
        sa.sa_flags = SA_SIGINFO | SA_RESTART;
        sa.sa_sigaction = signal_handler;

        ::sigemptyset(&sa.sa_mask);

        ::sigaction(SIGSEGV, &sa, nullptr);
        ::sigaction(SIGFPE, &sa, nullptr);
        ::sigaction(SIGABRT, &sa, nullptr);
        ::sigaction(SIGILL, &sa, nullptr);
    }

} // namespace qos::core