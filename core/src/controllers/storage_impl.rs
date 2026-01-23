//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::{poll_math, storage_math};
use crate::config::{kernel_limits, loop_settings};
use crate::daemon::{state, traits, types};
use crate::hal::{cached_file, filesystem, kernel};
use crate::monitors::{disk_monitor, psi_monitor};
use crate::resources::sys_paths;

use std::{fs, io, os, time};

pub struct StorageController {
    fd: fs::File,
    read_ahead: cached_file::CachedFile,
    nr_requests: cached_file::CachedFile,
    psi_monitor: psi_monitor::PsiMonitor,
    disk_monitor: disk_monitor::DiskMonitor,
    prev_io_stats: disk_monitor::IoStats,
    workload_state: storage_math::WorkloadState,
    storage_math_config: storage_math::StorageMathConfig,
    storage_kernel_limits: storage_math::StorageKernelLimits,
    last_tick: time::Instant,
    current_read_ahead: f32,
    current_nr_requests: f32,
    poller: poll_math::AdaptivePoller,
    next_wake_ms: i32,
}

impl StorageController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("StorageController: Initializing...");
        let config_limits = kernel_limits::GlobalConfig::default().storage_config;
        let storage_math_config = storage_math::StorageMathConfig::default();
        let storage_kernel_limits = storage_math::StorageKernelLimits {
            min_read_ahead: config_limits.min_read_ahead as f32,
            max_read_ahead: config_limits.max_read_ahead as f32,
            min_nr_requests: config_limits.min_nr_requests as f32,
            max_nr_requests: config_limits.max_nr_requests as f32,
        };
        let raw_fd = kernel::register_psi_trigger(sys_paths::K_PSI_IO_PATH, 250000, 1000000)
            .map_err(|e| types::QosError::FfiError(format!("Storage PSI Error: {}", e)))?;
        let fd = unsafe { os::fd::FromRawFd::from_raw_fd(raw_fd) };
        let ra_path = sys_paths::get_read_ahead_path();
        let nr_path = sys_paths::get_nr_requests_path();
        let read_ahead = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(ra_path.to_str().unwrap_or_default()).ok(),
            0,
        );
        let nr_requests = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(nr_path.to_str().unwrap_or_default()).ok(),
            0,
        );
        if !read_ahead.is_active() && !nr_requests.is_active() {
            return Err(types::QosError::SystemCheckFailed(
                "No storage block tunables found.".to_string(),
            ));
        }
        let psi_monitor = psi_monitor::PsiMonitor::new(sys_paths::K_PSI_IO_PATH)?;
        let stats_path = sys_paths::get_diskstats_path();
        let mut disk_monitor =
            disk_monitor::DiskMonitor::new(stats_path.to_str().unwrap_or_default())?;
        let initial_stats = disk_monitor
            .read_stats()
            .unwrap_or(disk_monitor::IoStats::default());
        let poller = poll_math::AdaptivePoller::new(1.0, 1.0, poll_math::PollerConfig::default());
        let mut controller = Self {
            fd,
            read_ahead,
            nr_requests,
            psi_monitor,
            disk_monitor,
            prev_io_stats: initial_stats,
            workload_state: storage_math::WorkloadState::default(),
            storage_math_config,
            storage_kernel_limits,
            last_tick: time::Instant::now(),
            current_read_ahead: config_limits.min_read_ahead as f32,
            current_nr_requests: config_limits.max_nr_requests as f32,
            poller,
            next_wake_ms: loop_settings::MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_io_logic(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<(), types::QosError> {
        let psi_data = self.psi_monitor.read_state()?;
        let current_io_stats = self.disk_monitor.read_stats()?;
        let now = time::Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        let dt_safe = dt.max(0.001);
        let delta =
            storage_math::calculate_io_deltas(&current_io_stats, &self.prev_io_stats, dt_safe);
        self.prev_io_stats = current_io_stats;
        self.last_tick = now;
        context.pressure.io_psi = psi_data.some.avg10;
        context.pressure.io_saturation = current_io_stats.in_flight as f32;
        let req_size_ratio =
            storage_math::calculate_request_size_ratio(&delta, &self.storage_math_config);
        let merge_ratio = storage_math::calculate_merge_ratio(&delta);
        let pressure_ratio = storage_math::calculate_pressure_ratio(
            current_io_stats.in_flight as f32,
            &self.storage_math_config,
        );
        let sequentiality = storage_math::resolve_sequentiality_factor(
            &mut self.workload_state,
            req_size_ratio,
            merge_ratio,
            pressure_ratio,
            &self.storage_math_config,
        );
        let calculated_ra =
            storage_math::calculate_target_read_ahead(sequentiality, &self.storage_kernel_limits);
        let lambda_eff =
            storage_math::calculate_weighted_throughput(&delta, &self.storage_math_config);
        let target_latency =
            storage_math::calculate_target_latency(psi_data.some.avg10, &self.storage_math_config);
        let current_latency = storage_math::calculate_effective_latency(
            &delta,
            lambda_eff,
            current_io_stats.in_flight as f32,
        );
        let calculated_nr = storage_math::calculate_next_queue_depth(
            lambda_eff,
            current_latency,
            target_latency,
            self.current_nr_requests,
            psi_data.full.avg10,
            &self.storage_math_config,
            &self.storage_kernel_limits,
        );
        if storage_math::should_update_nr_requests(
            calculated_nr,
            self.current_nr_requests,
            &self.storage_math_config,
            &self.storage_kernel_limits,
        ) {
            self.current_nr_requests = calculated_nr;
        }
        self.current_read_ahead = calculated_ra;
        if storage_math::is_congestion_critical(
            psi_data.some.avg10,
            current_io_stats.in_flight as f32,
            &self.storage_math_config,
        ) {
            self.next_wake_ms = loop_settings::MIN_POLLING_MS as i32;
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
            self.storage_kernel_limits.max_read_ahead as u64,
            32,
        );
        let nr_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_nr_requests,
            self.storage_kernel_limits.min_nr_requests as u64,
            16,
        );
        self.read_ahead
            .update(ra_u64, force, cached_file::CheckStrategy::Absolute(32));
        self.nr_requests
            .update(nr_u64, force, cached_file::CheckStrategy::Absolute(16));
    }
}

impl traits::EventHandler for StorageController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.fd)
    }
    fn on_event(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        let mut buf = [0u8; 8];
        let _ = io::Read::read(&mut self.fd, &mut buf);
        if let Err(e) = self.update_io_logic(context) {
            log::warn!("Storage Error: {}", e);
        }
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        if let Err(e) = self.update_io_logic(context) {
            log::warn!("Storage Timeout Error: {}", e);
        }
        Ok(traits::LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::PRI | rustix::event::epoll::EventFlags::ERR
    }
}
