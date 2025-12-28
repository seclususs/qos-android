//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::state::SHUTDOWN_REQUESTED;
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;

use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::sync::atomic::Ordering;

pub struct SignalController {
    file: File,
}

impl SignalController {
    pub fn new(fd: RawFd) -> Self {
        unsafe {
            Self { file: File::from_raw_fd(fd) }
        }
    }
}

impl EventHandler for SignalController {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        log::info!("SignalController: Signal received from Kernel.");
        let mut buf = [0u8; 128];
        match self.file.read(&mut buf) {
            Ok(_) => {
                log::info!("SignalController: Requesting shutdown...");
                SHUTDOWN_REQUESTED.store(true, Ordering::Release);
                Ok(LoopAction::Continue)
            },
            Err(e) => {
                Err(QosError::IoError(e))
            }
        }
    }
}