# Changelog

## v2.0 (Latest)
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