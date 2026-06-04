//! Author: [Seclususs](https://github.com/seclususs)

use crate::config::limits;

use std::sync;

pub static SHUTDOWN_REQUESTED: sync::atomic::AtomicBool = sync::atomic::AtomicBool::new(false);
pub static BLOCKER_SERVICE_ENABLED: sync::atomic::AtomicBool = sync::atomic::AtomicBool::new(false);
pub static CLEANER_SERVICE_ENABLED: sync::atomic::AtomicBool = sync::atomic::AtomicBool::new(false);
pub static CPU_SERVICE_ENABLED: sync::atomic::AtomicBool = sync::atomic::AtomicBool::new(false);
pub static STORAGE_SERVICE_ENABLED: sync::atomic::AtomicBool = sync::atomic::AtomicBool::new(false);
pub static TWEAKS_ENABLED: sync::atomic::AtomicBool = sync::atomic::AtomicBool::new(false);

pub static CPU_LIMITS_OVERRIDE: sync::OnceLock<limits::CpuLimitsConfig> = sync::OnceLock::new();
pub static STORAGE_LIMITS_OVERRIDE: sync::OnceLock<limits::StorageLimitsConfig> =
    sync::OnceLock::new();

#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalPressure {
    pub cpu_psi: f32,
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
