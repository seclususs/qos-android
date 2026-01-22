//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};
use crate::hal::{thermal, traversal};
use crate::monitors::psi_monitor;
use crate::resources::sys_paths;

use std::{collections, ffi, fs, io, os, path, sync, thread, time};

#[derive(Debug, Clone, Copy)]
struct CleanerTunables {
    sweep_interval_ms: i32,
    bloat_limit_bytes: u64,
    storage_critical_threshold: f32,
    age_stale_media: time::Duration,
    age_stale_code: time::Duration,
    age_bloat: time::Duration,
    age_emergency: time::Duration,
    age_trash: time::Duration,
}

impl Default for CleanerTunables {
    fn default() -> Self {
        Self {
            sweep_interval_ms: 600_000,
            bloat_limit_bytes: 512 * 1024 * 1024,
            storage_critical_threshold: 10.0,
            age_stale_media: time::Duration::from_secs(7 * 86400),
            age_stale_code: time::Duration::from_secs(30 * 86400),
            age_bloat: time::Duration::from_secs(24 * 3600),
            age_emergency: time::Duration::from_secs(3600),
            age_trash: time::Duration::from_secs(3600),
        }
    }
}

struct CleanerWorker {
    tunables: CleanerTunables,
    rx: sync::mpsc::Receiver<()>,
}

impl CleanerWorker {
    fn new(tunables: CleanerTunables, rx: sync::mpsc::Receiver<()>) -> Self {
        Self { tunables, rx }
    }
    fn run(&self) {
        while self.rx.recv().is_ok() {
            let items = self.perform_cycle();
            if items > 0 {
                log::info!("Cleaner: Cycle complete. Removed {} items.", items);
            }
        }
    }
    fn get_active_packages(&self) -> collections::HashSet<String> {
        let mut active = collections::HashSet::new();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.as_bytes().first().is_some_and(|b| b.is_ascii_digit())
                {
                    let mut buf = [0u8; 128];
                    if let Ok(mut f) = fs::File::open(path.join("cmdline"))
                        && let Ok(n) = io::Read::read(&mut f, &mut buf)
                        && let Ok(s) = std::str::from_utf8(&buf[..n])
                    {
                        let pkg = s.split('\0').next().unwrap_or("").trim();
                        if pkg.contains('.') {
                            active.insert(pkg.to_string());
                        }
                    }
                }
            }
        }
        active
    }
    fn is_storage_critical(&self) -> bool {
        if let Ok(stats) = rustix::fs::statvfs("/data") {
            let total = stats.f_blocks * stats.f_frsize;
            let free = stats.f_bavail * stats.f_frsize;
            if total > 0 {
                let pct = (free as f32 / total as f32) * 100.0;
                return pct < self.tunables.storage_critical_threshold;
            }
        }
        false
    }
    #[inline(always)]
    fn is_safe_name(name: &ffi::OsStr) -> bool {
        let bytes = os::unix::ffi::OsStrExt::as_bytes(name);
        let len = bytes.len();
        if len > 3 && bytes[len - 3..] == *b".db" {
            return true;
        }
        if len > 4 {
            let tail = &bytes[len - 4..];
            if tail == b".xml" || tail == b".obb" || tail == b".pak" || tail == b".dat" {
                return true;
            }
        }
        if len > 5 {
            let tail = &bytes[len - 5..];
            if tail == b".json" || tail == b".lock" || tail == b".pref" || tail == b".conf" {
                return true;
            }
        }
        if bytes.ends_with(b"-journal") || bytes.ends_with(b"-wal") || bytes.ends_with(b"-shm") {
            return true;
        }
        false
    }
    #[inline(always)]
    fn is_trash_ext(name: &ffi::OsStr) -> bool {
        let bytes = os::unix::ffi::OsStrExt::as_bytes(name);
        if bytes.ends_with(b".tmp")
            || bytes.ends_with(b".temp")
            || bytes.ends_with(b".log")
            || bytes.ends_with(b".bak")
            || bytes.ends_with(b".old")
            || bytes.ends_with(b".thumb")
            || bytes.ends_with(b".exo")
        {
            return true;
        }
        false
    }
    fn perform_cycle(&self) -> usize {
        let active_pkgs = self.get_active_packages();
        let is_critical = self.is_storage_critical();
        let now = time::SystemTime::now();
        let mut total_cleaned = 0;
        let tunables = self.tunables;
        for sys in ["/data/anr", "/data/tombstones"] {
            let p = path::Path::new(sys);
            if p.exists() {
                let policy = |entry: &fs::DirEntry, _depth: usize| -> traversal::TraversalAction {
                    if Self::is_safe_name(&entry.file_name()) {
                        return traversal::TraversalAction::Keep;
                    }
                    if let Ok(meta) = entry.metadata() {
                        let threshold = if Self::is_trash_ext(&entry.file_name()) {
                            tunables.age_trash
                        } else {
                            tunables.age_stale_media
                        };
                        if let Ok(modified) = meta.modified()
                            && let Ok(diff) = now.duration_since(modified)
                            && diff > threshold
                        {
                            return traversal::TraversalAction::DeleteFile;
                        }
                    }
                    traversal::TraversalAction::Keep
                };
                total_cleaned += traversal::walk_and_act(p, &policy, 0);
            }
        }
        for root in ["/data/data", "/sdcard/Android/data"] {
            let root_path = path::Path::new(root);
            if !root_path.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(root_path) {
                for entry in entries.flatten() {
                    let app_dir = entry.path();
                    if !app_dir.is_dir() {
                        continue;
                    }
                    let pkg = app_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if active_pkgs.contains(pkg) && !is_critical {
                        continue;
                    }
                    let cache_dir = app_dir.join("cache");
                    if cache_dir.exists() {
                        let size = traversal::get_tree_size_capped(
                            &cache_dir,
                            tunables.bloat_limit_bytes + 1024,
                        );
                        let age = if is_critical {
                            tunables.age_emergency
                        } else if size > tunables.bloat_limit_bytes {
                            tunables.age_bloat
                        } else {
                            tunables.age_stale_media
                        };
                        let policy =
                            |entry: &fs::DirEntry, _depth: usize| -> traversal::TraversalAction {
                                if !is_critical && Self::is_safe_name(&entry.file_name()) {
                                    return traversal::TraversalAction::Keep;
                                }
                                if let Ok(meta) = entry.metadata() {
                                    let threshold = if Self::is_trash_ext(&entry.file_name()) {
                                        tunables.age_trash
                                    } else {
                                        age
                                    };
                                    if let Ok(modified) = meta.modified()
                                        && let Ok(diff) = now.duration_since(modified)
                                        && diff > threshold
                                    {
                                        return traversal::TraversalAction::DeleteFile;
                                    }
                                }
                                traversal::TraversalAction::Keep
                            };
                        total_cleaned += traversal::walk_and_act(&cache_dir, &policy, 0);
                    }
                    let code_dir = app_dir.join("code_cache");
                    if code_dir.exists() {
                        let age = if is_critical {
                            tunables.age_emergency
                        } else {
                            tunables.age_stale_code
                        };
                        let policy =
                            |entry: &fs::DirEntry, _depth: usize| -> traversal::TraversalAction {
                                if !is_critical && Self::is_safe_name(&entry.file_name()) {
                                    return traversal::TraversalAction::Keep;
                                }
                                if let Ok(meta) = entry.metadata() {
                                    let threshold = if Self::is_trash_ext(&entry.file_name()) {
                                        tunables.age_trash
                                    } else {
                                        age
                                    };
                                    if let Ok(modified) = meta.modified()
                                        && let Ok(diff) = now.duration_since(modified)
                                        && diff > threshold
                                    {
                                        return traversal::TraversalAction::DeleteFile;
                                    }
                                }
                                traversal::TraversalAction::Keep
                            };
                        total_cleaned += traversal::walk_and_act(&code_dir, &policy, 0);
                    }
                }
            }
        }
        total_cleaned
    }
}

pub struct CleanerController {
    io_monitor: psi_monitor::PsiMonitor,
    cpu_monitor: psi_monitor::PsiMonitor,
    thermal: thermal::ThermalSensor,
    tunables: CleanerTunables,
    last_sweep: time::Instant,
    dummy_fd: fs::File,
    tx: sync::mpsc::Sender<()>,
}

impl CleanerController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("CleanerController: Initializing...");
        let dummy = fs::File::open("/dev/null")
            .map_err(|e| types::QosError::SystemCheckFailed(format!("Placeholder error: {}", e)))?;
        let tunables = CleanerTunables::default();
        let (tx, rx) = sync::mpsc::channel();
        let worker_tunables = tunables;
        thread::Builder::new()
            .name("CleanerWorker".into())
            .spawn(move || {
                let worker = CleanerWorker::new(worker_tunables, rx);
                worker.run();
            })
            .map_err(|e| {
                types::QosError::SystemCheckFailed(format!("Failed to spawn cleaner thread: {}", e))
            })?;
        Ok(Self {
            io_monitor: psi_monitor::PsiMonitor::new(sys_paths::K_PSI_IO_PATH)?,
            cpu_monitor: psi_monitor::PsiMonitor::new(sys_paths::K_PSI_CPU_PATH)?,
            thermal: thermal::ThermalSensor::new(sys_paths::K_BATTERY_TEMP_PATH, 35.0),
            tunables,
            last_sweep: time::Instant::now() - time::Duration::from_secs(86000),
            dummy_fd: dummy,
            tx,
        })
    }
    fn is_storage_critical(&self) -> bool {
        if let Ok(stats) = rustix::fs::statvfs("/data") {
            let total = stats.f_blocks * stats.f_frsize;
            let free = stats.f_bavail * stats.f_frsize;
            if total > 0 {
                let pct = (free as f32 / total as f32) * 100.0;
                return pct < self.tunables.storage_critical_threshold;
            }
        }
        false
    }
}

impl traits::EventHandler for CleanerController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.dummy_fd)
    }
    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        let now = time::Instant::now();
        if now.duration_since(self.last_sweep).as_millis() < self.tunables.sweep_interval_ms as u128
        {
            return Ok(traits::LoopAction::Continue);
        }
        let io_busy = self
            .io_monitor
            .read_state()
            .map(|d| d.some.avg10 > 5.0)
            .unwrap_or(false);
        let cpu_busy = self
            .cpu_monitor
            .read_state()
            .map(|d| d.some.avg10 > 10.0)
            .unwrap_or(false);
        let temp = self.thermal.read();
        let is_emergency = self.is_storage_critical();
        if !is_emergency {
            if io_busy || cpu_busy || temp > 40.0 {
                return Ok(traits::LoopAction::Continue);
            }
        } else if temp > 46.0
            || (cpu_busy && self.cpu_monitor.read_state().unwrap().some.avg10 > 80.0)
        {
            return Ok(traits::LoopAction::Continue);
        }
        match self.tx.send(()) {
            Ok(_) => {
                self.last_sweep = now;
            }
            Err(e) => {
                log::error!("CleanerController: Failed to signal worker: {}", e);
            }
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
