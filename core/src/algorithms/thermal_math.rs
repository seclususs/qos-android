//! Author: [Seclususs](https://github.com/seclususs)

use crate::utils::tier::DeviceTier;

use std::time;

const SMITH_BUFFER_SIZE: usize = 512;

#[derive(Clone, Copy, Debug)]
pub struct ThermalConfig {
    pub hard_limit_cpu: f32,
    pub hard_limit_bat: f32,
    pub sched_temp_cool: f32,
    pub sched_temp_hot: f32,
    pub kp_base: f32,
    pub ki_base: f32,
    pub kd_base: f32,
    pub kp_fast: f32,
    pub ki_fast: f32,
    pub kd_fast: f32,
    pub anti_windup_k: f32,
    pub deriv_filter_n: f32,
    pub ff_gain: f32,
    pub ff_lead_time: f32,
    pub ff_lag_time: f32,
    pub smith_gain: f32,
    pub smith_tau: f32,
    pub smith_delay_sec: f32,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        let tier = DeviceTier::get();
        match tier {
            DeviceTier::Flagship => Self {
                hard_limit_cpu: 56.0,
                hard_limit_bat: 42.5,
                sched_temp_cool: 24.0,
                sched_temp_hot: 49.0,
                kp_base: 0.72,
                ki_base: 0.014,
                kd_base: 1.05,
                kp_fast: 4.6,
                ki_fast: 0.068,
                kd_fast: 4.2,
                anti_windup_k: 1.75,
                deriv_filter_n: 21.0,
                ff_gain: 3.1,
                ff_lead_time: 3.8,
                ff_lag_time: 1.9,
                smith_gain: 1.75,
                smith_tau: 9.5,
                smith_delay_sec: 1.4,
            },
            DeviceTier::MidRange => Self {
                hard_limit_cpu: 53.5,
                hard_limit_bat: 41.5,
                sched_temp_cool: 25.0,
                sched_temp_hot: 46.5,
                kp_base: 0.66,
                ki_base: 0.0115,
                kd_base: 0.92,
                kp_fast: 4.25,
                ki_fast: 0.0625,
                kd_fast: 3.8,
                anti_windup_k: 1.85,
                deriv_filter_n: 19.5,
                ff_gain: 2.8,
                ff_lead_time: 4.2,
                ff_lag_time: 2.1,
                smith_gain: 1.62,
                smith_tau: 10.5,
                smith_delay_sec: 1.6,
            },
            DeviceTier::LowEnd => Self {
                hard_limit_cpu: 52.5,
                hard_limit_bat: 40.5,
                sched_temp_cool: 25.5,
                sched_temp_hot: 45.5,
                kp_base: 0.61,
                ki_base: 0.0095,
                kd_base: 0.82,
                kp_fast: 4.05,
                ki_fast: 0.059,
                kd_fast: 3.55,
                anti_windup_k: 1.95,
                deriv_filter_n: 18.5,
                ff_gain: 2.55,
                ff_lead_time: 4.8,
                ff_lag_time: 2.3,
                smith_gain: 1.52,
                smith_tau: 11.5,
                smith_delay_sec: 1.7,
            },
        }
    }
}

struct LeadLagFilter {
    prev_y: f32,
    prev_u: f32,
    first_run: bool,
}

impl LeadLagFilter {
    fn new() -> Self {
        Self {
            prev_y: 0.0,
            prev_u: 0.0,
            first_run: true,
        }
    }
    fn update(&mut self, u: f32, dt: f32, k: f32, t_lead: f32, t_lag: f32) -> f32 {
        if self.first_run {
            self.prev_u = u;
            self.prev_y = u * k;
            self.first_run = false;
            return self.prev_y;
        }
        let a = 2.0 * t_lag + dt;
        let b = 2.0 * t_lag - dt;
        let c = k * (2.0 * t_lead + dt);
        let d = k * (2.0 * t_lead - dt);
        let y = (c * u + d * self.prev_u - b * self.prev_y) / a;
        self.prev_u = u;
        self.prev_y = y;
        y
    }
}

#[derive(Clone, Copy)]
struct HistoryPoint {
    value: f32,
    timestamp: time::Instant,
}

impl Default for HistoryPoint {
    fn default() -> Self {
        Self {
            value: 0.0,
            timestamp: time::Instant::now(),
        }
    }
}

struct SmithPredictor {
    model_output_no_delay: f32,
    delay_buffer: [HistoryPoint; SMITH_BUFFER_SIZE],
    head: usize,
    count: usize,
    capacity: usize,
}

impl SmithPredictor {
    fn new(capacity: usize) -> Self {
        let safe_capacity = capacity.min(SMITH_BUFFER_SIZE);
        let now = time::Instant::now();
        let init_point = HistoryPoint {
            value: 0.0,
            timestamp: now,
        };
        Self {
            model_output_no_delay: 0.0,
            delay_buffer: [init_point; SMITH_BUFFER_SIZE],
            head: 0,
            count: 0,
            capacity: safe_capacity,
        }
    }
    fn update(
        &mut self,
        u_control: f32,
        dt: f32,
        k_gain: f32,
        tau: f32,
        delay_sec: f32,
    ) -> (f32, f32) {
        let alpha = dt / (tau + dt);
        let y_no_delay = alpha * (u_control * k_gain) + (1.0 - alpha) * self.model_output_no_delay;
        self.model_output_no_delay = y_no_delay;
        let now = time::Instant::now();
        self.delay_buffer[self.head] = HistoryPoint {
            value: y_no_delay,
            timestamp: now,
        };
        let current_head_idx = self.head;
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
        let target_delay = time::Duration::from_secs_f32(delay_sec);
        let mut y_delayed = y_no_delay;
        for i in 0..self.count {
            let idx = (current_head_idx + self.capacity - i) % self.capacity;
            let point = &self.delay_buffer[idx];
            let age = now.duration_since(point.timestamp);
            if age >= target_delay {
                y_delayed = point.value;
                break;
            }
        }
        (y_no_delay, y_delayed)
    }
}

pub struct ThermalManager {
    last_tick: time::Instant,
    integral_accum: f32,
    prev_adjusted_pv: f32,
    prev_deriv_output: f32,
    prev_output_sat: f32,
    feedforward: LeadLagFilter,
    smith_predictor: SmithPredictor,
}

impl Default for ThermalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThermalManager {
    pub fn new() -> Self {
        Self {
            last_tick: time::Instant::now(),
            integral_accum: 0.0,
            prev_adjusted_pv: 0.0,
            prev_deriv_output: 0.0,
            prev_output_sat: 0.0,
            feedforward: LeadLagFilter::new(),
            smith_predictor: SmithPredictor::new(512),
        }
    }
    pub fn update(
        &mut self,
        cpu_temp: f32,
        bat_temp: f32,
        psi_load: f32,
        tunables: &ThermalConfig,
    ) -> f32 {
        let now = time::Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        let dt_safe = dt.clamp(0.01, 1.0);
        self.last_tick = now;
        let sigma = ((bat_temp - tunables.sched_temp_cool)
            / (tunables.sched_temp_hot - tunables.sched_temp_cool))
            .clamp(0.0, 1.0);
        let k_p = tunables.kp_base + sigma * (tunables.kp_fast - tunables.kp_base);
        let k_i = tunables.ki_base + sigma * (tunables.ki_fast - tunables.ki_base);
        let k_d = tunables.kd_base + sigma * (tunables.kd_fast - tunables.kd_base);
        let bat_margin = (tunables.hard_limit_bat - bat_temp).max(0.0);
        let control_margin = if bat_margin < 5.0 {
            5.0 - bat_margin
        } else {
            0.0
        };
        let setpoint = tunables.hard_limit_cpu - control_margin;
        let u_ff = self.feedforward.update(
            psi_load,
            dt_safe,
            tunables.ff_gain,
            tunables.ff_lead_time,
            tunables.ff_lag_time,
        );
        let current_control_effort = self.prev_output_sat;
        let (y_pred_no_delay, y_pred_delayed) = self.smith_predictor.update(
            current_control_effort,
            dt_safe,
            tunables.smith_gain,
            tunables.smith_tau,
            tunables.smith_delay_sec,
        );
        let pred_error_term = y_pred_no_delay - y_pred_delayed;
        let adjusted_pv = cpu_temp + pred_error_term;
        let error = adjusted_pv - setpoint;
        let p_term = k_p * error;
        let i_increment = k_i * error * dt_safe;
        self.integral_accum += i_increment;
        self.integral_accum = self.integral_accum.clamp(-50.0, 50.0);
        let t_d = if k_p > 1e-6 { k_d / k_p } else { 0.0 };
        let n = tunables.deriv_filter_n;
        let denominator = t_d + n * dt_safe;
        let d_term = if denominator > 1e-6 {
            let alpha = t_d / denominator;
            let beta = (k_d * n) / denominator;
            let delta_pv = adjusted_pv - self.prev_adjusted_pv;
            alpha * self.prev_deriv_output + beta * delta_pv
        } else {
            0.0
        };
        self.prev_adjusted_pv = adjusted_pv;
        self.prev_deriv_output = d_term;
        let u_raw = p_term + self.integral_accum + d_term + u_ff;
        let u_sat = u_raw.clamp(0.0, 100.0);
        if (u_raw - u_sat).abs() > 0.001 {
            let excess = u_raw - u_sat;
            self.integral_accum -= excess * tunables.anti_windup_k * dt_safe;
        }
        self.prev_output_sat = u_sat;
        let pid_saturation = u_sat / 100.0;
        let final_scale = 1.0 - pid_saturation;
        if bat_temp >= tunables.hard_limit_bat {
            return final_scale.min(0.2);
        }
        final_scale.clamp(0.1, 1.0)
    }
}
