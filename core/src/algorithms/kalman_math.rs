//! Author: [Seclususs](https://github.com/seclususs)

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct KalmanConfig {
    pub q_base: f64,
    pub r_base: f64,
    pub fading_factor: f64,
    pub window_size: usize,
}

impl Default for KalmanConfig {
    fn default() -> Self {
        Self {
            q_base: 2.0,
            r_base: 10.0,
            fading_factor: 1.05,
            window_size: 10,
        }
    }
}

pub struct KalmanFilter {
    x: f64,
    p: f64,
    last_nis: f64,
    config: KalmanConfig,
    history: VecDeque<f64>,
    first_run: bool,
}

impl KalmanFilter {
    pub fn new(config: KalmanConfig) -> Self {
        Self {
            x: 0.0,
            p: 1.0,
            last_nis: 0.0,
            config,
            history: VecDeque::with_capacity(config.window_size),
            first_run: true,
        }
    }
    pub fn reset(&mut self) {
        self.first_run = true;
        self.p = self.config.r_base;
        self.x = 0.0;
        self.last_nis = 0.0;
        self.history.clear();
    }
    pub fn update(&mut self, mut z_measured: f64, dt_sec: f64) -> f64 {
        if !z_measured.is_finite() {
            return self.x;
        }
        z_measured = z_measured.clamp(0.0, 100.0);
        if dt_sec > 5.0 {
            self.reset();
        }
        if self.first_run {
            self.x = z_measured;
            self.p = self.config.r_base;
            self.last_nis = 0.0;
            self.first_run = false;
            return self.x;
        }
        let x_pred = self.x;
        let q_k_base = self.config.q_base * dt_sec;
        let p_pred = (self.config.fading_factor * self.p) + q_k_base;
        let innovation = z_measured - x_pred;
        if self.history.len() >= self.config.window_size {
            self.history.pop_front();
        }
        self.history.push_back(innovation);
        let sum_sq_innov: f64 = self.history.iter().map(|&y| y * y).sum();
        let count = self.history.len() as f64;
        let c_y = if count > 0.0 {
            sum_sq_innov / count
        } else {
            0.0
        };
        let r_adaptive = c_y - p_pred;
        let r_eff = r_adaptive.max(self.config.r_base);
        let s_temp = p_pred + r_eff;
        let nis = if s_temp > 1e-6 {
            (innovation * innovation) / s_temp
        } else {
            0.0
        };
        self.last_nis = nis;
        let q_adaptive = if nis > 2.0 {
            let scale = nis.min(10.0);
            q_k_base * scale
        } else {
            q_k_base
        };
        let p_pred_final = p_pred + (q_adaptive - q_k_base).max(0.0);
        let s_k = p_pred_final + r_eff;
        let k_gain = if s_k > 1e-6 { p_pred_final / s_k } else { 0.0 };
        self.x = (x_pred + (k_gain * innovation)).clamp(0.0, 100.0);
        self.p = (1.0 - k_gain) * p_pred_final;
        self.x
    }
    pub fn get_last_nis(&self) -> f64 {
        self.last_nis
    }
}