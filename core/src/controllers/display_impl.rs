//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};
use crate::hal::{filesystem, surface_flinger};
use crate::resources::sys_paths;

use std::{fs, io, os, sync, thread, time};

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

#[derive(Debug, Clone, Copy)]
pub struct DisplayTunables {
    pub touch_idle_timeout_ms: i32,
    pub activity_throttle_ms: u128,
}

impl Default for DisplayTunables {
    fn default() -> Self {
        Self {
            touch_idle_timeout_ms: 3000,
            activity_throttle_ms: 200,
        }
    }
}

pub struct DisplayController {
    touch_fd: fs::File,
    state: DisplayState,
    current_mode: DisplayMode,
    tunables: DisplayTunables,
    last_activity: time::Instant,
    last_transition: time::Instant,
    next_wake_ms: i32,
    tx: sync::mpsc::Sender<DisplayMode>,
    io_buffer: Box<[u8; BUFFER_CAPACITY]>,
}

impl DisplayController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("DisplayController: Initializing...");
        let touch_fd = filesystem::open_file_for_read(sys_paths::K_TOUCH_DEVICE_PATH)?;
        let fd_raw = os::fd::AsRawFd::as_raw_fd(&touch_fd);
        let borrowed_fd = unsafe { os::fd::BorrowedFd::borrow_raw(fd_raw) };
        let flags = rustix::fs::fcntl_getfl(borrowed_fd)
            .map_err(|e: rustix::io::Errno| types::QosError::IoError(e.into()))?;
        rustix::fs::fcntl_setfl(borrowed_fd, flags | rustix::fs::OFlags::NONBLOCK)
            .map_err(|e: rustix::io::Errno| types::QosError::IoError(e.into()))?;
        let (tx, rx) = sync::mpsc::channel::<DisplayMode>();
        thread::Builder::new()
            .name("DisplayWorker".into())
            .spawn(move || {
                while let Ok(mode) = rx.recv() {
                    let _ = surface_flinger::set_refresh_rate(mode as i32);
                }
            })
            .map_err(|e| {
                types::QosError::SystemCheckFailed(format!("Spawn worker failed: {}", e))
            })?;
        let _ = tx.send(DisplayMode::Low60Hz);
        Ok(Self {
            touch_fd,
            state: DisplayState::Idle,
            current_mode: DisplayMode::Low60Hz,
            tunables: DisplayTunables::default(),
            last_activity: time::Instant::now(),
            last_transition: time::Instant::now(),
            next_wake_ms: -1,
            tx,
            io_buffer: Box::new([0u8; BUFFER_CAPACITY]),
        })
    }
    fn transition_to(&mut self, new_state: DisplayState) {
        if self.state == new_state {
            return;
        }
        let now = time::Instant::now();
        if now.duration_since(self.last_transition).as_millis() < self.tunables.activity_throttle_ms
        {
            return;
        }
        match (self.state, new_state) {
            (DisplayState::Idle, DisplayState::Active) => {
                self.current_mode = DisplayMode::High90Hz;
                let _ = self.tx.send(DisplayMode::High90Hz);
                self.next_wake_ms = self.tunables.touch_idle_timeout_ms;
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

impl traits::EventHandler for DisplayController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.touch_fd)
    }
    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        loop {
            match io::Read::read(&mut self.touch_fd, self.io_buffer.as_mut_slice()) {
                Ok(0) => break,
                Ok(_) => {
                    self.last_activity = time::Instant::now();
                    if state::DISPLAY_SERVICE_ENABLED.load(sync::atomic::Ordering::Relaxed) {
                        self.transition_to(DisplayState::Active);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    log::warn!("Display: read error: {}", e);
                    break;
                }
            }
        }
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        if !state::DISPLAY_SERVICE_ENABLED.load(sync::atomic::Ordering::Relaxed) {
            return Ok(traits::LoopAction::Continue);
        }
        if self.state != DisplayState::Active {
            return Ok(traits::LoopAction::Continue);
        }
        let elapsed = time::Instant::now()
            .duration_since(self.last_activity)
            .as_millis() as i32;
        if elapsed >= self.tunables.touch_idle_timeout_ms {
            self.transition_to(DisplayState::Idle);
        } else {
            self.next_wake_ms = self.tunables.touch_idle_timeout_ms - elapsed;
        }
        Ok(traits::LoopAction::Continue)
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
