//! Author: [Seclususs](https://github.com/seclususs)

use std::time::Instant;

#[derive(Clone, Copy)]
pub struct ThermalTunables {
    pub pid_kp: f64,
    pub pid_ki: f64,
    pub pid_kd: f64,
    pub target_headroom: f64,
    pub hard_limit_cpu: f64,
    pub hard_limit_bat: f64,
    pub leakage_k: f64,
    pub leakage_start_temp: f64,
    pub bucket_capacity: f64,
    pub bucket_leak_base: f64,
    pub psi_threshold: f64,
    pub psi_strength: f64,
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

pub struct ThermalManager {
    pid: PidController,
    energy_bucket: f64,
    last_tick: Instant,
}

impl ThermalManager {
    pub fn new() -> Self {
        Self {
            pid: PidController::new(),
            energy_bucket: 0.0,
            last_tick: Instant::now(),
        }
    }
    pub fn update(&mut self, cpu_temp: f64, bat_temp: f64, psi_load: f64, tunables: &ThermalTunables) -> f64 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        let dt_safe = dt.clamp(0.01, 1.0); 
        self.last_tick = now;
        let excess_load = (psi_load - tunables.psi_threshold).max(0.0);
        let anticipatory_penalty = excess_load * tunables.psi_strength;
        let bat_safety_margin = (tunables.hard_limit_bat - bat_temp).max(0.0);
        let safety_scaling = (bat_safety_margin / 10.0).clamp(0.0, 1.0);
        let base_headroom = tunables.target_headroom * safety_scaling;
        let base_target = (bat_temp + base_headroom).min(tunables.hard_limit_cpu);
        let dynamic_target = base_target - anticipatory_penalty;
        let error = cpu_temp - dynamic_target;
        let pid_output = self.pid.update(error, dt_safe, tunables);
        let leakage_penalty = if cpu_temp > tunables.leakage_start_temp {
            let excess = cpu_temp - tunables.leakage_start_temp;
            (excess * tunables.leakage_k).exp()
        } else {
            1.0
        };
        let cooling_potential = (cpu_temp - bat_temp).max(1.0);
        let leak_rate = tunables.bucket_leak_base * (cooling_potential / 20.0);
        if error > 0.0 {
            self.energy_bucket += error * dt_safe * 5.0; 
        } else {
            self.energy_bucket -= leak_rate * dt_safe;
        }
        self.energy_bucket = self.energy_bucket.clamp(0.0, tunables.bucket_capacity);
        let base_throttle = pid_output.max(0.0);
        let phys_throttle = base_throttle * leakage_penalty;
        let bucket_fill_ratio = self.energy_bucket / tunables.bucket_capacity;
        let sustained_penalty = bucket_fill_ratio * 0.5;
        let total_severity = phys_throttle + sustained_penalty;
        let damping = 1.0 / (1.0 + total_severity);
        if bat_temp >= tunables.hard_limit_bat {
            return damping.min(0.2); 
        }
        damping.clamp(0.1, 1.0)
    }
}