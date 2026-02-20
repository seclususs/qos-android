//! Author: [Seclususs](https://github.com/seclususs)

#[derive(Debug, Clone, Copy)]
pub struct KalmanConfig {
    pub q_pos: f32,
    pub q_vel: f32,
    pub r_meas: f32,
    pub fading_factor: f32,
}

impl Default for KalmanConfig {
    fn default() -> Self {
        Self {
            q_pos: 0.08,
            q_vel: 8.0,
            r_meas: 4.0,
            fading_factor: 1.02,
        }
    }
}

pub struct KalmanFilter {
    x_pos: f32,
    x_vel: f32,
    p00: f32,
    p01: f32,
    p10: f32,
    p11: f32,
    config: KalmanConfig,
    first_run: bool,
    last_nis: f32,
}

impl KalmanFilter {
    pub fn new(config: KalmanConfig) -> Self {
        Self {
            x_pos: 0.0,
            x_vel: 0.0,
            p00: 100.0,
            p01: 0.0,
            p10: 0.0,
            p11: 100.0,
            config,
            first_run: true,
            last_nis: 0.0,
        }
    }
    pub fn reset(&mut self) {
        self.first_run = true;
        self.x_pos = 0.0;
        self.x_vel = 0.0;
        self.p00 = 100.0;
        self.p01 = 0.0;
        self.p10 = 0.0;
        self.p11 = 100.0;
        self.last_nis = 0.0;
    }
    pub fn update(&mut self, z_meas: f32, dt_sec: f32) -> f32 {
        if !z_meas.is_finite() {
            return self.x_pos;
        }
        let z = z_meas.clamp(0.0, 500.0);
        if dt_sec > 5.0 {
            self.reset();
        }
        if self.first_run {
            self.x_pos = z;
            self.x_vel = 0.0;
            self.first_run = false;
            return z;
        }
        let dt = dt_sec.max(0.0001);
        let x_pos_pred = self.x_pos + self.x_vel * dt;
        let x_vel_pred = self.x_vel;
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;
        let dt4 = dt2 * dt2;
        let q_scale = self.config.q_vel;
        let q00 = q_scale * dt4 * 0.25 + self.config.q_pos * dt;
        let q01 = q_scale * dt3 * 0.5;
        let q10 = q01;
        let q11 = q_scale * dt2 + self.config.q_vel * dt;
        let f_p00 = self.p00 + self.p10 * dt;
        let f_p01 = self.p01 + self.p11 * dt;
        let f_p10 = self.p10;
        let f_p11 = self.p11;
        let alpha = self.config.fading_factor;
        let p00_pred = (f_p00 + f_p01 * dt) * alpha + q00;
        let p01_pred = f_p01 * alpha + q01;
        let p10_pred = (f_p10 + f_p11 * dt) * alpha + q10;
        let p11_pred = f_p11 * alpha + q11;
        let y = z - x_pos_pred;
        let s = p00_pred + self.config.r_meas;
        let inv_s = if s.abs() > 1e-9 { 1.0 / s } else { 0.0 };
        let k0 = p00_pred * inv_s;
        let k1 = p10_pred * inv_s;
        self.x_pos = x_pos_pred + k0 * y;
        self.x_vel = x_vel_pred + k1 * y;
        let p00_new = (1.0 - k0) * p00_pred;
        let p01_new = (1.0 - k0) * p01_pred;
        let p10_new = -k1 * p00_pred + p10_pred;
        let p11_new = -k1 * p01_pred + p11_pred;
        self.p00 = p00_new;
        self.p01 = p01_new;
        self.p10 = p10_new;
        self.p11 = p11_new;
        self.last_nis = y * y * inv_s;
        self.x_pos.max(0.0)
    }
    #[inline]
    pub fn get_velocity(&self) -> f32 {
        self.x_vel
    }
    #[inline]
    pub fn get_last_nis(&self) -> f32 {
        self.last_nis
    }
}
