//! Author: [Seclususs](https://github.com/seclususs)

use android_logger::Config;
use log::LevelFilter;

pub fn init() {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let config = Config::default()
        .with_tag("QoS")
        .with_max_level(level)
        .format(|f, record| write!(f, "{}", record.args()));

    android_logger::init_once(config);
}
