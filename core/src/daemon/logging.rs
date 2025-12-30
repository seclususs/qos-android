//! Author: [Seclususs](https://github.com/seclususs)

use android_logger::Config;
use log::LevelFilter;

pub fn init() {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    android_logger::init_once(Config::default().with_tag("QoS").with_max_level(level));
}