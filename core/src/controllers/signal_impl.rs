//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::state::{DaemonContext, SHUTDOWN_REQUESTED};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::sync::atomic::Ordering;

pub struct SignalController {
    file: File,
}

impl SignalController {
    /// # Safety
    /// The caller must ensure that `fd` is a valid, open file descriptor that
    /// this process has ownership of. The `SignalController` will take ownership
    /// of this FD and close it when dropped.
    pub unsafe fn new(fd: RawFd) -> Self {
        Self {
            file: unsafe { File::from_raw_fd(fd) },
        }
    }
}

impl EventHandler for SignalController {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
    fn on_event(&mut self, _context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        log::info!("SignalController: Signal received from Kernel.");
        let mut buf = [0u8; 128];
        match self.file.read(&mut buf) {
            Ok(bytes_read) if bytes_read > 0 => {
                log::info!("SignalController: Requesting shutdown...");
                SHUTDOWN_REQUESTED.store(true, Ordering::Release);
                Ok(LoopAction::Continue)
            }
            Ok(_) => {
                Ok(LoopAction::Continue)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                Ok(LoopAction::Continue)
            }
            Err(e) => {
                Err(QosError::IoError(e))
            }
        }
    }
}