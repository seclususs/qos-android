//! Author: [Seclususs](https://github.com/seclususs)

#[cfg(target_os = "android")]
use crate::ffi;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "android")]
use std::{ffi::CStr, mem, thread, time::Duration};

#[cfg(target_os = "android")]
const K_TCP_CONGESTION_PATH: &str = "/proc/sys/net/ipv4/tcp_congestion_control";
#[cfg(target_os = "android")]
const K_ALGO_CUBIC: &str = "cubic";
#[cfg(target_os = "android")]
const K_ALGO_WESTWOOD: &str = "westwood";
#[cfg(target_os = "android")]
const K_WLAN_INTERFACE: &str = "wlan0";
#[cfg(target_os = "android")]
const K_BUFFER_SIZE: usize = 4096;

#[cfg(target_os = "android")]
const RTMGRP_LINK: u32 = 1;

#[cfg(target_os = "android")]
#[repr(C)]
struct ifinfomsg {
    ifi_family: u8,
    __ifi_pad: u8,
    ifi_type: u16,
    ifi_index: i32,
    ifi_flags: u32,
    ifi_change: u32,
}

#[cfg(target_os = "android")]
fn align_nl(len: usize) -> usize {
    (len + 3) & !3
}

pub fn monitor_network(shutdown_requested: &AtomicBool) {
    #[cfg(target_os = "android")]
    {
        ffi::log_info("NetworkManager: Starting monitor...");
        let fd = unsafe {
            libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW,
                libc::NETLINK_ROUTE,
            )
        };
        if fd < 0 {
            ffi::log_error("NetworkManager: Failed to open Netlink socket.");
            return;
        }
        let mut sa: libc::sockaddr_nl = unsafe { mem::zeroed() };
        sa.nl_family = libc::AF_NETLINK as libc::sa_family_t;
        sa.nl_groups = RTMGRP_LINK;
        unsafe {
            if libc::bind(
                fd,
                &sa as *const _ as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
            ) < 0
            {
                ffi::log_error("NetworkManager: Failed to bind Netlink socket.");
                libc::close(fd);
                return;
            }
        }
        let mut buf = [0u8; K_BUFFER_SIZE];
        while !shutdown_requested.load(Ordering::Acquire) {
            let len = unsafe {
                libc::recv(
                    fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len() as libc::size_t,
                    0,
                )
            };
            if len < 0 {
                ffi::log_error("NetworkManager: recv failed.");
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            if len == 0 {
                continue;
            }
            let mut current_ptr = buf.as_ptr();
            let mut remaining_len = len as usize;
            while remaining_len >= mem::size_of::<libc::nlmsghdr>() {
                let nh = current_ptr as *const libc::nlmsghdr;
                let msg_len = unsafe { (*nh).nlmsg_len } as usize;
                if msg_len < mem::size_of::<libc::nlmsghdr>() || msg_len > remaining_len {
                    break;
                }
                let msg_type = unsafe { (*nh).nlmsg_type };
                if msg_type == libc::RTM_NEWLINK || msg_type == libc::RTM_DELLINK {
                    let ifinfo = unsafe {
                        (current_ptr.add(mem::size_of::<libc::nlmsghdr>())) as *const ifinfomsg
                    };
                    let if_index = unsafe { (*ifinfo).ifi_index };
                    let if_flags = unsafe { (*ifinfo).ifi_flags };
                    let mut name_buf = [0 as libc::c_char; libc::IF_NAMESIZE];
                    unsafe {
                        if !libc::if_indextoname(if_index as u32, name_buf.as_mut_ptr()).is_null() {
                            let if_name = CStr::from_ptr(name_buf.as_ptr())
                                .to_string_lossy();
                            if if_name == K_WLAN_INTERFACE {
                                let is_up = (if_flags & (libc::IFF_LOWER_UP as u32)) != 0;
                                let action = if is_up {
                                    ffi::apply_tweak(K_TCP_CONGESTION_PATH, K_ALGO_CUBIC);
                                    "Connected (Cubic)"
                                } else {
                                    ffi::apply_tweak(K_TCP_CONGESTION_PATH, K_ALGO_WESTWOOD);
                                    "Disconnected (Westwood)"
                                };
                                ffi::log_info(&format!("NetworkManager: wlan0 status changed -> {}", action));
                            }
                        }
                    }
                }
                let aligned_len = align_nl(msg_len);
                if aligned_len >= remaining_len {
                    break;
                }
                unsafe {
                    current_ptr = current_ptr.add(aligned_len);
                }
                remaining_len -= aligned_len;
            }
        }
        unsafe { libc::close(fd) };
        ffi::log_info("NetworkManager: Monitoring stopped.");
    }
    #[cfg(not(target_os = "android"))]
    {
        use std::thread;
        use std::time::Duration;
        while !shutdown_requested.load(Ordering::Acquire) {
            thread::sleep(Duration::from_secs(1));
        }
    }
}