//! Author: [Seclususs](https://github.com/seclususs)

use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Copy)]
pub struct ThermalTunables {
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

struct SmithPredictor {
    model_output_no_delay: f32,
    delay_buffer: VecDeque<f32>,
    capacity: usize,
}

impl SmithPredictor {
    fn new(capacity: usize) -> Self {
        Self {
            model_output_no_delay: 0.0,
            delay_buffer: VecDeque::with_capacity(capacity),
            capacity,
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
        if self.delay_buffer.len() >= self.capacity {
            self.delay_buffer.pop_front();
        }
        self.delay_buffer.push_back(y_no_delay);
        let steps_needed = (delay_sec / dt.max(0.001)).round() as usize;
        let current_len = self.delay_buffer.len();
        let y_delayed = if steps_needed < current_len {
            let idx = current_len.saturating_sub(1).saturating_sub(steps_needed);
            *self.delay_buffer.get(idx).unwrap_or(&0.0)
        } else {
            *self.delay_buffer.front().unwrap_or(&0.0)
        };
        (y_no_delay, y_delayed)
    }
}

pub struct ThermalManager {
    last_tick: Instant,
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
            last_tick: Instant::now(),
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
        tunables: &ThermalTunables,
    ) -> f32 {
        let now = Instant::now();
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