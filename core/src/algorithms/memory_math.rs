//! Author: [Seclususs](https://github.com/seclususs)

use crate::monitors::vm_monitor::VmStats;

#[derive(Debug, Clone, Copy)]
pub struct MemoryTunables {
    pub min_swappiness: f64,
    pub max_swappiness: f64,
    pub min_dirty_expire: f64,
    pub max_dirty_expire: f64,
    pub min_stat_interval: f64,
    pub max_stat_interval: f64,
    pub min_watermark_scale: f64,
    pub max_watermark_scale: f64,
    pub min_extfrag_threshold: f64,
    pub max_extfrag_threshold: f64,
    pub min_dirty: f64,
    pub max_dirty: f64,
    pub min_dirty_bg: f64,
    pub max_dirty_bg: f64,
    pub min_dirty_writeback: f64,
    pub max_dirty_writeback: f64,
    pub min_page_cluster: f64,
    pub max_page_cluster: f64,
    pub min_vfs: f64,
    pub max_vfs: f64,
    pub hydraulic_kp: f64,
    pub hydraulic_kd: f64,
    pub turbulence_factor: f64,
    pub thermal_vfs_k: f64,
    pub entropy_watermark_k: f64,
    pub wss_penalty_factor: f64,
    pub zram_thermal_penalty: f64,
    pub general_smooth_factor: f64,
    pub watermark_smooth_factor: f64,
}

#[derive(Default)]
pub struct FluidDelta {
    pub efficiency: f64,
    pub refault_index: f64,
    pub scan_rate: f64,
}

pub fn calculate_fluid_dynamics(current: &VmStats, prev: &VmStats, dt_sec: f64) -> FluidDelta {
    if dt_sec <= 0.0 {
        return FluidDelta::default();
    }
    let delta_scan = current.pgscan.saturating_sub(prev.pgscan) as f64;
    let delta_steal = current.pgsteal.saturating_sub(prev.pgsteal) as f64;
    let delta_refault = current
        .workingset_refault
        .saturating_sub(prev.workingset_refault) as f64;
    let efficiency = if delta_scan > 0.0 {
        delta_steal / (delta_scan + 0.001)
    } else {
        1.0
    };
    let refault_index = if delta_scan > 0.0 || delta_refault > 0.0 {
        delta_refault / (delta_scan + delta_refault + 0.001)
    } else {
        0.0
    };
    FluidDelta {
        efficiency: efficiency.clamp(0.0, 1.0),
        refault_index: refault_index.clamp(0.0, 1.0),
        scan_rate: delta_scan / dt_sec,
    }
}

pub fn calculate_pressure_derivative(current_psi: f64, prev_psi: f64, dt: f64) -> f64 {
    if dt <= 0.0 {
        0.0
    } else {
        (current_psi - prev_psi) / dt
    }
}

pub fn calculate_swappiness_physics(
    p_mem: f64,
    dp_dt: f64,
    fluid: &FluidDelta,
    cpu_temp: f64,
    io_sat: f64,
    tunables: &MemoryTunables,
) -> f64 {
    let base_swap = tunables.min_swappiness;
    let p_term = tunables.hydraulic_kp * p_mem;
    let d_term = tunables.hydraulic_kd * dp_dt;
    let turbulence = tunables.turbulence_factor * (1.0 - fluid.efficiency);
    let mut target_swap = base_swap + p_term + d_term + turbulence;
    let thermal_stress = (cpu_temp - 50.0).max(0.0) / 20.0;
    let thermal_throttle = (1.0 - (thermal_stress * tunables.zram_thermal_penalty)).clamp(0.0, 1.0);
    let io_throttle = (1.0 - (io_sat * 0.6)).clamp(0.2, 1.0);
    target_swap = target_swap * thermal_throttle * io_throttle;
    let thrashing_penalty = (fluid.refault_index * tunables.wss_penalty_factor).powi(2);
    let wss_protection = (1.0 - thrashing_penalty).clamp(0.0, 1.0);
    let final_swap = target_swap * wss_protection;
    final_swap.clamp(tunables.min_swappiness, tunables.max_swappiness)
}

pub fn calculate_vfs_thermodynamics(p_mem: f64, tunables: &MemoryTunables) -> f64 {
    let range = tunables.max_vfs - tunables.min_vfs;
    let decay = (-tunables.thermal_vfs_k * p_mem).exp();
    let inverse_decay = 1.0 - decay;
    let vfs = tunables.min_vfs + (range * inverse_decay);
    vfs.clamp(tunables.min_vfs, tunables.max_vfs)
}

pub fn calculate_sediment_control(io_sat: f64, tunables: &MemoryTunables) -> (f64, f64) {
    let pipe_capacity = (1.0 - io_sat).clamp(0.1, 1.0);
    let target_dirty = tunables.max_dirty * pipe_capacity;
    let target_dirty_bg = tunables.max_dirty_bg * pipe_capacity;
    (
        target_dirty.clamp(tunables.min_dirty, tunables.max_dirty),
        target_dirty_bg.clamp(tunables.min_dirty_bg, tunables.max_dirty_bg),
    )
}

pub fn calculate_sediment_time(io_sat: f64, tunables: &MemoryTunables) -> f64 {
    let t = io_sat.clamp(0.0, 1.0);
    let expire =
        tunables.min_dirty_expire + (tunables.max_dirty_expire - tunables.min_dirty_expire) * t;
    expire.clamp(tunables.min_dirty_expire, tunables.max_dirty_expire)
}

pub fn calculate_dirty_writeback(target_expire: f64, tunables: &MemoryTunables) -> f64 {
    let t_wb = (target_expire - tunables.min_dirty_expire)
        / (tunables.max_dirty_expire - tunables.min_dirty_expire);
    let wb = tunables.min_dirty_writeback
        + (tunables.max_dirty_writeback - tunables.min_dirty_writeback) * t_wb;
    wb.clamp(tunables.min_dirty_writeback, tunables.max_dirty_writeback)
}

pub fn calculate_watermark_entropy(
    p_mem: f64,
    fragmentation: f64,
    tunables: &MemoryTunables,
) -> f64 {
    let pressure_factor = (p_mem / 100.0).clamp(0.0, 1.0);
    let entropy_impact = tunables.entropy_watermark_k * fragmentation * pressure_factor;
    let target_wm = tunables.min_watermark_scale * (1.0 + entropy_impact);
    target_wm.clamp(tunables.min_watermark_scale, tunables.max_watermark_scale)
}

pub fn calculate_extfrag_physics(p_cpu: f64, tunables: &MemoryTunables) -> f64 {
    if p_cpu > 50.0 {
        tunables.max_extfrag_threshold
    } else {
        tunables.min_extfrag_threshold
    }
}

pub fn calculate_viscosity(p_cpu: f64, tunables: &MemoryTunables) -> f64 {
    if p_cpu > 25.0 {
        tunables.min_page_cluster
    } else {
        1.0
    }
}

pub fn calculate_sampling_rate(p_mem: f64, tunables: &MemoryTunables) -> f64 {
    let urgency = (p_mem / 50.0).clamp(0.0, 1.0);
    let interval = tunables.max_stat_interval
        - (urgency * (tunables.max_stat_interval - tunables.min_stat_interval));
    interval.clamp(tunables.min_stat_interval, tunables.max_stat_interval)
}

pub fn smooth_value(current: f64, target: f64, alpha: f64) -> f64 {
    current * (1.0 - alpha) + target * alpha
}