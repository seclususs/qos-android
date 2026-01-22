//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, types};

use std::os;

#[derive(Debug, PartialEq)]
pub enum LoopAction {
    Continue,
}

pub trait EventHandler {
    fn as_raw_fd(&self) -> os::fd::RawFd;
    fn on_event(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<LoopAction, types::QosError>;
    fn get_timeout_ms(&self) -> i32 {
        -1
    }
    fn on_timeout(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<LoopAction, types::QosError> {
        let _ = context;
        Ok(LoopAction::Continue)
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::IN
            | rustix::event::epoll::EventFlags::PRI
            | rustix::event::epoll::EventFlags::ERR
    }
}
