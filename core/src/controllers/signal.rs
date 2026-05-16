//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};

use std::{fs, io, os, sync};

pub struct SignalController {
    signal_file: fs::File,
}

impl SignalController {
    /// # Safety
    /// The caller must ensure that `signal_fd` is a valid, open file descriptor that
    /// this process has ownership of. The `SignalController` will take ownership
    /// of this FD and close it when dropped.
    pub unsafe fn new(signal_fd: os::fd::RawFd) -> Self {
        Self {
            // Safety: Inherits the safety requirements of FromRawFd::from_raw_fd
            signal_file: unsafe { os::fd::FromRawFd::from_raw_fd(signal_fd) },
        }
    }
}

impl traits::EventHandler for SignalController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.signal_file)
    }
    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        log::info!("SignalController: Signal received from Kernel.");
        let mut buffer = [0u8; 128];
        match io::Read::read(&mut self.signal_file, &mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                log::info!("SignalController: Requesting shutdown...");
                state::SHUTDOWN_REQUESTED.store(true, sync::atomic::Ordering::Release);
                Ok(traits::LoopAction::Continue)
            }
            Ok(_) => Ok(traits::LoopAction::Continue),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(traits::LoopAction::Continue),
            Err(e) => Err(types::QosError::IoError(e)),
        }
    }
}
