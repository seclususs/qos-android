# Changelog

## v2.3 (Latest)
- **Cleaner:** Migrated internal signaling to eventfd via rustix, replacing dummy file handles to reduce syscall overhead and improve resource efficiency.
- **Display:** Removed experimental touch-boost and refresh rate control to streamline architecture and reduce runtime overhead.
- **Native:** Removed display service initialization logic and diagnostics from the C++ runtime, but retained the low-level SurfaceFlinger FFI bridge for future use.
- **Memory:** Optimized resident footprint by constraining stack limits to 512KB and removing aggressive heap purging to streamline allocation dynamics.
- **CPU:** Tuned scheduler update thresholds to minimize redundant sysfs writes and filter out transient fluctuations.

---

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

---

### v2.1
- **Memory:** Refined control behavior with bounded history tracking and smoother extfrag scaling
- **Monitoring:** Improved vmstat parsing robustness by avoiding implicit default values
- **CPU:** Retuned control parameters to improve stability under transient load
- **Thermal:** Simplified delay handling with bounded predictor buffers
- **Architecture:** Removed overengineered control paths and unused tunables
- **Memory Usage:** Reduced runtime allocation overhead across control loops
- **Polling:** Tightened adaptive polling bounds for more responsive and stable operation

---

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

---

### v1.9
- **Core:** Implemented Closed-Loop PID Thermal Regulation
- **Scheduling:** Enforced Daemon UClamp & Timer Slack Coalescing
- **Polling:** Added Stochastic Polling Jitter with Quantization
- **Affinity:** Refined Core Affinity with Topology Fallback
- **Safety:** Added Deep Sleep Time Discontinuity Detection
- **Diagnostics:** Implemented Granular Kernel Feature Discovery
- **Optimization:** Optimized Memory Footprint via Immediate Decay (`mallopt`)
- **Config:** Refactored Config Parser for Fault Tolerance

---

## History (Archived)

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