//! Author: [Seclususs](https://github.com/seclususs)

use crate::config::loop_settings::{
    BOOT_POLL_INTERVAL_SEC, BOOT_WAIT_RETRY_LIMIT, COOLDOWN_DURATION_SEC, MAX_EPOLL_TIMEOUT_MS,
    MAX_EVENTS, STABILIZATION_DELAY_SEC,
};
use crate::daemon::state::SHUTDOWN_REQUESTED;
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::properties::get_system_property;
use crate::hal::{bridge, filesystem, properties};
use crate::registry::boot_tweaks;

use rustix::event::epoll;
use std::io::ErrorKind;
use std::os::fd::{AsRawFd, BorrowedFd, RawFd};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

pub struct RecoverableService {
    pub name: &'static str,
    pub handler: Option<Box<dyn EventHandler>>,
    pub factory: Box<dyn Fn() -> Result<Box<dyn EventHandler>, QosError> + Send + Sync>,
    pub cooldown_start: Option<Instant>,
    pub last_tick: Instant,
    pub registered_in_epoll: bool,
    pub is_permanently_disabled: bool,
}

impl RecoverableService {
    pub fn new<F>(name: &'static str, factory: F) -> Self
    where
        F: Fn() -> Result<Box<dyn EventHandler>, QosError> + Send + Sync + 'static,
    {
        let initial_cooldown = Instant::now()
            .checked_sub(Duration::from_secs(COOLDOWN_DURATION_SEC))
            .unwrap_or(Instant::now());
        Self {
            name,
            handler: None,
            factory: Box::new(factory),
            cooldown_start: Some(initial_cooldown),
            last_tick: Instant::now(),
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
                self.last_tick = Instant::now();
                true
            }
            Err(e) => {
                match &e {
                    QosError::IoError(io_err) if io_err.kind() == ErrorKind::NotFound => {
                        log::error!(
                            "Service '{}' failed FATALLY: {}. Disabling permanently.",
                            self.name,
                            e
                        );
                        self.is_permanently_disabled = true;
                    }
                    QosError::SystemCheckFailed(msg) | QosError::PermissionDenied(msg) => {
                        log::error!(
                            "Service '{}' failed: {}. Disabling permanently.",
                            self.name,
                            msg
                        );
                        self.is_permanently_disabled = true;
                    }
                    _ => {
                        log::error!(
                            "Failed to initialize service '{}': {}. Retrying...",
                            self.name,
                            e
                        );
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
                epoll_mod(
                    epoll_fd,
                    h.as_raw_fd(),
                    id,
                    libc::EPOLL_CTL_DEL,
                    epoll::EventFlags::empty(),
                );
            }
            self.registered_in_epoll = false;
        }
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
        if (op == libc::EPOLL_CTL_DEL && errno == libc::ENOENT)
            || (op == libc::EPOLL_CTL_ADD && errno == libc::EEXIST)
        {
            return true;
        }
        log::warn!("Epoll op {} failed for ID {}: {}", op, id, e);
        return false;
    }
    true
}

fn is_fatal_runtime_error(e: &QosError) -> bool {
    match e {
        QosError::IoError(io) => matches!(
            io.kind(),
            ErrorKind::NotFound | ErrorKind::BrokenPipe | ErrorKind::PermissionDenied
        ),
        QosError::SystemCheckFailed(_) | QosError::PermissionDenied(_) => true,
        _ => false,
    }
}

pub fn apply_system_tweaks() {
    log::info!("Rust: Applying boot tweaks...");
    let file_tweaks = boot_tweaks::get_file_tweaks();
    let mut success_count = 0;
    for tweak in file_tweaks.iter() {
        match filesystem::write_to_file(tweak.path, tweak.value) {
            Ok(_) => success_count += 1,
            Err(e) => log::debug!("Failed to apply tweak {}: {}", tweak.path, e),
        }
    }
    log::info!(
        "Rust: Applied {}/{} file tweaks.",
        success_count,
        file_tweaks.len()
    );
    let prop_tweaks = boot_tweaks::get_prop_tweaks();
    for tweak in prop_tweaks.iter() {
        if let Err(e) = properties::set_system_property(tweak.key, tweak.value) {
            log::warn!("Failed to set prop {}: {}", tweak.key, e);
        }
    }
    log::info!("Rust: Tweaks process finished.");
}

pub fn wait_for_boot_completion(tag: &str) {
    log::info!("Rust [{}]: Waiting for sys.boot_completed...", tag);
    let mut retry_count = 0;
    loop {
        if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
            return;
        }
        if let Ok(val) = get_system_property("sys.boot_completed")
            && val == "1"
        {
            break;
        }
        retry_count += 1;
        if retry_count > BOOT_WAIT_RETRY_LIMIT {
            log::warn!("Rust [{}]: Boot property timeout.", tag);
            break;
        }
        thread::sleep(Duration::from_secs(BOOT_POLL_INTERVAL_SEC));
    }
    if !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        log::info!(
            "Rust [{}]: Stabilizing for {}s...",
            tag,
            STABILIZATION_DELAY_SEC
        );
        thread::sleep(Duration::from_secs(STABILIZATION_DELAY_SEC));
    }
}

pub fn run_event_loop(mut services: Vec<RecoverableService>) -> Result<(), QosError> {
    let epoll_fd = epoll::create(epoll::CreateFlags::CLOEXEC)
        .map_err(|e| QosError::SystemCheckFailed(format!("Failed to create epoll: {}", e)))?;
    let cooldown_dur = Duration::from_secs(COOLDOWN_DURATION_SEC);
    let mut events: [libc::epoll_event; MAX_EVENTS] =
        [libc::epoll_event { events: 0, u64: 0 }; MAX_EVENTS];
    while !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        let now = Instant::now();
        let mut next_wakeup = now + Duration::from_millis(MAX_EPOLL_TIMEOUT_MS as u64);
        for (i, service) in services.iter_mut().enumerate() {
            if service.is_permanently_disabled {
                continue;
            }
            if service.handler.is_none() {
                if let Some(start) = service.cooldown_start {
                    let elapsed = now.duration_since(start);
                    if elapsed >= cooldown_dur {
                        if service.try_initialize()
                            && let Some(ref h) = service.handler
                        {
                            let flags = h.get_poll_flags();
                            if !epoll_mod(
                                epoll_fd.as_raw_fd(),
                                h.as_raw_fd(),
                                i as u64,
                                libc::EPOLL_CTL_ADD,
                                flags,
                            ) {
                                service.handler = None;
                                service.cooldown_start = Some(now);
                            } else {
                                service.registered_in_epoll = true;
                            }
                        }
                    } else {
                        let wakeup_time = now + (cooldown_dur - elapsed);
                        if wakeup_time < next_wakeup {
                            next_wakeup = wakeup_time;
                        }
                    }
                }
            } else if let Some(ref handler) = service.handler {
                let interval_ms = handler.get_timeout_ms();
                if interval_ms > 0 {
                    let deadline = service.last_tick + Duration::from_millis(interval_ms as u64);
                    if deadline < next_wakeup {
                        next_wakeup = deadline;
                    }
                }
            }
        }
        let min_wait_ms = next_wakeup.saturating_duration_since(now).as_millis() as i32;
        let nfds = unsafe {
            libc::epoll_wait(
                epoll_fd.as_raw_fd(),
                events.as_mut_ptr(),
                MAX_EVENTS as i32,
                min_wait_ms,
            )
        };
        if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
            break;
        }
        if nfds < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::Interrupted {
                thread::sleep(Duration::from_millis(500));
            }
            continue;
        }
        for event in events.iter().take(nfds as usize) {
            let id = event.u64 as usize;
            if let Some(service) = services.get_mut(id)
                && let Some(ref mut handler) = service.handler
            {
                match handler.on_event() {
                    Ok(LoopAction::Pause) => {
                        service.unregister_if_active(epoll_fd.as_raw_fd(), id as u64)
                    }
                    Ok(LoopAction::Resume) => {
                        if !service.registered_in_epoll {
                            let flags = handler.get_poll_flags();
                            epoll_mod(
                                epoll_fd.as_raw_fd(),
                                handler.as_raw_fd(),
                                id as u64,
                                libc::EPOLL_CTL_ADD,
                                flags,
                            );
                            service.registered_in_epoll = true;
                        }
                    }
                    Ok(LoopAction::Continue) => {}
                    Err(e) => {
                        log::error!("Service '{}' event error: {}", service.name, e);
                        service.unregister_if_active(epoll_fd.as_raw_fd(), id as u64);
                        service.handler = None;
                        service.cooldown_start = if is_fatal_runtime_error(&e) {
                            service.is_permanently_disabled = true;
                            None
                        } else {
                            Some(Instant::now())
                        };
                    }
                }
            }
        }
        let now_after_wait = Instant::now();
        for (i, service) in services.iter_mut().enumerate() {
            if service.is_permanently_disabled {
                continue;
            }
            if let Some(ref mut handler) = service.handler {
                let interval_ms = handler.get_timeout_ms();
                if interval_ms > 0 {
                    let elapsed =
                        now_after_wait.duration_since(service.last_tick).as_millis() as i32;
                    if elapsed >= interval_ms {
                        service.last_tick = now_after_wait;
                        match handler.on_timeout() {
                            Ok(LoopAction::Resume) => {
                                if !service.registered_in_epoll {
                                    let flags = handler.get_poll_flags();
                                    epoll_mod(
                                        epoll_fd.as_raw_fd(),
                                        handler.as_raw_fd(),
                                        i as u64,
                                        libc::EPOLL_CTL_ADD,
                                        flags,
                                    );
                                    service.registered_in_epoll = true;
                                }
                            }
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("Service '{}' timeout error: {}", service.name, e);
                                service.unregister_if_active(epoll_fd.as_raw_fd(), i as u64);
                                service.handler = None;
                                service.cooldown_start = if is_fatal_runtime_error(&e) {
                                    service.is_permanently_disabled = true;
                                    None
                                } else {
                                    Some(Instant::now())
                                };
                            }
                        }
                    }
                }
            }
        }
    }
    for (i, service) in services.iter_mut().enumerate() {
        service.unregister_if_active(epoll_fd.as_raw_fd(), i as u64);
    }
    bridge::notify_service_death("Shutdown Clean");
    Ok(())
}