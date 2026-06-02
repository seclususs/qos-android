//! Author: [Seclususs](https://github.com/seclususs)

use crate::config::limits;
use crate::controllers::{blocker, cleaner, cpu, signal, storage};
use crate::daemon::{bridge, logging, runtime, state, types};

use std::{os, sync, thread, time};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiCpuLimits {
    pub min_latency_ns: u64,
    pub max_latency_ns: u64,
    pub min_granularity_ns: u64,
    pub max_granularity_ns: u64,
    pub min_wakeup_ns: u64,
    pub max_wakeup_ns: u64,
    pub min_migration_cost: u64,
    pub max_migration_cost: u64,
    pub min_walt_init_pct: u64,
    pub max_walt_init_pct: u64,
    pub min_uclamp_min: u64,
    pub max_uclamp_min: u64,
}

impl From<FfiCpuLimits> for limits::CpuLimitsConfig {
    fn from(ffi: FfiCpuLimits) -> Self {
        let def = Self::default();

        let resolve = |val: u64, default: u64| {
            if val == u64::MAX { default } else { val }
        };

        let mut config = Self {
            min_latency_ns: resolve(ffi.min_latency_ns, def.min_latency_ns),
            max_latency_ns: resolve(ffi.max_latency_ns, def.max_latency_ns),
            min_granularity_ns: resolve(ffi.min_granularity_ns, def.min_granularity_ns),
            max_granularity_ns: resolve(ffi.max_granularity_ns, def.max_granularity_ns),
            min_wakeup_ns: resolve(ffi.min_wakeup_ns, def.min_wakeup_ns),
            max_wakeup_ns: resolve(ffi.max_wakeup_ns, def.max_wakeup_ns),
            min_migration_cost: resolve(ffi.min_migration_cost, def.min_migration_cost),
            max_migration_cost: resolve(ffi.max_migration_cost, def.max_migration_cost),
            min_walt_init_pct: resolve(ffi.min_walt_init_pct, def.min_walt_init_pct),
            max_walt_init_pct: resolve(ffi.max_walt_init_pct, def.max_walt_init_pct),
            min_uclamp_min: resolve(ffi.min_uclamp_min, def.min_uclamp_min),
            max_uclamp_min: resolve(ffi.max_uclamp_min, def.max_uclamp_min),
        };

        if config.min_latency_ns > config.max_latency_ns {
            std::mem::swap(&mut config.min_latency_ns, &mut config.max_latency_ns);
        }

        if config.min_granularity_ns > config.max_granularity_ns {
            std::mem::swap(
                &mut config.min_granularity_ns,
                &mut config.max_granularity_ns,
            );
        }

        if config.min_wakeup_ns > config.max_wakeup_ns {
            std::mem::swap(&mut config.min_wakeup_ns, &mut config.max_wakeup_ns);
        }

        if config.min_migration_cost > config.max_migration_cost {
            std::mem::swap(
                &mut config.min_migration_cost,
                &mut config.max_migration_cost,
            );
        }

        if config.min_walt_init_pct > config.max_walt_init_pct {
            std::mem::swap(&mut config.min_walt_init_pct, &mut config.max_walt_init_pct);
        }

        if config.min_uclamp_min > config.max_uclamp_min {
            std::mem::swap(&mut config.min_uclamp_min, &mut config.max_uclamp_min);
        }

        config
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiStorageLimits {
    pub min_read_ahead: u64,
    pub max_read_ahead: u64,
    pub min_nr_requests: u64,
    pub max_nr_requests: u64,
}

impl From<FfiStorageLimits> for limits::StorageLimitsConfig {
    fn from(ffi: FfiStorageLimits) -> Self {
        let def = Self::default();

        let resolve = |val: u64, default: u64| {
            if val == u64::MAX { default } else { val }
        };

        let mut config = Self {
            min_read_ahead: resolve(ffi.min_read_ahead, def.min_read_ahead),
            max_read_ahead: resolve(ffi.max_read_ahead, def.max_read_ahead),
            min_nr_requests: resolve(ffi.min_nr_requests, def.min_nr_requests),
            max_nr_requests: resolve(ffi.max_nr_requests, def.max_nr_requests),
        };

        if config.min_read_ahead > config.max_read_ahead {
            std::mem::swap(&mut config.min_read_ahead, &mut config.max_read_ahead);
        }

        if config.min_nr_requests > config.max_nr_requests {
            std::mem::swap(&mut config.min_nr_requests, &mut config.max_nr_requests);
        }

        config
    }
}

/// # Safety
/// * `limits` must be a valid pointer to an initialized `FfiCpuLimits` struct, or a null pointer.
/// * The caller must ensure the pointer remains valid for the duration of this function call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_cpu_limits(limits: *const FfiCpuLimits) {
    if !limits.is_null() {
        unsafe {
            let _ = state::CPU_LIMITS_OVERRIDE.set((*limits).into());
        }
    }
}

/// # Safety
/// * `limits` must be a valid pointer to an initialized `FfiStorageLimits` struct, or a null pointer.
/// * The caller must ensure the pointer remains valid for the duration of this function call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_storage_limits(limits: *const FfiStorageLimits) {
    if !limits.is_null() {
        unsafe {
            let _ = state::STORAGE_LIMITS_OVERRIDE.set((*limits).into());
        }
    }
}

static MAIN_THREAD: sync::Mutex<Option<thread::JoinHandle<()>>> = sync::Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn set_blocker_service(enabled: bool) {
    state::BLOCKER_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_cleaner_service(enabled: bool) {
    state::CLEANER_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_cpu_service(enabled: bool) {
    state::CPU_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_storage_service(enabled: bool) {
    state::STORAGE_SERVICE_ENABLED.store(enabled, sync::atomic::Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn set_tweaks(enabled: bool) {
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
pub unsafe extern "C" fn start_services(signal_fd: i32) -> i32 {
    let owned_signal_fd: os::fd::OwnedFd = unsafe { os::fd::FromRawFd::from_raw_fd(signal_fd) };

    {
        match MAIN_THREAD.lock() {
            Ok(guard) => {
                if guard.is_some() {
                    log::error!("Attempted to start services while already running!");
                    return -1;
                }
            }
            Err(e) => {
                log::error!("MAIN_THREAD mutex poison detected: {e}. Resetting...");
                return -1;
            }
        }
    }

    logging::init();

    let (tx, rx) = sync::mpsc::channel::<()>();
    let result = std::panic::catch_unwind(move || {
        log::debug!("Service entry point reached. Signal FD: {signal_fd}");

        thread::Builder::new()
            .name("Tweaks".into())
            .stack_size(64 * 1024)
            .spawn(|| {
                if !state::TWEAKS_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    log::debug!("System Tweaks are DISABLED by config.");
                    return;
                }

                runtime::apply_prop_tweaks();
                runtime::wait_for_boot_completion("Tweaker");

                if state::SHUTDOWN_REQUESTED.load(sync::atomic::Ordering::Acquire) {
                    return;
                }

                runtime::apply_file_tweaks();
            })
            .expect("Failed to spawn Tweaks thread");

        state::SHUTDOWN_REQUESTED.store(false, sync::atomic::Ordering::Release);

        let handle = thread::Builder::new()
            .name("MainLoop".into())
            .stack_size(128 * 1024)
            .spawn(move || {
                if let Err(e) = tx.send(()) {
                    log::error!("Failed to send handshake: {e}.");
                }

                runtime::wait_for_boot_completion("MainLoop");

                if state::SHUTDOWN_REQUESTED.load(sync::atomic::Ordering::Acquire) {
                    return;
                }

                log::debug!("Constructing Service Vector...");
                let mut services = Vec::new();

                services.push(runtime::RecoverableService::new("Signal", move || {
                    let instance_fd = owned_signal_fd
                        .try_clone()
                        .map_err(types::QosError::IoError)?;

                    Ok(Box::new(signal::SignalController::new(instance_fd)))
                }));

                if state::STORAGE_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Storage", || {
                        Ok(Box::new(storage::StorageController::new()?))
                    }));
                }

                if state::CPU_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("CPU", || {
                        Ok(Box::new(cpu::CpuController::new()?))
                    }));
                }

                if state::CLEANER_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Cleaner", || {
                        Ok(Box::new(cleaner::CleanerController::new()?))
                    }));
                }

                if state::BLOCKER_SERVICE_ENABLED.load(sync::atomic::Ordering::Acquire) {
                    services.push(runtime::RecoverableService::new("Blocker", || {
                        Ok(Box::new(blocker::BlockerController::new()?))
                    }));
                }

                let svc_len = services.len();
                log::debug!("Initializing Event Loop with {svc_len} services...");

                if let Err(e) = runtime::run_event_loop(services) {
                    log::error!("Fatal error in event loop: {e}");
                }
            })
            .expect("Failed to spawn MainLoop thread");

        match MAIN_THREAD.lock() {
            Ok(mut guard) => *guard = Some(handle),
            Err(poisoned) => *poisoned.into_inner() = Some(handle),
        }
    });

    if let Err(cause) = result {
        log::error!("Critical Panic during startup: {cause:?}");
        bridge::notify_service_death("Startup Panic");
        return -1;
    }

    match rx.recv_timeout(time::Duration::from_secs(5)) {
        Ok(()) => 0,
        Err(e) => {
            log::error!("Handshake failed: {e}");
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
pub unsafe extern "C" fn join_threads() {
    log::debug!("Requested join threads.");
    let handle_opt = match MAIN_THREAD.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };

    if let Some(handle) = handle_opt
        && let Err(e) = handle.join()
    {
        log::error!("Main thread panicked during join: {e:?}");
    }
}
