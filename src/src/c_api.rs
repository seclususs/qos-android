//! Author: [Seclususs](https://github.com/seclususs)

use crate::{ffi, traits::EventHandler};
use crate::memory_logic::MemoryManager;
use crate::refresh_logic::RefreshManager;
use crate::storage_logic::StorageManager;
use crate::vm_logic::VmManager;

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::sync::Mutex;
use std::time::Duration;
use std::os::fd::{AsRawFd, OwnedFd, FromRawFd};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
static MAIN_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

const MAX_EVENTS: usize = 16;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_start_services() {
    ffi::log_info("Rust services starting...");
    SHUTDOWN_REQUESTED.store(false, Ordering::Release);
    let handle = thread::spawn(|| {
        if let Err(e) = run_event_loop() {
            ffi::log_error(&format!("Event loop failed: {}", e));
        }
    });
    *MAIN_THREAD.lock().expect("Failed to lock MAIN_THREAD: Mutex poisoned") = Some(handle);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_stop_services() {
    ffi::log_info("Rust services stopping...");
    SHUTDOWN_REQUESTED.store(true, Ordering::Release);
    if let Some(handle) = MAIN_THREAD.lock().expect("Failed to lock MAIN_THREAD during stop").take() {
        if let Err(e) = handle.join() {
            ffi::log_error(&format!("Main thread panicked: {:?}", e));
        }
    }
    ffi::log_info("Rust services stopped.");
}

fn run_event_loop() -> Result<(), String> {
    let epoll_raw = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    if epoll_raw < 0 { return Err("Failed to create epoll".to_string()); }
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
                return Err(format!("Failed to register manager index {}", i));
            }
        }
    }
    let mut events: [libc::epoll_event; MAX_EVENTS] = unsafe { std::mem::zeroed() };
    ffi::log_info("Event Loop Started.");
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
                ffi::log_error("Epoll wait error");
                thread::sleep(Duration::from_secs(1));
            }
            continue;
        }
        for manager in managers.iter_mut() {
            manager.on_timeout();
        }
        for i in 0..nfds as usize {
            let index = events[i].u64 as usize;
            if let Some(manager) = managers.get_mut(index) {
                manager.on_event();
            }
        }
    }
    Ok(())
}