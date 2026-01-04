//! Author: [Seclususs](https://github.com/seclususs)

use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Copy)]
pub struct ThermalTunables {
    pub pid_kp: f64,
    pub pid_ki: f64,
    pub pid_kd: f64,
    pub hard_limit_cpu: f64,
    pub hard_limit_bat: f64,
    pub dth_start_temp: f64,
    pub dth_k_thermal: f64,
    pub tga_k_anticipation: f64,
    pub leakage_k: f64,
    pub leakage_start_temp: f64,
    pub bucket_capacity: f64,
    pub bucket_leak_base: f64,
    pub psi_threshold: f64,
    pub psi_strength: f64,
    pub mpc_horizon: usize,
    pub rls_lambda: f64,
    pub rls_sigma: f64,
    pub rls_deadzone: f64,
    pub model_alpha_init: f64,
    pub model_beta_init: f64,
    pub dob_smoothing: f64,
}

struct PidController {
    integral_error: f64,
    prev_error: f64,
    prev_derivative: f64,
}

impl PidController {
    fn new() -> Self {
        Self {
            integral_error: 0.0,
            prev_error: 0.0,
            prev_derivative: 0.0,
        }
    }
    fn update(&mut self, error: f64, dt_sec: f64, tunables: &ThermalTunables) -> f64 {
        self.integral_error = (self.integral_error + (error * dt_sec)).clamp(-100.0, 100.0);
        let raw_derivative = if dt_sec > 0.0 {
            (error - self.prev_error) / dt_sec
        } else {
            0.0
        };
        self.prev_error = error;
        let alpha = 0.2;
        let smoothed_derivative = alpha * raw_derivative + (1.0 - alpha) * self.prev_derivative;
        self.prev_derivative = smoothed_derivative;
        let p_term = tunables.pid_kp * error;
        let i_term = tunables.pid_ki * self.integral_error;
        let d_term = tunables.pid_kd * smoothed_derivative;
        p_term + i_term + d_term
    }
}

struct RlsEstimator {
    p: [[f64; 2]; 2],
    theta: [f64; 2],
    prev_u_temp: f64,
    prev_load: f64,
    dist_est: f64,
    initialized: bool,
}

impl RlsEstimator {
    fn new(alpha_init: f64, beta_init: f64) -> Self {
        Self {
            p: [[100.0, 0.0], [0.0, 100.0]],
            theta: [alpha_init, beta_init],
            prev_u_temp: 0.0,
            prev_load: 0.0,
            dist_est: 0.0,
            initialized: false,
        }
    }
    fn update(
        &mut self,
        current_temp: f64,
        ambient_temp: f64,
        current_load: f64,
        _dt: f64,
        tunables: &ThermalTunables,
    ) -> (f64, f64, f64) {
        let u_curr = current_temp - ambient_temp;
        if !self.initialized {
            self.prev_u_temp = u_curr;
            self.prev_load = current_load;
            self.initialized = true;
            return (self.theta[0], self.theta[1], 0.0);
        }
        let phi = [self.prev_u_temp, self.prev_load];
        let prediction = self.theta[0] * phi[0] + self.theta[1] * phi[1];
        let error = u_curr - prediction;
        let robust_error = if error.abs() <= tunables.rls_deadzone {
            0.0
        } else {
            error - tunables.rls_deadzone * error.signum()
        };
        if robust_error.abs() > 0.0 {
            let p_phi = [
                self.p[0][0] * phi[0] + self.p[0][1] * phi[1],
                self.p[1][0] * phi[0] + self.p[1][1] * phi[1],
            ];
            let phi_p_phi = phi[0] * p_phi[0] + phi[1] * p_phi[1];
            let lambda = tunables.rls_lambda;
            let denom = lambda + phi_p_phi;
            let k = [p_phi[0] / denom, p_phi[1] / denom];
            let sigma = tunables.rls_sigma;
            let theta_default = [tunables.model_alpha_init, tunables.model_beta_init];
            for i in 0..2 {
                let leakage = sigma * (theta_default[i] - self.theta[i]);
                let learning = k[i] * robust_error;
                self.theta[i] += learning + leakage;
            }
            self.theta[0] = self.theta[0].clamp(0.80, 0.999);
            self.theta[1] = self.theta[1].clamp(0.001, 2.0);
            let term = [
                [k[0] * p_phi[0], k[0] * p_phi[1]],
                [k[1] * p_phi[0], k[1] * p_phi[1]],
            ];
            for (i, row) in term.iter().enumerate() {
                for (j, val) in row.iter().enumerate() {
                    self.p[i][j] = (self.p[i][j] - val) / lambda;
                }
            }
        }
        let dynamic_load = (u_curr - self.theta[0] * self.prev_u_temp) / self.theta[1];
        let raw_disturbance = dynamic_load - self.prev_load;
        self.dist_est = tunables.dob_smoothing * self.dist_est
            + (1.0 - tunables.dob_smoothing) * raw_disturbance;
        self.prev_u_temp = u_curr;
        self.prev_load = current_load;
        (self.theta[0], self.theta[1], self.dist_est)
    }
}

pub struct ThermalManager {
    pid: PidController,
    rls: RlsEstimator,
    energy_bucket: f64,
    last_tick: Instant,
    tga_history: VecDeque<(Instant, f64)>,
}

impl Default for ThermalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThermalManager {
    pub fn new() -> Self {
        Self {
            pid: PidController::new(),
            rls: RlsEstimator::new(0.90, 0.3),
            energy_bucket: 0.0,
            last_tick: Instant::now(),
            tga_history: VecDeque::with_capacity(20),
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
        let (alpha, beta, dist) = self
            .rls
            .update(cpu_temp, bat_temp, psi_load, dt_safe, tunables);
        let mut t_pred = cpu_temp;
        let t_amb = bat_temp;
        let effective_load = psi_load + dist;
        let mut max_future_temp = t_pred;
        for _ in 0..tunables.mpc_horizon {
            let u_k = t_pred - t_amb;
            let u_next = alpha * u_k + beta * effective_load;
            t_pred = u_next + t_amb;
            if t_pred > max_future_temp {
                max_future_temp = t_pred;
            }
        }
        let mpc_damping = if max_future_temp > tunables.hard_limit_cpu {
            let headroom = tunables.hard_limit_cpu - t_amb;
            let projected_excess = max_future_temp - t_amb;
            if projected_excess > 0.0 {
                (headroom / projected_excess).clamp(0.1, 1.0)
            } else {
                0.1
            }
        } else {
            1.0
        };
        while let Some(front) = self.tga_history.front() {
            if now.duration_since(front.0).as_secs_f64() > 10.0 {
                self.tga_history.pop_front();
            } else {
                break;
            }
        }
        self.tga_history.push_back((now, bat_temp));
        let gradient_penalty = if let Some(oldest) = self.tga_history.front() {
            let delta_t = now.duration_since(oldest.0).as_secs_f64();
            if delta_t > 1.0 {
                let delta_temp = bat_temp - oldest.1;
                let g_bat = delta_temp / delta_t;
                (tunables.tga_k_anticipation * g_bat).max(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        let s_thermal = (bat_temp - tunables.dth_start_temp).max(0.0);
        let dth_penalty = tunables.dth_k_thermal * s_thermal;
        let excess_load = (psi_load - tunables.psi_threshold).max(0.0);
        let psi_penalty = excess_load * tunables.psi_strength;
        let target_cpu_dynamic =
            tunables.hard_limit_cpu - dth_penalty - gradient_penalty - psi_penalty;
        let final_target = target_cpu_dynamic.max(45.0);
        let error = cpu_temp - final_target;
        let pid_output = self.pid.update(error, dt_safe, tunables);
        let headroom_bat = tunables.hard_limit_bat - tunables.dth_start_temp;
        let current_headroom = tunables.hard_limit_bat - bat_temp;
        let eta_cooling = if headroom_bat > 0.0 {
            (current_headroom / headroom_bat).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let adaptive_leak_rate = tunables.bucket_leak_base * eta_cooling;
        if error > 0.0 {
            self.energy_bucket += error * dt_safe * 5.0;
        } else {
            self.energy_bucket -= adaptive_leak_rate * dt_safe;
        }
        self.energy_bucket = self.energy_bucket.clamp(0.0, tunables.bucket_capacity);
        let leakage_penalty_factor = if cpu_temp > tunables.leakage_start_temp {
            let excess = cpu_temp - tunables.leakage_start_temp;
            (excess * tunables.leakage_k).exp()
        } else {
            1.0
        };
        let base_throttle = pid_output.max(0.0);
        let phys_throttle = base_throttle * leakage_penalty_factor;
        let bucket_fill_ratio = self.energy_bucket / tunables.bucket_capacity;
        let sustained_penalty = bucket_fill_ratio * 0.5;
        let total_severity = phys_throttle + sustained_penalty;
        let pid_damping = (1.0 / (1.0 + total_severity)).clamp(0.1, 1.0);
        let final_damping = pid_damping.min(mpc_damping);
        if bat_temp >= tunables.hard_limit_bat {
            return final_damping.min(0.2);
        }
        final_damping
    }
}