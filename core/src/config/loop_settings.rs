//! Author: [Seclususs](https://github.com/seclususs)

pub const MIN_POLLING_MS: u64 = 3000;
pub const MAX_POLLING_MS: u64 = 10000;
pub const MAX_EPOLL_TIMEOUT_MS: i32 = 10000;
pub const MAX_EVENTS: usize = 16;
pub const STABILIZATION_DELAY_SEC: u64 = 60;
pub const COOLDOWN_DURATION_SEC: u64 = 5;
pub const BOOT_WAIT_RETRY_LIMIT: u32 = 300;
pub const BOOT_POLL_INTERVAL_SEC: u64 = 1;
