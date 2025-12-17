//! Author: [Seclususs](https://github.com/seclususs)

use crate::common::error::QosError;
use crate::common::traits::{EventHandler, LoopAction};
use std::fs::{File, OpenOptions};
use std::io::{Read, ErrorKind};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const INPUT_DEVICE_PATH: &str = "/dev/input/event3";
const CMD_BINARY: &str = "/system/bin/cmd";
const SMOOTH_RATE: &str = "90.0";
const LOW_POWER_RATE: &str = "60.0";
const SMOOTH_TIMEOUT_MS: u64 = 10000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DisplayMode {
    Smooth,
    LowPower,
}

pub struct DisplayController {
    file: File,
    current_mode: DisplayMode,
    deadline: Option<Instant>,
    tx: mpsc::Sender<DisplayMode>,
}

impl DisplayController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("DisplayController: Initializing...");
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK) 
            .open(INPUT_DEVICE_PATH)
            .map_err(|e| {
                QosError::SystemCheckFailed(format!("Failed to open input device {}: {}", INPUT_DEVICE_PATH, e))
            })?;
        let (tx, rx) = mpsc::channel::<DisplayMode>();
        thread::spawn(move || {
            log::info!("DisplayController: Started.");
            while let Ok(target_mode) = rx.recv() {
                let rate_value = match target_mode {
                    DisplayMode::Smooth => SMOOTH_RATE,
                    DisplayMode::LowPower => LOW_POWER_RATE,
                };
                match Command::new(CMD_BINARY)
                    .args(["settings", "put", "system", "min_refresh_rate", rate_value])
                    .output()
                {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            log::error!("DisplayController: Failed to set {:?}. Stderr: {}", target_mode, stderr);
                        } else {
                            log::debug!("DisplayController: Worker applied {:?}", target_mode);
                        }
                    },
                    Err(e) => {
                        log::error!("DisplayController: Failed to spawn shell command: {}", e);
                    }
                }
            }
            log::info!("DisplayController: Worker thread stopping.");
        });
        let mut controller = Self {
            file,
            current_mode: DisplayMode::Smooth,
            deadline: None,
            tx,
        };
        controller.apply_mode(DisplayMode::LowPower)?;
        Ok(controller)
    }
    fn apply_mode(&mut self, target_mode: DisplayMode) -> Result<(), QosError> {
        if self.current_mode == target_mode {
            return Ok(());
        }
        log::debug!("DisplayController: Requesting transition to {:?}", target_mode);
        self.current_mode = target_mode;
        if let Err(e) = self.tx.send(target_mode) {
            log::error!("DisplayController: Failed to send command to worker: {}", e);
            return Err(QosError::SystemCheckFailed("Display worker thread unavailable".to_string()));
        }
        Ok(())
    }
}

impl EventHandler for DisplayController {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 256];
        let mut input_detected = false;
        loop {
            match self.file.read(&mut buf) {
                Ok(0) => {
                    log::error!("DisplayController: Input device EOF detected.");
                    return Err(QosError::IoError(std::io::Error::new(
                        ErrorKind::UnexpectedEof, 
                        "Input device sent EOF"
                    )));
                },
                Ok(_) => {
                    input_detected = true;
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break;
                },
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                },
                Err(e) => {
                    log::error!("DisplayController: Error reading input: {}", e);
                    return Err(QosError::IoError(e));
                }
            }
        }
        if input_detected {
            if self.current_mode != DisplayMode::Smooth {
                self.apply_mode(DisplayMode::Smooth)?;
            }
            self.deadline = Some(Instant::now() + Duration::from_millis(SMOOTH_TIMEOUT_MS));
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        if self.current_mode == DisplayMode::LowPower {
            return -1;
        }
        match self.deadline {
            Some(deadline) => {
                let now = Instant::now();
                if now >= deadline {
                    0
                } else {
                    (deadline - now).as_millis() as i32
                }
            },
            None => -1,
        }
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if self.current_mode == DisplayMode::Smooth {
            self.apply_mode(DisplayMode::LowPower)?;
            self.deadline = None;
        }
        Ok(LoopAction::Continue)
    }
}