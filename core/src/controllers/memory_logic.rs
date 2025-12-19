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
const MIN_SWAPPINESS: u64 = 30;
const MAX_SWAPPINESS: u64 = 60;
const MIN_VFS: u64 = 100;
const MAX_VFS: u64 = 200;
const MAX_DIRTY: u64 = 25;
const MIN_DIRTY: u64 = 15;
const MAX_DIRTY_BG: u64 = 12;
const MIN_DIRTY_BG: u64 = 8;
const PSI_MEMORY_CEILING: f64 = 35.0;
const IO_CONGESTION_THRESHOLD: f64 = 25.0;
const DECAY_FACTOR: f64 = 0.1;
const POLLING_INTERVAL_MS: u64 = 3000;

struct KernelConfigCache {
    swappiness: u64,
    vfs_cache_pressure: u64,
    dirty_ratio: u64,
    dirty_bg_ratio: u64,
}

pub struct MemoryController {
    fd: File,
    swap_file: File,
    vfs_file: File,
    dirty_ratio_file: Option<File>,
    dirty_bg_file: Option<File>,
    psi_monitor: PsiMonitor,
    current_swappiness: f64,
    current_vfs: f64,
    current_dirty: f64,
    current_dirty_bg: f64,
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
        let psi_monitor = PsiMonitor::new(K_PSI_MEMORY_PATH)?;
        let mut controller = Self {
            fd,
            swap_file,
            vfs_file,
            dirty_ratio_file,
            dirty_bg_file,
            psi_monitor,
            current_swappiness: MIN_SWAPPINESS as f64,
            current_vfs: MIN_VFS as f64,
            current_dirty: MAX_DIRTY as f64,
            current_dirty_bg: MAX_DIRTY_BG as f64,
            cache: KernelConfigCache { 
                swappiness: 0, vfs_cache_pressure: 0, 
                dirty_ratio: 0, dirty_bg_ratio: 0 
            },
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn lerp(&self, input: f64, out_min: u64, out_max: u64) -> f64 {
        let ratio = (input / PSI_MEMORY_CEILING).clamp(0.0, 1.0);
        let diff = out_max as f64 - out_min as f64;
        (out_min as f64) + (diff * ratio)
    }
    fn lerp_inv(&self, input: f64, out_start: u64, out_end: u64) -> f64 {
        let ratio = (input / PSI_MEMORY_CEILING).clamp(0.0, 1.0);
        let diff = out_start as f64 - out_end as f64;
        (out_start as f64) - (diff * ratio)
    }
    fn update_dynamics(&mut self, mem_psi: f64) {
        update_memory_pressure(mem_psi);
        let io_psi = get_io_pressure();
        let mut target_swap = self.lerp(mem_psi, MIN_SWAPPINESS, MAX_SWAPPINESS);
        let target_vfs = self.lerp(mem_psi, MIN_VFS, MAX_VFS);
        let target_dirty = self.lerp_inv(mem_psi, MAX_DIRTY, MIN_DIRTY);
        let target_dirty_bg = self.lerp_inv(mem_psi, MAX_DIRTY_BG, MIN_DIRTY_BG);
        let mut emergency_brake = false;
        if io_psi > IO_CONGESTION_THRESHOLD {
            let limit = 40.0;
            if target_swap > limit {
                log::warn!("Cross-Controller: IO Congested ({:.1}).", io_psi);
                target_swap = limit;
                emergency_brake = true;
            }
        }
        let apply_dynamic = |current: &mut f64, target: f64, is_emergency: bool| {
            if target > *current {
                *current = target;
            } else {
                if is_emergency {
                    *current = target; 
                } else {
                    *current += (target - *current) * DECAY_FACTOR;
                }
            }
        };
        let apply_dynamic_inv = |current: &mut f64, target: f64| {
            if target < *current {
                *current = target;
            } else {
                *current += (target - *current) * DECAY_FACTOR;
            }
        };
        apply_dynamic(&mut self.current_swappiness, target_swap, emergency_brake);
        apply_dynamic(&mut self.current_vfs, target_vfs, false);
        apply_dynamic_inv(&mut self.current_dirty, target_dirty);
        apply_dynamic_inv(&mut self.current_dirty_bg, target_dirty_bg);
        self.apply_values(false);
    }
    fn apply_values(&mut self, force: bool) {
        let swap_u64 = self.current_swappiness.round() as u64;
        let vfs_u64 = self.current_vfs.round() as u64;
        let dirty_u64 = self.current_dirty.round() as u64;
        let dbg_u64 = self.current_dirty_bg.round() as u64;
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