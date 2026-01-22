//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};

use std::{fs, io, os, sync};

pub struct SignalController {
    file: fs::File,
}

impl SignalController {
    /// # Safety
    /// The caller must ensure that `fd` is a valid, open file descriptor that
    /// this process has ownership of. The `SignalController` will take ownership
    /// of this FD and close it when dropped.
    pub unsafe fn new(fd: os::fd::RawFd) -> Self {
        Self {
            // UFCS: Menggunakan FromRawFd dari modul os::fd
            file: unsafe { os::fd::FromRawFd::from_raw_fd(fd) },
        }
    }
}

impl traits::EventHandler for SignalController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.file)
    }
    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        log::info!("SignalController: Signal received from Kernel.");
        let mut buf = [0u8; 128];
        match io::Read::read(&mut self.file, &mut buf) {
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
