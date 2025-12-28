//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::thermal_math::ThermalState;

pub struct StorageTunables {
    pub min_read_ahead: f64,
    pub max_read_ahead: f64,
    pub min_nr_requests: f64,
    pub max_nr_requests: f64,
    pub min_fifo_batch: f64,
    pub max_fifo_batch: f64,
    pub io_sat_beta: f64,
    pub epsilon: f64,
    pub io_read_ahead_threshold: f64,
    pub io_scaling_factor: f64,
    pub io_tactical_multiplier: f64,
}

pub fn calculate_read_ahead(p_curr: f64, tunables: &StorageTunables, thermal_state: ThermalState) -> f64 {
    if thermal_state == ThermalState::Conservation {
        return tunables.min_read_ahead;
    }
    let base_read_ahead = if p_curr < tunables.io_read_ahead_threshold {
        tunables.max_read_ahead
    } else {
        let normalized_p = (p_curr - tunables.io_read_ahead_threshold).max(tunables.epsilon);
        let scaling = tunables.io_scaling_factor / normalized_p; 
        let result = tunables.min_read_ahead + (scaling * (tunables.max_read_ahead - tunables.min_read_ahead));
        result.clamp(tunables.min_read_ahead, tunables.max_read_ahead)
    };
    if thermal_state == ThermalState::Balanced {
        base_read_ahead.min((tunables.max_read_ahead + tunables.min_read_ahead) / 2.0)
    } else {
        base_read_ahead
    }
}

pub fn calculate_io_saturation(avg10: f64, some_avg10: f64, tunables: &StorageTunables) -> f64 {
    let i_sat_raw = avg10 / (some_avg10 + tunables.epsilon);
    i_sat_raw.clamp(0.0, 1.0)
}

pub fn calculate_queue_params(i_sat: f64, tunables: &StorageTunables, thermal_state: ThermalState) -> (f64, f64) {
    let sat_factor = i_sat.powf(tunables.io_sat_beta);
    let mut target_nr = (tunables.max_nr_requests * (1.0 - sat_factor)) + (tunables.min_nr_requests * sat_factor);
    let mut target_fifo = (tunables.max_fifo_batch * (1.0 - sat_factor)) + (tunables.min_fifo_batch * sat_factor);
    match thermal_state {
        ThermalState::Conservation => {
            target_nr = tunables.min_nr_requests;
            target_fifo = tunables.min_fifo_batch;
        },
        ThermalState::Balanced => {
            target_nr = target_nr.min((tunables.max_nr_requests + tunables.min_nr_requests) * 0.6);
        },
        _ => {}
    }
    (target_nr, target_fifo)
}