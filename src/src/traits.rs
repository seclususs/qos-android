use std::os::fd::RawFd;

pub trait EventHandler {
    fn as_raw_fd(&self) -> RawFd;
    fn on_event(&mut self);
    fn get_timeout_ms(&self) -> i32 {
        -1
    }
    fn on_timeout(&mut self) {}
}