# QoS Parameter Tuning Documentation

---

## CpuTunables
→ [`cpu_impl.rs`](../core/src/controllers/cpu_impl.rs)

These parameters control CPU scheduler behavior, load prediction, and frequency management.

### `latency_gran_ratio` (Default: `0.65`)

**Purpose**: Binds the ratio between granularity (minimum task run duration) to latency to maintain Linux kernel proportionality.

**Formula/Logic**:
```rust
final_granularity = adjusted_latency * tunables.latency_gran_ratio
```

- **Impact if increased**: Tasks run longer before preemption, improving cache efficiency (L1/L2) but reducing input responsiveness (UI feels "heavy").
- **Impact if decreased**: High interactivity, but kernel overhead increases (more frequent spinlocks), wastes power.

### `decay_coeff` (Default: `0.10`)

**Purpose**: Controls exponential decay of wakeup_granularity in response to load.

**Formula/Logic**:
```rust
decay = (-tunables.decay_coeff * pressure).exp()
```

- **Impact if increased**: Wakeup_granularity drops quickly under light load, highly responsive for touch boost.
- **Impact if decreased**: Slower granularity reduction, more power efficient.

### `nr_migrate_k` (Default: `0.20`)

**Purpose**: Scales the number of task migrations between cores based on CPU pressure to reduce lock overhead.

**Formula/Logic**:
```rust
denom = 1.0 + (tunables.nr_migrate_k * pressure);
target_nr = min + (range / denom);
```

- **Impact if increased**: Migration aggressively limited under high load, reduces rq->lock overhead, throughput increases.
- **Impact if decreased**: Migration remains active under busy CPU, more aggressive load balancing.

### `uclamp_k` (Default: `0.12`)

**Purpose**: Controls the steepness of the sigmoid curve for uClamp (minimum utility) activation based on pressure.

**Formula/Logic**:
```rust
exponent = -uclamp_k * (pressure - uclamp_mid);
val = min + (range / (1.0 + exp(exponent)));
```

- **Impact if increased**: Sharp clamping transition, minimum frequency rises suddenly.
- **Impact if decreased**: Smooth transition, more gradual clamping.

### `uclamp_mid` (Default: `25.0`)

**Purpose**: PSI pressure midpoint that triggers minimum frequency clamping.

**Formula/Logic**:
```rust
exponent = -uclamp_k * (pressure - uclamp_mid);
```

- **Impact if increased**: Clamping activates only under heavy load, saves battery under light-medium load.
- **Impact if decreased**: Clamping activates early, prevents FPS drops in fluctuating games.

### `response_gain` (Default: `50.0`)

**Purpose**: Main proportional gain (Kp) for frequency response to PSI displacement.

**Formula/Logic**:
```rust
prop_term = k_final * (target_psi - current_psi);
```

- **Impact if increased**: Aggressive/instant frequency response, risk of oscillation and heat.
- **Impact if decreased**: Slow response, power efficient but lags on sudden load.

### `stability_ratio` (Default: `1.40`)

**Purpose**: Damping factor to brake frequency oscillation.

**Formula/Logic**:
```rust
critical_damping = 2.0 * sqrt(k_final);
damping_term = critical_damping * tunables.stability_ratio * rate;
```

- **Impact if increased**: Stable system, frequency difficult to change (less responsive).
- **Impact if decreased**: Agile system, increased oscillation risk.

### `stability_margin` (Default: `1.6`)

**Purpose**: Stability cost multiplier in damping calculation.

**Formula/Logic**:
```rust
c_stability = stability_damping_req.clamp(0.0, base_damp * 4.0) * tunables.stability_margin;
```

- **Impact if increased**: Tight stability, reduces oscillation but lowers responsiveness.
- **Impact if decreased**: Agile system, increased oscillation risk.

### `gain_scheduling_alpha` (Default: `1.2`)

**Purpose**: Adaptive gain multiplier when load trend increases.

**Formula/Logic**:
```rust
k_dynamic = k_base * (1.0 + (tunables.gain_scheduling_alpha * trend_factor));
```

- **Impact if increased**: "Turbo Boost" sensitivity when load rises sharply.
- **Impact if decreased**: More conservative trend adaptation.

### `alpha_smooth` (Default: `0.6`)

**Purpose**: EMA factor for smoothing PSI load delta.

**Formula/Logic**:
```rust
delta_smooth = (current_delta * alpha) + ((1 - alpha) * prev_smooth);
```

- **Impact if increased**: Responsive but noisy (>0.8).
- **Impact if decreased**: Smooth but laggy (<0.3).

### `sigmoid_k` (Default: `0.20`)

**Purpose**: Steepness of the latency transition curve from max to min.

**Formula/Logic**:
```rust
factor = (pressure - mid).exp() ^ k
```

- **Impact if increased**: Sharp transition like On/Off.
- **Impact if decreased**: Smooth/gradual transition.

### `sigmoid_mid` (Default: `30.0`)

**Purpose**: PSI pressure midpoint for performance mode (low latency).

**Formula/Logic**:
```rust
offset = pressure - tunables.sigmoid_mid
```

- **Impact if increased**: Responsive mode requires heavy load, saves battery under light-medium load.
- **Impact if decreased**: Mode activates early, snappy UI but power hungry.

### `lookahead_time` (Default: `0.06` seconds)

**Purpose**: Prediction window for future load via linear regression.

**Formula/Logic**:
```rust
slope = regression_slope(history);
prediction = current_psi + (slope * tunables.lookahead_time);
```

- **Impact if increased**: Far prediction, good for stable loads (video rendering).
- **Impact if decreased**: Short prediction, accurate for random loads (gaming).

### `variance_sensitivity` (Default: `0.10`)

**Purpose**: PID gain multiplier based on historical fluctuation std_dev.

**Formula/Logic**:
```rust
gain_multiplier = 1.0 + (tunables.variance_sensitivity * std_dev);
```

- **Impact if increased**: Aggressive on unstable loads (open-world games).
- **Impact if decreased**: Less sensitive to noise.

### `efficiency_gain` (Default: `2.5`)

**Purpose**: Frequency penalty during memory/IO stalls.

**Formula/Logic**:
```rust
throughput_ratio = 1.0 / (1.0 + (stall_ratio * tunables.efficiency_gain));
```

- **Impact if increased**: Frequency held back when RAM/disk is busy, saves battery.
- **Impact if decreased**: Brute force frequency even when data isn't ready.

### `trend_amplification` (Default: `0.10`)

**Purpose**: Load amplifier based on short-medium term trends.

**Formula/Logic**:
```rust
p_response = load_demand * (1.0 + trend_factor * tunables.trend_amplification);
```

- **Impact if increased**: Overreacts to new load.
- **Impact if decreased**: More conservative trend response.

### `surge_threshold` (Default: `40.0`)

**Purpose**: Threshold for instant load surge detection.

**Formula/Logic**:
```rust
if load_rate > surge_threshold {
    rate += rate * surge_gain;
}
```

- **Impact if increased**: Surge mode difficult to activate.
- **Impact if decreased**: Surge activates easily, good for anti-stutter animations.

### `surge_gain` (Default: `0.30`)

**Purpose**: Rate boost when surge is detected.

**Formula/Logic**:
```rust
if load_rate > surge_threshold {
    rate += rate * surge_gain;
}
```

- **Impact if increased**: Large frequency boost during surge.
- **Impact if decreased**: Weak surge effect.

### `transient_rate_threshold` (Default: `0.25`)

**Purpose**: Rate threshold for transient detection.

**Formula/Logic**:
```rust
if rate > rate_thresh || diff > diff_thresh {
    // Transient Mode Active
}
```

- **Impact if increased**: Transient difficult to detect.
- **Impact if decreased**: Responsive to small changes.

### `transient_diff_threshold` (Default: `1.5`)

**Purpose**: Diff threshold for transient detection.

**Formula/Logic**:
```rust
if rate > rate_thresh || diff > diff_thresh {
    // Transient Mode Active
}
```

- **Impact if increased**: Transient difficult to detect.
- **Impact if decreased**: Responsive to small changes.

### `transient_poll_interval` (Default: `50.0` ms)

**Purpose**: Forced polling interval during transient.

**Formula/Logic**: Directly sets next_wake_ms to this value during transient.

- **Impact if increased**: Infrequent polling, saves power but slow response.
- **Impact if decreased**: Maximum responsiveness during app transitions.

### `nis_threshold` (Default: `8.0`)

**Purpose**: NIS Kalman Filter threshold for reset (structural break).

**Formula/Logic**:
```rust
if nis > threshold {
    // Reset Kalman Filter
}
```

- **Impact if increased**: Filter tolerates noise.
- **Impact if decreased**: Sensitive, fast response but unstable.

### `safe_temp_limit` (Default: `60.0`)

**Purpose**: Safe temperature threshold to start calculating integral throttling.

**Formula/Logic**:
```rust
limit_violation = max(0, cpu_temp - safe_temp_limit);
```

- **Impact if increased**: Throttling starts later (hotter).
- **Impact if decreased**: Throttling starts early (cooler).

### `max_temp_limit` (Default: `80.0`)

**Purpose**: Maximum temperature threshold for thermal cost normalization.

**Formula/Logic**:
```rust
limit_violation = max(0, cpu_temp - safe_temp_limit);
```

- **Impact if increased**: Shallow thermal cost curve, allows hotter operation.
- **Impact if decreased**: Steep curve, aggressive throttling.

### `temp_cost_weight` (Default: `5.0`)

**Purpose**: CPU temperature penalty weight in integral.

**Formula/Logic**:
```rust
term_cpu = weight * (temp / max_limit).powi(2);
```

- **Impact if increased**: Large heat penalty, strong throttling.
- **Impact if decreased**: Heat tolerant, higher performance.

### `bat_temp_weight` (Default: `4.0`)

**Purpose**: Battery temperature penalty weight in integral.

**Formula/Logic**:
```rust
term_bat_temp = weight * (bat_temp / 45.0).clamp(0.0, 1.0);
```

- **Impact if increased**: Large battery temperature penalty, protects battery.
- **Impact if decreased**: Tolerates battery temperature.

### `bat_level_weight` (Default: `60.0`)

**Purpose**: Low battery level penalty weight.

**Formula/Logic**:
```rust
depletion = (100 - level) / 100;
term_bat = weight * depletion.powi(3);
```

- **Impact if increased**: Performance cut when battery is low.
- **Impact if decreased**: Performance remains high even when battery is low.

### `integral_acc_rate` (Default: `0.2`)

**Purpose**: Throttling accumulation rate when temperature is unsafe.

**Formula/Logic**:
```rust
accumulated_throttle += tunables.integral_acc_rate * limit_violation;
```

- **Impact if increased**: Throttling accumulates quickly when hot.
- **Impact if decreased**: Slow accumulation, risk of overheat.

### `memory_migration_alpha` (Default: `1.5`)

**Purpose**: Additional cost for task migration when memory is full.

**Formula/Logic**:
```rust
cost_scale = 1.0 + (tunables.memory_migration_alpha * (psi_memory / 100));
```

- **Impact if increased**: Tasks locked to core when RAM is busy.
- **Impact if decreased**: Migration remains active even when memory is full.

### `memory_granularity_scaling` (Default: `0.8`)

**Purpose**: Scales latency during high memory pressure to prevent thrashing.

**Formula/Logic**:
```rust
scaling = 1.0 + (tunables.memory_granularity_scaling * (psi_memory / 100.0));
final_latency *= scaling;
```

- **Impact if increased**: Aggressively long latency when RAM is full, reduces context switches.
- **Impact if decreased**: Ignores memory, risk of cache misses.

### `memory_volatility_cost` (Default: `1.5`)

**Purpose**: Reduces trend sensitivity when memory is unstable.

**Formula/Logic**:
```rust
trend_gain = base_gain / (1.0 + (memory_psi * tunables.memory_volatility_cost));
```

- **Impact if increased**: Anti-overreact to spurious spikes from swapping.
- **Impact if decreased**: Sensitive to trends even when memory is volatile.

---

## ThermalTunables
→ [`cpu_impl.rs`](../core/src/controllers/cpu_impl.rs)

### `hard_limit_cpu` (Default: `70.0`)

**Purpose**: Absolute CPU temperature threshold for PID target.

**Formula/Logic**:
```rust
error = current_temp - tunables.hard_limit_cpu;
```

- **Impact if increased**: Allows device to run hotter.
- **Impact if decreased**: Earlier throttling.

### `hard_limit_bat` (Default: `40.0`)

**Purpose**: Battery temperature threshold for PID setpoint modification.

**Formula/Logic**:
```rust
bat_margin = (tunables.hard_limit_bat - bat_temp).max(0.0);
control_margin = if bat_margin < 5.0 { 5.0 - bat_margin } else { 0.0 };
setpoint = tunables.hard_limit_cpu - control_margin;
```

- **Impact if increased**: Battery runs hotter before aggressive throttling.
- **Impact if decreased**: Early throttling protects battery.

### `sched_temp_cool` (Default: `30.0`)

**Purpose**: Lower bound of battery temperature range for PID interpolation.

**Formula/Logic**:
```rust
sigma = ((bat_temp - tunables.sched_temp_cool) / (tunables.sched_temp_hot - tunables.sched_temp_cool)).clamp(0.0, 1.0);
```

- **Impact if increased**: Narrow range, fast PID transition.
- **Impact if decreased**: Wide range, gradual transition.

### `sched_temp_hot` (Default: `40.0`)

**Purpose**: Upper bound of battery temperature range for PID interpolation.

**Formula/Logic**:
```rust
sigma = ((bat_temp - tunables.sched_temp_cool) / (tunables.sched_temp_hot - tunables.sched_temp_cool)).clamp(0.0, 1.0);
```

- **Impact if increased**: Wide range, gradual transition.
- **Impact if decreased**: Narrow range, fast transition.

### `kp_base` (Default: `1.5`)

**Purpose**: Base proportional gain for instant response.

**Formula/Logic**:
```rust
k_p = tunables.kp_base + sigma * (tunables.kp_fast - tunables.kp_base);
p_term = k_p * error;
```

- **Impact if increased**: Strong braking when temperature rises.
- **Impact if decreased**: Weak response.

### `ki_base` (Default: `0.02`)

**Purpose**: Base integral gain for error accumulation.

**Formula/Logic**:
```rust
k_i = tunables.ki_base + sigma * (tunables.ki_fast - tunables.ki_base);
i_increment = k_i * error * dt_safe;
integral_accum += i_increment;
```

- **Impact if increased**: Fast throttling accumulation on sustained error.
- **Impact if decreased**: Slow integral response, reduces overshoot.

### `kd_base` (Default: `0.5`)

**Purpose**: Base derivative gain for error change.

**Formula/Logic**:
```rust
k_d = tunables.kd_base + sigma * (tunables.kd_fast - tunables.kd_base);
t_d = if k_p > 1e-6 { k_d / k_p } else { 0.0 };
d_term = // filter calculation
```

- **Impact if increased**: Sensitive to temperature changes, reduces oscillation.
- **Impact if decreased**: Less responsive to fluctuations, oscillation risk.

### `kp_fast` (Default: `5.0`)

**Purpose**: Fast proportional gain for hot mode.

**Formula/Logic**: Interpolated with kp_base via sigma.

- **Impact if increased**: Very strong braking in hot mode.
- **Impact if decreased**: Weak response in hot mode.

### `ki_fast` (Default: `0.10`)

**Purpose**: Fast integral gain for hot mode.

**Formula/Logic**: Interpolated with ki_base.

- **Impact if increased**: Fast accumulation in hot mode.
- **Impact if decreased**: Slow, reduces overshoot.

### `kd_fast` (Default: `3.0`)

**Purpose**: Fast derivative gain for hot mode.

**Formula/Logic**: Interpolated with kd_base.

- **Impact if increased**: Sensitive in hot mode, reduces oscillation.
- **Impact if decreased**: Oscillation risk in hot mode.

### `anti_windup_k` (Default: `0.8`)

**Purpose**: Anti-windup integral coefficient during saturation.

**Formula/Logic**:
```rust
integral -= excess * k * dt;
```

- **Impact if increased**: Integral discarded quickly, fast performance recovery when temperature drops.
- **Impact if decreased**: Windup persists, slow recovery.

### `deriv_filter_n` (Default: `10.0`)

**Purpose**: Derivative low-pass filter coefficient.

**Formula/Logic**:
```rust
denominator = t_d + n * dt;
```

- **Impact if increased**: Pure derivative signal but noisy.
- **Impact if decreased**: Strong filter, slow response.

### `ff_gain` (Default: `1.5`)

**Purpose**: Feedforward load prediction gain.

**Formula/Logic**:
```rust
u_ff = feedforward.update(psi_load, dt_safe, ff_gain, ff_lead_time, ff_lag_time);
```

- **Impact if increased**: Aggressive prediction, increased responsiveness.
- **Impact if decreased**: Conservative prediction, reduces overshoot.

### `ff_lead_time` (Default: `4.0`)

**Purpose**: Lead time in feedforward lead-lag filter.

**Formula/Logic**: Part of lead-lag filter formula.

- **Impact if increased**: Initial surge response is stronger.
- **Impact if decreased**: Weak initial response.

### `ff_lag_time` (Default: `2.0`)

**Purpose**: Lag time in feedforward lead-lag filter.

**Formula/Logic**: Part of filter formula.

- **Impact if increased**: Braking effect persists longer.
- **Impact if decreased**: Effect dissipates quickly.

### `smith_gain` (Default: `1.0`)

**Purpose**: Smith Predictor model sensitivity.

**Formula/Logic**:
```rust
y_no_delay = alpha * (u_control * k_gain) + (1.0 - alpha) * model_output_no_delay;
```

- **Impact if increased**: Predicts larger throttling effect.
- **Impact if decreased**: Weak prediction.

### `smith_tau` (Default: `10.0`)

**Purpose**: Heatsink thermal time constant in Smith Predictor.

**Formula/Logic**:
```rust
alpha = dt / (tau + dt);
```

- **Impact if increased**: Assumes thick material (slow heat/cool).
- **Impact if decreased**: Assumes thin material (fast heat).

### `smith_delay_sec` (Default: `3.0`)

**Purpose**: Dead time estimation for heat propagation.

**Formula/Logic**: Uses history for delayed prediction.

- **Impact if increased**: Assumes long delay, early throttling.
- **Impact if decreased**: Oscillation risk if actual delay is large.

---

## MemoryTunables
→ [`memory_impl.rs`](../core/src/controllers/memory_impl.rs)

### `pressure_kp` (Default: `0.8`)

**Purpose**: Proportional gain for increasing swappiness.

**Formula/Logic**:
```rust
target_swap = base + (pressure_kp * psi_memory);
```

- **Impact if increased**: Aggressively moves memory to ZRAM.
- **Impact if decreased**: Swappiness increases slowly.

### `pressure_kd` (Default: `0.2`)

**Purpose**: Derivative gain for swappiness.

**Formula/Logic**:
```rust
d_term = pressure_kd * dp_dt;
```

- **Impact if increased**: Sensitive to pressure changes, reduces oscillation.
- **Impact if decreased**: Less responsive to fluctuations.

### `inefficiency_cost` (Default: `25.0`)

**Purpose**: Swappiness penalty if reclaim efficiency is low.

**Formula/Logic**:
```rust
cost = inefficiency_cost * (1.0 - (pgsteal / pgscan));
```

- **Impact if increased**: Switches to anonymous swap if file cache is difficult.
- **Impact if decreased**: Tolerates inefficiency.

### `pressure_vfs_k` (Default: `0.10`)

**Purpose**: Exponential decay of vfs_cache_pressure.

**Formula/Logic**:
```rust
vfs_pressure = min + (range * (1.0 - exp(-k * pressure)));
```

- **Impact if increased**: VFS rises sharply when memory is full, aggressively discards inode cache.
- **Impact if decreased**: Gradual increase.

### `fragmentation_impact_k` (Default: `2.0`)

**Purpose**: Fragmentation impact on watermark scale.

**Formula/Logic**:
```rust
impact = k * fragmentation * pressure;
```

- **Impact if increased**: Kswapd wakes early if RAM is fragmented.
- **Impact if decreased**: Ignores fragmentation.

### `wss_cost_factor` (Default: `3.0`)

**Purpose**: Working set protection based on refault.

**Formula/Logic**:
```rust
preservation = 1.0 - (refault_risk * wss_cost_factor).powi(2);
```

- **Impact if increased**: Highly protective, prevents swap if data is frequently accessed.
- **Impact if decreased**: Less protective.

### `zram_thermal_cost` (Default: `1.5`)

**Purpose**: Reduces ZRAM when CPU is hot.

**Formula/Logic**:
```rust
throttle = 1.0 - (temp_stress * zram_thermal_cost);
```

- **Impact if increased**: Prioritizes CPU temperature over ZRAM compression.
- **Impact if decreased**: Tolerates heat for ZRAM.

### `general_smooth_factor` (Default: `0.20`)

**Purpose**: EMA smoothing for general output.

**Formula/Logic**: smooth_value(current, target, alpha)

- **Impact if increased**: Sysfs values change quickly/roughly.
- **Impact if decreased**: Smooth but slow.

### `watermark_smooth_factor` (Default: `0.1`)

**Purpose**: Specific smoothing for watermark_scale_factor.

**Formula/Logic**: smooth_value for watermark.

- **Impact if increased**: Fast watermark changes.
- **Impact if decreased**: Slow changes.

### `queue_history_size` (Default: `16`)

**Purpose**: Number of queue history samples for variability analysis.

**Formula/Logic**: VecDeque with this capacity.

- **Impact if increased**: More accurate variability analysis but memory intensive.
- **Impact if decreased**: Fast analysis but less accurate.

### `queue_smoothing_alpha` (Default: `0.2`)

**Purpose**: Queue rate statistics smoothing.

**Formula/Logic**:
```rust
smoothed_rate = alpha * raw + (1 - alpha) * prev;
```

- **Impact if increased**: Responsive to queue changes.
- **Impact if decreased**: Smooth but slow adaptation.

### `residence_time_threshold` (Default: `30.0`)

**Purpose**: Target minimum page residence time in RAM to prevent thrashing.

**Formula/Logic**:
```rust
risk_ratio = threshold / residence_time;
```

- **Impact if increased**: Paranoid, considers pages <30s as dangerous.
- **Impact if decreased**: Tolerates thrashing.

### `protection_curve_k` (Default: `3.0`)

**Purpose**: Residence time protection curve exponent.

**Formula/Logic**:
```rust
protection = 1.0 / (1.0 + risk.powi(k));
```

- **Impact if increased**: Sharp protection after threshold.
- **Impact if decreased**: Gradual protection.

### `congestion_scaling_factor` (Default: `2.5`)

**Purpose**: Stability penalty based on scan rate CV.

**Formula/Logic**:
```rust
variability = 1.0 + (cv * scaling);
target /= variability;
```

- **Impact if increased**: Target pressure drops drastically if unstable.
- **Impact if decreased**: Tolerates variability.

---

## StorageTunables
→ [`storage_impl.rs`](../core/src/controllers/storage_impl.rs)

### `write_cost_factor` (Default: `5.0`)

**Purpose**: Write vs read cost weight.

**Formula/Logic**:
```rust
effective_load = read_bw + (write_bw * write_cost_factor);
```

- **Impact if increased**: Sensitive to background writes.
- **Impact if decreased**: Tolerates writes.

### `target_latency_base_ms` (Default: `75.0`)

**Purpose**: Base I/O latency target for queue optimization.

**Formula/Logic**: Gradient descent for nr_requests.

- **Impact if increased**: Long queue, increased throughput.
- **Impact if decreased**: Short queue, responsive.

### `hysteresis_threshold` (Default: `0.15`)

**Purpose**: Change tolerance before updating nr_requests.

**Formula/Logic**:
```rust
error_ratio = abs(calc - current) / current;
```

- **Impact if increased**: Stable queue depth, changes infrequently.
- **Impact if decreased**: Changes frequently, jittery.

### `critical_threshold_psi` (Default: `40.0`)

**Purpose**: I/O PSI threshold for panic mode.

**Formula/Logic**:
```rust
if psi > critical {
    queue_depth = min_nr_requests;
}
```

- **Impact if increased**: Panic mode difficult to activate.
- **Impact if decreased**: Early panic.

### `queue_pressure_low` (Default: `1.0`)

**Purpose**: In-flight lower bound for pressure normalization.

**Formula/Logic**:
```rust
ratio = (in_flight - low) / (high - low);
```

- **Impact if increased**: Queue considered congested later.
- **Impact if decreased**: Queue congested earlier.

### `queue_pressure_high` (Default: `4.0`)

**Purpose**: In-flight upper bound for normalization.

**Formula/Logic**: Same as low.

- **Impact if increased**: Tolerates long queues.
- **Impact if decreased**: Long queues considered critical earlier.

### `smoothing_factor` (Default: `0.25`)

**Purpose**: EMA for sequential pattern detection.

**Formula/Logic**:
```rust
smoothed = (raw * alpha) + (prev * (1 - alpha));
```

- **Impact if increased**: Fast pattern detection.
- **Impact if decreased**: Smooth but slow detection.

---

## Constraints
→ [`tunables.rs`](../core/src/config/tunables.rs)

This section defines "Hard Limits" or safe operational boundaries. Dynamic algorithms will calculate target values based on load, but the final results will always be clamped within these ranges to maintain system stability.

### **CPU Constraints**

These parameters limit the dynamic range for CPU scheduler and task scheduling.

#### `SCHED_LATENCY_NS`

* **Range**: `8,000,000` (8ms) - `24,000,000` (24ms)
* **Unit**: Nanoseconds
* **Description**: Scheduler rotation period.
* **Logic**: Algorithm will lower value to 8ms under high load (for responsiveness) and raise to 24ms when idle (for throughput/battery).

#### `SCHED_MIN_GRANULARITY_NS`

* **Range**: `6,000,000` (6ms) - `18,000,000` (18ms)
* **Unit**: Nanoseconds
* **Description**: Minimum guaranteed execution time for a task before it can be preempted. Prevents excessive context switching (thrashing) on CPU.

#### `SCHED_WAKEUP_GRANULARITY_NS`

* **Range**: `3,000,000` (3ms) - `6,000,000` (6ms)
* **Unit**: Nanoseconds
* **Description**: Resistance to preemption on wakeup.
* **Low**: Newly awakened tasks will immediately displace running tasks (Snappy).
* **High**: Reduces interruption to currently running tasks (Stable).

#### `SCHED_MIGRATION_COST_NS`

* **Range**: `200,000` (0.2ms) - `600,000` (0.6ms)
* **Unit**: Nanoseconds
* **Description**: Estimated "cost" of lost cache time when moving tasks between cores. Higher values make tasks more "sticky" to the current core.

#### `SCHED_NR_MIGRATE`

* **Range**: `8` - `32`
* **Unit**: Tasks
* **Description**: Maximum number of tasks allowed to migrate in a single load balancing operation. Limits the duration a core holds the lock (`rq->lock`).

#### `SCHED_WALT_INIT_TASK_LOAD_PCT`

* **Range**: `15` - `45`
* **Unit**: Percent (%)
* **Description**: Initial load value assigned to new tasks (specific to WALT/Qualcomm scheduler).
* **Logic**: Higher values make new tasks appear "heavy" from the start, so the scheduler immediately places them on big cores/high frequency (helps with app launch).

#### `SCHED_UCLAMP_MIN`

* **Range**: `0` - `256` (0% - 25% approx on scale 1024)
* **Unit**: Utilisation Value
* **Description**: Lower bound for CPU frequency *clamping*. Dynamic value calculated based on thermal pressure and CPU pressure.

### **Memory Constraints**

These parameters limit how aggressively virtual memory (VM) management and dirty page cache management operate.

#### `SWAPPINESS`

* **Range**: `20` - `60`
* **Description**: Kernel's tendency to swap anonymous memory to ZRAM/Swap.
* **Logic**: When memory pressure is low, value approaches 20. When pressure is high and CPU is cool, value approaches 60.

#### `VFS_CACHE_PRESSURE`

* **Range**: `80` - `200`
* **Description**: Controls reclaim of inode/dentry cache vs page cache.
* **>100**: Kernel is more aggressive in discarding file structure cache (inodes) to free RAM.

#### `DIRTY_RATIO` & `DIRTY_BACKGROUND_RATIO`

* **Total Ratio Range**: `10%` - `20%`
* **Background Range**: `5%` - `10%`
* **Description**: Dirty memory limit (data not yet written to disk) before forced cleaning. This range is tightly controlled to prevent *IO Stutter* during large data flushes.

#### `DIRTY_EXPIRE_CENTISECS`

* **Range**: `1000` (10s) - `2000` (20s)
* **Unit**: Centiseconds
* **Description**: Maximum age of dirty data in RAM before it's considered expired and must be written to disk.

#### `DIRTY_WRITEBACK_CENTISECS`

* **Range**: `300` (3s) - `1000` (10s)
* **Unit**: Centiseconds
* **Description**: Interval for background flusher thread to wake up and write dirty data to disk.
* **Logic**: Short interval (3s) when memory is full to prevent accumulation, long interval (10s) when idle to save battery.

#### `WATERMARK_SCALE_FACTOR`

* **Range**: `8` (0.08%) - `15` (0.15%)
* **Description**: Controls how early `kswapd` wakes up before memory is actually exhausted. Dynamic value increases when memory fragmentation is high.

#### `EXTFRAG_THRESHOLD`

* **Range**: `400` - `600`
* **Description**: External fragmentation index threshold. Determines when kernel should perform *compaction* rather than discarding cache.

#### `VM_STAT_INTERVAL`

* **Range**: `1` - `5`
* **Unit**: Seconds
* **Description**: Update interval for VM (virtual memory) statistics.
* **Logic**: 1 second when memory pressure is high (needs accurate data), 5 seconds when normal (saves overhead).

#### `PAGE_CLUSTER`

* **Range**: `0` - `1`
* **Unit**: Power of 2 (Pages)
* **Description**: Number of pages read at once during swap-in (prefetch).
* `0` (2^0 = 1 page): Optimal for ZRAM/ZSWAP (low latency random access).
* `1` (2^1 = 2 pages): Slight prefetch if CPU load is low.

### **Storage Constraints**

These parameters control block device I/O queue (MMC/UFS).

#### `READ_AHEAD_KB`

* **Range**: `128 KB` - `256 KB`
* **Description**: Amount of data read ahead (prefetch) into cache.
* **Logic**: Increases when sequential access pattern is detected, decreases for random patterns (random IO).

#### `NR_REQUESTS`

* **Range**: `128` - `256`
* **Description**: I/O scheduler queue depth.
* **Low**: Low latency for UI (random read).
* **High**: High throughput for file copying (sequential write).

#### `FIFO_BATCH`

* **Range**: `8` - `16`
* **Description**: Number of requests processed in one batch (for `deadline` or `mq-deadline` scheduler) before switching direction (read vs write).