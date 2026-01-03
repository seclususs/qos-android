//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::poll_math::AdaptivePoller;
use crate::algorithms::storage_math::{self, StorageTunables};
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::config::tunables::*;
use crate::daemon::state::{update_io_pressure, update_io_saturation};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::cached_file::{CachedFile, CheckStrategy};
use crate::hal::filesystem;
use crate::hal::kernel;
use crate::monitors::disk_monitor::{DiskMonitor, IoStats};
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::*;

use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::time::Instant;

pub struct StorageController {
    fd: File,
    read_ahead: CachedFile,
    nr_requests: CachedFile,
    fifo_batch: CachedFile,
    psi_monitor: PsiMonitor,
    disk_monitor: DiskMonitor,
    prev_io_stats: IoStats,
    last_tick: Instant,
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
        let raw_fd = kernel::register_psi_trigger(K_PSI_IO_PATH, 250000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Storage PSI Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let read_ahead = CachedFile::new(filesystem::open_file_for_write(K_READ_AHEAD_PATH)?, 0);
        let nr_requests = CachedFile::new(filesystem::open_file_for_write(K_NR_REQUESTS_PATH)?, 0);
        let fifo_batch =
            CachedFile::new_opt(filesystem::open_file_for_write(K_FIFO_BATCH_PATH).ok(), 0);
        let psi_monitor = PsiMonitor::new(K_PSI_IO_PATH)?;
        let mut disk_monitor = DiskMonitor::new(K_MMC_DISKSTATS_PATH)?;
        let initial_stats = disk_monitor.read_stats().unwrap_or(IoStats::default());
        let tunables = StorageTunables {
            min_read_ahead: MIN_READ_AHEAD as f64,
            max_read_ahead: MAX_READ_AHEAD as f64,
            min_nr_requests: MIN_NR_REQUESTS as f64,
            max_nr_requests: MAX_NR_REQUESTS as f64,
            min_fifo_batch: MIN_FIFO_BATCH as f64,
            max_fifo_batch: MAX_FIFO_BATCH as f64,
            write_cost_factor: 5.0,
            target_latency_base_ms: 75.0,
            congestion_beta: 0.5,
            hysteresis_threshold: 0.15,
            panic_threshold_psi: 40.0,
            urgent_poll_psi: 10.0,
            urgent_poll_inflight: 4.0,
        };
        let poller = AdaptivePoller::new(1.0, 1.0);
        let mut controller = Self {
            fd,
            read_ahead,
            nr_requests,
            fifo_batch,
            psi_monitor,
            disk_monitor,
            prev_io_stats: initial_stats,
            last_tick: Instant::now(),
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
        let psi_data = self.psi_monitor.read_state()?;
        let current_io_stats = self.disk_monitor.read_stats()?;
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        let dt_safe = dt.max(0.001);
        let delta =
            storage_math::calculate_io_deltas(&current_io_stats, &self.prev_io_stats, dt_safe);
        self.prev_io_stats = current_io_stats;
        self.last_tick = now;
        update_io_pressure(psi_data.some.avg10);
        update_io_saturation(current_io_stats.in_flight as f64);
        let lambda_eff = storage_math::calculate_weighted_throughput(&delta, &self.tunables);
        let target_latency =
            storage_math::calculate_target_latency(psi_data.some.avg10, &self.tunables);
        let current_latency = storage_math::calculate_effective_latency(
            &delta,
            lambda_eff,
            current_io_stats.in_flight as f64,
        );
        let calculated_nr = storage_math::calculate_next_queue_depth(
            lambda_eff,
            current_latency,
            target_latency,
            self.current_nr_requests,
            psi_data.full.avg10,
            &self.tunables,
        );
        let calculated_ra = storage_math::calculate_read_ahead(
            delta.throughput_read,
            psi_data.some.avg10,
            &self.tunables,
        );
        if storage_math::should_update_nr_requests(
            calculated_nr,
            self.current_nr_requests,
            &self.tunables,
        ) {
            self.current_nr_requests = calculated_nr;
        }
        let calculated_fifo =
            storage_math::calculate_fifo_batch(self.current_nr_requests, &self.tunables);
        self.current_read_ahead = calculated_ra;
        self.current_fifo_batch = calculated_fifo;
        if psi_data.some.avg10 > self.tunables.urgent_poll_psi
            || current_io_stats.in_flight as f64 > self.tunables.urgent_poll_inflight
        {
            self.next_wake_ms = MIN_POLLING_MS as i32;
        } else {
            self.next_wake_ms = self
                .poller
                .calculate_next_interval(psi_data.some.avg10, psi_data.some.avg300)
                as i32;
        }
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let ra_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_read_ahead,
            self.tunables.max_read_ahead as u64,
            32,
        );
        let nr_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_nr_requests,
            self.tunables.min_nr_requests as u64,
            16,
        );
        let fifo_u64 = crate::algorithms::sanitize_to_u64(
            self.current_fifo_batch,
            self.tunables.max_fifo_batch as u64,
        );
        self.read_ahead
            .update(ra_u64, force, CheckStrategy::Absolute(32));
        self.nr_requests
            .update(nr_u64, force, CheckStrategy::Absolute(16));
        self.fifo_batch
            .update(fifo_u64, force, CheckStrategy::Absolute(2));
    }
}

impl EventHandler for StorageController {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
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