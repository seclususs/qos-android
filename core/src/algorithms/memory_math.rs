//! Author: [Seclususs](https://github.com/seclususs)

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
    pub swap_sigmoid_k: f64,
    pub swap_sigmoid_mid: f64,
    pub dirty_decay_coeff: f64,
    pub dirty_ratio_decay: f64,
    pub watermark_sigmoid_k: f64,
    pub watermark_sigmoid_mid: f64,
    pub extfrag_cpu_threshold: f64,
    pub vfs_low_threshold: f64,
    pub vfs_high_threshold: f64,
    pub vfs_base: f64,
    pub vfs_max_val: f64,
    pub vfs_slope: f64,
    pub page_cluster_threshold: f64,
    pub cpu_pow_alpha: f64,
}

pub fn calculate_swappiness(p_curr: f64, p_avg60: f64, tunables: &MemoryTunables) -> f64 {
    let anchor_p = p_curr.max(p_avg60);
    let denom = 1.0 + (-tunables.swap_sigmoid_k * (anchor_p - tunables.swap_sigmoid_mid)).exp();
    let res = tunables.min_swappiness + ((tunables.max_swappiness - tunables.min_swappiness) / denom);
    res.clamp(tunables.min_swappiness, tunables.max_swappiness)
}

pub fn calculate_dirty_expire(p_eff: f64, tunables: &MemoryTunables) -> f64 {
    let decay = (-tunables.dirty_decay_coeff * p_eff).exp();
    let res = tunables.min_dirty_expire + (tunables.max_dirty_expire - tunables.min_dirty_expire) * decay;
    res.clamp(tunables.min_dirty_expire, tunables.max_dirty_expire)
}

pub fn calculate_stat_interval(p_cpu: f64, tunables: &MemoryTunables) -> f64 {
    let t = (p_cpu / 100.0).clamp(0.0, 1.0);
    let interval = tunables.min_stat_interval + (tunables.max_stat_interval - tunables.min_stat_interval) * t;
    interval.clamp(tunables.min_stat_interval, tunables.max_stat_interval)
}

pub fn calculate_watermark_scale(p_mem: f64, tunables: &MemoryTunables) -> f64 {
    let denom = 1.0 + (-tunables.watermark_sigmoid_k * (p_mem - tunables.watermark_sigmoid_mid)).exp();
    let res = tunables.min_watermark_scale + ((tunables.max_watermark_scale - tunables.min_watermark_scale) / denom);
    res.clamp(tunables.min_watermark_scale, tunables.max_watermark_scale)
}

pub fn calculate_extfrag_threshold(p_cpu: f64, tunables: &MemoryTunables) -> f64 {
    if p_cpu > tunables.extfrag_cpu_threshold {
        tunables.max_extfrag_threshold
    } else {
        tunables.min_extfrag_threshold
    }
}

pub fn calculate_target_vfs(p_mem: f64, tunables: &MemoryTunables) -> f64 {
    if p_mem < tunables.vfs_low_threshold {
        tunables.vfs_base
    } else if p_mem > tunables.vfs_high_threshold {
        tunables.vfs_max_val
    } else {
        tunables.vfs_base + (p_mem - tunables.vfs_low_threshold) * tunables.vfs_slope
    }
}

pub fn calculate_dirty_params(p_mem: f64, tunables: &MemoryTunables) -> (f64, f64) {
    let decay = (-tunables.dirty_ratio_decay * p_mem).exp();
    let target_dirty = tunables.min_dirty + (tunables.max_dirty - tunables.min_dirty) * decay;
    let target_dirty_bg = tunables.min_dirty_bg + (tunables.max_dirty_bg - tunables.min_dirty_bg) * decay;
    (target_dirty, target_dirty_bg)
}

pub fn calculate_dirty_writeback(target_expire: f64, tunables: &MemoryTunables) -> f64 {
    let t_wb = (target_expire - tunables.min_dirty_expire) / (tunables.max_dirty_expire - tunables.min_dirty_expire);
    let target_wb = tunables.min_dirty_writeback + (tunables.max_dirty_writeback - tunables.min_dirty_writeback) * t_wb;
    target_wb
}

pub fn calculate_page_cluster(avg10: f64, tunables: &MemoryTunables) -> f64 {
    if avg10 > tunables.page_cluster_threshold {
        tunables.min_page_cluster
    } else {
        tunables.max_page_cluster
    }
}

pub fn calculate_final_swap(s_base: f64, p_cpu: f64, i_sat: f64, tunables: &MemoryTunables) -> f64 {
    let cpu_penalty = 1.0 - (p_cpu / 100.0).powf(tunables.cpu_pow_alpha);
    let storage_penalty = if i_sat > 0.8 {
        1.0 - ((i_sat - 0.8) * 2.5)
    } else {
        1.0
    };
    s_base * cpu_penalty * storage_penalty
}