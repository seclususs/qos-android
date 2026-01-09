# Porting & Customization Guide

This document is a technical reference for developers who want to adapt **QoS** to specific Android devices.

### ⚠️ Read Before Starting

Because the Android ecosystem is highly fragmented (different SoCs, different thermal paths, different storage types), **QoS is not "Universal Plug-and-Play"**. You **must** modify the Rust source code (`core/`) to match your target device before compilation.

Target audience for this document:

* **Device Maintainer / Porter**
* **Kernel Developer**
* **Performance Engineer**

---

## 1. System Path Configuration (Mandatory)

**Target File:** [`core/src/resources/sys_paths.rs`](../core/src/resources/sys_paths.rs)

This file is the "location map" for the daemon. The daemon does not perform automatic scanning; it trusts the paths you write here literally.

**If paths are incorrect:**
Related services (e.g., CPU Controller) will fail to load and **die permanently (disabled)**. The main daemon may continue running, but that feature will not function.

### A. Storage Interface (UFS vs eMMC)

QoS controls I/O queue directly. You must ensure the `queue` path points to the main block device.

| Storage Type | Common Path | Target Device |
| --- | --- | --- |
| **UFS** | `/dev/sda` | Flagship / Mid-range |
| **eMMC** | `/dev/mmcblk0` | Entry-level |

**Validation Method:**
Check via ADB Shell:

```bash
ls -l /sys/block/sda      # If it appears, use sda
ls -l /sys/block/mmcblk0  # If it appears, use mmcblk0
```

**Implementation (`sys_paths.rs`):**

```rust
// Example for UFS (sda)
pub const K_READ_AHEAD_PATH: &str = "/sys/block/sda/queue/read_ahead_kb";
pub const K_NR_REQUESTS_PATH: &str = "/sys/block/sda/queue/nr_requests";

// Example for eMMC (mmcblk0)
// pub const K_READ_AHEAD_PATH: &str = "/sys/block/mmcblk0/queue/read_ahead_kb";
```

### B. Thermal Zone (Critical)

**⚠️ DANGER ZONE**

The `K_CPU_TEMP_PATH` constant **MUST** point to the SoC/CPU core temperature sensor. Incorrectly selecting a sensor (e.g., choosing battery sensor or *skin temp* instead) will disrupt PID calculations, causing delayed or overly aggressive throttling.

**How to Find the Correct Sensor:**
Run this command to inspect sensor types:

```bash
grep . /sys/class/thermal/thermal_zone*/type
```

Look for output like: `cpu-thermal`, `soc_thermal`, `mtktscpu`, or `tsens_tz_sensor0`.

**Implementation:**

```rust
// Example: CPU sensor is at thermal_zone7
pub const K_CPU_TEMP_PATH: &str = "/sys/class/thermal/thermal_zone7/temp";
```

---

## 2. Logic & Algorithm Tuning

**Target Directory:** [`core/src/controllers/*_impl.rs`](../core/src/controllers/)

This is where the daemon's brain operates. These modules use mathematics (PID, Kalman Filter, Regression) to determine performance.

**Important Rule:**
Don't change numbers ("Magic Numbers") here without understanding the side effects. Use **[TUNING.md](TUNING.md)** as your dictionary.

### Controller Map

| Controller File | Main Function | Dictionary Reference |
| --- | --- | --- |
| **`cpu_impl.rs`** | Load prediction, frequency, thermal management. | [CpuTunables](TUNING.md#cputunables) |
| **`memory_impl.rs`** | Memory PSI, ZRAM, cache management. | [MemoryTunables](TUNING.md#memorytunables) |
| **`storage_impl.rs`** | I/O pattern detection, queue depth. | [StorageTunables](TUNING.md#storagetunables) |

### Example Case: Making Device More "Snappy"

If you want more instant touch response (with slightly higher battery consumption risk), you can tune `CpuTunables` in `core/src/controllers/cpu_impl.rs`:

```rust
let tunables = CpuTunables {
    // Increase 'Gain' so PID reacts strongly to small loads
    response_gain: 65.0, // Default: 50.0

    // Lower threshold so 'Surge' (Boost) mode activates more easily
    surge_threshold: 25.0, // Default: 40.0

    ..Default::default()
};
```

*Make sure you understand the risks by reading the `response_gain` explanation in TUNING.md*

---

## 3. Safety Limits

**Target File:** [`core/src/config/tunables.rs`](../core/src/config/tunables.rs)

This file is the **Safety Fence**. Whatever the dynamic algorithm calculates, the values will be *clamped* to not exceed the Min/Max limits in this file.

**Purpose:** Prevent extreme values that could cause kernel panic or device *soft-brick*.

**When Does It Need Changing?**

* **Large RAM (≥12GB):** Needs more relaxed `read_ahead` or `swappiness` limits.
* **Custom Kernel:** If kernel supports non-standard features or different limits.

**Example Adjustment:**

```rust
// For large RAM devices, allow higher read_ahead
pub const MIN_READ_AHEAD: u64 = 512;
pub const MAX_READ_AHEAD: u64 = 2048;

// Give more breathing room for swap
pub const MIN_SWAPPINESS: u64 = 20;
pub const MAX_SWAPPINESS: u64 = 80;
```

---

## 4. C++ Adaptation (Hardware Abstraction)

**Target File:** [`native/src/runtime/scheduler.cpp`](../native/src/runtime/scheduler.cpp)

The daemon uses a C++ layer to make low-level system calls (`syscall`), one of which is **CPU Affinity** (locking the daemon process to specific cores).

By default, the source code is configured **hardcoded** for **MediaTek Helio G88** SoC (Architecture 2x Big + 6x Little), where the daemon is locked to **Core 0-5** (Little Cluster) for maximum power efficiency.

### Porting Issue

If you compile this code as-is for a SoC with different topology (e.g., Snapdragon 8 Gen 2 or older Quad-Core devices), the daemon may:

1. Run on the wrong core (battery drain).
2. Fail to bind if core index doesn't exist.

### Modification Guide

You must adjust the `apply_little_core_affinity` function to match your target device's CPU topology.

**Step 1: Identify Power-Efficient Cores (Little Cluster)**
Find out the range of small cores on your device (usually Core 0 to X).

| Chipset | Common Topology | Target Index (Little) |
| --- | --- | --- |
| **Helio G88** | 2 Big + 6 Little | `0` to `5` |
| **Snapdragon 865** | 1 Prime + 3 Gold + 4 Little | `0` to `3` |
| **Snapdragon 8 Gen 2** | 1+4+3 (Unique) | `0` to `2` (Silver Cores) |
| **Older Quad Core** | 4 Identical Cores | `0` to `3` (All cores) |

**Step 2: Edit C++ Loop**

Find the following code block in `scheduler.cpp` and change the loop limit number.

**Original Code (Target G88):**

```cpp
// Lock to cores 0, 1, 2, 3, 4, 5
for (int i = 0; i <= 5; ++i) {
    CPU_SET(i, &cpuset);
}
```

**Modification (Example: Snapdragon 865 - Core 0-3 Little):**

```cpp
// Change 'i' limit to 3
for (int i = 0; i <= 3; ++i) {
    CPU_SET(i, &cpuset);
}
```

### Advanced Option: Changing Strategy to Performance

If you're porting this daemon for a gaming device always connected to charger (e.g., Android Console/TV Box), you might want to lock the daemon to **Performance Cores** for maximum responsiveness, ignoring efficiency.

**Modification Example (Target Big Cores):**

```cpp
// Target Cores 4 to 7 (Example on standard Octa-core)
for (int i = 4; i <= 7; ++i) {
    CPU_SET(i, &cpuset);
}
```

> **Note:** Changes to C++ files require a complete rebuild (not just *hot-reload*). Make sure to do a `clean build` after modifying native files.

---

## 5. Compilation & Verification

After adjusting the code:

1. **Rebuild** the binary.
2. **Repack** into Magisk module.
3. **Flash** & Reboot.

### Check Logs (Logcat)

To ensure the daemon is alive and healthy:

```bash
logcat -s QoS:V
```

**How to Read Status:**

| Log Output | Status | Meaning & Action |
| --- | --- | --- |
| *(Silent / Empty)* | **HEALTHY** | Daemon running normally. |
| `Service '...' failed FATALLY` | **PARTIAL** | Path in `sys_paths.rs` is wrong. That service is permanently disabled, but daemon continues. |
| `Critical Panic` | **CRASH** | Fatal error (e.g., permission denied on vital file or thread crash). Check root access. |
| `No such file or directory` | **MISSING** | Target file not found. Re-verify your `ls -l` path. |

> **Tips:** If you want to see activity logs ("Core services running"), compile binary without `--release` flag (Debug Mode). See guide in **[BUILD.md](BUILD.md)**.