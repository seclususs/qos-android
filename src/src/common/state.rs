//! Author: [Seclususs](https://github.com/seclususs)

use std::sync::atomic::AtomicBool;

pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
pub static DISPLAY_SERVICE_ENABLED: AtomicBool = AtomicBool::new(false);