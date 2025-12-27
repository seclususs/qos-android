//! Author: [Seclususs](https://github.com/seclususs)

pub const MIN_POLLING_MS: u64 = 1000;
pub const MAX_POLLING_MS: u64 = 10000;
pub const SLEEP_TOLERANCE_MS: u64 = 500;
pub const JITTER_PERCENT: u64 = 5;
pub const QUANTIZATION_STEP_MS: u64 = 100;
pub const ATTACK_COEFF: f64 = 1.0;
pub const DECAY_COEFF: f64 = 0.1;
pub const HYSTERESIS_THRESHOLD_MS: u64 = 200;
pub const MAX_EPOLL_TIMEOUT_MS: i32 = 5000;
pub const MAX_EVENTS: usize = 16;
pub const STABILIZATION_DELAY_SEC: u64 = 60;
pub const COOLDOWN_DURATION_SEC: u64 = 5;
pub const BOOT_WAIT_RETRY_LIMIT: u32 = 300;
pub const BOOT_POLL_INTERVAL_SEC: u64 = 1;
pub const THERMAL_SYNC_INTERVAL_SEC: u64 = 30;