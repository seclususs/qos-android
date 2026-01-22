//! Author: [Seclususs](https://github.com/seclususs)

use crate::algorithms::{memory_math, poll_math};
use crate::config::{loop_settings, tunables};
use crate::daemon::{state, traits, types};
use crate::hal::{cached_file, filesystem, kernel, thermal};
use crate::monitors::{psi_monitor, vm_monitor};
use crate::resources::sys_paths;

use std::{fs, io, os, time};

pub struct MemoryController {
    fd: fs::File,
    swap: cached_file::CachedFile,
    vfs: cached_file::CachedFile,
    psi_monitor: psi_monitor::PsiMonitor,
    vm_monitor: vm_monitor::VmMonitor,
    cpu_sensor: thermal::ThermalSensor,
    prev_vm_stats: vm_monitor::VmStats,
    prev_psi_mem: f32,
    last_tick: time::Instant,
    current_swappiness: f32,
    current_vfs: f32,
    tunables: memory_math::MemoryTunables,
    poller: poll_math::AdaptivePoller,
    queue_state: memory_math::QueueState,
    next_wake_ms: i32,
}

impl MemoryController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("MemoryController: Initializing...");
        let config = tunables::GlobalConfig::default();
        let raw_fd = kernel::register_psi_trigger(sys_paths::K_PSI_MEMORY_PATH, 150000, 1000000)
            .map_err(|e| types::QosError::FfiError(format!("Memory PSI Error: {}", e)))?;
        let fd = unsafe { os::fd::FromRawFd::from_raw_fd(raw_fd) };
        let swap = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_SWAPPINESS_PATH).ok(),
            0,
        );
        let vfs = cached_file::CachedFile::new_opt(
            filesystem::open_file_for_write(sys_paths::K_VFS_CACHE_PRESSURE_PATH).ok(),
            0,
        );
        if !swap.is_active() && !vfs.is_active() {
            return Err(types::QosError::SystemCheckFailed(
                "No memory tunables found.".to_string(),
            ));
        }
        let psi_monitor = psi_monitor::PsiMonitor::new(sys_paths::K_PSI_MEMORY_PATH)?;
        let mut vm_monitor = vm_monitor::VmMonitor::new(sys_paths::K_VMSTAT_PATH)?;
        let cpu_path = sys_paths::get_cpu_temp_path();
        let cpu_sensor = thermal::ThermalSensor::new(cpu_path.to_str().unwrap_or_default(), 45.0);
        let initial_vm_stats = vm_monitor
            .read_stats()
            .unwrap_or(vm_monitor::VmStats::default());
        let tunables = memory_math::MemoryTunables {
            min_swappiness: config.memory.min_swappiness as f32,
            max_swappiness: config.memory.max_swappiness as f32,
            min_vfs: config.memory.min_vfs as f32,
            max_vfs: config.memory.max_vfs as f32,
            pressure_kp: config.memory.pressure_kp,
            pressure_kd: config.memory.pressure_kd,
            inefficiency_cost: config.memory.inefficiency_cost,
            pressure_vfs_k: config.memory.pressure_vfs_k,
            fragmentation_impact_k: config.memory.fragmentation_impact_k,
            wss_cost_factor: config.memory.wss_cost_factor,
            zram_thermal_cost: config.memory.zram_thermal_cost,
            general_smooth_factor: config.memory.general_smooth_factor,
            queue_history_size: config.memory.queue_history_size,
            queue_smoothing_alpha: config.memory.queue_smoothing_alpha,
            residence_time_threshold: config.memory.residence_time_threshold,
            protection_curve_k: config.memory.protection_curve_k,
            congestion_scaling_factor: config.memory.congestion_scaling_factor,
        };
        let poller = poll_math::AdaptivePoller::new(1.5, 0.5, poll_math::PollerConfig::default());
        let mut controller = Self {
            fd,
            swap,
            vfs,
            psi_monitor,
            vm_monitor,
            cpu_sensor,
            prev_vm_stats: initial_vm_stats,
            prev_psi_mem: 0.0,
            last_tick: time::Instant::now(),
            current_swappiness: config.memory.min_swappiness as f32,
            current_vfs: config.memory.min_vfs as f32,
            tunables,
            poller,
            queue_state: memory_math::QueueState::default(),
            next_wake_ms: loop_settings::MIN_POLLING_MS as i32,
        };
        controller.apply_values(true);
        Ok(controller)
    }
    fn update_pressure_state(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<(), types::QosError> {
        let psi_data = self.psi_monitor.read_state()?;
        let vm_stats = self.vm_monitor.read_stats()?;
        let cpu_temp = self.cpu_sensor.read();
        let io_sat = context.pressure.io_saturation;
        let now = time::Instant::now();
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
            .update(swap_u64, force, cached_file::CheckStrategy::Absolute(5));
        self.vfs
            .update(vfs_u64, force, cached_file::CheckStrategy::Absolute(10));
    }
}

impl traits::EventHandler for MemoryController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        os::fd::AsRawFd::as_raw_fd(&self.fd)
    }
    fn on_event(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        let mut buf = [0u8; 8];
        let _ = io::Read::read(&mut self.fd, &mut buf);
        if let Err(e) = self.update_pressure_state(context) {
            log::warn!("Mem Error: {}", e);
        }
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        if let Err(e) = self.update_pressure_state(context) {
            log::warn!("Mem Timeout Error: {}", e);
        }
        Ok(traits::LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        self.next_wake_ms
    }
    fn get_poll_flags(&self) -> rustix::event::epoll::EventFlags {
        rustix::event::epoll::EventFlags::PRI | rustix::event::epoll::EventFlags::ERR
    }
}
