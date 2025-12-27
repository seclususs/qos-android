//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::cached_file::{CachedFile, CheckStrategy};
use crate::hal::filesystem;
use crate::hal::kernel;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::*;
use crate::config::tunables::*;
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::algorithms::storage_math::{self, StorageTunables};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::core::state::{update_io_pressure, update_io_saturation};
use crate::core::interfaces::{EventHandler, LoopAction};
use crate::core::types::QosError;

use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

const IO_TACTICAL_MULTIPLIER: f64 = 2.0;

pub struct StorageController {
    fd: File,
    read_ahead: CachedFile,
    nr_requests: CachedFile,
    fifo_batch: CachedFile,
    psi_monitor: PsiMonitor,
    current_read_ahead: f64,
    current_nr_requests: f64,
    current_fifo_batch: f64,
    tunables: StorageTunables,
    poller: AdaptivePoller,
    next_wake_ms: i32,
}

impl StorageController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("StorageController: Initializing...");
        let raw_fd = kernel::register_psi_trigger(K_PSI_IO_PATH, 100000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Storage PSI Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let read_ahead = CachedFile::new(filesystem::open_file_for_write(K_READ_AHEAD_PATH)?, 0);
        let nr_requests = CachedFile::new(filesystem::open_file_for_write(K_NR_REQUESTS_PATH)?, 0);
        let fifo_batch = CachedFile::new_opt(filesystem::open_file_for_write(K_FIFO_BATCH_PATH).ok(), 0);
        let psi_monitor = PsiMonitor::new(K_PSI_IO_PATH)?;
        let tunables = StorageTunables {
            min_read_ahead: MIN_READ_AHEAD as f64,
            max_read_ahead: MAX_READ_AHEAD as f64,
            min_nr_requests: MIN_NR_REQUESTS as f64,
            max_nr_requests: MAX_NR_REQUESTS as f64,
            min_fifo_batch: MIN_FIFO_BATCH as f64,
            max_fifo_batch: MAX_FIFO_BATCH as f64,
            io_sat_beta: 2.5,
            epsilon: 0.01,
            io_read_ahead_threshold: 6.0,
            io_scaling_factor: 20.0,
        };
        let poller = AdaptivePoller::new(1.0, 1.0);
        let mut controller = Self { 
            fd,
            read_ahead,
            nr_requests,
            fifo_batch,
            psi_monitor,
            current_read_ahead: MAX_READ_AHEAD as f64,
            current_nr_requests: MAX_NR_REQUESTS as f64,
            current_fifo_batch: MAX_FIFO_BATCH as f64,
            tunables,
            poller,
            next_wake_ms: MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_fluid_dynamics(&mut self) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let full = data.full;
        update_io_pressure(some.avg10);
        let i_sat = storage_math::calculate_io_saturation(full.avg10, some.avg10, &self.tunables);
        update_io_saturation(i_sat);
        if i_sat > 0.0 {
            self.next_wake_ms = MIN_POLLING_MS as i32;
            self.poller.calculate_next_interval(100.0); 
        } else {
            self.next_wake_ms = self.poller.calculate_next_interval(some.avg10) as i32;
        }
        let (target_nr, target_fifo) = storage_math::calculate_queue_params(i_sat, &self.tunables);
        let tactical_p = some.current.max(full.current * IO_TACTICAL_MULTIPLIER);
        let target_ra = storage_math::calculate_read_ahead(tactical_p, &self.tunables);
        self.current_read_ahead = target_ra;
        self.current_nr_requests = target_nr.clamp(self.tunables.min_nr_requests, self.tunables.max_nr_requests);
        self.current_fifo_batch = target_fifo.clamp(self.tunables.min_fifo_batch, self.tunables.max_fifo_batch);
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let ra_u64 = self.current_read_ahead.round() as u64;
        let nr_u64 = self.current_nr_requests.round() as u64;
        let fifo_u64 = self.current_fifo_batch.round() as u64;
        self.read_ahead.update(ra_u64, force, CheckStrategy::Absolute(32));
        self.nr_requests.update(nr_u64, force, CheckStrategy::Absolute(16));
        self.fifo_batch.update(fifo_u64, force, CheckStrategy::Absolute(2));
    }
}

impl EventHandler for StorageController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Err(e) = self.update_fluid_dynamics() {
            log::warn!("Storage Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_fluid_dynamics() {
            log::warn!("Storage Timeout Error: {}", e);
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