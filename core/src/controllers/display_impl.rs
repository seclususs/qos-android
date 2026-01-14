//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::state::{DISPLAY_SERVICE_ENABLED, DaemonContext};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::filesystem;
use crate::resources::sys_paths::K_TOUCH_DEVICE_PATH;

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::fd::{AsRawFd, RawFd};
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const SERVICE_BINARY: &str = "/system/bin/service";
const TOUCH_IDLE_TIMEOUT_MS: i32 = 4000;
const ACTIVITY_THROTTLE_MS: u128 = 100;
const BUFFER_CAPACITY: usize = 1024;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DisplayMode {
    Low60Hz,
    High90Hz,
}

pub struct DisplayController {
    touch_fd: File,
    last_activity: Instant,
    last_throttle: Instant,
    current_mode: DisplayMode,
    next_wake_ms: i32,
    tx: mpsc::Sender<DisplayMode>,
    io_buffer: Box<[u8; BUFFER_CAPACITY]>,
}

impl DisplayController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("DisplayController: Initializing...");
        let touch_fd = filesystem::open_file_for_read(K_TOUCH_DEVICE_PATH)?;
        let fd_raw = touch_fd.as_raw_fd();
        let borrowed_fd = unsafe { std::os::fd::BorrowedFd::borrow_raw(fd_raw) };
        let flags = rustix::fs::fcntl_getfl(borrowed_fd)
            .map_err(|e: rustix::io::Errno| QosError::IoError(e.into()))?;
        rustix::fs::fcntl_setfl(borrowed_fd, flags | rustix::fs::OFlags::NONBLOCK)
            .map_err(|e: rustix::io::Errno| QosError::IoError(e.into()))?;
        let (tx, rx) = mpsc::channel::<DisplayMode>();
        thread::Builder::new()
            .name("DisplayWorker".into())
            .spawn(move || {
                while let Ok(mode) = rx.recv() {
                    let param = match mode {
                        DisplayMode::High90Hz => "1",
                        DisplayMode::Low60Hz => "0",
                    };
                    let _ = Command::new(SERVICE_BINARY)
                        .args(["call", "SurfaceFlinger", "1035", "i32", param])
                        .status();
                }
            })
            .map_err(|e| QosError::SystemCheckFailed(format!("Spawn worker failed: {}", e)))?;
        let _ = tx.send(DisplayMode::Low60Hz);
        Ok(Self {
            touch_fd,
            last_activity: Instant::now(),
            last_throttle: Instant::now(),
            current_mode: DisplayMode::Low60Hz,
            next_wake_ms: -1,
            tx,
            io_buffer: Box::new([0u8; BUFFER_CAPACITY]),
        })
    }
    fn set_mode(&mut self, mode: DisplayMode) {
        if self.current_mode != mode
            && self.tx.send(mode).is_ok() {
                self.current_mode = mode;
            }
    }
}

impl EventHandler for DisplayController {
    fn as_raw_fd(&self) -> RawFd {
        self.touch_fd.as_raw_fd()
    }
    fn on_event(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        if !DISPLAY_SERVICE_ENABLED.load(Ordering::Relaxed) {
            loop {
                match self.touch_fd.read(self.io_buffer.as_mut_slice()) {
                    Ok(0) => break,
                    Ok(_) => continue,
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
            return Ok(LoopAction::Continue);
        }
        match self.touch_fd.read(self.io_buffer.as_mut_slice()) {
            Ok(n) if n > 0 => {
                let now = Instant::now();
                if now.duration_since(self.last_throttle).as_millis() > ACTIVITY_THROTTLE_MS {
                    self.last_activity = now;
                    self.last_throttle = now;
                    self.set_mode(DisplayMode::High90Hz);
                    self.next_wake_ms = TOUCH_IDLE_TIMEOUT_MS;
                }
                if n == BUFFER_CAPACITY {
                    loop {
                        match self.touch_fd.read(self.io_buffer.as_mut_slice()) {
                            Ok(0) => break,
                            Ok(_) => continue,
                            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                            Err(_) => break,
                        }
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => log::warn!("Display: Read err: {}", e),
            _ => {}
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        if !DISPLAY_SERVICE_ENABLED.load(Ordering::Relaxed) {
            return Ok(LoopAction::Continue);
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_activity).as_millis() as i32;
        if elapsed >= TOUCH_IDLE_TIMEOUT_MS {
            self.set_mode(DisplayMode::Low60Hz);
            self.next_wake_ms = -1;
        } else {
            self.next_wake_ms = TOUCH_IDLE_TIMEOUT_MS - elapsed;
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::IN | rustix::event::epoll::EventFlags::ERR
    }
}