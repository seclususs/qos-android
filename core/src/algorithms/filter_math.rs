//! Author: [Seclususs](https://github.com/seclususs)

const MAX_WINDOW_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct KalmanConfig {
    pub q_base: f32,
    pub r_base: f32,
    pub fading_factor: f32,
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
    x: f32,
    p: f32,
    last_nis: f32,
    config: KalmanConfig,
    history: [f32; MAX_WINDOW_SIZE],
    head: usize,
    count: usize,
    first_run: bool,
    sum_sq_innov: f32,
}

impl KalmanFilter {
    pub fn new(config: KalmanConfig) -> Self {
        let safe_config = if config.window_size > MAX_WINDOW_SIZE {
            let mut c = config;
            c.window_size = MAX_WINDOW_SIZE;
            c
        } else {
            config
        };
        Self {
            x: 0.0,
            p: 1.0,
            last_nis: 0.0,
            config: safe_config,
            history: [0.0; MAX_WINDOW_SIZE],
            head: 0,
            count: 0,
            first_run: true,
            sum_sq_innov: 0.0,
        }
    }
    pub fn reset(&mut self) {
        self.first_run = true;
        self.p = self.config.r_base;
        self.x = 0.0;
        self.last_nis = 0.0;
        self.head = 0;
        self.count = 0;
        self.sum_sq_innov = 0.0;
        self.history = [0.0; MAX_WINDOW_SIZE];
    }
    pub fn update(&mut self, mut z_measured: f32, dt_sec: f32) -> f32 {
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
        let innov_sq = innovation * innovation;
        let old_val = self.history[self.head];
        self.sum_sq_innov -= old_val * old_val;
        self.history[self.head] = innovation;
        self.sum_sq_innov += innov_sq;
        if self.sum_sq_innov < 0.0 {
            self.sum_sq_innov = 0.0;
        }
        self.head = (self.head + 1) % self.config.window_size;
        if self.count < self.config.window_size {
            self.count += 1;
        }
        let c_y = if self.count > 0 {
            self.sum_sq_innov / (self.count as f32)
        } else {
            0.0
        };
        let r_adaptive = c_y - p_pred;
        let r_eff = r_adaptive.max(self.config.r_base);
        let s_temp = p_pred + r_eff;
        let nis = if s_temp > 1e-6 {
            innov_sq / s_temp
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
    pub fn get_last_nis(&self) -> f32 {
        self.last_nis
    }
}
