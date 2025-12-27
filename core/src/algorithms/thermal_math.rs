//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::thermal;
use crate::config::loop_settings::THERMAL_SYNC_INTERVAL_SEC;

use std::time::Instant;

pub struct ThermalTunables {
    pub alpha_heating: f64,
    pub lambda_cooling: f64,
    pub max_virtual_temp: f64,
    pub bucket_size: f64,
    pub bucket_leak_rate: f64,
    pub threshold_warm: f64,
    pub threshold_hot: f64,
    pub hysteresis_gap: f64,
}

pub const THERMAL_CONFIG: ThermalTunables = ThermalTunables {
    alpha_heating: 0.85,
    lambda_cooling: 0.09,
    max_virtual_temp: 95.0,
    bucket_size: 1200.0,
    bucket_leak_rate: 20.0,
    threshold_warm: 65.0,
    threshold_hot: 78.0,
    hysteresis_gap: 5.0,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalState {
    Performance,
    Balanced,
    Conservation
}

pub struct ThermalPredictor {
    virtual_temp: f64,
    energy_bucket: f64,
    current_state: ThermalState,
    last_tick: Instant,
    ambient_temp_assumption: f64,
}

impl ThermalPredictor {
    pub fn new(start_temp_c: f64) -> Self {
        let ambient = if start_temp_c < 35.0 {
            start_temp_c
        } else {
            35.0
        };
        Self {
            virtual_temp: start_temp_c, 
            energy_bucket: 0.0,
            current_state: ThermalState::Performance,
            last_tick: Instant::now(),
            ambient_temp_assumption: ambient,
        }
    }
    pub fn sync_with_sensor(&mut self, real_temp: f64) {
        self.virtual_temp = real_temp;
        self.last_tick = Instant::now();
        if real_temp < self.ambient_temp_assumption {
            self.ambient_temp_assumption = real_temp;
        } else if real_temp > self.ambient_temp_assumption + 10.0 {
            self.ambient_temp_assumption = self.ambient_temp_assumption.max(30.0);
        }
    }
    pub fn update(&mut self, input_pressure: f64, tunables: &ThermalTunables) -> f64 {
        let now = Instant::now();
        let dt_sec = now.duration_since(self.last_tick).as_secs_f64();
        if dt_sec > THERMAL_SYNC_INTERVAL_SEC as f64 {
            let real_temp = thermal::read_initial_thermal_state();
            self.sync_with_sensor(real_temp);
            self.energy_bucket = 0.0;
            self.last_tick = now;
            self.update_state_machine(tunables);
            return self.calculate_damping_factor(tunables);
        }
        self.last_tick = now; 
        let safe_dt = if dt_sec < 0.001 { 0.001 } else { dt_sec };
        let heat_in = input_pressure * tunables.alpha_heating;
        let delta_t_ambient = self.virtual_temp - self.ambient_temp_assumption;
        let heat_out = delta_t_ambient * tunables.lambda_cooling;
        let delta_temp = (heat_in - heat_out) * safe_dt;
        self.virtual_temp = (self.virtual_temp + delta_temp)
            .clamp(self.ambient_temp_assumption, tunables.max_virtual_temp);
        let bucket_in = input_pressure * safe_dt;
        let bucket_leak = tunables.bucket_leak_rate * safe_dt;
        self.energy_bucket = (self.energy_bucket + bucket_in - bucket_leak)
            .clamp(0.0, tunables.bucket_size);  
        self.update_state_machine(tunables);
        self.calculate_damping_factor(tunables)
    }
    #[allow(dead_code)]
    fn hard_reset(&mut self) {
        self.virtual_temp = self.ambient_temp_assumption;
        self.energy_bucket = 0.0;
        self.current_state = ThermalState::Performance;
    }
    fn update_state_machine(&mut self, tunables: &ThermalTunables) {
        match self.current_state {
            ThermalState::Performance => {
                if self.virtual_temp > tunables.threshold_warm 
                   || self.energy_bucket > (tunables.bucket_size * 0.75) {
                    self.current_state = ThermalState::Balanced;
                }
            },
            ThermalState::Balanced => {
                if self.virtual_temp > tunables.threshold_hot {
                    self.current_state = ThermalState::Conservation;
                }
                else if self.virtual_temp < (tunables.threshold_warm - tunables.hysteresis_gap) 
                        && self.energy_bucket < (tunables.bucket_size * 0.25) {
                    self.current_state = ThermalState::Performance;
                }
            },
            ThermalState::Conservation => {
                if self.virtual_temp < (tunables.threshold_hot - tunables.hysteresis_gap) {
                    self.current_state = ThermalState::Balanced;
                }
            }
        }
    }
    fn calculate_damping_factor(&self, tunables: &ThermalTunables) -> f64 {
        if self.current_state == ThermalState::Performance && self.energy_bucket < 50.0 {
            return 1.0;
        }
        let temp_range = tunables.max_virtual_temp - self.ambient_temp_assumption;
        let curr_offset = self.virtual_temp - self.ambient_temp_assumption;
        let ratio = (curr_offset / temp_range).clamp(0.0, 1.0);
        let quadratic_drop = ratio * ratio;
        let mut damping = 1.0 - quadratic_drop;
        match self.current_state {
            ThermalState::Performance => {}, 
            ThermalState::Balanced => {
                damping *= 0.90;
            },
            ThermalState::Conservation => {
                damping *= 0.60;
            }
        }
        damping.clamp(0.2, 1.0)
    }
}