# Changelog

## v1.8
- Replaced integral PSI metrics with Real-Time Differential Load Sensing
- Introduced Multi-Scale PSI Logic for responsive and stable decisions
- Added Asymmetric EMA Filtering (Fast Attack, Fast Decay) with Cold-Start Protection
- Implemented Trend-Aware Dynamic Gain using Non-Linear Control Functions
- Added Impulse-Based CPU Burst Detection with Low-Pass Filtered Response
- Introduced Hysteresis-Driven Scheduler and Task Migration Logic
- Implemented Cross-Coupled Control between CPU, Memory, and Storage subsystems
- Added Storage Saturation Index with Cubic Queue Throttling for Anti-Bufferbloat
- Implemented Triple-Domain ZRAM Elasticity
- Enforced strict constraints, output clamping, and numerical safety guards
- Implemented Energy-Aware Adaptive Polling Engine for autonomous power efficiency
- Added Precision-Gated I/O Control with fuzzy tolerance to minimize overhead

---

## v1.7
- Introduced non-linear control curves (sigmoid, parabolic, logistic, exponential)
- Added adaptive EMA filtering to suppress PSI noise and parameter jitter
- Applied inverse sigmoid and parabolic curves for stable CPU scheduler control
- Introduced derived granularity for consistent, latency-aware preemption
- Improved memory pressure handling with logistic growth and exponential decay
- Implemented adaptive I/O congestion control and optimized queue batching
- Added new dynamically controlled parameters
- Introduced configurable subsystem control

---

## v1.6
- Refactored to Continuous Dynamic Control (Linear Interpolation)
- Implemented active CPU Scheduler Controller with burst detection
- Added global state awareness for cross-controller optimization
- Optimized PSI monitoring with persistent file descriptors
- Added support for user configuration file (`config.ini`)

---

## v1.5
- Implemented self-healing architecture with auto-recovery
- Migrated to synchronous signal handling (signalfd)
- Offloaded display operations to async worker threads
- Optimized I/O with persistent file descriptors
- Enhanced security with strict path validation
- Added boot completion safety check

---

## v1.4
- Tuned kernel parameters
- Dependency updates
- Security hardening

---

## v1.3
- Optimized parameters for better balance
- Significantly reduced background CPU usage
- Adjusted touch boost timeout logic
- Migrated core logic to Rust for maximum stability

---

## v1.2
- Refactor to context-aware FSM with hysteresis
- Improved stability
- Added auto-recovery & active polling
- Added memoization for syscalls
- Optimized kernel params

---

## v1.1
- Impl Event-Driven (Epoll) Arch
- Optimized CPU & RAM Usage

---

## v1.0
- Adaptive Resource Control
- Smart Memory Management
- Intelligent I/O Scheduler
- Dynamic Refresh Rate
- Network Optimization
- Kernel Optimization
- VM Tuning
- Enhanced Security Settings
- Custom Boot Animation