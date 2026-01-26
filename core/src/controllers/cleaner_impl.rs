//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};
use crate::hal::{thermal, traversal};
use crate::monitors::psi_monitor;
use crate::resources::sys_paths;

use std::{collections, ffi, fs, io, os, path, sync, thread, time};

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x10000001b3;

#[inline(always)]
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

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
            age_stale_media: time::Duration::from_secs(7 * 86400),
            age_stale_code: time::Duration::from_secs(30 * 86400),
            age_bloat: time::Duration::from_secs(24 * 3600),
            age_emergency: time::Duration::from_secs(3600),
            age_trash: time::Duration::from_secs(3600),
        }
    }
}

struct CleanerWorker {
    tunables: CleanerConfig,
    rx: sync::mpsc::Receiver<()>,
}

impl CleanerWorker {
    fn new(tunables: CleanerConfig, rx: sync::mpsc::Receiver<()>) -> Self {
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
    fn get_active_packages_hash(&self) -> collections::HashSet<u64> {
        let mut active_hashes = collections::HashSet::with_capacity(512);
        let mut buf = [0u8; 256];
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if !ft.is_dir() {
                        continue;
                    }
                } else {
                    continue;
                }
                let file_name = entry.file_name();
                let name_bytes = os::unix::ffi::OsStrExt::as_bytes(file_name.as_os_str());
                if !name_bytes.first().is_some_and(|b| b.is_ascii_digit()) {
                    continue;
                }
                let path = entry.path().join("cmdline");
                if let Ok(mut f) = fs::File::open(path)
                    && let Ok(n) = io::Read::read(&mut f, &mut buf)
                    && n > 0
                {
                    let slice = &buf[..n];
                    let pkg_name = slice.split(|&b| b == 0).next().unwrap_or(slice);
                    if pkg_name.contains(&b'.') {
                        active_hashes.insert(hash_bytes(pkg_name));
                    }
                }
            }
        }
        active_hashes
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
        let active_pkgs_hash = self.get_active_packages_hash();
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
                    if let Ok(ft) = entry.file_type() {
                        if !ft.is_dir() {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    let pkg_os_str = entry.file_name();
                    let pkg_bytes = os::unix::ffi::OsStrExt::as_bytes(pkg_os_str.as_os_str());
                    let pkg_hash = hash_bytes(pkg_bytes);
                    if active_pkgs_hash.contains(&pkg_hash) && !is_critical {
                        continue;
                    }
                    let app_dir = entry.path();
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
    tunables: CleanerConfig,
    last_sweep: time::Instant,
    dummy_fd: fs::File,
    tx: sync::mpsc::Sender<()>,
}

impl CleanerController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("CleanerController: Initializing...");
        let dummy = fs::File::open("/dev/null")
            .map_err(|e| types::QosError::SystemCheckFailed(format!("Placeholder error: {}", e)))?;
        let tunables = CleanerConfig::default();
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
