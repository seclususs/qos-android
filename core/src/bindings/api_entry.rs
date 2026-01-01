//! Author: [Seclususs](https://github.com/seclususs)

use crate::controllers::cpu_impl::CpuController;
use crate::controllers::memory_impl::MemoryController;
use crate::controllers::signal_impl::SignalController;
use crate::controllers::storage_impl::StorageController;
use crate::daemon::logging;
use crate::daemon::runtime::{self, RecoverableService};
use crate::daemon::state::{
    CPU_SERVICE_ENABLED, MEMORY_SERVICE_ENABLED, SHUTDOWN_REQUESTED, STORAGE_SERVICE_ENABLED,
    TWEAKS_ENABLED,
};
use crate::hal::bridge::notify_service_death;

use std::sync::atomic::Ordering;
use std::sync::{Mutex, mpsc};
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

static MAIN_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_cpu_service_enabled(enabled: bool) {
    CPU_SERVICE_ENABLED.store(enabled, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_memory_service_enabled(enabled: bool) {
    MEMORY_SERVICE_ENABLED.store(enabled, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_storage_service_enabled(enabled: bool) {
    STORAGE_SERVICE_ENABLED.store(enabled, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_tweaks_enabled(enabled: bool) {
    TWEAKS_ENABLED.store(enabled, Ordering::Release);
}

/// # Safety
/// Initializes the Rust runtime and starts background services.
/// # Requirements
/// * `signal_fd` must be a valid, open file descriptor.
/// * **Ownership Transfer**: The ownership of `signal_fd` is transferred to Rust.
///   The C++ caller must NOT close or use this FD after calling this function,
///   as Rust will close it upon shutdown.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_start_services(signal_fd: i32) -> i32 {
    {
        match MAIN_THREAD.lock() {
            Ok(guard) => {
                if guard.is_some() {
                    log::error!("Rust: Attempted to start services while already running!");
                    return -1;
                }
            }
            Err(e) => {
                log::error!(
                    "Rust: MAIN_THREAD mutex poison detected: {}. Resetting...",
                    e
                );
                return -1;
            }
        }
    }
    logging::init();
    let (tx, rx) = mpsc::channel::<()>();
    let result = std::panic::catch_unwind(move || {
        log::info!(
            "Rust: Service entry point reached. Signal FD: {}",
            signal_fd
        );
        Builder::new()
            .name("Tweaks".into())
            .stack_size(256 * 1024)
            .spawn(|| {
                if !TWEAKS_ENABLED.load(Ordering::Acquire) {
                    log::info!("Rust: System Tweaks are DISABLED by config.");
                    return;
                }
                runtime::wait_for_boot_completion("Tweaker");
                if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                    return;
                }
                runtime::apply_system_tweaks();
            })
            .expect("Failed to spawn Tweaks thread");
        SHUTDOWN_REQUESTED.store(false, Ordering::Release);
        let handle = Builder::new()
            .name("MainLoop".into())
            .stack_size(1024 * 1024)
            .spawn(move || {
                if let Err(e) = tx.send(()) {
                    log::error!("Rust: Failed to send handshake: {}.", e);
                }
                runtime::wait_for_boot_completion("MainLoop");
                if SHUTDOWN_REQUESTED.load(Ordering::Acquire) {
                    return;
                }
                log::info!("Rust: Constructing Service Vector...");
                let mut services = Vec::new();
                services.push(RecoverableService::new("Signal", move || {
                    Ok(Box::new(unsafe { SignalController::new(signal_fd) }))
                }));
                if MEMORY_SERVICE_ENABLED.load(Ordering::Acquire) {
                    services.push(RecoverableService::new("Memory", || {
                        Ok(Box::new(MemoryController::new()?))
                    }));
                }
                if STORAGE_SERVICE_ENABLED.load(Ordering::Acquire) {
                    services.push(RecoverableService::new("Storage", || {
                        Ok(Box::new(StorageController::new()?))
                    }));
                }
                if CPU_SERVICE_ENABLED.load(Ordering::Acquire) {
                    services.push(RecoverableService::new("CPU", || {
                        Ok(Box::new(CpuController::new()?))
                    }));
                }
                log::info!(
                    "Rust: Initializing Event Loop with {} services...",
                    services.len()
                );
                if let Err(e) = runtime::run_event_loop(services) {
                    log::error!("Fatal error in event loop: {}", e);
                }
            })
            .expect("Failed to spawn MainLoop thread");
        match MAIN_THREAD.lock() {
            Ok(mut guard) => *guard = Some(handle),
            Err(poisoned) => *poisoned.into_inner() = Some(handle),
        }
    });
    if let Err(cause) = result {
        log::error!("Rust: Critical Panic during startup: {:?}", cause);
        notify_service_death("Startup Panic");
        return -1;
    }
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("Rust: Handshake failed: {}", e);
            -1
        }
    }
}

/// # Safety
/// Joins the main event loop thread.
/// The caller must ensure that this function is **not** called from within the
/// Rust background thread itself (e.g., via a callback), as attempting to
/// join the current thread will result in a deadlock or panic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_join_threads() {
    log::info!("Rust: C++ requested join threads.");
    let handle_opt = match MAIN_THREAD.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    if let Some(handle) = handle_opt
        && let Err(e) = handle.join()
    {
        log::error!("Main thread panicked during join: {:?}", e);
    }
}