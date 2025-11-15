//! Author: [Seclususs](https://github.com/seclususs)


use crate::ffi;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const K_SWAPPINESS_LOW: &str = "20";
const K_VFS_CACHE_PRESSURE_LOW: &str = "50";
const K_SWAPPINESS_MID: &str = "100";
const K_VFS_CACHE_PRESSURE_MID: &str = "100";
const K_SWAPPINESS_HIGH: &str = "150";
const K_VFS_CACHE_PRESSURE_HIGH: &str = "200";
const K_GO_TO_HIGH_THRESHOLD: i32 = 20;
const K_GO_TO_LOW_THRESHOLD: i32 = 45;
const K_RETURN_TO_MID_FROM_LOW_THRESHOLD: i32 = 40;
const K_RETURN_TO_MID_FROM_HIGH_THRESHOLD: i32 = 25;
const K_SWAPPINESS_PATH: &str = "/proc/sys/vm/swappiness";
const K_VFS_CACHE_PRESSURE_PATH: &str = "/proc/sys/vm/vfs_cache_pressure";

#[derive(Debug, PartialEq, Copy, Clone)]
enum MemoryState {
    Low,
    Mid,
    High,
    Unknown,
}

fn apply_memory_tweaks(new_state: MemoryState, current_state: &mut MemoryState) {
    if new_state == *current_state {
        return;
    }
    match new_state {
        MemoryState::Low => {
            ffi::log_info("MemoryManager: RAM ample. Profile: LOW");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_LOW);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_VFS_CACHE_PRESSURE_LOW);
        }
        MemoryState::Mid => {
            ffi::log_info("MemoryManager: Moderate RAM usage. Profile: MID");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_MID);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_VFS_CACHE_PRESSURE_MID);
        }
        MemoryState::High => {
            ffi::log_info("MemoryManager: RAM nearly full. Profile: HIGH");
            ffi::apply_tweak(K_SWAPPINESS_PATH, K_SWAPPINESS_HIGH);
            ffi::apply_tweak(K_VFS_CACHE_PRESSURE_PATH, K_VFS_CACHE_PRESSURE_HIGH);
        }
        MemoryState::Unknown => {}
    }
    *current_state = new_state;
}

fn check_ram_percentage(current_state: &mut MemoryState) {
    let free_ram_percent = ffi::get_free_ram_percentage();
    if free_ram_percent < 0 {
        return;
    }
    ffi::log_debug(&format!("MemoryManager: Free RAM percentage: {}%", free_ram_percent));
    let new_state = match *current_state {
        MemoryState::Unknown => {
            if free_ram_percent < K_GO_TO_HIGH_THRESHOLD { MemoryState::High }
            else if free_ram_percent > K_GO_TO_LOW_THRESHOLD { MemoryState::Low }
            else { MemoryState::Mid }
        }
        MemoryState::High => {
            if free_ram_percent >= K_RETURN_TO_MID_FROM_HIGH_THRESHOLD { MemoryState::Mid }
            else { *current_state }
        }
        MemoryState::Mid => {
            if free_ram_percent < K_GO_TO_HIGH_THRESHOLD { MemoryState::High }
            else if free_ram_percent > K_GO_TO_LOW_THRESHOLD { MemoryState::Low }
            else { *current_state }
        }
        MemoryState::Low => {
            if free_ram_percent < K_RETURN_TO_MID_FROM_LOW_THRESHOLD { MemoryState::Mid }
            else { *current_state }
        }
    };
    apply_memory_tweaks(new_state, current_state);
}

pub fn monitor_memory(shutdown_requested: &AtomicBool) {
    ffi::log_info("MemoryManager: Starting event-driven monitoring...");
    let mut current_state = MemoryState::Unknown;
    check_ram_percentage(&mut current_state);
    let netlink_fd = ffi::create_netlink_socket();
    if netlink_fd < 0 {
        ffi::log_error("MemoryManager: Failed to create netlink socket. Falling back to 15s polling.");
        while !shutdown_requested.load(Ordering::Acquire) {
            check_ram_percentage(&mut current_state);
            thread::sleep(Duration::from_secs(15));
        }
        return;
    }
    let mut event_buffer = [0u8; 2048];
    while !shutdown_requested.load(Ordering::Acquire) {
        let poll_result = ffi::poll_fd(netlink_fd, 5000);
        if poll_result > 0 {
            if let Some(event_str) = ffi::read_netlink_event(netlink_fd, &mut event_buffer) {
                if event_str.contains("SUBSYSTEM=lowmemorykiller") {
                    ffi::log_info("MemoryManager: LMK event received! Applying HIGH profile.");
                    apply_memory_tweaks(MemoryState::High, &mut current_state);
                    thread::sleep(Duration::from_secs(10));
                    check_ram_percentage(&mut current_state);
                }
            }
        } else if poll_result == 0 {
            if current_state == MemoryState::High {
                ffi::log_debug("MemoryManager: Re-checking recovery from HIGH state...");
                check_ram_percentage(&mut current_state);
            } else if current_state == MemoryState::Unknown {
                check_ram_percentage(&mut current_state);
            }
        } else {
            ffi::log_error("MemoryManager: Netlink poll error. Recreating socket.");
            ffi::close_fd(netlink_fd);
            thread::sleep(Duration::from_secs(5));
            let _ = ffi::create_netlink_socket();
        }
    }
    ffi::log_info("MemoryManager: Monitoring stopped.");
    ffi::close_fd(netlink_fd);
}