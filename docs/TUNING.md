# Parameter Configuration

---

## Device Tier Awareness

QoS automatically adapts many parameters based on **DeviceTier** detection (`Flagship`, `MidRange`, or `LowEnd`). Detection occurs once at startup in `tier.rs` by evaluating:
- Maximum CPU frequency (`cpuinfo_max_freq`)
- Number of big cores (≥ 2.1 GHz)
- Total RAM (in MB)

**Flagship** values prioritize performance and responsiveness. **MidRange** and **LowEnd** values are progressively more conservative to maintain stability and thermal headroom on weaker hardware.

Defaults shown below are for **Flagship** devices. Refer to the source files (`cpu_math.rs`, `thermal_math.rs`, `storage_math.rs`) for exact values in other tiers.

**Important**: All changes require recompiling the binary and repackaging the Magisk module.

---

## CpuMathConfig

**Source**: `core/src/algorithms/cpu_math.rs`

Controls Kalman-filtered load tracking, PID-style response, uClamp boosting, latency/granularity scheduling, WALT initialization, migration cost, and transient/surge handling.

### `latency_gran_ratio` (Default: `0.32`)
**Purpose**: Determines scheduler granularity as a fraction of the computed target latency.  
**Logic**:
```rust
raw_gran = adjusted_latency * latency_gran_ratio;
final_gran = raw_gran.clamp(min_granularity_ns, max_granularity_ns).min(adjusted_latency);
```
- Increasing → coarser granularity → better throughput, higher latency risk.  
- Decreasing → finer granularity → improved interactivity, higher context-switch overhead.

### `decay_coeff` (Default: `0.014`)
**Purpose**: Controls decay rate of wakeup granularity as effective pressure rises.  
**Logic**:
```rust
x = decay_coeff * p_eff;
decay = 1.0 / (1.0 + x + 0.5 * x * x);
raw_wake = min_wakeup_ns + range * decay;
```
- Increasing → faster reduction in wakeup latency under load.  
- Decreasing → slower, more gradual decay.

### `uclamp_k` (Default: `0.21`)
**Purpose**: Steepness of the sigmoid curve for minimum utilization (uClamp) boost.  
**Logic**:
```rust
x = uclamp_k * (pressure - uclamp_mid);
sigmoid_val = 0.5 * (x / (1.0 + x.abs()) + 1.0);
ideal_uclamp = min_uclamp_min + range * sigmoid_val;
final_uclamp = (ideal_uclamp * thermal_scale).clamp(...);
```
- Increasing → sharper transition from low to high boost.  
- Decreasing → smoother, near-linear boost curve.

### `uclamp_mid` (Default: `7.5`)
**Purpose**: PSI pressure midpoint where uClamp boost reaches approximately 50% effectiveness.  
- Increasing → boost activates later (under heavier load).  
- Decreasing → earlier activation (more aggressive boosting).

### `response_gain` (Default: `36.0`)
**Purpose**: Base proportional gain in the load-demand tracking controller.  
**Logic**:
```rust
k_base = response_gain;
k_dynamic = k_base * (1.0 + gain_scheduling_alpha * trend_factor);
k_final = k_dynamic * thermal_scale.powi(2);
prop_term = k_final * displacement;
```
- Increasing → faster reaction to load changes.  
- Decreasing → smoother, more stable response.

### `stability_ratio` (Default: `2.05`)
**Purpose**: Multiplier for critical damping to achieve overdamped behavior and reduce oscillation.  
**Logic**:
```rust
crit_damp = 2.0 * k_final.sqrt();
base_damp = crit_damp * stability_ratio;
c_final = max(c_thermal_adjusted, stability_damping_req * stability_margin);
```
- Increasing → stronger damping → more stable but slower settling.  
- Decreasing → closer to critical damping → faster response with mild ringing risk.

### `stability_margin` (Default: `4.0`)
**Purpose**: Safety multiplier for the rate-derived stability damping term.  
- Increasing → stronger penalty on rapid rate changes.  
- Decreasing → permits faster rate adjustments.

### `gain_scheduling_alpha` (Default: `0.982`)
**Purpose**: Adaptive scaling of response gain based on load trend (velocity).  
**Logic**:
```rust
k_dynamic = k_base * (1.0 + gain_scheduling_alpha * trend_factor);
```
- Increasing → stronger gain boost during accelerating load.  
- Decreasing → weaker adaptation to trend.

### `sigmoid_k` (Default: `0.085`)
**Purpose**: Steepness of the sigmoid used for latency scaling.  
**Logic**:
```rust
sigmoid_val = sigmoid_param(p_eff, sigmoid_k, sigmoid_mid);
factor = 1.0 - sigmoid_val;
normal_latency = min_latency_ns + range * factor;
```
- Increasing → more abrupt switch between power-saving and performance latency regimes.  
- Decreasing → smoother transition.

### `sigmoid_mid` (Default: `5.5`)
**Purpose**: PSI pressure midpoint for latency sigmoid transition.  
- Increasing → stays in high-latency (power-saving) mode longer.  
- Decreasing → switches to low latency sooner.

### `lookahead_time` (Default: `0.14`)
**Purpose**: Prediction horizon (seconds) for future PSI based on current velocity.  
**Logic**:
```rust
prediction_target = target_psi + load_rate * lookahead_time;
```
- Increasing → earlier reaction to rising pressure (risk of overshoot).  
- Decreasing → primarily reacts to current measured pressure.

### `efficiency_gain` (Default: `6.2`)
**Purpose**: Strength of I/O stall penalty in effective pressure calculation.  
**Logic**:
```rust
ratio_stall = io_psi / (load_demand + 1.0);
throughput_ratio = 1.0 / (1.0 + ratio_stall * efficiency_gain);
p_eff = p_response * throughput_ratio;
```
- Increasing → stronger CPU demand throttling when I/O is bottlenecked.  
- Decreasing → less sensitivity to I/O stalls.

### `trend_amplification` (Default: `0.135`)
**Purpose**: Amplification of tanh-transformed velocity in effective pressure.  
**Logic**:
```rust
p_response = load_demand * (1.0 + trend_factor * trend_amplification);
```
- Increasing → exaggerates response to upward load trends.  
- Decreasing → closer to linear response.

### `surge_threshold` (Default: `15.0`)
**Purpose**: Absolute velocity threshold triggering surge-mode rate boost.  
**Logic**:
```rust
if load_rate.abs() > surge_threshold {
    state.rate += load_rate * surge_gain;
}
```
- Increasing → surge mode activates less frequently.  
- Decreasing → more frequent activation on moderate spikes.

### `surge_gain` (Default: `0.115`)
**Purpose**: Additional rate gain during surge conditions.  
- Increasing → stronger instantaneous rate boost on spikes.  
- Decreasing → milder surge effect.

### `transient_rate_threshold` (Default: `0.095`)
**Purpose**: Rate threshold for transient (burst) state detection.  
**Logic**:
```rust
is_transient = rate.abs() > transient_rate_threshold || (psi_value - target_psi).abs() > transient_diff_threshold;
```
- Increasing → ignores smaller fluctuations.  
- Decreasing → detects micro-bursts as transients.

### `transient_diff_threshold` (Default: `0.45`)
**Purpose**: PSI error magnitude threshold for transient detection.  
- Increasing → requires larger deviation to trigger fast polling.  
- Decreasing → higher sensitivity.

### `transient_poll_interval` (Default: `45.0`)
**Purpose**: Maximum polling interval (ms) enforced during transient states.  
**Logic**:
```rust
calculated_poll = calculated_poll.min(transient_poll_interval as i32);
```
- Increasing → slower updates during bursts.  
- Decreasing → very frequent polling (higher CPU overhead).

### `nis_threshold` (Default: `6.5`)
**Purpose**: Normalized Innovation Squared threshold for Kalman filter structural break reset.  
**Logic**:
```rust
is_structural_break = nis > nis_threshold;
```
- Increasing → filter more "sticky", ignores noise.  
- Decreasing → frequent resets on anomalies.

### `bat_level_weight` (Default: `94.0`)
**Purpose**: Scaling factor for battery depletion cost heuristic.  
**Logic**:
```rust
depletion = (100.0 - bat_level).max(0.0) / 100.0;
cost_heuristic = bat_level_weight * depletion.powi(3);
```
- Increasing → more aggressive throttling as battery drops.  
- Decreasing → weaker battery influence.

### Helper Functions (CpuMathConfig)
- `sigmoid_param(val, k, mid)` → Smooth sigmoid transition (used in latency & uClamp).  
- `decay(val, coeff)` → Quadratic decay approximation (used for wakeup granularity).  
- `tanh(x)` → Hyperbolic tangent for trend amplification.

---

## ThermalConfig

**Source**: `core/src/algorithms/thermal_math.rs`

Tiered PID controller with Smith Predictor (dead-time compensation) and Lead-Lag feed-forward.

Flagship defaults shown.

### `hard_limit_cpu` (Default: `56.0 °C`)
**Purpose**: Primary CPU temperature setpoint.  
**Logic**:
```rust
setpoint = hard_limit_cpu - control_margin;
error = adjusted_pv - setpoint;
```
- Increasing → permits higher operating temperature.  
- Decreasing → earlier throttling.

### `hard_limit_bat` (Default: `42.5 °C`)
**Purpose**: Battery temperature threshold that reduces CPU setpoint.  
**Logic**:
```rust
bat_margin = (hard_limit_bat - bat_temp).max(0.0);
control_margin = (5.0 - bat_margin).max(0.0);
setpoint = hard_limit_cpu - control_margin;
```
- Increasing → allows battery to heat more.  
- Decreasing → stronger battery protection.

### `sched_temp_cool` (Default: `24.0 °C`) / `sched_temp_hot` (Default: `49.0 °C`)
**Purpose**: Battery temperature range for interpolating PID gains (cool → hot).  
**Logic**:
```rust
sigma = ((bat_temp - sched_temp_cool) / (sched_temp_hot - sched_temp_cool)).clamp(0.0, 1.0);
kp = kp_base + sigma * (kp_fast - kp_base);
```

### PID Gains (Flagship)
- `kp_base: 0.72`, `kp_fast: 4.6`  
- `ki_base: 0.014`, `ki_fast: 0.068`  
- `kd_base: 1.05`, `kd_fast: 4.2`

Higher values increase reaction speed/strength at hotter battery temperatures. **PID gains are linearly interpolated** based on battery temperature between `sched_temp_cool` and `sched_temp_hot`.

### `anti_windup_k` (Default: `1.75`)
**Purpose**: Coefficient for conditional integration anti-windup on saturation.  
**Logic**:
```rust
if saturated {
    integral_accum -= excess * anti_windup_k * dt_safe;
}
```
- Increasing → faster integral recovery after saturation.  
- Decreasing → slower recovery.

### `deriv_filter_n` (Default: `21.0`)
**Purpose**: Low-pass filter coefficient for derivative term.  
**Logic**:
```rust
denominator = t_d + deriv_filter_n * dt_safe;
d_term = alpha * prev_deriv_output + beta * delta_pv;
```
- Increasing → smoother derivative (less noise).  
- Decreasing → faster but noisier derivative.

### `ff_gain` (Default: `3.1`), `ff_lead_time` (Default: `3.8`), `ff_lag_time` (Default: `1.9`)
**Purpose**: Lead-Lag feed-forward compensator parameters driven by PSI load.  
- Higher `ff_gain` → stronger proactive throttling.  
- Higher `ff_lead_time` → earlier anticipation.  
- Higher `ff_lag_time` → slower decay of anticipatory effect.

### `smith_gain` (Default: `1.75`), `smith_tau` (Default: `9.5`), `smith_delay_sec` (Default: `1.4`)
**Purpose**: Internal first-order model parameters for Smith Predictor.  
- `smith_gain`: scales predicted temperature change.  
- `smith_tau`: thermal time constant.  
- `smith_delay_sec`: estimated sensor/actuator delay.

---

## StorageMathConfig

**Source**: `core/src/algorithms/storage_math.rs`

Flagship defaults shown.

### `min_req_size_kb` (Default: `6.0`) / `max_req_size_kb` (Default: `768.0`)
**Purpose**: Range for normalizing average read request size into sequentiality ratio.

### `write_cost_factor` (Default: `3.5`)
**Purpose**: Relative cost weighting of writes vs reads.  
**Logic**:
```rust
weighted_throughput = throughput_read + write_cost_factor * throughput_write;
```

### `target_latency_base_ms` (Default: `30.0`)
**Purpose**: Baseline target latency scaled by PSI pressure.  
**Logic**:
```rust
target = target_latency_base_ms * (1.0 - (psi_some_avg10 / 100.0).clamp(0.0, 1.0));
```

### `hysteresis_threshold` (Default: `0.25`)
**Purpose**: Relative error ratio required to update `nr_requests`.  
**Logic**:
```rust
error_ratio > hysteresis_threshold || out_of_bounds;
```

### `critical_threshold_psi` (Default: `18.0`)
**Purpose**: PSI level forcing minimum queue depth (congestion panic mode).

### `queue_pressure_low` (Default: `0.15`) / `queue_pressure_high` (Default: `6.0`)
**Purpose**: Bounds for in-flight request pressure ratio.

### `smoothing_factor` (Default: `0.55`)
**Purpose**: EMA alpha for sequentiality smoothing.  
**Logic**:
```rust
smoothed = raw * smoothing_factor + old * (1.0 - smoothing_factor);
```

---

## CleanerConfig

**Source**: `core/src/controllers/cleaner_impl.rs`

Fixed values (not tiered).

### `sweep_interval_ms` (Default: `600_000`)
**Purpose**: Interval between cache cleaning cycles (10 minutes).

### `bloat_limit_bytes` (Default: `512 * 1024 * 1024`)
**Purpose**: Cache directory size triggering shorter retention age.

### `storage_critical_threshold` (Default: `10.0`)
**Purpose**: Free space percentage triggering emergency cleaning mode.

### Age Thresholds
- `age_stale_media`: 7 days  
- `age_stale_code`: 30 days  
- `age_bloat`: 1 day  
- `age_emergency`: 1 hour  
- `age_trash`: 1 hour  

These determine deletion eligibility for media, code cache, bloated folders, trash files, and emergency conditions.

---

## General Tuning Recommendations

- **More "snappy" / responsive**: Increase `response_gain`, decrease `surge_threshold`, `transient_poll_interval`, and `lookahead_time`.
- **Better battery & thermal efficiency**: Decrease `hard_limit_cpu`, increase `decay_coeff`, `stability_ratio`, and `anti_windup_k`.
- **Higher stability**: Increase `stability_margin` and `stability_ratio`, decrease `gain_scheduling_alpha`.
- **More aggressive toward I/O bottlenecks**: Increase `efficiency_gain`.
- **Always test** with heavy workloads (gaming + multitasking + intensive I/O) after changing parameters.
- Monitor temperature, PSI values, and logcat (`logcat -s QoS:V`) during testing.

---

**Note**: Parameter tuning affects the balance between performance, responsiveness, stability, and power efficiency. Change values gradually and test thoroughly.