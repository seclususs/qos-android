// Author: [Seclususs](https://github.com/seclususs)

#include "runtime/sentinel.h"

#include <csignal>
#include <cstring>
#include <unistd.h>

namespace qos::runtime {

// Raw signal handler. Must be async-signal-safe.
void signal_handler(int sig, siginfo_t *info, void *context) {
  const char msg[] = "!!! SENTINEL TRIGGERED: Fatal Signal Received !!!\n";

  // Use direct write() syscall as standard I/O (printf, etc.) is not safe
  // inside a signal handler.
  write(STDERR_FILENO, msg, sizeof(msg) - 1);

  // Reset the signal action to default and reraise the signal.
  // This ensures that after our logging, the process terminates as expected
  // by the OS (e.g., generating a tombstone/coredump).
  signal(sig, SIG_DFL);
  raise(sig);
}

void Sentinel::arm() {
  struct sigaction sa;
  memset(&sa, 0, sizeof(sa));
  sa.sa_flags = SA_SIGINFO | SA_RESTART;
  sa.sa_sigaction = signal_handler;

  // Register handlers for common crash signals.
  sigaction(SIGSEGV, &sa, nullptr); // Segfault
  sigaction(SIGFPE, &sa, nullptr);  // Floating point exception
  sigaction(SIGABRT, &sa, nullptr); // Abort
  sigaction(SIGILL, &sa, nullptr);  // Illegal instruction
}

} // namespace qos::runtime