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
    dirty_ratio: CachedFile,
    dirty_bg: CachedFile,
    dirty_expire: CachedFile,
    stat_interval: CachedFile,
    watermark_scale: CachedFile,
    extfrag: CachedFile,
    dirty_writeback: CachedFile,
    page_cluster: CachedFile,
    psi_monitor: PsiMonitor,
    vm_monitor: VmMonitor,
    cpu_sensor: ThermalSensor,
    prev_vm_stats: VmStats,
    prev_psi_mem: f32,
    last_tick: Instant,
    current_swappiness: f32,
    current_vfs: f32,
    current_dirty: f32,
    current_dirty_bg: f32,
    current_dirty_expire: f32,
    current_stat_interval: f32,
    current_watermark_scale: f32,
    current_extfrag_threshold: f32,
    current_dirty_writeback: f32,
    current_page_cluster: f32,
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
        let dirty_ratio =
            CachedFile::new_opt(filesystem::open_file_for_write(K_DIRTY_RATIO).ok(), 0);
        let dirty_bg =
            CachedFile::new_opt(filesystem::open_file_for_write(K_DIRTY_BG_RATIO).ok(), 0);
        let dirty_expire = CachedFile::new_opt(
            filesystem::open_file_for_write(K_DIRTY_EXPIRE_CENTISECS).ok(),
            0,
        );
        let stat_interval =
            CachedFile::new_opt(filesystem::open_file_for_write(K_STAT_INTERVAL).ok(), 0);
        let watermark_scale = CachedFile::new_opt(
            filesystem::open_file_for_write(K_WATERMARK_SCALE_FACTOR).ok(),
            0,
        );
        let extfrag =
            CachedFile::new_opt(filesystem::open_file_for_write(K_EXTFRAG_THRESHOLD).ok(), 0);
        let dirty_writeback = CachedFile::new_opt(
            filesystem::open_file_for_write(K_DIRTY_WRITEBACK_CENTISECS).ok(),
            0,
        );
        let page_cluster =
            CachedFile::new_opt(filesystem::open_file_for_write(K_PAGE_CLUSTER).ok(), 0);
        let psi_monitor = PsiMonitor::new(K_PSI_MEMORY_PATH)?;
        let mut vm_monitor = VmMonitor::new(K_VMSTAT_PATH, K_BUDDYINFO_PATH)?;
        let cpu_sensor = ThermalSensor::new(K_CPU_TEMP_PATH, 45.0);
        let initial_vm_stats = vm_monitor.read_stats().unwrap_or(VmStats::default());
        let tunables = MemoryTunables {
            min_swappiness: MIN_SWAPPINESS as f32,
            max_swappiness: MAX_SWAPPINESS as f32,
            min_dirty_expire: MIN_DIRTY_EXPIRE as f32,
            max_dirty_expire: MAX_DIRTY_EXPIRE as f32,
            min_stat_interval: MIN_STAT_INTERVAL as f32,
            max_stat_interval: MAX_STAT_INTERVAL as f32,
            min_watermark_scale: MIN_WATERMARK_SCALE as f32,
            max_watermark_scale: MAX_WATERMARK_SCALE as f32,
            min_extfrag_threshold: MIN_EXTFRAG_THRESHOLD as f32,
            max_extfrag_threshold: MAX_EXTFRAG_THRESHOLD as f32,
            min_dirty: MIN_DIRTY as f32,
            max_dirty: MAX_DIRTY as f32,
            min_dirty_bg: MIN_DIRTY_BG as f32,
            max_dirty_bg: MAX_DIRTY_BG as f32,
            min_dirty_writeback: MIN_DIRTY_WRITEBACK as f32,
            max_dirty_writeback: MAX_DIRTY_WRITEBACK as f32,
            min_page_cluster: MIN_PAGE_CLUSTER as f32,
            max_page_cluster: MAX_PAGE_CLUSTER as f32,
            min_vfs: MIN_VFS as f32,
            max_vfs: MAX_VFS as f32,
            pressure_kp: 0.5,
            pressure_kd: 0.25,
            inefficiency_penalty: 20.0,
            thermal_vfs_k: 0.05,
            fragmentation_impact_k: 2.0,
            wss_penalty_factor: 2.5,
            zram_thermal_penalty: 2.5,
            general_smooth_factor: 0.25,
            watermark_smooth_factor: 0.1,
            queue_history_size: 16,
            queue_smoothing_alpha: 0.15,
            residence_time_threshold: 60.0,
            protection_curve_k: 2.5,
            congestion_scaling_factor: 2.0,
        };
        let poller = AdaptivePoller::new(1.5, 0.5);
        let mut controller = Self {
            fd,
            swap,
            vfs,
            dirty_ratio,
            dirty_bg,
            dirty_expire,
            stat_interval,
            watermark_scale,
            extfrag,
            dirty_writeback,
            page_cluster,
            psi_monitor,
            vm_monitor,
            cpu_sensor,
            prev_vm_stats: initial_vm_stats,
            prev_psi_mem: 0.0,
            last_tick: Instant::now(),
            current_swappiness: MIN_SWAPPINESS as f32,
            current_vfs: MIN_VFS as f32,
            current_dirty: MAX_DIRTY as f32,
            current_dirty_bg: MAX_DIRTY_BG as f32,
            current_dirty_expire: MAX_DIRTY_EXPIRE as f32,
            current_stat_interval: MIN_STAT_INTERVAL as f32,
            current_watermark_scale: MIN_WATERMARK_SCALE as f32,
            current_extfrag_threshold: MAX_EXTFRAG_THRESHOLD as f32,
            current_dirty_writeback: MAX_DIRTY_WRITEBACK as f32,
            current_page_cluster: MAX_PAGE_CLUSTER as f32,
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
        let p_cpu = context.pressure.cpu_psi;
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
        let (target_dirty, target_dirty_bg) =
            memory_math::calculate_dirty_limits(io_sat, &self.tunables);
        let target_expire = memory_math::calculate_dirty_time(io_sat, &self.tunables);
        let target_wb = memory_math::calculate_dirty_writeback(target_expire, &self.tunables);
        let target_wm = memory_math::calculate_watermark_scale(
            p_mem,
            vm_stats.fragmentation_index,
            &self.tunables,
        );
        let target_stat = memory_math::calculate_sampling_rate(p_mem, &self.tunables);
        let target_ext = memory_math::calculate_extfrag_threshold(p_cpu, &self.tunables);
        let target_pc = memory_math::calculate_clustering_factor(p_cpu, &self.tunables);
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
        self.current_dirty = target_dirty;
        self.current_dirty_bg = target_dirty_bg;
        self.current_dirty_expire = target_expire;
        self.current_dirty_writeback = target_wb;
        self.current_stat_interval = target_stat;
        self.current_watermark_scale = memory_math::smooth_value(
            self.current_watermark_scale,
            target_wm,
            self.tunables.watermark_smooth_factor,
        );
        self.current_extfrag_threshold = target_ext;
        self.current_page_cluster = target_pc;
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
        let dirty_u64 =
            crate::algorithms::sanitize_to_u64(self.current_dirty, self.tunables.max_dirty as u64);
        let dbg_u64 = crate::algorithms::sanitize_to_u64(
            self.current_dirty_bg,
            self.tunables.max_dirty_bg as u64,
        );
        let expire_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_dirty_expire,
            self.tunables.max_dirty_expire as u64,
            50,
        );
        let stat_u64 = crate::algorithms::sanitize_to_u64(
            self.current_stat_interval,
            self.tunables.min_stat_interval as u64,
        );
        let wm_u64 = crate::algorithms::sanitize_to_u64(
            self.current_watermark_scale,
            self.tunables.min_watermark_scale as u64,
        );
        let ext_u64 = crate::algorithms::sanitize_to_u64(
            self.current_extfrag_threshold,
            self.tunables.max_extfrag_threshold as u64,
        );
        let dwb_u64 = crate::algorithms::sanitize_to_clean_u64(
            self.current_dirty_writeback,
            self.tunables.max_dirty_writeback as u64,
            50,
        );
        let pc_u64 = crate::algorithms::sanitize_to_u64(
            self.current_page_cluster,
            self.tunables.max_page_cluster as u64,
        );
        self.swap
            .update(swap_u64, force, CheckStrategy::Absolute(5));
        self.vfs.update(vfs_u64, force, CheckStrategy::Absolute(10));
        self.dirty_ratio
            .update(dirty_u64, force, CheckStrategy::Absolute(1));
        self.dirty_bg
            .update(dbg_u64, force, CheckStrategy::Absolute(1));
        self.dirty_expire
            .update(expire_u64, force, CheckStrategy::Relative(0.1));
        self.stat_interval
            .update(stat_u64, force, CheckStrategy::Strict);
        self.watermark_scale
            .update(wm_u64, force, CheckStrategy::Absolute(2));
        self.extfrag
            .update(ext_u64, force, CheckStrategy::Absolute(20));
        self.dirty_writeback
            .update(dwb_u64, force, CheckStrategy::Relative(0.1));
        self.page_cluster
            .update(pc_u64, force, CheckStrategy::Strict);
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