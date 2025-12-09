/**
 * @file daemon_interface.h
 * @brief Quality of Service (QoS) Daemon Foreign Function Interface (FFI) Specification.
 * @author Seclususs <https://github.com/seclususs>
 * @copyright Copyright (c) 2025 Seclususs. All rights reserved.
 *
 * ======================================================================================
 * SYSTEM ARCHITECTURE OVERVIEW
 * ======================================================================================
 *
 * This header file defines the strict ABI (Application Binary Interface) contract between
 * the Android Native C++ layer (Host) and the Rust Core Logic library (Guest).
 * The architecture follows a "Host-Guest" model where the C++ runtime manages the
 * process lifecycle, OS signals, and JNI interaction, while the Rust static library
 * (`libqos_logic.a`) encapsulates the heuristic algorithms and state machines.
 *
 * <h3>Data Flow Diagram</h3>
 *
 * <pre>
 * +----------------------+        FFI Boundary        +------------------------+
 * |   Android Runtime    |                            |    Rust Core Logic     |
 * |      (C++ Host)      | <========================> |      (Static Lib)      |
 * +----------------------+                            +------------------------+
 * |                      |                            |                        |
 * |  [Main Thread]       | -- calls start/stop ---->  |  [Lifecycle Manager]   |
 * |   - Signal Handling  |                            |   - Thread Spawning    |
 * |   - Service Init     |                            |   - Cleanup            |
 * |                      |                            |                        |
 * |  [Logging Shim]      | <--- raw pointers -------  |  [Logger Module]       |
 * |   - __android_log    |        (const char*)       |   - macros (info!)     |
 * |                      |                            |                        |
 * |  [Emergency Stop]    | <--- function call ------  |  [Panic Handler]       |
 * |   - exit/restart     |      (death signal)        |   - catch_unwind       |
 * |                      |                            |                        |
 * |  [Hardware Abstr.]   | <--- syscall wrappers ---  |  [Event Loop]          |
 * |   - open/read/write  |        (file desc)         |   - epoll_wait         |
 * |   - ioctl            |                            |   - PSI Monitors       |
 * |                      |                            |   - Touch Listeners    |
 * +----------------------+                            +------------------------+
 * </pre>
 *
 * ======================================================================================
 * MEMORY MANAGEMENT CONTRACT
 * ======================================================================================
 *
 * 1. <b>String Ownership:</b>
 * - All strings passed from Rust to C++ (e.g., logging messages) are borrowed.
 * - The C++ layer strictly observes "read-only" access and must not attempt to
 * free() or modify these pointers.
 * - Rust guarantees null-termination for all `const char*` arguments.
 *
 * 2. <b>File Descriptors (FDs):</b>
 * - FDs opened by C++ on behalf of Rust (e.g., via `cpp_open_touch_device`)
 * transfer ownership to the Rust runtime immediately upon return.
 * - The Rust `OwnedFd` type ensures the FD is closed when the struct drops.
 *
 * 3. <b>Stack vs Heap:</b>
 * - Heavy buffers (like input event reads) should be allocated on the stack
 * within the C++ implementation to avoid heap fragmentation, as they are ephemeral.
 *
 * ======================================================================================
 * THREAD SAFETY AND CONCURRENCY
 * ======================================================================================
 *
 * - <b>Reentrancy:</b> The logging functions (`cpp_log_*`) are reentrant and thread-safe,
 * relying on the thread-safety guarantees of the underlying Android logging facility.
 * - <b>Lifecycle:</b> `rust_start_services` and `rust_stop_services` are NOT thread-safe
 * relative to each other. They must be called sequentially from the main thread.
 * - <b>Blocking Operations:</b> Functions exposed to Rust (like `cpp_read_touch_events`)
 * must remain non-blocking (O_NONBLOCK) to ensure the Rust `epoll` loop maintains
 * predictable latency.
 */

#ifndef DAEMON_INTERFACE_H
#define DAEMON_INTERFACE_H

#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>

/*
 * Ensure C linkage to prevent C++ name mangling, allowing the Rust linker
 * to locate these symbols successfully.
 */
#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Bootstraps the Rust runtime environment and initializes background services.
 *
 * @details
 * This function serves as the primary entry point for the library. It is designed to be
 * called exactly once during the application startup phase.
 *
 * <b>Internal Operations:</b>
 * 1. <b>Thread Spawning:</b> Initializes the Rust runtime and spawns the main `epoll`
 * event loop thread. This thread is detached from the caller but managed internally via
 * a global `JoinHandle`.
 * 2. <b>System Tweaker:</b> Launches a secondary, short-lived thread to apply static
 * kernel parameters (sysctl) asynchronously, preventing boot-time latency.
 * 3. <b>State Initialization:</b> Resets the global `SHUTDOWN_REQUESTED` atomic flag.
 *
 * @warning
 * This function initiates detached threads. The caller must ensure the process does not
 * exit immediately. Typically, the C++ `main()` should enter a `while(running)` loop
 * or `sigsuspend()` after calling this.
 *
 * @note
 * If called multiple times without an intervening `rust_stop_services()`, the behavior
 * is undefined and may lead to resource contention or panic in the Rust runtime.
 *
 * @see rust_stop_services
 */
void rust_start_services(void);

/**
 * @brief Initiates a graceful shutdown sequence for all Rust subsystems.
 *
 * @details
 * This function acts as a synchronization barrier. It instructs the Rust event loop
 * to terminate and waits for all managed threads to join.
 *
 * <b>Shutdown Sequence:</b>
 * 1. Atomically sets the `SHUTDOWN_REQUESTED` flag to `true` with Release ordering.
 * 2. If the event loop is blocked in `epoll_wait`, it will naturally wake up on the
 * next timeout or event, check the flag, and exit.
 * 3. The function blocks the calling C++ thread until the Rust main thread handles
 * have joined successfully.
 *
 * <b>Performance Implications:</b>
 * This is a blocking call. Depending on the `epoll` timeout configured in the Rust
 * logic (typically 60s max), this might block for a short duration, though normally
 * it completes instantly.
 *
 * @return void
 */
void rust_stop_services(void);

/**
 * @brief Emits an informational log message to the system buffer.
 *
 * @details
 * Maps to the `ANDROID_LOG_INFO` priority. This should be used for high-level
 * lifecycle events (e.g., "Service Started", "Profile Switched to Balanced").
 *
 * @param[in] message
 * A null-terminated UTF-8 string. The pointer is valid only for the duration of the call.
 *
 * @note
 * In production builds, `INFO` logs are generally preserved. Avoid putting high-frequency
 * loop data here to prevent log spam and buffer rotation.
 */
void cpp_log_info(const char* message);

/**
 * @brief Emits a debug log message for development and diagnostics.
 *
 * @details
 * Maps to the `ANDROID_LOG_DEBUG` priority.
 *
 * <b>Conditional Compilation:</b>
 * Depending on the `logging.h` preprocessor definitions (e.g., `#ifdef NDEBUG`),
 * implementation of this function may be compiled out to a no-op instruction to
 * reduce binary size and runtime overhead.
 *
 * @param[in] message
 * A null-terminated C-string containing diagnostic data (e.g., variable states, raw sensor values).
 */
void cpp_log_debug(const char* message);

/**
 * @brief Emits an error log message indicating a failure condition.
 *
 * @details
 * Maps to the `ANDROID_LOG_ERROR` priority. This is the highest severity used by the daemon.
 *
 * <b>Usage Guidelines:</b>
 * - Call this when a syscall fails (e.g., `open()` returns -1).
 * - Call this when the heuristic logic encounters an impossible state.
 * - These logs should always be visible in release builds for triage.
 *
 * @param[in] message
 * A null-terminated C-string describing the error context.
 */
void cpp_log_error(const char* message);

/**
 * @brief Reports a critical, unrecoverable failure from Rust to the C++ runtime.
 *
 * @details
 * This function acts as a "Dead Man's Switch". If the Rust thread panics (crashes)
 * or encounters a fatal state despite `catch_unwind`, it calls this function to
 * notify the host process.
 *
 * <b>Behavior:</b>
 * The implementation should typically terminate the process (exit/abort) to allow
 * the Android init system (or watchdog) to restart the daemon cleanly.
 *
 * @param[in] context
 * A short string describing why the death signal was sent (e.g., "Panic in Event Loop").
 */
void cpp_notify_service_death(const char* context);

/**
 * @brief Opens a character device node with specific flags for non-blocking I/O.
 *
 * @details
 * This is a direct wrapper around the POSIX `open()` syscall, but it enforces
 * specific flags required by the Rust `epoll` architecture.
 *
 * <b>Enforced Flags:</b>
 * - `O_RDONLY`: Read-only access.
 * - `O_NONBLOCK`: Essential for Edge-Triggered or Level-Triggered epoll usage without freezing the thread.
 * - `O_CLOEXEC`: Prevents file descriptor leakage to child processes.
 *
 * @param[in] path
 * Absolute path to the device node (e.g., `/dev/input/event0`).
 *
 * @return
 * - On Success: A non-negative file descriptor (int).
 * - On Failure: Returns -1. The specific `errno` is logged via `cpp_log_error` internally.
 */
int cpp_open_touch_device(const char* path);

/**
 * @brief Drains the input event buffer for a specific file descriptor.
 *
 * @details
 * When the `epoll` loop detects activity (`EPOLLIN`) on a touch device, this function
 * is invoked to consume the data.
 *
 * <b>Mechanism:</b>
 * It performs a `read()` in a loop until it receives `EAGAIN` or reads 0 bytes.
 * This effectively "clears" the interrupt state of the device driver.
 *
 * <b>Data Handling:</b>
 * The actual content of the `input_event` struct is discarded. The daemon's logic uses
 * the <i>existence</i> of an event as a signal for user activity, rather than analyzing
 * the specific X/Y coordinates or keycodes.
 *
 * @param[in] fd
 * The valid file descriptor previously opened via `cpp_open_touch_device`.
 */
void cpp_read_touch_events(int fd);

/**
 * @brief Configures and registers a Pressure Stall Information (PSI) trigger.
 *
 * @details
 * The Linux PSI subsystem allows userspace to monitor resource contention (CPU, IO, Memory).
 * This function handles the complex handshake required to register a trigger with the kernel.
 *
 * <b>Protocol Sequence:</b>
 * 1. Opens the PSI file (e.g., `/proc/pressure/memory`) with Read/Write permissions.
 * 2. Constructs a protocol string: `"some <threshold> <window>"`.
 * 3. Writes this string to the FD.
 * 4. Checks for write errors (which usually indicate kernel incompatibility).
 *
 * @param[in] path
 * Path to the PSI interface. Valid values typically include:
 * - `/proc/pressure/memory`
 * - `/proc/pressure/io`
 * - `/proc/pressure/cpu`
 *
 * @param[in] threshold_us
 * The stall threshold in microseconds. If the system stalls for longer than this
 * value within the specified window, the FD becomes readable.
 *
 * @param[in] window_us
 * The sliding window size in microseconds.
 *
 * @return
 * - On Success: A valid file descriptor representing the registered trigger.
 * - On Failure: Returns -1. Commonly occurs if the kernel was compiled without `CONFIG_PSI=y`.
 */
int cpp_register_psi_trigger(const char* path, int threshold_us, int window_us);

#ifdef __cplusplus
}
#endif

#endif // DAEMON_INTERFACE_H