# Customization Guide

---

## Important Warning

The Android ecosystem is highly fragmented (different SoCs, kernels, thermal paths, storage devices, and scheduler behaviors). **QoS is not universal plug-and-play**. You **must** modify the Rust source code in the `core/` directory to match your target device before compiling.

Failure to adapt paths, limits, or detection logic may result in:
- Incorrect thermal or performance behavior
- Kernel panics or soft-bricks in extreme cases

---

## Target Audience

- Device Maintainers / Porters
- Kernel Developers
- Performance Engineers

---

## Algorithm & Logic Tuning

**Directory**: `core/src/algorithms/`

This is the core intelligence of the daemon. Modules use advanced mathematics (Kalman Filter, PID with Smith Predictor, adaptive polling, workload pattern detection) to make real-time decisions.

**Key Files**:
- `cpu_math.rs` → Load prediction, uClamp, latency/granularity, WALT, migration cost
- `thermal_math.rs` → PID + Smith Predictor for thermal management
- `storage_math.rs` → I/O pattern detection, queue depth, read-ahead

**Rule**: Do not change numeric values ("magic numbers") without understanding their impact. Refer to **[TUNING.md](TUNING.md)** for detailed explanations of each parameter.

**Example**: To make the device feel more responsive (at the cost of slightly higher battery usage):
```rust
// In cpu_math.rs, Flagship block (or your tier)
response_gain: 45.0,           // Default: 36.0
surge_threshold: 13.0,         // Default: 15.0
transient_poll_interval: 35.0, // Default: 45.0
```

---

## Safety Limits & Clamping

**File**: `core/src/config/kernel_limits.rs`

These are **hard safety fences**. Dynamic algorithms will never exceed the min/max values defined here.

**Purpose**: Prevent dangerous values (e.g., latency = 0 ns, nr_requests = 1, excessive read-ahead) that could cause instability or kernel panic.

**Common Reasons to Adjust**:
- High-refresh-rate panels (120 Hz / 144 Hz) → lower `min_latency_ns`
- Large RAM (12 GB+) → increase `max_read_ahead` and `max_nr_requests`
- Specific kernel scheduler quirks → adjust migration/wakeup costs

**Example**:
```rust
// Flagship block in CpuKernelLimitsConfig
min_latency_ns: 5_000_000,   // 5 ms (was 6 ms)
max_latency_ns: 18_000_000,
max_read_ahead: 2048,        // Allow larger read-ahead on high-RAM devices
```

All computed values are clamped before writing to sysfs.

---

## Hardware Discovery & Paths

**Key Files**:
- `resources/discovery.rs` → Automatic detection of storage device and CPU thermal zone
- `resources/sys_paths.rs` → Central constants and getters for PSI, scheduler, battery, and thermal paths

**Common Customizations**:
- Add your device's thermal zone names to `THERMAL_PRIORITY_LIST`
- Add misidentified zones to `THERMAL_BLACKLIST`
- Override storage device detection if your device uses non-standard names (e.g., `sda`, custom nvme, zram)

If automatic detection fails, hard-code the correct paths in `sys_paths.rs` or extend the detection functions.

---

## Compilation, Packaging & Verification

### Build Steps
1. Modify the source code as needed
2. Rebuild the binary (see **BUILD.md** for instructions)
3. Repackage into the Magisk module
4. Flash and reboot

### Logcat Verification
```bash
logcat -s QoS:V
```

**Common Status Indicators**:
- No output or only initialization messages → Daemon is healthy
- "No such file or directory" or "Failed to open ..." → Path discovery problem → fix in `discovery.rs` or `sys_paths.rs`
- "Service ... failed FATALLY" → Permission, SELinux, or thread issue
- Frequent warnings → Overly aggressive polling or mis-tuned parameters

**Tip**: Build without `--release` (Debug mode) for more verbose internal logs.

---

## Additional Considerations

- **SELinux**: Ensure the module correctly sets contexts for sysfs writes
- **Kernel Compatibility**: QoS depends on specific interfaces (PSI, sched_* tunables, WALT). Verify they exist on your kernel
- **Thermal & Battery Sensors**: Confirm scaling (most battery/temp nodes report ×10; handled by `ThermalSensor`)
- **Testing**: Use realistic workloads (gaming, multitasking, heavy I/O) and monitor temperatures, PSI, responsiveness, and battery drain
- **Device Tier**: After changes, verify `DeviceTier::get()` returns the expected tier and that limits are applied correctly