//! Author: [Seclususs](https://github.com/seclususs)

use crate::controllers::{
    cleaner_impl, cpu_impl, display_impl, memory_impl, signal_impl, storage_impl,
};
use crate::daemon::{logging, runtime, state};
use crate::hal::bridge;

use std::{sync, thread, time};

static MAIN_THREAD: sync::Mutex<Option<thread::JoinHandle<()>>> = sync::Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_cleaner_service_enabled(enabled: bool) {
    state::CLEANER_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_cpu_service_enabled(enabled: bool) {
    state::CPU_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_display_service_enabled(enabled: bool) {
    state::DISPLAY_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_memory_service_enabled(enabled: bool) {
    state::MEMORY_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_storage_service_enabled(enabled: bool) {
    state::STORAGE_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_set_tweaks_enabled(enabled: bool) {
    state::TWEAKS_ENABLED.store(enabled, sync::atomic::Ordering::Release);
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
    let (tx, rx) = sync::mpsc::channel::<()>();
    let result = std::panic::catch_unwind(move || {
        log::info!(
            "Rust: Service entry point reached. Signal FD: {}",
            signal_fd
        );
        thread::Builder::new()
            .name("Tweaks".into())
            .stack_size(256 * 1024)
            .spawn(|| {
                if !state::TWEAKS_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    log::info!("Rust: System Tweaks are DISABLED by config.");
                    return;
                }
                runtime::wait_for_boot_completion("Tweaker");
                if state::SHUTDOWN_REQUESTED.load(sync::atomic::Ordering::Acquire) {
                    return;
                }
                runtime::apply_system_tweaks();
            })
            .expect("Failed to spawn Tweaks thread");
        state::SHUTDOWN_REQUESTED.store(false, sync::atomic::Ordering::Release);
        let handle = thread::Builder::new()
            .name("MainLoop".into())
            .stack_size(1024 * 1024)
            .spawn(move || {
                if let Err(e) = tx.send(()) {
                    log::error!("Rust: Failed to send handshake: {}.", e);
                }
                runtime::wait_for_boot_completion("MainLoop");
                if state::SHUTDOWN_REQUESTED.load(sync::atomic::Ordering::Acquire) {
                    return;
                }
                log::info!("Rust: Constructing Service Vector...");
                let mut services = Vec::new();
                services.push(runtime::RecoverableService::new("Signal", move || {
                    Ok(Box::new(unsafe {
                        signal_impl::SignalController::new(signal_fd)
                    }))
                }));
                if state::MEMORY_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Memory", || {
                        Ok(Box::new(memory_impl::MemoryController::new()?))
                    }));
                }
                if state::STORAGE_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Storage", || {
                        Ok(Box::new(storage_impl::StorageController::new()?))
                    }));
                }
                if state::CPU_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("CPU", || {
                        Ok(Box::new(cpu_impl::CpuController::new()?))
                    }));
                }
                if state::DISPLAY_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Display", || {
                        Ok(Box::new(display_impl::DisplayController::new()?))
                    }));
                }
                if state::CLEANER_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Cleaner", || {
                        Ok(Box::new(cleaner_impl::CleanerController::new()?))
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
        bridge::notify_service_death("Startup Panic");
        return -1;
    }
    match rx.recv_timeout(time::Duration::from_secs(5)) {
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
