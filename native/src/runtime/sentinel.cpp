/**
 * @brief Implementation of signal handling.
 * 
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 * 
 */

#include "runtime/sentinel.h"

#include <csignal>
#include <cstring>
#include <unistd.h>

namespace qos::runtime {
    
    /**
     * @brief Raw signal handler function.
     */
    void signal_handler(int sig, siginfo_t* info, void* context) {
        const char msg[] = "!!! SENTINEL TRIGGERED: Fatal Signal Received !!!\n";
        // Use write() as it is async-signal-safe, unlike printf/LOGE.
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        
        // Reset to default handler and raise again to generate core dump if enabled.
        signal(sig, SIG_DFL);
        raise(sig);
    }

    void Sentinel::arm() {
        struct sigaction sa;
        memset(&sa, 0, sizeof(sa));
        sa.sa_flags = SA_SIGINFO | SA_RESTART;
        sa.sa_sigaction = signal_handler;
        
        // Monitor standard fatal signals
        sigaction(SIGSEGV, &sa, nullptr);
        sigaction(SIGFPE, &sa, nullptr);
        sigaction(SIGABRT, &sa, nullptr);
        sigaction(SIGILL, &sa, nullptr);
    }

}