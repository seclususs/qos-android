//! Author: [Seclususs](https://github.com/seclususs)

use android_logger;
use log;

pub fn init() {
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Error
    };
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("QoS")
            .with_max_level(level),
    );
}
