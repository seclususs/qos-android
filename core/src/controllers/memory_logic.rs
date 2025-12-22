//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::state::update_memory_pressure;
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
const MAX_PAGE_CLUSTER: u64 = 1;
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
    last_psi_raw: f64,
    smoothed_psi: f64,
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
            last_psi_raw: 0.0,
            smoothed_psi: 0.0,
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
    fn get_adaptive_smoothed_psi(&mut self, raw_psi: f64) -> f64 {
        let delta = raw_psi - self.last_psi_raw;
        let alpha = if delta > 2.0 { 0.7 } else { 0.15 }; 
        self.smoothed_psi = alpha * raw_psi + (1.0 - alpha) * self.smoothed_psi;
        self.last_psi_raw = raw_psi;
        self.smoothed_psi
    }
    fn logistic_growth(&self, psi: f64, min: f64, max: f64, midpoint: f64, steepness: f64) -> f64 {
        let denominator = 1.0 + (-steepness * (psi - midpoint)).exp();
        min + ((max - min) / denominator)
    }
    fn exponential_decay(&self, psi: f64, min: f64, max: f64, lambda: f64) -> f64 {
        min + (max - min) * (-lambda * psi).exp()
    }
    fn inverse_sigmoid(&self, psi: f64, min: f64, max: f64, midpoint: f64, steepness: f64) -> f64 {
        let denominator = 1.0 + (steepness * (psi - midpoint)).exp();
        min + ((max - min) / denominator)
    }
    fn update_dynamics_logistic(&mut self, raw_psi: f64) {
        let psi = self.get_adaptive_smoothed_psi(raw_psi);
        update_memory_pressure(psi);
        let target_swap = self.logistic_growth(psi, MIN_SWAPPINESS as f64, MAX_SWAPPINESS as f64, 30.0, 0.15);
        let vfs_calc = 100.0 + (psi * 2.0);
        let target_vfs = vfs_calc.clamp(MIN_VFS as f64, MAX_VFS as f64);
        let target_dirty = self.exponential_decay(psi, MIN_DIRTY as f64, MAX_DIRTY as f64, 0.05);
        let target_dirty_bg = self.exponential_decay(psi, MIN_DIRTY_BG as f64, MAX_DIRTY_BG as f64, 0.05);
        let t = (psi / 50.0).clamp(0.0, 1.0);
        let target_dirty_expire = MAX_DIRTY_EXPIRE as f64 - (t * (MAX_DIRTY_EXPIRE - MIN_DIRTY_EXPIRE) as f64);
        let target_dirty_wb = MAX_DIRTY_WRITEBACK as f64 - (t * (MAX_DIRTY_WRITEBACK - MIN_DIRTY_WRITEBACK) as f64);
        let target_stat = MIN_STAT_INTERVAL as f64 + (t * (MAX_STAT_INTERVAL - MIN_STAT_INTERVAL) as f64);
        let target_watermark = self.logistic_growth(psi, MIN_WATERMARK_SCALE as f64, MAX_WATERMARK_SCALE as f64, 25.0, 0.2);
        let target_extfrag = self.inverse_sigmoid(psi, MIN_EXTFRAG_THRESHOLD as f64, MAX_EXTFRAG_THRESHOLD as f64, 30.0, 0.1);
        let target_page_cluster = if psi > 40.0 {
            MIN_PAGE_CLUSTER as f64
        } else {
            MAX_PAGE_CLUSTER as f64 
        };
        self.current_swappiness = target_swap;
        self.current_vfs = target_vfs;
        self.current_dirty = target_dirty;
        self.current_dirty_bg = target_dirty_bg;
        self.current_dirty_expire = target_dirty_expire;
        self.current_stat_interval = target_stat;
        self.current_watermark_scale = target_watermark;
        self.current_extfrag_threshold = target_extfrag;
        self.current_dirty_writeback = target_dirty_wb;
        self.current_page_cluster = target_page_cluster;
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
            if write_to_stream(&mut self.swap_file, &swap_u64.to_string()).is_ok() {
                self.cache.swappiness = swap_u64;
            }
        }
        if force || self.cache.vfs_cache_pressure != vfs_u64 {
            if write_to_stream(&mut self.vfs_file, &vfs_u64.to_string()).is_ok() {
                self.cache.vfs_cache_pressure = vfs_u64;
            }
        }
        if let Some(ref mut f) = self.dirty_ratio_file {
            if force || self.cache.dirty_ratio != dirty_u64 {
                if write_to_stream(f, &dirty_u64.to_string()).is_ok() {
                    self.cache.dirty_ratio = dirty_u64;
                }
            }
        }
        if let Some(ref mut f) = self.dirty_bg_file {
            if force || self.cache.dirty_bg_ratio != dbg_u64 {
                if write_to_stream(f, &dbg_u64.to_string()).is_ok() {
                    self.cache.dirty_bg_ratio = dbg_u64;
                }
            }
        }
        if let Some(ref mut f) = self.dirty_expire_file {
            if force || self.cache.dirty_expire_centisecs != expire_u64 {
                if write_to_stream(f, &expire_u64.to_string()).is_ok() {
                    self.cache.dirty_expire_centisecs = expire_u64;
                }
            }
        }
        if let Some(ref mut f) = self.stat_interval_file {
            if force || self.cache.stat_interval != stat_u64 {
                if write_to_stream(f, &stat_u64.to_string()).is_ok() {
                    self.cache.stat_interval = stat_u64;
                }
            }
        }
        if let Some(ref mut f) = self.watermark_scale_file {
            if force || self.cache.watermark_scale_factor != wm_u64 {
                if write_to_stream(f, &wm_u64.to_string()).is_ok() {
                    self.cache.watermark_scale_factor = wm_u64;
                }
            }
        }
        if let Some(ref mut f) = self.extfrag_file {
            if force || self.cache.extfrag_threshold != ext_u64 {
                if write_to_stream(f, &ext_u64.to_string()).is_ok() {
                    self.cache.extfrag_threshold = ext_u64;
                }
            }
        }
        if let Some(ref mut f) = self.dirty_writeback_file {
            if force || self.cache.dirty_writeback_centisecs != dwb_u64 {
                if write_to_stream(f, &dwb_u64.to_string()).is_ok() {
                    self.cache.dirty_writeback_centisecs = dwb_u64;
                }
            }
        }
        if let Some(ref mut f) = self.page_cluster_file {
            if force || self.cache.page_cluster != pc_u64 {
                if write_to_stream(f, &pc_u64.to_string()).is_ok() {
                    self.cache.page_cluster = pc_u64;
                }
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
            self.update_dynamics_logistic(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Ok(psi) = self.psi_monitor.read_avg10() {
            self.update_dynamics_logistic(psi);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        POLLING_INTERVAL_MS as i32
    }
}