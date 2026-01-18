//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::state::{DISPLAY_SERVICE_ENABLED, DaemonContext};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::{filesystem, surface_flinger};
use crate::resources::sys_paths::K_TOUCH_DEVICE_PATH;

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::fd::{AsRawFd, RawFd};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const TOUCH_IDLE_TIMEOUT_MS: i32 = 3000;
const ACTIVITY_THROTTLE_MS: u128 = 200;
const BUFFER_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayState {
    Idle,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Low60Hz = 0,
    High90Hz = 1,
}

pub struct DisplayController {
    touch_fd: File,
    state: DisplayState,
    current_mode: DisplayMode,
    last_activity: Instant,
    last_transition: Instant,
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
                    let _ = surface_flinger::set_refresh_rate(mode as i32);
                }
            })
            .map_err(|e| QosError::SystemCheckFailed(format!("Spawn worker failed: {}", e)))?;
        let _ = tx.send(DisplayMode::Low60Hz);
        Ok(Self {
            touch_fd,
            state: DisplayState::Idle,
            current_mode: DisplayMode::Low60Hz,
            last_activity: Instant::now(),
            last_transition: Instant::now(),
            next_wake_ms: -1,
            tx,
            io_buffer: Box::new([0u8; BUFFER_CAPACITY]),
        })
    }
    fn transition_to(&mut self, new_state: DisplayState) {
        if self.state == new_state {
            return;
        }
        let now = Instant::now();
        if now.duration_since(self.last_transition).as_millis() < ACTIVITY_THROTTLE_MS {
            return;
        }
        match (self.state, new_state) {
            (DisplayState::Idle, DisplayState::Active) => {
                self.current_mode = DisplayMode::High90Hz;
                let _ = self.tx.send(DisplayMode::High90Hz);
                self.next_wake_ms = TOUCH_IDLE_TIMEOUT_MS;
            }
            (DisplayState::Active, DisplayState::Idle) => {
                self.current_mode = DisplayMode::Low60Hz;
                let _ = self.tx.send(DisplayMode::Low60Hz);
                self.next_wake_ms = -1;
            }
            _ => return,
        }
        self.state = new_state;
        self.last_transition = now;
    }
}

impl EventHandler for DisplayController {
    fn as_raw_fd(&self) -> RawFd {
        self.touch_fd.as_raw_fd()
    }
    fn on_event(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        loop {
            match self.touch_fd.read(self.io_buffer.as_mut_slice()) {
                Ok(0) => break,
                Ok(_) => {
                    self.last_activity = Instant::now();
                    if DISPLAY_SERVICE_ENABLED.load(Ordering::Relaxed) {
                        self.transition_to(DisplayState::Active);
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    log::warn!("Display: read error: {}", e);
                    break;
                }
            }
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        if !DISPLAY_SERVICE_ENABLED.load(Ordering::Relaxed) {
            return Ok(LoopAction::Continue);
        }
        if self.state != DisplayState::Active {
            return Ok(LoopAction::Continue);
        }
        let elapsed = Instant::now()
            .duration_since(self.last_activity)
            .as_millis() as i32;
        if elapsed >= TOUCH_IDLE_TIMEOUT_MS {
            self.transition_to(DisplayState::Idle);
        } else {
            self.next_wake_ms = TOUCH_IDLE_TIMEOUT_MS - elapsed;
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        match self.state {
            DisplayState::Idle => -1,
            DisplayState::Active => self.next_wake_ms,
        }
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::IN | rustix::event::epoll::EventFlags::ERR
    }
}