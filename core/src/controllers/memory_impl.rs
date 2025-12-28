//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::cached_file::{CachedFile, CheckStrategy};
use crate::hal::filesystem;
use crate::hal::kernel;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::*;
use crate::config::tunables::*;
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::algorithms::memory_math::{self, MemoryTunables};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::daemon::state::{
    update_memory_pressure,
    get_io_pressure,
    get_cpu_pressure,
    get_io_saturation,
    get_thermal_state
};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;

use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

pub struct MemoryController {
    fd: File,
    swap: CachedFile,
    vfs: CachedFile,
    dirty_ratio: CachedFile,
    dirty_bg: CachedFile,
    dirty_expire: CachedFile,
    stat_interval: CachedFile,
    watermark_scale: CachedFile,
    extfrag: CachedFile,
    dirty_writeback: CachedFile,
    page_cluster: CachedFile,
    psi_monitor: PsiMonitor,
    current_swappiness: f64,
    current_vfs: f64,
    current_dirty: f64,
    current_dirty_bg: f64,
    current_dirty_expire: f64,
    current_stat_interval: f64,
    current_watermark_scale: f64,
    current_extfrag_threshold: f64,
    current_dirty_writeback: f64,
    current_page_cluster: f64,
    tunables: MemoryTunables,
    poller: AdaptivePoller,
    next_wake_ms: i32,
}

impl MemoryController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("MemoryController: Initializing...");
        let raw_fd = kernel::register_psi_trigger(K_PSI_MEMORY_PATH, 100000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Memory PSI Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let swap = CachedFile::new(filesystem::open_file_for_write(K_SWAPPINESS_PATH)?, 0);
        let vfs = CachedFile::new(filesystem::open_file_for_write(K_VFS_CACHE_PRESSURE_PATH)?, 0);
        let dirty_ratio = CachedFile::new_opt(filesystem::open_file_for_write(K_DIRTY_RATIO).ok(), 0);
        let dirty_bg = CachedFile::new_opt(filesystem::open_file_for_write(K_DIRTY_BG_RATIO).ok(), 0);
        let dirty_expire = CachedFile::new_opt(filesystem::open_file_for_write(K_DIRTY_EXPIRE_CENTISECS).ok(), 0);
        let stat_interval = CachedFile::new_opt(filesystem::open_file_for_write(K_STAT_INTERVAL).ok(), 0);
        let watermark_scale = CachedFile::new_opt(filesystem::open_file_for_write(K_WATERMARK_SCALE_FACTOR).ok(), 0);
        let extfrag = CachedFile::new_opt(filesystem::open_file_for_write(K_EXTFRAG_THRESHOLD).ok(), 0);
        let dirty_writeback = CachedFile::new_opt(filesystem::open_file_for_write(K_DIRTY_WRITEBACK_CENTISECS).ok(), 0);
        let page_cluster = CachedFile::new_opt(filesystem::open_file_for_write(K_PAGE_CLUSTER).ok(), 0);
        let psi_monitor = PsiMonitor::new(K_PSI_MEMORY_PATH)?;
        let tunables = MemoryTunables {
            min_swappiness: MIN_SWAPPINESS as f64,
            max_swappiness: MAX_SWAPPINESS as f64,
            min_dirty_expire: MIN_DIRTY_EXPIRE as f64,
            max_dirty_expire: MAX_DIRTY_EXPIRE as f64,
            min_stat_interval: MIN_STAT_INTERVAL as f64,
            max_stat_interval: MAX_STAT_INTERVAL as f64,
            min_watermark_scale: MIN_WATERMARK_SCALE as f64,
            max_watermark_scale: MAX_WATERMARK_SCALE as f64,
            min_extfrag_threshold: MIN_EXTFRAG_THRESHOLD as f64,
            max_extfrag_threshold: MAX_EXTFRAG_THRESHOLD as f64,
            min_dirty: MIN_DIRTY as f64,
            max_dirty: MAX_DIRTY as f64,
            min_dirty_bg: MIN_DIRTY_BG as f64,
            max_dirty_bg: MAX_DIRTY_BG as f64,
            min_dirty_writeback: MIN_DIRTY_WRITEBACK as f64,
            max_dirty_writeback: MAX_DIRTY_WRITEBACK as f64,
            min_page_cluster: MIN_PAGE_CLUSTER as f64,
            max_page_cluster: MAX_PAGE_CLUSTER as f64,
            min_vfs: MIN_VFS as f64,
            max_vfs: MAX_VFS as f64,
            swap_sigmoid_k: 0.15,
            swap_sigmoid_mid: 30.0,
            dirty_decay_coeff: 0.1,
            dirty_ratio_decay: 0.05,
            watermark_sigmoid_k: 0.1,
            watermark_sigmoid_mid: 30.0,
            extfrag_cpu_threshold: 40.0,
            vfs_low_threshold: 30.0,
            vfs_high_threshold: 70.0,
            vfs_base: 80.0,
            vfs_max_val: 200.0,
            vfs_slope: 3.0,
            page_cluster_threshold: 10.0,
            cpu_pow_alpha: 2.0,
            mem_smooth_fast: 0.1,
            mem_smooth_slow: 0.01,
            mem_smooth_fallback: 0.8,
            mem_pressure_high_threshold: 40.0,
        };
        let poller = AdaptivePoller::new(1.5, 0.5);
        let mut controller = Self {
            fd, 
            swap, vfs, dirty_ratio, dirty_bg, dirty_expire,
            stat_interval, watermark_scale, extfrag, dirty_writeback, page_cluster, 
            psi_monitor,
            current_swappiness: MIN_SWAPPINESS as f64,
            current_vfs: MIN_VFS as f64,
            current_dirty: MAX_DIRTY as f64,
            current_dirty_bg: MAX_DIRTY_BG as f64,
            current_dirty_expire: MAX_DIRTY_EXPIRE as f64,
            current_stat_interval: MIN_STAT_INTERVAL as f64,
            current_watermark_scale: MIN_WATERMARK_SCALE as f64,
            current_extfrag_threshold: MAX_EXTFRAG_THRESHOLD as f64,
            current_dirty_writeback: MAX_DIRTY_WRITEBACK as f64,
            current_page_cluster: MAX_PAGE_CLUSTER as f64,
            tunables,
            poller,
            next_wake_ms: MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_thermodynamics(&mut self) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let p_mem = some.current.max(some.avg10).max(data.full.avg10 * 2.0);
        let thermal_state = get_thermal_state();
        update_memory_pressure(p_mem);
        self.next_wake_ms = self.poller.calculate_next_interval(p_mem, some.avg300) as i32;
        let p_cpu = get_cpu_pressure();
        let i_sat = get_io_saturation();
        let p_io = get_io_pressure();
        let p_combined = (p_mem + p_io).min(100.0);
        let s_base = memory_math::calculate_swappiness(p_mem, some.avg60, &self.tunables, thermal_state);
        let target_swap = memory_math::calculate_final_swap(s_base, p_cpu, i_sat, &self.tunables).clamp(self.tunables.min_swappiness, self.tunables.max_swappiness);
        let target_vfs = memory_math::calculate_target_vfs(p_mem, &self.tunables);
        let (target_dirty, target_dirty_bg) = memory_math::calculate_dirty_params(p_mem, &self.tunables);
        let target_expire = memory_math::calculate_dirty_expire(p_combined, &self.tunables);
        let target_wb = memory_math::calculate_dirty_writeback(target_expire, &self.tunables);
        let target_stat = memory_math::calculate_stat_interval(p_cpu, &self.tunables);
        let target_wm = memory_math::calculate_watermark_scale(p_mem, &self.tunables);
        let target_ext = memory_math::calculate_extfrag_threshold(p_cpu, &self.tunables);
        let target_pc = memory_math::calculate_page_cluster(data.full.avg10, &self.tunables, thermal_state);
        let decay_factor = if some.avg60 > self.tunables.mem_pressure_high_threshold { 
            self.tunables.mem_smooth_slow 
        } else { 
            self.tunables.mem_smooth_fast 
        };
        let smoothing = if target_swap < self.current_swappiness { 
            self.tunables.mem_smooth_fallback 
        } else { 
            decay_factor 
        };
        self.current_swappiness = smoothing * target_swap + (1.0 - smoothing) * self.current_swappiness;
        self.current_vfs = target_vfs.clamp(self.tunables.min_vfs, self.tunables.max_vfs);
        self.current_dirty = target_dirty.clamp(self.tunables.min_dirty, self.tunables.max_dirty);
        self.current_dirty_bg = target_dirty_bg.clamp(self.tunables.min_dirty_bg, self.tunables.max_dirty_bg);
        self.current_dirty_expire = target_expire.clamp(self.tunables.min_dirty_expire, self.tunables.max_dirty_expire);
        self.current_dirty_writeback = target_wb.clamp(self.tunables.min_dirty_writeback, self.tunables.max_dirty_writeback);
        self.current_stat_interval = target_stat.clamp(self.tunables.min_stat_interval, self.tunables.max_stat_interval);
        self.current_watermark_scale = target_wm.clamp(self.tunables.min_watermark_scale, self.tunables.max_watermark_scale);
        self.current_extfrag_threshold = target_ext.clamp(self.tunables.min_extfrag_threshold, self.tunables.max_extfrag_threshold);
        self.current_page_cluster = target_pc;
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let swap_u64 = self.current_swappiness.round() as u64;
        let vfs_u64 = self.current_vfs.round() as u64;
        let dirty_u64 = self.current_dirty.round() as u64;
        let dbg_u64 = self.current_dirty_bg.round() as u64;
        let expire_u64 = self.current_dirty_expire.round() as u64;
        let stat_u64 = self.current_stat_interval.round() as u64;
        let wm_u64 = self.current_watermark_scale.round() as u64;
        let ext_u64 = self.current_extfrag_threshold.round() as u64;
        let dwb_u64 = self.current_dirty_writeback.round() as u64;
        let pc_u64 = self.current_page_cluster.round() as u64;
        self.swap.update(swap_u64, force, CheckStrategy::Absolute(3));
        self.vfs.update(vfs_u64, force, CheckStrategy::Absolute(10));
        self.dirty_ratio.update(dirty_u64, force, CheckStrategy::Absolute(1));
        self.dirty_bg.update(dbg_u64, force, CheckStrategy::Absolute(1));
        self.dirty_expire.update(expire_u64, force, CheckStrategy::Relative(0.05));
        self.stat_interval.update(stat_u64, force, CheckStrategy::Strict);
        self.watermark_scale.update(wm_u64, force, CheckStrategy::Absolute(2));
        self.extfrag.update(ext_u64, force, CheckStrategy::Absolute(25));
        self.dirty_writeback.update(dwb_u64, force, CheckStrategy::Relative(0.10));
        self.page_cluster.update(pc_u64, force, CheckStrategy::Strict);
    }
}

impl EventHandler for MemoryController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Err(e) = self.update_thermodynamics() {
            log::warn!("Mem Logic Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_thermodynamics() {
            log::warn!("Mem Timeout Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::PRI | rustix::event::epoll::EventFlags::ERR
    }
}