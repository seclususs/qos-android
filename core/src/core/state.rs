//! Author: [Seclususs](https://github.com/seclususs)

use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

pub static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
pub static CPU_SERVICE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static MEMORY_SERVICE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static STORAGE_SERVICE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static TWEAKS_ENABLED: AtomicBool = AtomicBool::new(true);

pub struct GlobalPressure {
    pub cpu_psi: f64,
    pub memory_psi: f64,
    pub io_psi: f64,
    pub io_saturation: f64,
}

impl GlobalPressure {
    pub const fn new() -> Self {
        Self {
            cpu_psi: 0.0,
            memory_psi: 0.0,
            io_psi: 0.0,
            io_saturation: 0.0,
        }
    }
}

pub static SYSTEM_PRESSURE: RwLock<GlobalPressure> = RwLock::new(GlobalPressure::new());

pub fn update_cpu_pressure(psi: f64) {
    if let Ok(mut guard) = SYSTEM_PRESSURE.write() {
        guard.cpu_psi = psi;
    }
}

pub fn update_memory_pressure(psi: f64) {
    if let Ok(mut guard) = SYSTEM_PRESSURE.write() {
        guard.memory_psi = psi;
    }
}

pub fn update_io_pressure(psi: f64) {
    if let Ok(mut guard) = SYSTEM_PRESSURE.write() {
        guard.io_psi = psi;
    }
}

pub fn update_io_saturation(sat: f64) {
    if let Ok(mut guard) = SYSTEM_PRESSURE.write() {
        guard.io_saturation = sat;
    }
}

pub fn get_cpu_pressure() -> f64 {
    if let Ok(guard) = SYSTEM_PRESSURE.read() {
        guard.cpu_psi
    } else {
        0.0
    }
}

pub fn get_memory_pressure() -> f64 {
    if let Ok(guard) = SYSTEM_PRESSURE.read() {
        guard.memory_psi
    } else {
        0.0
    }
}

pub fn get_io_pressure() -> f64 {
    if let Ok(guard) = SYSTEM_PRESSURE.read() {
        guard.io_psi
    } else {
        0.0
    }
}

pub fn get_io_saturation() -> f64 {
    if let Ok(guard) = SYSTEM_PRESSURE.read() {
        guard.io_saturation
    } else {
        0.0
    }
}