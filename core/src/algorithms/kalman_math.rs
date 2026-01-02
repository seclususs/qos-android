//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct KalmanConfig {
    pub q_base: f64,
    pub r_base: f64,
    pub fading_factor: f64,
}

impl Default for KalmanConfig {
    fn default() -> Self {
        Self {
            q_base: 2.0,
            r_base: 10.0,
            fading_factor: 1.05,
        }
    }
}

pub struct KalmanFilter {
    x: f64,
    p: f64,
    config: KalmanConfig,
    first_run: bool,
}

impl KalmanFilter {
    pub fn new(config: KalmanConfig) -> Self {
        Self {
            x: 0.0,
            p: 1.0,
            config,
            first_run: true,
        }
    }
    pub fn reset(&mut self) {
        self.first_run = true;
        self.p = self.config.r_base;
        self.x = 0.0;
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
            self.first_run = false;
            return self.x;
        }
        let x_pred = self.x;
        let q_k = self.config.q_base * dt_sec;
        let p_pred = (self.config.fading_factor * self.p) + q_k;
        let innovation = z_measured - x_pred;
        let s_k = p_pred + self.config.r_base;
        let k_gain = p_pred / s_k;
        self.x = x_pred + (k_gain * innovation);
        self.p = (1.0 - k_gain) * p_pred;
        self.x.clamp(0.0, 100.0)
    }
}