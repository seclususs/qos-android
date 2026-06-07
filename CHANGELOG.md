# Changelog

## v2.8 (Latest)

- **Resource Management:** Eliminated a critical file descriptor leak within the FFI boundary during daemon initialization. By taking absolute ownership of the raw `signal_fd` at the very start of the `start_services` execution block, the Rust backend is now guaranteed to safely drop and close the kernel file descriptor even if execution aborts early, strictly honoring the C++ memory contract.

- **FFI Boundary:** Eliminated heap allocations during kernel and Android C++ bridging calls by replacing dynamic `CString` generation with a strict stack-allocated byte array wrapper. This achieves a pure zero-allocation FFI layer when manipulating system properties and PSI triggers.

- **Error Handling:** Transformed the `QosError` enum into a zero-allocation structure by replacing heavy `String` types with `Cow<'static, str>`. This allows static error literals to be passed strictly by reference without triggering heap allocations, while gracefully falling back to dynamic memory only for heavily formatted runtime strings.

- **Traversal Engine:** Resolved pseudo-filesystem dependencies and SELinux compatibility issues by removing the `/proc/self/fd/` string reconstruction method. The engine now transitions to native POSIX file-descriptor relative operations via `rustix::fs::openat` and `rustix::fs::unlinkat`, guaranteeing immunity against TOCTOU symlink hijacking without triggering strict Android `procfs` access denials.

- **Memory Footprint:** Achieved true zero-allocation directory traversal in the `CleanerWorker` fast-loop. The engine abandons Rust's high-level `std::fs::read_dir`—which allocates heavy `PathBuf` strings on the heap for every file—in favor of `rustix::fs::Dir::read_from`. It now parses raw kernel C-string pointers directly into byte slices, completely eliminating heap fragmentation during massive cache sweeps.

- **Syscall Optimization:** Drastically reduced kernel-space overhead during file inspection by replacing the heavy `entry.metadata()` attribute, which invokes a full `stat` syscall, with pinpoint `rustix::fs::statx` queries. By explicitly masking only the exact attributes needed like `MTIME`, `SIZE`, or `TYPE`, the OS avoids gathering and populating unused file metadata, yielding massive I/O throughput gains on deep directory trees.

- **Micro-Optimization:** Eliminated expensive `SystemTime::duration_since` object instantiations and temporal calculations inside the tight cleaner loop. The engine shifts the temporal baseline to a single, pre-calculated Unix timestamp prior to execution, replacing heavy time-struct logic with lightning-fast integer arithmetic. This shaves off millions of redundant CPU cycles during aggressive storage evictions.

- **String Processing:** Optimized target extension matching by abandoning `ffi::OsStr` to byte array conversions. File names are now evaluated directly at the kernel boundary as raw C-string bytes, providing instantaneous constant-time matching for safe and trash extensions without intermediate allocations.

- **Hardware Clamping:** Bulletproofed the C++ configuration loader by introducing absolute hardware boundaries via `std::clamp` and `std::swap`. The engine now automatically intercepts and corrects inverted parameters to prevent Rust backend panics, while forcefully clamping extreme CPU and I/O inputs to safe kernel thresholds. This guarantees total OS stability against destructive user configurations without sacrificing safe tuning freedom.

- **Config Hardening:** Eliminated silent parsing failures in the configuration loader by enforcing strict boundary validation within the zero-allocation `std::from_chars` engine. The C++ parser now explicitly rejects malformed inputs containing trailing garbage or unit mismatches by validating the exact memory pointer termination, completely shielding the Rust backend from receiving silently truncated and potentially destructive kernel values.

- **Sentinel Handling:** Hardened the Rust FFI translation layer by implementing a strict sentinel value resolution mechanism. This ensures that empty or explicitly defaulted C++ configuration properties are seamlessly mapped to the internal safe defaults within the Rust backend, preventing invalid maximum-integer logic errors during limits initialization.

- **Config Resolution:** Eliminated parsing blind spots within the C++ configuration loader by expanding the evaluation matrix to dynamically capture all limit permutations. The engine now flawlessly intercepts isolated partial overrides—such as modifying only UClamp or read-ahead values—without demanding a monolithic block, guaranteeing that granular user constraints are accurately propagated.

- **Execution Hardening:** Eliminated the risk of daemon cloning and unauthorized shell executions by establishing a strict UID 0 root gatekeeper and a POSIX single instance lock at the C++ bootstrap layer. This zero-overhead, fail-fast mechanism guarantees exclusive ownership of kernel nodes, shielding the Rust backend from fatal PID collisions and mathematical race conditions during concurrent startup anomalies.

- **Runtime Resilience:** Eliminated silent thread zombies during fatal panics by elevating local `SIGTERM` signals to a process-wide broadcast. Additionally, the engine fortifies the Android property FFI boundary against buffer overflows by enforcing strict memory copy length boundaries and guaranteed null-termination when reading raw system properties.

- **RAM Hardening:** Guaranteed complete daemon immunity against kernel swapping and Android zRAM eviction by enforcing the `MCL_FUTURE` flag during the memory locking fallback sequence. This strictly pins the dynamically spawned Rust background threads and service vectors to physical memory, preventing destructive paging latency and preserving absolute responsiveness after the C++ bootstrap phase.

- **App Optimization:** Reduced application complexity and footprint by removing daemon startup, restart, and telemetry functionality. The Companion App continues to support daemon termination, but configuration changes now require a device reboot to take effect, which securely restarts the daemon with the updated parameters. These features were removed because daemon control and telemetry operations could be blocked by restrictive Android security policies, resulting in inconsistent behavior across different devices. The user interface was also redesigned to reflect this new streamlined workflow, providing a cleaner, more intuitive experience for managing configurations.

---

## History (Archived)

### v2.7
- **Event Loop:** Fixed a critical timing race condition in `runtime.rs` by correctly updating `service.last_tick` inside the successful event execution branch, preventing heavy controller algorithms from incorrectly double-firing during timeout cycles.
- **Kernel I/O:** Eliminated dummy `eventfd` allocations in time-based controllers (Blocker, Cleaner) by changing the handler interface to `Option<RawFd>`. This drastically reduces kernel memory footprint and `epoll` tracking slot waste.
- **Thermal Predictor:** Rewrote the Smith Predictor history lookup from an O(N) iterative backward scan to an O(1) sliding-window approach using a moving tail pointer, heavily reducing CPU cycles during delay compensation.
- **Math Optimization:** Replaced expensive modulo (`%`) bounds checking in the PRNG Poller with *Lemire's Reduction* (multiplication and bitshift), transforming a heavy ARM UDIV instruction (10-40 cycles) into a lightning-fast UMULH (2-3 cycles).
- **Sensors:** Optimized thermal sensor scaling logic by replacing floating-point division (`FDIV`) with multiplication (`FMUL`), saving up to 14 clock cycles per read on the ARM FPU.
- **Micro-Optimization:** Unrolled battery depletion polynomial math (`.powi(3)`) into explicit sequential multiplication (`x * x * x`) to guarantee inline execution and prevent intrinsic function call overhead.
- **Sysfs I/O:** Removed risky `f32` type casting for absolute micro-timing (like 20 million ns `sched_latency`) to prevent mantissa precision loss. Replaced percentage tolerance logic with 100% pure integer arithmetic.
- **Syscall Handling:** Fixed a deliberate `-EINVAL` syscall failure in the C++ PSI registration bridge by removing the `null terminator` write attempt, directly using strict `\n` line endings.
- **Syscall Optimization:** Consolidated redundant `Instant::now()` clock syscalls across nested logic (CPU, Thermal, and Predictor) by capturing time once per loop and passing the reference downward, ensuring zero temporal jitter and minimal vDSO overhead.
- **Blocker:** Prevented ART/Dalvik "JVM Storms" when blocking GMS analytics by decoupling the 18-chain `cmd pm disable` sequence. Introduced a sequential execution loop with a 50ms cooldown sleep, eliminating massive CPU and RAM spikes. Fixed a critical scheduling bug where the Blocker service would permanently die after its first 24-hour cycle. Handed over timing control directly to the `epoll` event loop by returning a constant polling interval and streamlined its initialization flow.
- **Memory Footprint:** Eradicated heap fragmentation and sporadic `malloc` calls in the `CleanerWorker` fast-loop by implementing a Zero-Allocation traversal pattern. Replaced repetitive `Path::join` with mutable `PathBuf` `push()` and `pop()` operations.
- **Sysfs Security & I/O:** Eliminated a critical TOCTOU (Time-of-Check to Time-of-Use) vulnerability by shifting from a Check-Then-Act string validation to a secure Act-Then-Check File Descriptor validation via `/proc/self/fd/`. Removed the `O_TRUNC` flag to prevent destructive file wipeouts during symlink attacks. Simultaneously upgraded path resolution to O(1) performance and achieved pure Zero-Allocation by replacing `fs::read_link` and UTF-8 string overhead with a direct `sys::readlink` C FFI and raw byte-slice (`&[u8]`) matching.
- **Traversal Engine:** Eradicated TOCTOU (Time-of-Check to Time-of-Use) vulnerabilities and symlink escalation in the `CleanerWorker` by replacing race-prone `Path::exists()` checks with secure, act-then-check file descriptor validation via `rustix::fs::openat` (`O_NOFOLLOW` | `O_CLOEXEC`) routed strictly through `/proc/self/fd/`. Fixed a critical File Descriptor (FD) exhaustion vulnerability (Local DoS) by enforcing a strict 20-level recursion depth limit in directory tree size calculations. Additionally, achieved pure Zero-Allocation I/O by dropping expensive `PathBuf` string concatenations in favor of direct `entry.metadata()` polling after explicit symlink rejection.
- **Service Recovery:** Fixed a critical File Descriptor (FD) hijacking vulnerability and FD leaks during daemon self-healing. Replaced unsafe raw integer (`i32`) passing with strict `OwnedFd` type-state semantics. Implemented `.try_clone()` for atomic `O_CLOEXEC` duplication (`F_DUPFD_CLOEXEC`), guaranteeing that recovering services like `SignalController` receive isolated descriptors without silently stealing recycled FDs from active background threads.
- **Companion App:** Introduced an optional, root-powered Android application (Jetpack Compose) for seamless daemon management. It provides a GUI for real-time telemetry (CPU/RAM/Uptime polling), instant daemon lifecycle control (Start/Stop/Restart via `libsu`), and dynamic visual tuning of core modules and advanced kernel limits, securely bridging user inputs directly to the Magisk module's `config.ini` without requiring manual CLI interventions.

### v2.6 
- **Refactor(cleaner):** Removed the active package cache logic (`reusable_pkg_cache` and `refresh_active_packages_cache`) in `CleanerWorker` to simplify the cleaning evaluation process, and lowered the `age_stale_media` threshold from 7 days to 3 days.

### v2.5
- **Architecture:** Modernized the codebase by enforcing strict `clippy::pedantic` lints globally, replacing unsafe raw type casting with safe conversions, and utilizing modern inline string interpolation.
- **Performance:** Delegated inline decisions to the LLVM compiler, eliminated UTF-8 decoding overhead in string validation via raw ASCII processing, and replaced expensive floating-point rounding with highly optimized integer arithmetic.
- **CPU:** Introduced a 5-second caching layer to reduce `sysfs` I/O overhead during high-frequency polling, and achieved zero-allocation hardware capability detection utilizing stack buffers and the `itoa` crate.
- **Cleaner:** Optimized memory and cache efficiency during sweep cycles by replacing `HashSet` with a sorted `Vec` and binary search for active package tracking, while halting unnecessary empty directory removals to minimize I/O syscalls.
- **Sensors:** Rewrote thermal sensor data extraction to parse raw byte streams directly, bypassing expensive UTF-8 string allocations and parsing operations.
- **Blocker:** Migrated file descriptor management from unsafe `libc` routines to safe `rustix::fd::OwnedFd`, ensuring memory safety and automated resource cleanup.
- **Storage:** Refined controller dynamics by passing cache validation strategies by reference to avoid struct copying, and introduced a configurable idle polling bypass.
- **I/O:** Integrated the `itoa` crate for lightning-fast, zero-allocation integer-to-string conversions during stream writes, and hardened security path validation by strictly limiting access to `/sys/` and `/proc/`.
- **Reliability:** Hardened the daemon's core event loops against integer overflow panics during extreme uptimes by implementing safe bounds checking and saturating subtractions for `epoll` timeouts.

### v2.4
- **Blocker:** Introduced a new Component Blocker service to selectively disable targeted GMS background tasks, reducing resource drain and unnecessary wakeups.
- **Build:** Switched global optimization profile from Size (`-Oz`) to Speed (`-O3`) for both Native and Rust targets, prioritizing raw execution throughput over binary size.
- **Logging:** Enforced static compile-time log filtering (`release_max_level_warn`), completely stripping Debug and Info symbols from the release binary to minimize runtime overhead.
- **Polling:** Relaxed the minimum adaptive polling interval floor (50ms → 100ms) to reduce CPU cycle consumption during high-frequency sampling states.
- **Config:** Updated runtime configuration loader to support the new `blocker` module toggle and initialization sequence.

### v2.3
- **Cleaner:** Migrated internal signaling to eventfd via rustix, replacing dummy file handles to reduce syscall overhead and improve resource efficiency.
- **Display:** Removed experimental touch-boost and refresh rate control to streamline architecture and reduce runtime overhead.
- **Native:** Removed display service initialization logic and diagnostics from the C++ runtime, but retained the low-level SurfaceFlinger FFI bridge for future use.
- **Memory:** Optimized resident footprint by constraining stack limits to 512KB and removing aggressive heap purging to streamline allocation dynamics.
- **CPU:** Tuned scheduler update thresholds to minimize redundant sysfs writes and filter out transient fluctuations.

### v2.2

- **Thermal:** Refactored Smith Predictor to use timestamp-based lookups instead of array indexing, ensuring accurate thermal delay compensation under variable polling rates.
- **CPU:** Fixed "Time Dilation" bug by decoupling physical time (`dt_real`) from control time (`dt_safe`) to prevent load model freezing during long sleeps.
- **CPU & Memory:** Removed Memory PSI dependency from CPU controller, frequency scaling now strictly based on computational load and thermal constraints.
- **CPU Math:** Cleaned up algorithms by removing unused coefficients, and **inverted pressure scaling for migration cost** allowing tasks to migrate more freely under high load to improve balancing.
- **Storage Controller:** Switched metrics from `psi_full` to `psi_some`, fixed throughput calculations using real-time delta to prevent artificial spikes, and added aggressive 50ms polling fast-path during critical I/O congestion.
- **Storage Math:** **Refined queue depth scaling dynamics**, added idle/low-latency bypass and optimized gradient thresholds to prevent oscillation during I/O throttling.
- **Adaptive Polling:** Lowered minimum polling floor from 3000ms to 50ms, added **Asymmetric EMA smoothing** for responsive and stable operation.
- **Memory Controller:** Removed dedicated memory module to reduce runtime footprint and simplify logic.
- **Dependencies:** Updated startup logic in `main.cpp` to allow CPU service on kernels without Memory/IO PSI, Cleaner service still requires PSI.
- **Metrics Parsing:** Implemented **Zero-Copy parsing** for PSI and Disk statistics to reduce memory allocation overhead.
- **Kalman Filter:** Upgraded from static 1D to kinematic Constant Velocity (2D) model for zero-lag load tracking and accurate derivatives.
- **Stream I/O:** Updated low-level write handling to use positional I/O for predictability.
- **System Tweaks:** Expanded runtime property tweaks to reduce logging overhead and background noise.
- **Signal Handling:** Refined signal consumption logic for non-blocking reads and transient I/O states.
- **Cleaner Prerequisites:** Added runtime checks for storage and process filesystem accessibility.
- **DeviceCompat:** Integrated layer to validate feature compatibility against specific devices.
- **Display Service:** Added blacklist mechanism to auto-disable on incompatible firmware, implemented touch-driven frequency scaling (60Hz/90Hz) with optimized vfork/execve.
- **PSI Data Model:** Removed redundant fields (`avg60`, `total`) and implemented zero-copy skipping to reduce per-tick overhead.
- **Adaptive Poller:** Removed internal `rate_change` relying on variable dt, now accepts explicit `pressure_velocity` from Kalman filter.
- **Auto-Tuning:** Added Tier-Based Auto-Tuning, device classification (Low/Mid/Flagship) dynamically applies optimized PID coefficients and Storage latency targets.
- **Topology Detection:** Replaced hardcoded CPU affinity with universal detection using EAS capacity and peak frequency, supports all big.LITTLE architectures.
- **Affinity Fallback:** Implemented fail-safe to default to all cores if topology metrics unreadable.
- **Sysfs Helper:** Added `read_sysfs_long` for safe, error-tolerant kernel parameter reading during init.

### v2.1
- **Memory:** Refined control behavior with bounded history tracking and smoother extfrag scaling
- **Monitoring:** Improved vmstat parsing robustness by avoiding implicit default values
- **CPU:** Retuned control parameters to improve stability under transient load
- **Thermal:** Simplified delay handling with bounded predictor buffers
- **Architecture:** Removed overengineered control paths and unused tunables
- **Memory Usage:** Reduced runtime allocation overhead across control loops
- **Polling:** Tightened adaptive polling bounds for more responsive and stable operation

### v2.0
- **Core:** Shifted control flow toward unified predictive–reactive state-driven logic
- **Thermal:** Refined thermal regulation with combined predictive handling and reactive correction
- **Monitoring:** Expanded low-level system telemetry coverage for disk and virtual memory
- **CPU:** Improved load evaluation with trend-aware and transient-sensitive control logic
- **Memory:** Adjusted memory control behavior to better track reclaim activity and allocation pressure
- **Storage:** Extended I/O control logic with saturation and queue state awareness
- **Architecture:** Simplified runtime state handling by reducing global state dependencies
- **Memory Usage:** Reduced steady-state runtime memory footprint through tighter state management
- **Precision:** Standardized control calculations on single-precision floating point
- **Polling:** Tuned adaptive polling behavior for faster response and stable decay

### v1.9
- **Core:** Implemented Closed-Loop PID Thermal Regulation
- **Scheduling:** Enforced Daemon UClamp & Timer Slack Coalescing
- **Polling:** Added Stochastic Polling Jitter with Quantization
- **Affinity:** Refined Core Affinity with Topology Fallback
- **Safety:** Added Deep Sleep Time Discontinuity Detection
- **Diagnostics:** Implemented Granular Kernel Feature Discovery
- **Optimization:** Optimized Memory Footprint via Immediate Decay (`mallopt`)
- **Config:** Refactored Config Parser for Fault Tolerance

### v1.8
- Replaced integral PSI metrics with Real-Time Differential Load Sensing
- Introduced Multi-Scale PSI Logic for responsive and stable decisions
- Added Asymmetric EMA Filtering (Fast Attack, Fast Decay)
- Implemented Trend-Aware Dynamic Gain using Non-Linear Control Functions
- Added Impulse-Based CPU Burst Detection
- Introduced Hysteresis-Driven Scheduler and Task Migration Logic
- Implemented Cross-Coupled Control between CPU, Memory, and Storage subsystems
- Added Storage Saturation Index with Cubic Queue Throttling
- Implemented Triple-Domain ZRAM Elasticity
- Implemented Energy-Aware Adaptive Polling Engine

### v1.7
- Introduced non-linear control curves (sigmoid, parabolic, logistic)
- Added adaptive EMA filtering to suppress PSI noise
- Introduced derived granularity for latency-aware preemption
- Improved memory pressure handling with logistic growth
- Implemented adaptive I/O congestion control

### v1.6
- Refactored to Continuous Dynamic Control (Linear Interpolation)
- Implemented active CPU Scheduler Controller with burst detection
- Added global state awareness for cross-controller optimization
- Added support for user configuration file (`config.ini`)

### v1.5
- Implemented self-healing architecture with auto-recovery
- Migrated to synchronous signal handling (signalfd)
- Offloaded display operations to async worker threads
- Enhanced security with strict path validation

### v1.0 - v1.4 (Legacy / Pre-Stable)
- **v1.4:** Tuned kernel parameters and security hardening.
- **v1.3:** **Major Milestone:** Migrated core logic to Rust for maximum stability.
- **v1.2:** Context-aware FSM with hysteresis and memoization.
- **v1.0-v1.1:** Initial release with Event-Driven Epoll Architecture, Adaptive Resource Control, and Smart Memory Management.