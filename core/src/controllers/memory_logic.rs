//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::{update_memory_pressure, get_io_pressure};
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";
const K_DIRTY_RATIO: &str = "/proc/sys/vm/dirty_ratio";
const K_DIRTY_BG_RATIO: &str = "/proc/sys/vm/dirty_background_ratio";
const K_DIRTY_EXPIRE_CENTISECS: &str = "/proc/sys/vm/dirty_expire_centisecs";
const K_STAT_INTERVAL: &str = "/proc/sys/vm/stat_interval";
const K_WATERMARK_SCALE_FACTOR: &str = "/proc/sys/vm/watermark_scale_factor";
const K_EXTFRAG_THRESHOLD: &str = "/proc/sys/vm/extfrag_threshold";
const K_DIRTY_WRITEBACK_CENTISECS: &str = "/proc/sys/vm/dirty_writeback_centisecs";
const K_PAGE_CLUSTER: &str = "/proc/sys/vm/page-cluster";
const MIN_SWAPPINESS: u64 = 30;
const MAX_SWAPPINESS: u64 = 60;
const MIN_VFS: u64 = 100;
const MAX_VFS: u64 = 200;
const MAX_DIRTY: u64 = 25;
const MIN_DIRTY: u64 = 15;
const MAX_DIRTY_BG: u64 = 12;
const MIN_DIRTY_BG: u64 = 8;
const MIN_DIRTY_EXPIRE: u64 = 1500;
const MAX_DIRTY_EXPIRE: u64 = 3000;
const MIN_STAT_INTERVAL: u64 = 1;
const MAX_STAT_INTERVAL: u64 = 5;
const MIN_WATERMARK_SCALE: u64 = 8;
const MAX_WATERMARK_SCALE: u64 = 20;
const MIN_EXTFRAG_THRESHOLD: u64 = 400;
const MAX_EXTFRAG_THRESHOLD: u64 = 600;
const MIN_DIRTY_WRITEBACK: u64 = 300;
const MAX_DIRTY_WRITEBACK: u64 = 1000;
const MIN_PAGE_CLUSTER: u64 = 0;
const MAX_PAGE_CLUSTER: u64 = 2;
const PSI_MEMORY_CEILING: f64 = 35.0;
const DECAY_FACTOR: f64 = 0.1;
const POLLING_INTERVAL_MS: u64 = 2000;

struct KernelConfigCache {
    swappiness: u64,
    vfs_cache_pressure: u64,
    dirty_ratio: u64,
    dirty_bg_ratio: u64,
    dirty_expire_centisecs: u64,
    stat_interval: u64,
    watermark_scale_factor: u64,
    extfrag_threshold: u64,
    dirty_writeback_centisecs: u64,
    page_cluster: u64,
}

pub struct MemoryController {
    fd: File,
    swap_file: File,
    vfs_file: File,
    dirty_ratio_file: Option<File>,
    dirty_bg_file: Option<File>,
    dirty_expire_file: Option<File>,
    stat_interval_file: Option<File>,
    watermark_scale_file: Option<File>,
    extfrag_file: Option<File>,
    dirty_writeback_file: Option<File>,
    page_cluster_file: Option<File>,
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
    cache: KernelConfigCache,
}

impl MemoryController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("MemoryController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_MEMORY_PATH, 80000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Failed to register Memory PSI trigger: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let swap_file = fs::open_file_for_write(K_SWAPPINESS_PATH)?;
        let vfs_file = fs::open_file_for_write(K_VFS_CACHE_PRESSURE_PATH)?;
        let dirty_ratio_file = fs::open_file_for_write(K_DIRTY_RATIO).ok();
        let dirty_bg_file = fs::open_file_for_write(K_DIRTY_BG_RATIO).ok();
        let dirty_expire_file = fs::open_file_for_write(K_DIRTY_EXPIRE_CENTISECS).ok();
        let stat_interval_file = fs::open_file_for_write(K_STAT_INTERVAL).ok();
        let watermark_scale_file = fs::open_file_for_write(K_WATERMARK_SCALE_FACTOR).ok();
        let extfrag_file = fs::open_file_for_write(K_EXTFRAG_THRESHOLD).ok();
        let dirty_writeback_file = fs::open_file_for_write(K_DIRTY_WRITEBACK_CENTISECS).ok();
        let page_cluster_file = fs::open_file_for_write(K_PAGE_CLUSTER).ok();
        let psi_monitor = PsiMonitor::new(K_PSI_MEMORY_PATH)?;
        let mut controller = Self {
            fd,
            swap_file,
            vfs_file,
            dirty_ratio_file,
            dirty_bg_file,
            dirty_expire_file,
            stat_interval_file,
            watermark_scale_file,
            extfrag_file,
            dirty_writeback_file,
            page_cluster_file,
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
            cache: KernelConfigCache { 
                swappiness: 0, 
                vfs_cache_pressure: 0, 
                dirty_ratio: 0, 
                dirty_bg_ratio: 0,
                dirty_expire_centisecs: 0,
                stat_interval: 0,
                watermark_scale_factor: 0,
                extfrag_threshold: 0,
                dirty_writeback_centisecs: 0,
                page_cluster: 0,
            },
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn cubic_lerp(&self, psi: f64, max_scale: f64, start_val: u64, end_val: u64) -> f64 {
        let t = (psi / max_scale).clamp(0.0, 1.0);
        let t_cubic = t * t * t;
        let start = start_val as f64;
        let end = end_val as f64;
        start + (end - start) * t_cubic
    }
    fn update_dynamics(&mut self, mem_psi: f64) {
        update_memory_pressure(mem_psi);
        let io_psi = get_io_pressure();
        let mut target_swap = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MIN_SWAPPINESS, MAX_SWAPPINESS);
        let mut target_vfs = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MIN_VFS, MAX_VFS);
        let mut target_dirty = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MAX_DIRTY, MIN_DIRTY);
        let mut target_dirty_bg = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MAX_DIRTY_BG, MIN_DIRTY_BG);
        let target_dirty_expire = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MAX_DIRTY_EXPIRE, MIN_DIRTY_EXPIRE);
        let target_stat = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MIN_STAT_INTERVAL, MAX_STAT_INTERVAL);
        let target_watermark = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MIN_WATERMARK_SCALE, MAX_WATERMARK_SCALE);
        let target_extfrag = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MAX_EXTFRAG_THRESHOLD, MIN_EXTFRAG_THRESHOLD);
        let target_dirty_wb = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MAX_DIRTY_WRITEBACK, MIN_DIRTY_WRITEBACK);
        let target_page_cluster = self.cubic_lerp(mem_psi, PSI_MEMORY_CEILING, MAX_PAGE_CLUSTER, MIN_PAGE_CLUSTER);
        const PSI_MEM_RISING: f64 = 10.0;
        if mem_psi > PSI_MEM_RISING {
            if io_psi < 15.0 {
                target_swap = MAX_SWAPPINESS as f64;
                target_vfs = MAX_VFS as f64;
                target_dirty = MIN_DIRTY as f64;
            } else if io_psi > 25.0 {
                target_swap = MIN_SWAPPINESS as f64;
                target_dirty_bg = 12.0;
            }
        }
        let apply_smooth = |current: &mut f64, target: f64| {
            *current += (target - *current) * DECAY_FACTOR;
        };
        apply_smooth(&mut self.current_swappiness, target_swap);
        apply_smooth(&mut self.current_vfs, target_vfs);
        apply_smooth(&mut self.current_dirty, target_dirty);
        apply_smooth(&mut self.current_dirty_bg, target_dirty_bg);
        apply_smooth(&mut self.current_dirty_expire, target_dirty_expire);
        apply_smooth(&mut self.current_stat_interval, target_stat);
        apply_smooth(&mut self.current_watermark_scale, target_watermark);
        apply_smooth(&mut self.current_extfrag_threshold, target_extfrag);
        apply_smooth(&mut self.current_dirty_writeback, target_dirty_wb);
        apply_smooth(&mut self.current_page_cluster, target_page_cluster);
        self.apply_values(false);
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
        if force || self.cache.swappiness != swap_u64 {
            let s = swap_u64.to_string();
            if write_to_stream(&mut self.swap_file, &s).is_ok() {
                self.cache.swappiness = swap_u64;
            }
        }
        if force || self.cache.vfs_cache_pressure != vfs_u64 {
            let s = vfs_u64.to_string();
            if write_to_stream(&mut self.vfs_file, &s).is_ok() {
                self.cache.vfs_cache_pressure = vfs_u64;
            }
        }
        if let Some(ref mut f) = self.dirty_ratio_file {
            if force || self.cache.dirty_ratio != dirty_u64 {
                let s = dirty_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.dirty_ratio = dirty_u64; }
            }
        }
        if let Some(ref mut f) = self.dirty_bg_file {
            if force || self.cache.dirty_bg_ratio != dbg_u64 {
                let s = dbg_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.dirty_bg_ratio = dbg_u64; }
            }
        }
        if let Some(ref mut f) = self.dirty_expire_file {
            if force || self.cache.dirty_expire_centisecs != expire_u64 {
                let s = expire_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.dirty_expire_centisecs = expire_u64; }
            }
        }
        if let Some(ref mut f) = self.stat_interval_file {
            if force || self.cache.stat_interval != stat_u64 {
                let s = stat_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.stat_interval = stat_u64; }
            }
        }
        if let Some(ref mut f) = self.watermark_scale_file {
            if force || self.cache.watermark_scale_factor != wm_u64 {
                let s = wm_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.watermark_scale_factor = wm_u64; }
            }
        }
        if let Some(ref mut f) = self.extfrag_file {
            if force || self.cache.extfrag_threshold != ext_u64 {
                let s = ext_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.extfrag_threshold = ext_u64; }
            }
        }
        if let Some(ref mut f) = self.dirty_writeback_file {
            if force || self.cache.dirty_writeback_centisecs != dwb_u64 {
                let s = dwb_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.dirty_writeback_centisecs = dwb_u64; }
            }
        }
        if let Some(ref mut f) = self.page_cluster_file {
            if force || self.cache.page_cluster != pc_u64 {
                let s = pc_u64.to_string();
                if write_to_stream(f, &s).is_ok() { self.cache.page_cluster = pc_u64; }
            }
        }
    }
}

impl EventHandler for MemoryController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        POLLING_INTERVAL_MS as i32
    }
}