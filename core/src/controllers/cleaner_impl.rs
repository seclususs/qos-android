//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::state::DaemonContext;
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::thermal::ThermalSensor;
use crate::hal::traversal::{self, TraversalAction};
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::{K_BATTERY_TEMP_PATH, K_PSI_CPU_PATH, K_PSI_IO_PATH};

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, File};
use std::io::Read;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, Copy)]
struct CleanerTunables {
    sweep_interval_ms: i32,
    bloat_limit_bytes: u64,
    storage_critical_threshold: f64,
    age_stale_media: Duration,
    age_stale_code: Duration,
    age_bloat: Duration,
    age_emergency: Duration,
    age_trash: Duration,
}

impl Default for CleanerTunables {
    fn default() -> Self {
        Self {
            sweep_interval_ms: 600_000,
            bloat_limit_bytes: 512 * 1024 * 1024,
            storage_critical_threshold: 15.0,
            age_stale_media: Duration::from_secs(7 * 86400),
            age_stale_code: Duration::from_secs(30 * 86400),
            age_bloat: Duration::from_secs(24 * 3600),
            age_emergency: Duration::from_secs(3600),
            age_trash: Duration::from_secs(3600),
        }
    }
}

pub struct CleanerController {
    io_monitor: PsiMonitor,
    cpu_monitor: PsiMonitor,
    thermal: ThermalSensor,
    tunables: CleanerTunables,
    last_sweep: Instant,
    dummy_fd: File,
}

impl CleanerController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("CleanerController: Initializing...");
        let dummy = File::open("/dev/null")
            .map_err(|e| QosError::SystemCheckFailed(format!("Placeholder error: {}", e)))?;
        Ok(Self {
            io_monitor: PsiMonitor::new(K_PSI_IO_PATH)?,
            cpu_monitor: PsiMonitor::new(K_PSI_CPU_PATH)?,
            thermal: ThermalSensor::new(K_BATTERY_TEMP_PATH, 35.0),
            tunables: CleanerTunables::default(),
            last_sweep: Instant::now() - Duration::from_secs(86000),
            dummy_fd: dummy,
        })
    }
    fn get_active_packages(&self) -> HashSet<String> {
        let mut active = HashSet::new();
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
                        && let Ok(n) = f.read(&mut buf)
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
                let pct = (free as f64 / total as f64) * 100.0;
                return pct < self.tunables.storage_critical_threshold;
            }
        }
        false
    }
    #[inline(always)]
    fn is_safe_name(name: &OsStr) -> bool {
        let bytes = name.as_bytes();
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
    fn is_trash_ext(name: &OsStr) -> bool {
        let bytes = name.as_bytes();
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
    fn perform_cycle(&mut self) -> usize {
        let active_pkgs = self.get_active_packages();
        let is_critical = self.is_storage_critical();
        let now = SystemTime::now();
        let mut total_cleaned = 0;
        let tunables = self.tunables;
        for sys in ["/data/anr", "/data/tombstones"] {
            let p = Path::new(sys);
            if p.exists() {
                let policy = |entry: &DirEntry, _depth: usize| -> TraversalAction {
                    if Self::is_safe_name(&entry.file_name()) {
                        return TraversalAction::Keep;
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
                            return TraversalAction::DeleteFile;
                        }
                    }
                    TraversalAction::Keep
                };
                total_cleaned += traversal::walk_and_act(p, &policy, 0);
            }
        }
        for root in ["/data/data", "/sdcard/Android/data"] {
            let root_path = Path::new(root);
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
                        let policy = |entry: &DirEntry, _depth: usize| -> TraversalAction {
                            if !is_critical && Self::is_safe_name(&entry.file_name()) {
                                return TraversalAction::Keep;
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
                                    return TraversalAction::DeleteFile;
                                }
                            }
                            TraversalAction::Keep
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
                        let policy = |entry: &DirEntry, _depth: usize| -> TraversalAction {
                            if !is_critical && Self::is_safe_name(&entry.file_name()) {
                                return TraversalAction::Keep;
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
                                    return TraversalAction::DeleteFile;
                                }
                            }
                            TraversalAction::Keep
                        };
                        total_cleaned += traversal::walk_and_act(&code_dir, &policy, 0);
                    }
                }
            }
        }
        total_cleaned
    }
}

impl EventHandler for CleanerController {
    fn as_raw_fd(&self) -> RawFd {
        self.dummy_fd.as_raw_fd()
    }
    fn on_event(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        let now = Instant::now();
        if now.duration_since(self.last_sweep).as_millis() < self.tunables.sweep_interval_ms as u128
        {
            return Ok(LoopAction::Continue);
        }
        let io_busy = self
            .io_monitor
            .read_state()
            .map(|d| d.some.avg10 > 5.0)
            .unwrap_or(false);
        let cpu_busy = self
            .cpu_monitor
            .read_state()
            .map(|d| d.some.avg10 > 25.0)
            .unwrap_or(false);
        let temp = self.thermal.read();
        let is_emergency = self.is_storage_critical();
        if !is_emergency {
            if io_busy || cpu_busy || temp > 40.0 {
                return Ok(LoopAction::Continue);
            }
        } else if temp > 46.0
            || (cpu_busy && self.cpu_monitor.read_state().unwrap().some.avg10 > 80.0)
        {
            return Ok(LoopAction::Continue);
        }
        let items = self.perform_cycle();
        if items > 0 {
            log::info!("Cleaner: Cycle complete. Removed {} items.", items);
        }
        self.last_sweep = now;
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.tunables.sweep_interval_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::empty()
    }
}