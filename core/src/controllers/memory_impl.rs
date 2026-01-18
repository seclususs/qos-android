//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::memory_math::{self, MemoryTunables, QueueState};
use crate::algorithms::poll_math::AdaptivePoller;
use crate::config::loop_settings::MIN_POLLING_MS;
use crate::config::tunables::*;
use crate::daemon::state::DaemonContext;
use crate::daemon::traits::{EventHandler, LoopAction};
use crate::daemon::types::QosError;
use crate::hal::cached_file::{CachedFile, CheckStrategy};
use crate::hal::filesystem;
use crate::hal::kernel;
use crate::hal::thermal::ThermalSensor;
use crate::monitors::psi_monitor::PsiMonitor;
use crate::monitors::vm_monitor::{VmMonitor, VmStats};
use crate::resources::sys_paths::*;

use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::time::Instant;

pub struct MemoryController {
    fd: File,
    swap: CachedFile,
    vfs: CachedFile,
    psi_monitor: PsiMonitor,
    vm_monitor: VmMonitor,
    cpu_sensor: ThermalSensor,
    prev_vm_stats: VmStats,
    prev_psi_mem: f32,
    last_tick: Instant,
    current_swappiness: f32,
    current_vfs: f32,
    tunables: MemoryTunables,
    poller: AdaptivePoller,
    queue_state: QueueState,
    next_wake_ms: i32,
}

impl MemoryController {
    pub fn new() -> Result<Self, QosError> {
        log::info!("MemoryController: Initializing...");
        let raw_fd = kernel::register_psi_trigger(K_PSI_MEMORY_PATH, 150000, 1000000)
            .map_err(|e| QosError::FfiError(format!("Memory PSI Error: {}", e)))?;
        let fd = unsafe { File::from_raw_fd(raw_fd) };
        let swap = CachedFile::new(filesystem::open_file_for_write(K_SWAPPINESS_PATH)?, 0);
        let vfs = CachedFile::new(
            filesystem::open_file_for_write(K_VFS_CACHE_PRESSURE_PATH)?,
            0,
        );
        let psi_monitor = PsiMonitor::new(K_PSI_MEMORY_PATH)?;
        let mut vm_monitor = VmMonitor::new(K_VMSTAT_PATH)?;
        let cpu_sensor = ThermalSensor::new(K_CPU_TEMP_PATH, 45.0);
        let initial_vm_stats = vm_monitor.read_stats().unwrap_or(VmStats::default());
        let tunables = MemoryTunables {
            min_swappiness: MIN_SWAPPINESS as f32,
            max_swappiness: MAX_SWAPPINESS as f32,
            min_vfs: MIN_VFS as f32,
            max_vfs: MAX_VFS as f32,
            pressure_kp: 0.8,
            pressure_kd: 0.2,
            inefficiency_cost: 25.0,
            pressure_vfs_k: 0.10,
            fragmentation_impact_k: 2.0,
            wss_cost_factor: 3.0,
            zram_thermal_cost: 1.5,
            general_smooth_factor: 0.20,
            queue_history_size: 16,
            queue_smoothing_alpha: 0.2,
            residence_time_threshold: 30.0,
            protection_curve_k: 3.0,
            congestion_scaling_factor: 2.5,
        };
        let poller = AdaptivePoller::new(1.5, 0.5);
        let mut controller = Self {
            fd,
            swap,
            vfs,
            psi_monitor,
            vm_monitor,
            cpu_sensor,
            prev_vm_stats: initial_vm_stats,
            prev_psi_mem: 0.0,
            last_tick: Instant::now(),
            current_swappiness: MIN_SWAPPINESS as f32,
            current_vfs: MIN_VFS as f32,
            tunables,
            poller,
            queue_state: QueueState::default(),
            next_wake_ms: MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_pressure_state(&mut self, context: &mut DaemonContext) -> Result<(), QosError> {
        let psi_data = self.psi_monitor.read_state()?;
        let vm_stats = self.vm_monitor.read_stats()?;
        let cpu_temp = self.cpu_sensor.read();
        let io_sat = context.pressure.io_saturation;
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().max(0.001);
        self.last_tick = now;
        let activity_delta =
            memory_math::calculate_activity_state(&vm_stats, &self.prev_vm_stats, dt);
        let p_mem =
            memory_math::calculate_pressure_level(psi_data.some.current, psi_data.some.avg10);
        let dp_dt = memory_math::calculate_pressure_derivative(p_mem, self.prev_psi_mem, dt);
        self.prev_psi_mem = p_mem;
        self.prev_vm_stats = vm_stats;
        context.pressure.memory_psi = p_mem;
        let active_set = memory_math::calculate_active_set(&vm_stats);
        let queue_correction_factor = memory_math::update_congestion_model(
            &mut self.queue_state,
            active_set,
            activity_delta.scan_rate,
            &self.tunables,
        );
        self.next_wake_ms =
            self.poller
                .calculate_next_interval(p_mem, psi_data.some.avg300) as i32;
        let target_swap = memory_math::calculate_swappiness(
            p_mem,
            dp_dt,
            &activity_delta,
            cpu_temp,
            io_sat,
            queue_correction_factor,
            &self.tunables,
        );
        let target_vfs = memory_math::calculate_vfs_pressure(p_mem, &self.tunables);
        self.current_swappiness = memory_math::smooth_value(
            self.current_swappiness,
            target_swap,
            self.tunables.general_smooth_factor,
        );
        self.current_vfs = memory_math::smooth_value(
            self.current_vfs,
            target_vfs,
            self.tunables.general_smooth_factor,
        );
        self.apply_values(false);
        Ok(())
    }
    fn apply_values(&mut self, force: bool) {
        let swap_u64 = crate::algorithms::sanitize_to_u64(
            self.current_swappiness,
            self.tunables.min_swappiness as u64,
        );
        let vfs_u64 =
            crate::algorithms::sanitize_to_u64(self.current_vfs, self.tunables.min_vfs as u64);
        self.swap
            .update(swap_u64, force, CheckStrategy::Absolute(5));
        self.vfs.update(vfs_u64, force, CheckStrategy::Absolute(10));
    }
}

impl EventHandler for MemoryController {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
    fn on_event(&mut self, context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        let mut buf = [0u8; 8];
        let _ = self.fd.read(&mut buf);
        if let Err(e) = self.update_pressure_state(context) {
            log::warn!("Mem Error: {}", e);
        }
        Ok(LoopAction::Continue)
    }
    fn on_timeout(&mut self, context: &mut DaemonContext) -> Result<LoopAction, QosError> {
        if let Err(e) = self.update_pressure_state(context) {
            log::warn!("Mem Timeout Error: {}", e);
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