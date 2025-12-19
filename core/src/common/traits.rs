//! Author: [Seclususs](https://github.com/seclususs)

use std::os::fd::RawFd;
use crate::common::error::QosError;

#[derive(Debug, PartialEq)]
pub enum LoopAction {
    Continue,
    Pause,
    Resume,
}

pub trait EventHandler {
    fn as_raw_fd(&self) -> RawFd;
    fn on_event(&mut self) -> Result<LoopAction, QosError>;
    fn get_timeout_ms(&self) -> i32 {
        -1
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        Ok(LoopAction::Continue)
    }
}