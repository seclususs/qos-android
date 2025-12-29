//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::thermal_math::ThermalTunables;
use crate::hal::thermal;
use crate::hal::kernel;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::resources::sys_paths::K_PSI_CPU_PATH;
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::algorithms::poll_math::AdaptivePoller;
use crate::algorithms::thermal_math::ThermalPredictor;
use crate::daemon::state::{update_thermal_state, update_thermal_damping};
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;

use std::os::fd::{RawFd, AsRawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

pub struct ThermalController {
    fd: File,
    psi_monitor: PsiMonitor,
    thermal_model: ThermalPredictor,
    tunables: ThermalTunables,
    poller: AdaptivePoller,
    next_wake_ms: i32,
}

impl ThermalController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("ThermalController: Initializing...");
        let raw_fd = kernel::register_psi_trigger(K_PSI_CPU_PATH, 100000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Thermal Trigger Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let psi_monitor = PsiMonitor::new(K_PSI_CPU_PATH)?;
        let initial_temp = thermal::read_initial_thermal_state();
        let thermal_model = ThermalPredictor::new(initial_temp);
        let poller = AdaptivePoller::new(0.5, 0.5);
        let tunables = ThermalTunables {
            alpha_heating: 0.30,
            lambda_cooling: 0.005,
            max_virtual_temp: 60.0,
            bucket_size: 300.0,
            bucket_leak_rate: 5.0,
            threshold_warm: 36.0,
            threshold_hot: 42.0,
            hysteresis_gap: 2.0,
            lambda_degradation_k: 0.05, 
        };
        let mut controller = Self {
            fd,
            psi_monitor,
            thermal_model,
            tunables,
            poller,
            next_wake_ms: MIN_POLLING_MS as i32,
        };
        controller.update_thermal_model()?;
        Ok(controller)
    }
    fn update_thermal_model(&mut self) -> Result<(), QosError> {
        let data = self.psi_monitor.read_state()?;
        let some = data.some;
        let raw_p = some.current.max(some.avg10);
        let damping_factor = self.thermal_model.update(raw_p, some.avg300, &self.tunables);
        update_thermal_state(self.thermal_model.current_state);
        update_thermal_damping(damping_factor);
        self.next_wake_ms = self.poller.calculate_next_interval(raw_p, some.avg300) as i32;
        Ok(())
    }
}

impl EventHandler for ThermalController {
    fn as_raw_fd(&self) -> RawFd { self.fd.as_raw_fd() }
    fn on_event(&mut self) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Err(e) = self.update_thermal_model() {
            log::warn!("Thermal Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_thermal_model() {
            log::warn!("Thermal Timeout Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::PRI | rustix::event::epoll::EventFlags::ERR
    }
}