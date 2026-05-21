//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::{state, traits, types};

use std::{os, process, thread, time};

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

#[derive(Debug, Clone, Copy)]
struct ControllerConfig {
    interval_ms: i32,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            interval_ms: 86_400_000,
        }
    }
}

pub struct BlockerController {
    config: ControllerConfig,
    last_run: time::Instant,
}

impl BlockerController {
    pub fn new() -> types::Result<Self> {
        log::info!("BlockerController: Initializing...");

        let config = ControllerConfig::default();

        let mut controller = Self {
            config,
            last_run: time::Instant::now(),
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
        for component in TARGET_COMPONENTS {
            let _ = process::Command::new("cmd")
                .args(["pm", "disable", component])
                .status();
            thread::sleep(time::Duration::from_millis(50));
        }
    }
}

impl traits::EventHandler for BlockerController {
    fn as_raw_fd(&self) -> Option<os::fd::RawFd> {
        None
    }

    fn on_event(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> types::Result<traits::LoopAction> {
        Ok(traits::LoopAction::Continue)
    }

    fn on_timeout(
        &mut self,
        _context: &mut state::DaemonContext,
    ) -> types::Result<traits::LoopAction> {
        let now = time::Instant::now();

        let elapsed_ms = now.duration_since(self.last_run).as_millis();

        if elapsed_ms >= self.config.interval_ms as u128 {
            self.trigger_block_cycle();
        }

        Ok(traits::LoopAction::Continue)
    }

    fn get_timeout_ms(&self) -> i32 {
        self.config.interval_ms
    }
}
