//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::ffi;
use crate::common::fs::{self, write_to_stream};
use crate::common::psi::PsiMonitor;
use crate::common::traits::{EventHandler, LoopAction};
use crate::common::error::QosError;
use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;
use std::time::{Instant, Duration};

const K_PSI_MEMORY_PATH: &str = "/proc/pressure/memory";
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";
const K_DIRTY_RATIO: &str = "/proc/sys/vm/dirty_ratio";
const K_DIRTY_BG_RATIO: &str = "/proc/sys/vm/dirty_background_ratio";
const THRESHOLD_GREEN_TO_YELLOW: f64 = 8.0;
const THRESHOLD_YELLOW_TO_GREEN: f64 = 3.0;
const THRESHOLD_YELLOW_TO_RED: f64 = 35.0;
const THRESHOLD_RED_TO_YELLOW: f64 = 15.0;
const QUICK_RECHECK_MS: u64 = 5000; 
const HYSTERESIS_DURATION: Duration = Duration::from_secs(5);

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum MemoryState { Idle, Balanced, Pressure }

struct KernelConfigCache {
    swappiness: String,
    vfs_cache_pressure: String,
    dirty_ratio: String,
    dirty_bg_ratio: String,
}

impl KernelConfigCache {
    fn new() -> Self {
        Self { 
            swappiness: String::new(), 
            vfs_cache_pressure: String::new(),
            dirty_ratio: String::new(),
            dirty_bg_ratio: String::new(),
        }
    }
}

pub struct MemoryController {
    fd: File,
    current_state: MemoryState,
    cache: KernelConfigCache,
    swap_file: File,
    vfs_file: File,
    dirty_ratio_file: Option<File>,
    dirty_bg_file: Option<File>,
    psi_monitor: PsiMonitor,
    last_state_change: Instant,
    next_check: Instant,
    trigger_active: bool,
}

impl MemoryController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("MemoryController: Initializing...");
        let raw_fd = ffi::register_psi_trigger(K_PSI_MEMORY_PATH, 80000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Failed to register Memory PSI trigger: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let swap_file = fs::open_file_for_write(K_SWAPPINESS_PATH)
            .map_err(|e| QosError::SystemCheckFailed(format!("Failed to open Swappiness: {}", e)))?;
        let vfs_file = fs::open_file_for_write(K_VFS_CACHE_PRESSURE_PATH)
            .map_err(|e| QosError::SystemCheckFailed(format!("Failed to open VFS Cache: {}", e)))?;
        let dirty_ratio_file = fs::open_file_for_write(K_DIRTY_RATIO).ok();
        let dirty_bg_file = fs::open_file_for_write(K_DIRTY_BG_RATIO).ok();
        let psi_monitor = PsiMonitor::new(K_PSI_MEMORY_PATH)?;
        let mut manager = Self {
            fd,
            current_state: MemoryState::Idle,
            cache: KernelConfigCache::new(),
            swap_file,
            vfs_file,
            dirty_ratio_file,
            dirty_bg_file,
            psi_monitor,
            last_state_change: Instant::now() - HYSTERESIS_DURATION,
            next_check: Instant::now() + Duration::from_millis(QUICK_RECHECK_MS),
            trigger_active: true,
        };
        manager.apply_state(MemoryState::Idle, true);
        Ok(manager)
    }
    fn evaluate_next_state(&self, psi: f64) -> MemoryState {
        match self.current_state {
            MemoryState::Idle => {
                if psi > THRESHOLD_GREEN_TO_YELLOW { MemoryState::Balanced } else { MemoryState::Idle }
            },
            MemoryState::Balanced => {
                if psi > THRESHOLD_YELLOW_TO_RED { MemoryState::Pressure } 
                else if psi < THRESHOLD_YELLOW_TO_GREEN { MemoryState::Idle } 
                else { MemoryState::Balanced }
            },
            MemoryState::Pressure => {
                if psi < THRESHOLD_RED_TO_YELLOW { MemoryState::Balanced } else { MemoryState::Pressure }
            },
        }
    }
    fn apply_state(&mut self, new_state: MemoryState, force: bool) {
        let is_pressure_increase = new_state > self.current_state;
        if !force && !is_pressure_increase && self.last_state_change.elapsed() < HYSTERESIS_DURATION {
            return;
        }
        let (t_swap, t_vfs, t_dirty, t_dbg) = match new_state {
            MemoryState::Idle =>     ("30", "100", "25", "12"),
            MemoryState::Balanced => ("40", "150", "20", "10"),
            MemoryState::Pressure => ("60", "200", "15", "8"),
        };
        macro_rules! safe_write {
            ($file:expr, $val:expr, $cache:expr) => {
                if force || $cache != $val {
                    match write_to_stream($file, $val) {
                        Ok(_) => $cache = $val.to_string(),
                        Err(e) => log::error!("MemoryController: Apply failed: {}", e),
                    }
                }
            };
        }
        safe_write!(&mut self.swap_file, t_swap, self.cache.swappiness);
        safe_write!(&mut self.vfs_file, t_vfs, self.cache.vfs_cache_pressure);
        if let Some(ref mut f) = self.dirty_ratio_file {
            safe_write!(f, t_dirty, self.cache.dirty_ratio);
        }
        if let Some(ref mut f) = self.dirty_bg_file {
            safe_write!(f, t_dbg, self.cache.dirty_bg_ratio);
        }
        if self.current_state != new_state {
            log::info!("Unified Memory State: {:?} -> {:?}", self.current_state, new_state);
            self.current_state = new_state;
            self.last_state_change = Instant::now();
        }
    }
    fn perform_polling_check(&mut self) {
        if self.last_state_change.elapsed() < HYSTERESIS_DURATION {
            return;
        }
        match self.psi_monitor.read_avg10() {
            Ok(psi_value) => {
                let next_state = self.evaluate_next_state(psi_value);
                if next_state != self.current_state {
                    log::info!("Memory Poll: PSI={:.2} trigger change {:?}->{:?}", 
                        psi_value, self.current_state, next_state);
                    self.apply_state(next_state, false);
                }
            },
            Err(e) => {
                log::warn!("Memory Poll Error: {}", e);
            }
        }
    }
}

impl EventHandler for MemoryController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        let next_state = match self.current_state {
            MemoryState::Idle => MemoryState::Balanced,
            MemoryState::Balanced => MemoryState::Pressure,
            MemoryState::Pressure => MemoryState::Pressure,
        };
        if next_state != self.current_state {
            log::info!("Trigger Fired: Escalating to {:?}", next_state);
            self.apply_state(next_state, false);
            self.next_check = Instant::now() + Duration::from_millis(QUICK_RECHECK_MS);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> { 
        self.perform_polling_check();
        self.next_check = Instant::now() + Duration::from_millis(QUICK_RECHECK_MS);
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        if self.trigger_active && self.current_state == MemoryState::Idle {
            return -1;
        }
        let now = Instant::now();
        if now >= self.next_check {
            0
        } else {
            (self.next_check - now).as_millis() as i32
        }
    }
}