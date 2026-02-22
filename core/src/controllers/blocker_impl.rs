//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};

use std::{os, process, sync, thread, time};

const TARGET_COMPONENTS: &[&str] = &[
    "com.google.android.gms/com.google.android.gms.ads.AdRequestBrokerService",
    "com.google.android.gms/com.google.android.gms.ads.identifier.service.AdvertisingIdService",
    "com.google.android.gms/com.google.android.gms.ads.measurement.GmpConversionTrackingBrokerService",
    "com.google.android.gms/com.google.android.gms.ads.social.GcmSchedulerWakeupService",
    "com.google.android.gms/com.google.android.gms.ads.identifier.service.AdvertisingIdNotificationService",
    "com.google.android.gms/com.google.android.gms.ads.jams.NegotiationService",
    "com.google.android.gms/com.google.android.gms.growth.watchdog.GrowthWatchdogTaskService",
    "com.google.android.gms/com.google.android.gms.measurement.PackageMeasurementReceiver",
    "com.google.android.gms/com.google.android.gms.measurement.PackageMeasurementTaskService",
    "com.google.android.gms/com.google.android.gms.measurement.service.MeasurementBrokerService",
    "com.google.android.gms/com.google.android.gms.analytics.AnalyticsService",
    "com.google.android.gms/com.google.android.gms.analytics.AnalyticsTaskService",
    "com.google.android.gms/com.google.android.gms.common.stats.StatsUploadService",
    "com.google.android.gms/com.google.android.gms.clearcut.uploader.QosUploaderService",
    "com.google.android.gms/com.google.android.gms.audit.upload.AuditGcmTaskService",
    "com.google.android.gms/com.google.android.gms.analytics.AnalyticsReceiver",
    "com.google.android.gms/com.google.android.gms.feedback.LegacyBugReportService",
    "com.google.android.gms/com.google.android.gms.feedback.OfflineReportSendTaskService",
];

static CMD_CACHE: sync::OnceLock<String> = sync::OnceLock::new();

pub struct BlockerController {
    dummy_fd: rustix::fd::OwnedFd,
    interval_ms: i32,
    last_run: time::Instant,
}

impl BlockerController {
    pub fn new() -> Result<Self, types::QosError> {
        log::info!("BlockerController: Initializing...");
        let evt = rustix::event::eventfd(
            0,
            rustix::event::EventfdFlags::CLOEXEC | rustix::event::EventfdFlags::NONBLOCK,
        )
        .map_err(|e| types::QosError::SystemCheckFailed(format!("Eventfd fail: {e}")))?;
        let mut controller = Self {
            dummy_fd: evt,
            interval_ms: 86_400_000,
            last_run: time::Instant::now()
                .checked_sub(time::Duration::from_secs(86401))
                .unwrap_or_else(time::Instant::now),
        };
        controller.trigger_block_cycle();
        Ok(controller)
    }
    fn trigger_block_cycle(&mut self) {
        thread::Builder::new()
            .name("BlockerExec".into())
            .spawn(|| {
                Self::execute_batch_disable();
            })
            .ok();
        self.last_run = time::Instant::now();
    }
    fn execute_batch_disable() {
        let cmd = CMD_CACHE.get_or_init(|| {
            let capacity = TARGET_COMPONENTS.len() * 100;
            let mut chain = String::with_capacity(capacity);
            for component in TARGET_COMPONENTS {
                chain.push_str("cmd pm disable ");
                chain.push_str(component);
                chain.push_str(" >/dev/null 2>&1 ; ");
            }
            chain
        });
        let start = time::Instant::now();
        match process::Command::new("sh").arg("-c").arg(cmd).status() {
            Ok(_) => {
                let duration = start.elapsed();
                log::debug!("Blocker: Batch cycle done in {}ms", duration.as_millis());
            }
            Err(e) => {
                log::error!("Blocker: Exec failed: {e}");
            }
        }
    }
}

impl traits::EventHandler for BlockerController {
    fn as_raw_fd(&self) -> os::fd::RawFd {
        std::os::fd::AsRawFd::as_raw_fd(&self.dummy_fd)
    }
    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        let mut buf = [0u8; 8];
        let _ = rustix::io::read(&self.dummy_fd, &mut buf);
        Ok(traits::LoopAction::Continue)
    }
    fn on_timeout(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> Result<traits::LoopAction, types::QosError> {
        let now = time::Instant::now();
        let elapsed = now.duration_since(self.last_run).as_millis();
        if elapsed >= self.interval_ms as u128 {
            self.trigger_block_cycle();
        }
        Ok(traits::LoopAction::Continue)
    }
    fn get_timeout_ms(&self) -> i32 {
        let now = time::Instant::now();
        let elapsed = now.duration_since(self.last_run).as_millis();
        let remaining = (self.interval_ms as u128).saturating_sub(elapsed);
        if remaining > i32::MAX as u128 {
            i32::MAX
        } else {
            remaining as i32
        }
    }
}
