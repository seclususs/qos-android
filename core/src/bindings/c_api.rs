//! Author: [Seclususs](https://github.com/seclususs)

use crate::common::traits::{EventHandler, LoopAction};
use crate::common::state::{
    SHUTDOWN_REQUESTED, 
    CPU_SERVICE_ENABLED,
    MEMORY_SERVICE_ENABLED,
    STORAGE_SERVICE_ENABLED,
    TWEAKS_ENABLED
};
use crate::controllers::memory_logic::MemoryController;
use crate::controllers::signal_logic::SignalController;
use crate::controllers::storage_logic::StorageController;
use crate::controllers::tweaker_logic::SystemTweaker;
use crate::controllers::cpu_logic::CpuController;
use crate::bindings::ffi;
use crate::common::logger;
use crate::common::error::QosError;

use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};
use std::os::fd::{AsRawFd, RawFd, BorrowedFd};
use std::cmp;
use std::io::ErrorKind;
use rustix::event::epoll;

static MAIN_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

const MAX_EVENTS: usize = 16;
const MAX_EPOLL_TIMEOUT_MS: i32 = i32::MAX; 
const COOLDOWN_DURATION: Duration = Duration::from_secs(5);
const STABILIZATION_DELAY: Duration = Duration::from_secs(60);

fn get_service_flags(name: &str) -> epoll::EventFlags {
    match name {
        "Memory" | "Storage" | "CPU" => epoll::EventFlags::PRI | epoll::EventFlags::ERR,
        _ => epoll::EventFlags::IN | epoll::EventFlags::PRI | epoll::EventFlags::ERR,
    }
}

struct RecoverableService {
    name: &'static str,
    handler: Option<Box<dyn EventHandler>>,
    factory: Box<dyn Fn() -> Result<Box<dyn EventHandler>, QosError> + Send + Sync>, 
    cooldown_start: Option<Instant>,
    registered_in_epoll: bool,
    is_permanently_disabled: bool,
}

impl RecoverableService {
    fn new<F>(name: &'static str, factory: F) -> Self 
    where F: Fn() -> Result<Box<dyn EventHandler>, QosError> + Send + Sync + 'static {
        Self {
            name,
            handler: None,
            factory: Box::new(factory),
            cooldown_start: Some(Instant::now()), 
            registered_in_epoll: false,
            is_permanently_disabled: false,
        }
    }
    fn try_initialize(&mut self) -> bool {
        if self.is_permanently_disabled {
            return false;
        }
        match (self.factory)() {
            Ok(handler) => {
                log::info!("Service '{}' initialized successfully.", self.name);
                self.handler = Some(handler);
                self.cooldown_start = None;
                true
            },
            Err(e) => {
                match &e {
                    QosError::IoError(io_err) if io_err.kind() == ErrorKind::NotFound => {
                        log::error!("Service '{}' failed FATALLY: {}. Disabling permanently.", self.name, e);
                        self.is_permanently_disabled = true;
                    },
                    QosError::SystemCheckFailed(msg) => {
                        log::error!("Service '{}' failed SYSTEM CHECK: {}. Disabling permanently.", self.name, msg);
                        self.is_permanently_disabled = true;
                    },
                    QosError::PermissionDenied(msg) => {
                        log::error!("Service '{}' denied PERMISSION: {}. Disabling permanently.", self.name, msg);
                        self.is_permanently_disabled = true;
                    },
                    _ => {
                        log::error!("Failed to initialize service '{}': {}. Retrying in {:?}...", self.name, e, COOLDOWN_DURATION);
                        self.cooldown_start = Some(Instant::now());
                    }
                }
                false
            }
        }
    }
    fn unregister_if_active(&mut self, epoll_fd: RawFd, id: u64) {
        if self.registered_in_epoll {
            if let Some(ref h) = self.handler {
                log::debug!("Unregistering service '{}' from epoll before cleanup.", self.name);
                epoll_mod(epoll_fd, h.as_raw_fd(), id, libc::EPOLL_CTL_DEL, epoll::EventFlags::empty());
            }
            self.registered_in_epoll = false;
        }
    }
}

fn is_fatal_runtime_error(e: &QosError) -> bool {
    match e {
        QosError::IoError(io) => matches!(io.kind(), ErrorKind::NotFound | ErrorKind::BrokenPipe | ErrorKind::PermissionDenied),
        QosError::SystemCheckFailed(_) => true,
        QosError::PermissionDenied(_) => true,
        _ => false,
    }
}

fn wait_for_boot_completion(tag: &str) {
    log::info!("Rust [{}]: Waiting for sys.boot_completed...", tag);
    let mut retry_count = 0;
    loop {
        if SHUTDOWN_REQUESTED.load(Ordering::Acquire) { return; }
        match ffi::get_system_property("sys.boot_completed") {
            Ok(val) => {
                if val == "1" {
                    log::info!("Rust [{}]: Boot completed detected.", tag);
                    break;
                }
            },
            Err(_) => {}
        }
        retry_count += 1;
        if retry_count > 300 { 
            log::warn!("Rust [{}]: Boot property timeout. Proceeding anyway.", tag);
            break; 
        }
        thread::sleep(Duration::from_secs(1));
    }
    if !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        log::info!("Rust [{}]: Stabilizing for {}s...", tag, STABILIZATION_DELAY.as_secs());
        thread::sleep(STABILIZATION_DELAY);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_cpu_service_enabled(enabled: bool) {
    CPU_SERVICE_ENABLED.store(enabled, Ordering::Release);
    log::info!("Rust: CPU service enabled state set to: {}", enabled);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_memory_service_enabled(enabled: bool) {
    MEMORY_SERVICE_ENABLED.store(enabled, Ordering::Release);
    log::info!("Rust: Memory service enabled state set to: {}", enabled);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_storage_service_enabled(enabled: bool) {
    STORAGE_SERVICE_ENABLED.store(enabled, Ordering::Release);
    log::info!("Rust: Storage service enabled state set to: {}", enabled);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_tweaks_enabled(enabled: bool) {
    TWEAKS_ENABLED.store(enabled, Ordering::Release);
    log::info!("Rust: Tweaks enabled state set to: {}", enabled);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_start_services(signal_fd: i32) -> i32 {
    {
        match MAIN_THREAD.lock() {
            Ok(guard) => {
                if guard.is_some() {
                    log::error!("Rust: Attempted to start services while already running!");
                    return -1;
                }
            },
            Err(e) => {
                log::error!("Rust: MAIN_THREAD mutex poison detected: {}. Resetting...", e);
                return -1;
            }
        }
    }
    logger::init();
    let (tx, rx) = mpsc::channel::<()>();
    let result = std::panic::catch_unwind(move || {
        log::info!("Rust: Service entry point reached. Signal FD: {}", signal_fd);
        thread::spawn(|| {
            if !TWEAKS_ENABLED.load(Ordering::Acquire) {
                log::info!("Rust: System Tweaks are DISABLED by config. Skipping.");
                return;
            }
            wait_for_boot_completion("Tweaker");
            if SHUTDOWN_REQUESTED.load(Ordering::Acquire) { return; }
            SystemTweaker::apply_all();
        });
        SHUTDOWN_REQUESTED.store(false, Ordering::Release);
        let handle = thread::spawn(move || {
            if let Err(e) = tx.send(()) {
                 log::error!("Rust: Failed to send handshake: {}. (Main thread gave up?)", e);
            }
            wait_for_boot_completion("MainLoop");
            if SHUTDOWN_REQUESTED.load(Ordering::Acquire) { return; }
            log::info!("Rust: Initializing Event Loop...");
            if let Err(e) = run_event_loop(signal_fd) {
                log::error!("Fatal error in event loop: {}", e);
            }
        });
        match MAIN_THREAD.lock() {
            Ok(mut guard) => *guard = Some(handle),
            Err(poisoned) => *poisoned.into_inner() = Some(handle),
        }
    });
    if let Err(cause) = result {
        log::error!("Rust: Critical Panic during startup: {:?}", cause);
        ffi::notify_service_death("Startup Panic");
        return -1;
    }
    match rx.recv_timeout(Duration::from_secs(5)) { 
        Ok(_) => {
            log::info!("Rust: Main thread handshake received. Initialization complete.");
            0
        },
        Err(e) => {
            log::error!("Rust: Handshake failed: {}", e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_join_threads() {
    log::info!("Rust: C++ requested join threads.");
    let handle_opt = match MAIN_THREAD.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take()
    };
    if let Some(handle) = handle_opt {
        if let Err(e) = handle.join() {
            log::error!("Main thread panicked during join: {:?}", e);
        }
        log::info!("Rust: Main thread joined successfully.");
    } else {
        log::warn!("Rust: No thread to join.");
    }
}

fn epoll_mod(epoll_fd: RawFd, fd: RawFd, id: u64, op: i32, events: epoll::EventFlags) -> bool {
    let epoll_fd = unsafe { BorrowedFd::borrow_raw(epoll_fd) };
    let target_fd = unsafe { BorrowedFd::borrow_raw(fd) };
    let event_data = epoll::EventData::new_u64(id);
    let res = match op {
        libc::EPOLL_CTL_ADD => epoll::add(epoll_fd, target_fd, event_data, events),
        libc::EPOLL_CTL_DEL => epoll::delete(epoll_fd, target_fd),
        _ => return false,
    };
    if let Err(e) = res {
        let errno = e.raw_os_error();
        if (op == libc::EPOLL_CTL_DEL && errno == libc::ENOENT) || 
           (op == libc::EPOLL_CTL_ADD && errno == libc::EEXIST) {
            return true; 
        }
        log::warn!("Epoll op {} failed for ID {}: {}", op, id, e);
        return false;
    }
    true
}

fn run_event_loop(signal_fd: RawFd) -> Result<(), QosError> {
    let epoll_fd = epoll::create(epoll::CreateFlags::CLOEXEC)
        .map_err(|e| QosError::SystemCheckFailed(format!("Failed to create epoll: {}", e)))?;
    let mut services: Vec<RecoverableService> = Vec::new();
    let mut sig_service = RecoverableService::new("Signal", move || Ok(Box::new(SignalController::new(signal_fd))));
    sig_service.cooldown_start = None;
    services.push(sig_service);
    if MEMORY_SERVICE_ENABLED.load(Ordering::Acquire) {
        log::info!("Rust: Enabling MemoryController.");
        services.push(RecoverableService::new("Memory", || Ok(Box::new(MemoryController::new()?))));
    } else {
        log::info!("Rust: MemoryController service DISABLED by config.");
    }
    if STORAGE_SERVICE_ENABLED.load(Ordering::Acquire) {
        log::info!("Rust: Enabling StorageController.");
        services.push(RecoverableService::new("Storage", || Ok(Box::new(StorageController::new()?))));
    } else {
        log::info!("Rust: StorageController service DISABLED by config.");
    }
    if CPU_SERVICE_ENABLED.load(Ordering::Acquire) {
        log::info!("Rust: Enabling CpuController.");
        services.push(RecoverableService::new("CPU", || Ok(Box::new(CpuController::new()?))));
    } else {
        log::info!("Rust: CpuController service DISABLED by config.");
    }
    let mut events: [libc::epoll_event; MAX_EVENTS] = [libc::epoll_event { events: 0, u64: 0 }; MAX_EVENTS];
    while !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        let mut min_timeout_ms: i32 = -1;
        let mut min_finite_timeout_u128 = u128::MAX;
        let mut has_finite_timeout = false;
        for (i, service) in services.iter_mut().enumerate() {
            if service.is_permanently_disabled {
                continue;
            }
            if service.handler.is_none() {
                if let Some(start) = service.cooldown_start {
                    let elapsed = start.elapsed();
                    if elapsed >= COOLDOWN_DURATION {
                        log::info!("Attempting to recover service: {}", service.name);
                        if service.try_initialize() {
                            if let Some(ref h) = service.handler {
                                let flags = get_service_flags(service.name);
                                if !epoll_mod(epoll_fd.as_raw_fd(), h.as_raw_fd(), i as u64, libc::EPOLL_CTL_ADD, flags) {
                                    log::error!("Failed to register recovered service {} to epoll", service.name);
                                    service.handler = None;
                                    service.cooldown_start = Some(Instant::now());
                                } else {
                                    service.registered_in_epoll = true;
                                }
                            }
                        }
                    } else {
                        let remaining = COOLDOWN_DURATION - elapsed;
                        let remaining_u128 = remaining.as_millis();
                        if remaining_u128 < min_finite_timeout_u128 {
                            min_finite_timeout_u128 = remaining_u128;
                            has_finite_timeout = true;
                        }
                    }
                }
            } else if let Some(ref handler) = service.handler {
                let t = handler.get_timeout_ms();
                if t >= 0 {
                    let t_u128 = t as u128;
                    if t_u128 < min_finite_timeout_u128 {
                        min_finite_timeout_u128 = t_u128;
                        has_finite_timeout = true;
                    }
                }
            }
        }
        if has_finite_timeout {
            min_timeout_ms = cmp::min(min_finite_timeout_u128, MAX_EPOLL_TIMEOUT_MS as u128) as i32;
        }
        let nfds = unsafe {
            libc::epoll_wait(epoll_fd.as_raw_fd(), events.as_mut_ptr(), MAX_EVENTS as i32, min_timeout_ms)
        };
        if SHUTDOWN_REQUESTED.load(Ordering::Acquire) { break; }
        if nfds < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::Interrupted {
                log::error!("Epoll wait error: {}", err);
                thread::sleep(Duration::from_millis(500));
            }
            continue;
        }
        for i in 0..nfds as usize {
            let event = events[i];
            let id = event.u64 as usize;
            if let Some(service) = services.get_mut(id) {
                if let Some(ref mut handler) = service.handler {
                    match handler.on_event() {
                        Ok(LoopAction::Pause) => {
                             service.unregister_if_active(epoll_fd.as_raw_fd(), id as u64);
                        },
                        Ok(LoopAction::Resume) => {
                             if !service.registered_in_epoll {
                                 let flags = get_service_flags(service.name);
                                 epoll_mod(epoll_fd.as_raw_fd(), handler.as_raw_fd(), id as u64, libc::EPOLL_CTL_ADD, flags);
                                 service.registered_in_epoll = true;
                             }
                        },
                        Ok(LoopAction::Continue) => {},
                        Err(e) => {
                            log::error!("Service '{}' failed on_event: {}. Cleaning up...", service.name, e);
                            service.unregister_if_active(epoll_fd.as_raw_fd(), id as u64);
                            service.handler = None;
                            if is_fatal_runtime_error(&e) {
                                log::error!("Service '{}' encountered fatal error during runtime.", service.name);
                                service.is_permanently_disabled = true;
                                service.cooldown_start = None;
                            } else {
                                service.cooldown_start = Some(Instant::now());
                            }
                        }
                    }
                }
            }
        }
        for (i, service) in services.iter_mut().enumerate() {
            if service.is_permanently_disabled { continue; }
            if let Some(ref mut handler) = service.handler {
                if handler.get_timeout_ms() == 0 {
                    match handler.on_timeout() {
                        Ok(LoopAction::Resume) => {
                             if !service.registered_in_epoll {
                                let flags = get_service_flags(service.name);
                                epoll_mod(epoll_fd.as_raw_fd(), handler.as_raw_fd(), i as u64, libc::EPOLL_CTL_ADD, flags);
                                service.registered_in_epoll = true;
                            }
                        },
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("Service '{}' failed on_timeout: {}. Cleaning up...", service.name, e);
                            service.unregister_if_active(epoll_fd.as_raw_fd(), i as u64);
                            service.handler = None;
                            if is_fatal_runtime_error(&e) {
                                log::error!("Service '{}' encountered fatal error during timeout.", service.name);
                                service.is_permanently_disabled = true;
                                service.cooldown_start = None;
                            } else {
                                service.cooldown_start = Some(Instant::now());
                            }
                        }
                    }
                }
            }
        }
    }
    log::info!("Event Loop Exiting. Cleaning up epoll...");
    for (i, service) in services.iter_mut().enumerate() {
        service.unregister_if_active(epoll_fd.as_raw_fd(), i as u64);
    }
    Ok(())
}