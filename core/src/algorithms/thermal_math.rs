//! Author: [Seclususs](https://github.com/seclususs)

use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Copy)]
pub struct ThermalTunables {
    pub hard_limit_cpu: f64,
    pub hard_limit_bat: f64,
    pub throttling_start_temp: f64,
    pub sched_temp_cool: f64,
    pub sched_temp_hot: f64,
    pub anti_windup_k: f64,
    pub deriv_filter_n: f64,
    pub kp_base: f64,
    pub ki_base: f64,
    pub kd_base: f64,
    pub kp_agg: f64,
    pub ki_agg: f64,
    pub kd_agg: f64,
    pub ff_gain: f64,
    pub ff_lead_time: f64,
    pub ff_lag_time: f64,
    pub smith_delay_sec: f64,
    pub smith_tau: f64,
    pub smith_gain: f64,
}

struct LeadLagFilter {
    prev_y: f64,
    prev_u: f64,
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
    fn update(&mut self, u: f64, dt: f64, k: f64, t_lead: f64, t_lag: f64) -> f64 {
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
    model_output_no_delay: f64,
    delay_buffer: VecDeque<f64>,
}

impl SmithPredictor {
    fn new(capacity: usize) -> Self {
        Self {
            model_output_no_delay: 0.0,
            delay_buffer: VecDeque::with_capacity(capacity),
        }
    }
    fn update(
        &mut self,
        u_control: f64,
        dt: f64,
        k_gain: f64,
        tau: f64,
        delay_sec: f64,
    ) -> (f64, f64) {
        let alpha = dt / (tau + dt);
        let y_no_delay = alpha * (u_control * k_gain) + (1.0 - alpha) * self.model_output_no_delay;
        self.model_output_no_delay = y_no_delay;
        let steps = (delay_sec / dt.max(0.001)).round() as usize;
        if self.delay_buffer.len() > steps + 10 {
            self.delay_buffer.truncate(steps + 5);
        }
        self.delay_buffer.push_back(y_no_delay);
        let y_delayed = if self.delay_buffer.len() >= steps {
            self.delay_buffer.pop_front().unwrap_or(0.0)
        } else {
            *self.delay_buffer.front().unwrap_or(&0.0)
        };
        (y_no_delay, y_delayed)
    }
}

pub struct ThermalManager {
    last_tick: Instant,
    integral_accum: f64,
    prev_adjusted_pv: f64,
    prev_deriv_output: f64,
    prev_output_sat: f64,
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
        cpu_temp: f64,
        bat_temp: f64,
        psi_load: f64,
        tunables: &ThermalTunables,
    ) -> f64 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        let dt_safe = dt.clamp(0.01, 1.0);
        self.last_tick = now;
        let sigma = ((bat_temp - tunables.sched_temp_cool)
            / (tunables.sched_temp_hot - tunables.sched_temp_cool))
            .clamp(0.0, 1.0);
        let k_p = tunables.kp_base + sigma * (tunables.kp_agg - tunables.kp_base);
        let k_i = tunables.ki_base + sigma * (tunables.ki_agg - tunables.ki_base);
        let k_d = tunables.kd_base + sigma * (tunables.kd_agg - tunables.kd_base);
        let bat_headroom = (tunables.hard_limit_bat - bat_temp).max(0.0);
        let safety_margin = if bat_headroom < 5.0 {
            5.0 - bat_headroom
        } else {
            0.0
        };
        let setpoint = tunables.hard_limit_cpu - safety_margin;
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
        let pid_severity = u_sat / 100.0;
        let final_scale = 1.0 - pid_severity;
        if bat_temp >= tunables.hard_limit_bat {
            return final_scale.min(0.2);
        }
        final_scale.clamp(0.1, 1.0)
    }
}