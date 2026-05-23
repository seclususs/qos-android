//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};

use std::{fs, io, os, sync};

pub struct SignalController {
    signal_file: fs::File,
}

impl SignalController {
    pub fn new(signal_fd: os::fd::OwnedFd) -> Self {
        Self {
            signal_file: signal_fd.into(),
        }
    }
}

impl traits::EventHandler for SignalController {
    fn as_raw_fd(&self) -> Option<os::fd::RawFd> {
        Some(os::fd::AsRawFd::as_raw_fd(&self.signal_file))
    }

    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        log::debug!("Signal received from Kernel.");
        let mut buffer = [0u8; 128];

        match io::Read::read(&mut self.signal_file, &mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                log::info!("Requesting shutdown...");
                state::SHUTDOWN_REQUESTED.store(true, sync::atomic::Ordering::Release);
                Ok(traits::LoopAction::Continue)
            }

            Ok(_) => Ok(traits::LoopAction::Continue),

            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(traits::LoopAction::Continue),
            Err(e) => Err(types::QosError::IoError(e)),
        }
    }
}
