//! Author: [Seclususs](https://github.com/seclususs)

use std::sync::atomic::AtomicBool;

pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
pub static CPU_SERVICE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static MEMORY_SERVICE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static STORAGE_SERVICE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static TWEAKS_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalPressure {
    pub cpu_psi: f32,
    pub memory_psi: f32,
    pub io_psi: f32,
    pub io_saturation: f32,
}

#[derive(Debug, Default)]
pub struct DaemonContext {
    pub pressure: GlobalPressure,
}

impl DaemonContext {
    pub fn new() -> Self {
        Self {
            pressure: GlobalPressure::default(),
        }
    }
}