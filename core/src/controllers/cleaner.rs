//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::sys;
use crate::config::paths;
use crate::daemon::{state, traits, types};
use crate::hal::{monitors, sensors, traversal};

use std::{ffi, fs, io, os, path, sync, thread, time};

const MALLOPT_TRIM: i32 = -101;
const SECONDS_PER_DAY: u64 = 86400;

#[derive(Debug, Clone, Copy)]
struct CleanerConfig {
    sweep_interval_ms: i32,
    bloat_limit_bytes: u64,
    storage_critical_threshold: f32,
    age_stale_media: time::Duration,
    age_stale_code: time::Duration,
    age_bloat: time::Duration,
    age_emergency: time::Duration,
    age_trash: time::Duration,
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

struct CleanerWorker {
    tunables: CleanerConfig,
    sweep_rx: sync::mpsc::Receiver<()>,
}

impl CleanerWorker {
    fn new(tunables: CleanerConfig, sweep_rx: sync::mpsc::Receiver<()>) -> Self {
        Self { tunables, sweep_rx }
    }

    fn run(&mut self) {
        while self.sweep_rx.recv().is_ok() {
            let items = self.perform_cycle();
            if items > 0 {
                log::info!("Cleaner: Cycle complete. Removed {items} items.");
            }
            unsafe {
                sys::mallopt(MALLOPT_TRIM, 0);
            }
        }
    }

    fn is_storage_critical(&self) -> bool {
        let Ok(stats) = rustix::fs::statvfs("/data") else {
            return false;
        };

        let total = stats.f_blocks * stats.f_frsize;
        let free = stats.f_bavail * stats.f_frsize;

        if total > 0 {
            let percentage = (free as f32 / total as f32) * 100.0;
            return percentage < self.tunables.storage_critical_threshold;
        }
        false
    }

    #[inline]
    fn is_safe_name(name: &ffi::OsStr) -> bool {
        let bytes = os::unix::ffi::OsStrExt::as_bytes(name);
        if bytes.ends_with(b".db")
            || bytes.ends_with(b".xml")
            || bytes.ends_with(b".obb")
            || bytes.ends_with(b".pak")
            || bytes.ends_with(b".dat")
            || bytes.ends_with(b".json")
            || bytes.ends_with(b".lock")
            || bytes.ends_with(b".pref")
            || bytes.ends_with(b".conf")
        {
            return true;
        }

        if bytes.ends_with(b"-journal") || bytes.ends_with(b"-wal") || bytes.ends_with(b"-shm") {
            return true;
        }
        false
    }

    #[inline]
    fn is_trash_extension(name: &ffi::OsStr) -> bool {
        let bytes = os::unix::ffi::OsStrExt::as_bytes(name);
        bytes.ends_with(b".tmp")
            || bytes.ends_with(b".temp")
            || bytes.ends_with(b".log")
            || bytes.ends_with(b".bak")
            || bytes.ends_with(b".old")
            || bytes.ends_with(b".thumb")
            || bytes.ends_with(b".exo")
    }

    fn perform_cycle(&mut self) -> usize {
        let is_storage_critical = self.is_storage_critical();
        let now = time::SystemTime::now();
        let mut total_cleaned = 0;

        total_cleaned += self.clean_system_paths(now);
        total_cleaned += self.clean_app_caches(is_storage_critical, now);

        total_cleaned
    }

    fn clean_system_paths(&self, now: time::SystemTime) -> usize {
        let mut cleaned = 0;
        let tunables = self.tunables;

        for system_path in ["/data/anr", "/data/tombstones"] {
            let dir_path = path::Path::new(system_path);
            if !dir_path.exists() {
                continue;
            }

            let policy = |entry: &fs::DirEntry, _depth: usize| -> traversal::TraversalAction {
                if Self::is_safe_name(&entry.file_name()) {
                    return traversal::TraversalAction::Keep;
                }

                let Ok(file_metadata) = entry.metadata() else {
                    return traversal::TraversalAction::Keep;
                };

                let threshold = if Self::is_trash_extension(&entry.file_name()) {
                    tunables.age_trash
                } else {
                    tunables.age_stale_media
                };

                if let Ok(modified) = file_metadata.modified()
                    && let Ok(time_since_modified) = now.duration_since(modified)
                    && time_since_modified > threshold
                {
                    return traversal::TraversalAction::DeleteFile;
                }

                traversal::TraversalAction::Keep
            };

            cleaned += traversal::walk_and_act(dir_path, &policy, 0);
        }
        cleaned
    }

    fn clean_app_caches(&self, is_storage_critical: bool, now: time::SystemTime) -> usize {
        let mut cleaned = 0;
        let tunables = self.tunables;

        for root in ["/data/data", "/sdcard/Android/data"] {
            let root_path = path::Path::new(root);
            if !root_path.exists() {
                continue;
            }

            let Ok(entries) = fs::read_dir(root_path) else {
                continue;
            };

            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };

                if !file_type.is_dir() {
                    continue;
                }

                let app_dir = entry.path();
                let cache_dir = app_dir.join("cache");

                if cache_dir.exists() {
                    let cache_size_bytes = if is_storage_critical {
                        0
                    } else {
                        traversal::get_tree_size_capped(
                            &cache_dir,
                            tunables.bloat_limit_bytes + 1024,
                        )
                    };

                    let target_age = if is_storage_critical {
                        tunables.age_emergency
                    } else if cache_size_bytes > tunables.bloat_limit_bytes {
                        tunables.age_bloat
                    } else {
                        tunables.age_stale_media
                    };

                    let policy =
                        |entry: &fs::DirEntry, _depth: usize| -> traversal::TraversalAction {
                            if !is_storage_critical && Self::is_safe_name(&entry.file_name()) {
                                return traversal::TraversalAction::Keep;
                            }

                            let Ok(file_metadata) = entry.metadata() else {
                                return traversal::TraversalAction::Keep;
                            };

                            let threshold = if Self::is_trash_extension(&entry.file_name()) {
                                tunables.age_trash
                            } else {
                                target_age
                            };

                            if let Ok(modified) = file_metadata.modified()
                                && let Ok(time_since_modified) = now.duration_since(modified)
                                && time_since_modified > threshold
                            {
                                return traversal::TraversalAction::DeleteFile;
                            }

                            traversal::TraversalAction::Keep
                        };

                    cleaned += traversal::walk_and_act(&cache_dir, &policy, 0);
                }

                let code_dir = app_dir.join("code_cache");
                if code_dir.exists() {
                    let target_age = if is_storage_critical {
                        tunables.age_emergency
                    } else {
                        tunables.age_stale_code
                    };

                    let policy =
                        |entry: &fs::DirEntry, _depth: usize| -> traversal::TraversalAction {
                            if !is_storage_critical && Self::is_safe_name(&entry.file_name()) {
                                return traversal::TraversalAction::Keep;
                            }

                            let Ok(file_metadata) = entry.metadata() else {
                                return traversal::TraversalAction::Keep;
                            };

                            let threshold = if Self::is_trash_extension(&entry.file_name()) {
                                tunables.age_trash
                            } else {
                                target_age
                            };

                            if let Ok(modified) = file_metadata.modified()
                                && let Ok(time_since_modified) = now.duration_since(modified)
                                && time_since_modified > threshold
                            {
                                return traversal::TraversalAction::DeleteFile;
                            }

                            traversal::TraversalAction::Keep
                        };

                    cleaned += traversal::walk_and_act(&code_dir, &policy, 0);
                }
            }
        }
        cleaned
    }
}

pub struct CleanerController {
    io_monitor: monitors::PsiMonitor,
    cpu_monitor: monitors::PsiMonitor,
    thermal_sensor: sensors::ThermalSensor,
    tunables: CleanerConfig,
    last_sweep: time::Instant,
    event_fd: fs::File,
    sweep_tx: sync::mpsc::Sender<()>,
}

impl CleanerController {
    pub fn new() -> types::Result<Self> {
        log::info!("CleanerController: Initializing...");
        let event_fd_raw = rustix::event::eventfd(
            0,
            rustix::event::EventfdFlags::CLOEXEC | rustix::event::EventfdFlags::NONBLOCK,
        )
        .map_err(|e| {
            types::QosError::SystemCheckFailed(format!("Failed to create eventfd: {e}"))
        })?;
        let event_fd =
            unsafe { os::fd::FromRawFd::from_raw_fd(os::fd::IntoRawFd::into_raw_fd(event_fd_raw)) };

        let tunables = CleanerConfig::default();
        let (sweep_tx, sweep_rx) = sync::mpsc::channel();
        let worker_tunables = tunables;

        thread::Builder::new()
            .name("CleanerWorker".into())
            .stack_size(64 * 1024)
            .spawn(move || {
                let mut worker = CleanerWorker::new(worker_tunables, sweep_rx);
                worker.run();
            })
            .map_err(|e| {
                types::QosError::SystemCheckFailed(format!("Failed to spawn cleaner thread: {e}"))
            })?;

        Ok(Self {
            io_monitor: monitors::PsiMonitor::new(paths::K_PSI_IO_PATH)?,
            cpu_monitor: monitors::PsiMonitor::new(paths::K_PSI_CPU_PATH)?,
            thermal_sensor: sensors::ThermalSensor::new(paths::K_BATTERY_TEMP_PATH, 35.0),
            tunables,
            last_sweep: time::Instant::now()
                .checked_sub(time::Duration::from_secs(SECONDS_PER_DAY))
                .unwrap_or_else(time::Instant::now),
            event_fd,
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
            return percentage < self.tunables.storage_critical_threshold;
        }
        false
    }
}

impl traits::EventHandler for CleanerController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.event_fd)
    }

    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> types::Result<traits::LoopAction> {
        let mut buffer = [0u8; 8];
        let _ = io::Read::read(&mut self.event_fd, &mut buffer);
        Ok(traits::LoopAction::Continue)
    }

    fn on_timeout(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> types::Result<traits::LoopAction> {
        let now = time::Instant::now();
        if now.duration_since(self.last_sweep).as_millis() < self.tunables.sweep_interval_ms as u128
        {
            return Ok(traits::LoopAction::Continue);
        }

        let is_emergency = self.is_storage_critical();
        let temperature = self.thermal_sensor.read();

        if is_emergency {
            if temperature > 46.0 {
                return Ok(traits::LoopAction::Continue);
            }
        } else if temperature > 40.0 {
            return Ok(traits::LoopAction::Continue);
        }

        let is_io_busy = self
            .io_monitor
            .read_state()
            .is_ok_and(|d| d.some.avg10 > 3.0);

        if !is_emergency && is_io_busy {
            return Ok(traits::LoopAction::Continue);
        }

        let cpu_stats_opt = self.cpu_monitor.read_state().ok();
        let cpu_avg10 = cpu_stats_opt.as_ref().map_or(0.0, |d| d.some.avg10);
        let is_cpu_busy = cpu_avg10 > 3.0;

        if is_emergency {
            if is_cpu_busy && cpu_avg10 > 80.0 {
                return Ok(traits::LoopAction::Continue);
            }
        } else if is_cpu_busy {
            return Ok(traits::LoopAction::Continue);
        }

        match self.sweep_tx.send(()) {
            Ok(()) => self.last_sweep = now,
            Err(e) => log::error!("CleanerController: Failed to signal: {e}"),
        }
        Ok(traits::LoopAction::Continue)
    }

    fn get_timeout_ms(&self) -> i32 {
        self.tunables.sweep_interval_ms
    }

    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::empty()
    }
}
