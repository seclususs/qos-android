//! Author: [Seclususs](https://github.com/seclususs)

use crate::{ffi, memory_logic, network_logic, refresh_logic, storage_logic};
use std::sync::{atomic::{AtomicBool, Ordering}, Mutex};
use std::thread::{self, JoinHandle};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
static MEMORY_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static REFRESH_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static STORAGE_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static NETWORK_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_start_services() {
    ffi::log_info("Rust services starting...");
    SHUTDOWN_REQUESTED.store(false, Ordering::Release);
    let mem_handle = thread::spawn(|| {
        memory_logic::monitor_memory(&SHUTDOWN_REQUESTED);
    });
    *MEMORY_THREAD.lock().unwrap() = Some(mem_handle);
    let refresh_handle = thread::spawn(|| {
        refresh_logic::monitor_refresh_rate(&SHUTDOWN_REQUESTED);
    });
    *REFRESH_THREAD.lock().unwrap() = Some(refresh_handle);
    let storage_handle = thread::spawn(|| {
        storage_logic::monitor_storage(&SHUTDOWN_REQUESTED);
    });
    *STORAGE_THREAD.lock().unwrap() = Some(storage_handle);
    let network_handle = thread::spawn(|| {
        network_logic::monitor_network(&SHUTDOWN_REQUESTED);
    });
    *NETWORK_THREAD.lock().unwrap() = Some(network_handle);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_stop_services() {
    ffi::log_info("Rust services stopping...");
    SHUTDOWN_REQUESTED.store(true, Ordering::Release);
    if let Some(handle) = MEMORY_THREAD.lock().unwrap().take() {
        handle.join().unwrap_or_else(|e| ffi::log_error(&format!("Memory thread join failed: {:?}", e)));
    }
    if let Some(handle) = REFRESH_THREAD.lock().unwrap().take() {
        handle.join().unwrap_or_else(|e| ffi::log_error(&format!("Refresh thread join failed: {:?}", e)));
    }
    if let Some(handle) = STORAGE_THREAD.lock().unwrap().take() {
        handle.join().unwrap_or_else(|e| ffi::log_error(&format!("Storage thread join failed: {:?}", e)));
    }
    if let Some(handle) = NETWORK_THREAD.lock().unwrap().take() {
        handle.join().unwrap_or_else(|e| ffi::log_error(&format!("Network thread join failed: {:?}", e)));
    }
    ffi::log_info("Rust services stopped.");
}