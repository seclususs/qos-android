//! Author: [Seclususs](https://github.com/seclususs)

use crate::config::paths;
use crate::daemon::{state, traits, types, worker};
use crate::hal::{monitors, sensors};

use std::{os, sync, thread, time};

const INITIAL_SWEEP_BACKDATE_SECS: u64 = 86400;

#[derive(Debug, Clone, Copy)]
pub struct CleanerConfig {
    pub sweep_interval_ms: i32,
    pub bloat_limit_bytes: u64,
    pub storage_critical_threshold: f32,
    pub age_stale_media: time::Duration,
    pub age_stale_code: time::Duration,
    pub age_bloat: time::Duration,
    pub age_emergency: time::Duration,
    pub age_trash: time::Duration,
}

impl Default for CleanerConfig {
    fn default() -> Self {
        Self {
            sweep_interval_ms: 600_000,
            bloat_limit_bytes: 512 * 1024 * 1024,
            storage_critical_threshold: 10.0,
            age_stale_media: time::Duration::from_hours(72),
            age_stale_code: time::Duration::from_hours(720),
            age_bloat: time::Duration::from_hours(24),
            age_emergency: time::Duration::from_hours(1),
            age_trash: time::Duration::from_hours(1),
        }
    }
}

pub struct CleanerController {
    io_monitor: monitors::PsiMonitor,
    cpu_monitor: monitors::PsiMonitor,
    thermal_sensor: sensors::ThermalSensor,
    config: CleanerConfig,
    last_sweep: time::Instant,
    sweep_tx: sync::mpsc::Sender<bool>,
}

impl CleanerController {
    pub fn new() -> types::Result<Self> {
        log::debug!("Initializing Cleaner Controller...");

        let config = CleanerConfig::default();
        let (sweep_tx, sweep_rx) = sync::mpsc::channel();

        thread::Builder::new()
            .name("CleanerWorker".into())
            .stack_size(64 * 1024)
            .spawn(move || {
                let mut worker = worker::CleanerWorker::new(config, sweep_rx);
                worker.run();
            })
            .map_err(|e| {
                types::QosError::SystemCheckFailed(format!(
                    "Failed to spawn daemon cleaner thread: {e}"
                ))
            })?;

        Ok(Self {
            io_monitor: monitors::PsiMonitor::new(paths::K_PSI_IO_PATH)?,
            cpu_monitor: monitors::PsiMonitor::new(paths::K_PSI_CPU_PATH)?,
            thermal_sensor: sensors::ThermalSensor::new(paths::K_BATTERY_TEMP_PATH, 35.0),
            config,
            last_sweep: time::Instant::now()
                .checked_sub(time::Duration::from_secs(INITIAL_SWEEP_BACKDATE_SECS))
                .unwrap_or_else(time::Instant::now),
            sweep_tx,
        })
    }

    fn is_storage_critical(&self) -> bool {
        let Ok(stats) = rustix::fs::statvfs("/data") else {
            return false;
        };

        let total = stats.f_blocks * stats.f_frsize;
        let free = stats.f_bavail * stats.f_frsize;

        if total > 0 {
            let percentage = (free as f32 / total as f32) * 100.0;
            return percentage < self.config.storage_critical_threshold;
        }
        false
    }
}

impl traits::EventHandler for CleanerController {
    fn as_raw_fd(&self) -> Option<os::fd::RawFd> {
        None
    }

    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> types::Result<traits::LoopAction> {
        Ok(traits::LoopAction::Continue)
    }

    fn on_timeout(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> types::Result<traits::LoopAction> {
        let now = time::Instant::now();

        if now.duration_since(self.last_sweep).as_millis() < self.config.sweep_interval_ms as u128 {
            return Ok(traits::LoopAction::Continue);
        }

        let is_emergency = self.is_storage_critical();

        if is_emergency {
            log::info!(
                "CRITICAL: Storage space (/data) is nearly full! Aggressive cleaning mode enabled."
            );
        }

        let temperature = self.thermal_sensor.read();

        if is_emergency {
            if temperature > 46.0 {
                log::debug!("Cleaner sweep skipped: Thermal too high ({temperature}) in emergency");
                return Ok(traits::LoopAction::Continue);
            }
        } else if temperature > 40.0 {
            log::debug!("Cleaner sweep skipped: Thermal too high ({temperature})");
            return Ok(traits::LoopAction::Continue);
        }

        let is_io_busy = self
            .io_monitor
            .read_state()
            .is_ok_and(|d| d.some.avg10 > 3.0);

        if !is_emergency && is_io_busy {
            log::debug!("Cleaner sweep skipped: IO is busy");
            return Ok(traits::LoopAction::Continue);
        }

        let cpu_stats_opt = self.cpu_monitor.read_state().ok();
        let cpu_avg10 = cpu_stats_opt.as_ref().map_or(0.0, |d| d.some.avg10);
        let is_cpu_busy = cpu_avg10 > 3.0;

        if is_emergency {
            if is_cpu_busy && cpu_avg10 > 80.0 {
                log::debug!(
                    "Cleaner sweep skipped: CPU is extremely busy ({cpu_avg10}) in emergency"
                );
                return Ok(traits::LoopAction::Continue);
            }
        } else if is_cpu_busy {
            log::debug!("Cleaner sweep skipped: CPU is busy ({cpu_avg10})");
            return Ok(traits::LoopAction::Continue);
        }

        match self.sweep_tx.send(is_emergency) {
            Ok(()) => {
                self.last_sweep = now;
                log::debug!("Cleaning cycle sent to worker. Emergency mode: {is_emergency}");
            }
            Err(e) => log::error!("Failed to signal cleaner worker: {e}"),
        }

        Ok(traits::LoopAction::Continue)
    }

    fn get_timeout_ms(&self) -> i32 {
        self.config.sweep_interval_ms
    }

    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::empty()
    }
}
