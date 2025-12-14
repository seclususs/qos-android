//! Author: [Seclususs](https://github.com/seclususs)

use crate::traits::EventHandler;
use crate::memory_logic::MemoryManager;
use crate::refresh_logic::RefreshManager;
use crate::storage_logic::StorageManager;
use crate::vm_logic::VmManager;
use crate::tweaker_logic::SystemTweaker;
use crate::ffi;
use crate::logger;

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::sync::Mutex;
use std::time::Duration;
use std::os::fd::{AsRawFd, OwnedFd, FromRawFd};
use std::panic;

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
static MAIN_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

const MAX_EVENTS: usize = 16;
const TWEAKER_DELAY_SECONDS: usize = 60;
const SERVICE_START_DELAY_SECONDS: usize = 60;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_start_services() {
    logger::init();
    let result = panic::catch_unwind(|| {
        log::info!("Rust: Service entry point reached.");
        thread::spawn(|| {
            log::info!("Rust: Tweaker thread waiting for boot completion ({}s)...", TWEAKER_DELAY_SECONDS);
            for _ in 0..TWEAKER_DELAY_SECONDS {
                if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                    log::info!("Rust: Shutdown requested during tweaker delay. Aborting tweaks.");
                    return;
                }
                thread::sleep(Duration::from_secs(1));
            }
            if let Err(e) = panic::catch_unwind(|| {
                SystemTweaker::apply_all();
            }) {
                log::error!("Rust: SystemTweaker panicked: {:?}", e);
            }
        });
        SHUTDOWN_REQUESTED.store(false, Ordering::Release);
        let handle = thread::spawn(|| {
            log::info!("Rust: Main service waiting for boot completion ({}s)...", SERVICE_START_DELAY_SECONDS);
            for _ in 0..SERVICE_START_DELAY_SECONDS {
                if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                    log::info!("Rust: Shutdown requested during startup delay. Aborting main loop.");
                    return;
                }
                thread::sleep(Duration::from_secs(1));
            }
            let loop_result = panic::catch_unwind(|| {
                if let Err(e) = run_event_loop() {
                    log::error!("Event loop returned error: {}", e);
                    return false;
                }
                true
            });
            match loop_result {
                Ok(success) => {
                    if !success {
                        log::error!("Rust: Event loop failed logic. Requesting restart.");
                        ffi::notify_service_death("Event Loop Failed");
                    }
                },
                Err(cause) => {
                    log::error!("Rust: Event loop PANICKED! {:?}", cause);
                    ffi::notify_service_death("Event Loop Panic");
                }
            }
        });
        match MAIN_THREAD.lock() {
            Ok(mut guard) => *guard = Some(handle),
            Err(poisoned) => {
                log::error!("Rust: MAIN_THREAD mutex is poisoned. Recovering...");
                *poisoned.into_inner() = Some(handle);
            }
        }
    });
    if let Err(cause) = result {
        log::error!("Rust: Critical Panic during startup: {:?}", cause);
        ffi::notify_service_death("Startup Panic");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_stop_services() {
    let result = panic::catch_unwind(|| {
        log::info!("Rust services stopping...");
        SHUTDOWN_REQUESTED.store(true, Ordering::Release);
        let handle_opt = match MAIN_THREAD.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take()
        };
        if let Some(handle) = handle_opt {
            if let Err(e) = handle.join() {
                log::error!("Main thread panicked during join: {:?}", e);
            }
        }
        log::info!("Rust services stopped.");
    });
    if let Err(cause) = result {
        log::error!("Rust: Critical Panic in rust_stop_services: {:?}", cause);
    }
}

fn run_event_loop() -> Result<(), crate::error::QosError> {
    use crate::error::QosError;
    let epoll_raw = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    if epoll_raw < 0 { 
        return Err(QosError::SystemCheckFailed("Failed to create epoll".to_string())); 
    }
    let epoll_fd = unsafe { OwnedFd::from_raw_fd(epoll_raw) };
    let mut managers: Vec<Box<dyn EventHandler>> = Vec::new();
    managers.push(Box::new(MemoryManager::new()?));
    managers.push(Box::new(StorageManager::new()?));
    managers.push(Box::new(RefreshManager::new()?));
    managers.push(Box::new(VmManager::new()?));
    for (i, manager) in managers.iter().enumerate() {
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | libc::EPOLLPRI | libc::EPOLLERR) as u32,
            u64: i as u64,
        };
        unsafe {
            if libc::epoll_ctl(epoll_fd.as_raw_fd(), libc::EPOLL_CTL_ADD, manager.as_raw_fd(), &mut event) < 0 {
                return Err(QosError::SystemCheckFailed(format!("Failed to register manager index {}", i)));
            }
        }
    }
    let mut events: [libc::epoll_event; MAX_EVENTS] = [libc::epoll_event { events: 0, u64: 0 }; MAX_EVENTS];
    log::info!("Event Loop Started.");
    while !SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
        let timeout = managers.iter()
            .map(|m| m.get_timeout_ms())
            .filter(|&t| t >= 0)
            .min()
            .unwrap_or(-1);
        let nfds = unsafe {
            libc::epoll_wait(epoll_fd.as_raw_fd(), events.as_mut_ptr(), MAX_EVENTS as i32, timeout)
        };
        if nfds < 0 {
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
                log::error!("Epoll wait error");
                thread::sleep(Duration::from_secs(1));
            }
            continue;
        }
        for manager in managers.iter_mut() {
            manager.on_timeout();
        }
        for i in 0..nfds as usize {
            let event = events[i];
            let index = event.u64 as usize;
            if let Some(manager) = managers.get_mut(index) {
                let res = panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    manager.on_event();
                }));
                if let Err(_) = res {
                    log::error!("Panic in manager {} on_event", index);
                }
            }
        }
    }
    Ok(())
}