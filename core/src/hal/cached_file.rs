//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::filesystem::write_to_stream;

use std::fs::File;

#[inline(always)]
fn check_absolute(current: u64, target: u64, threshold: u64) -> bool {
    if current == target {
        return false;
    }
    current.abs_diff(target) >= threshold
}

#[inline(always)]
fn check_relative(current: u64, target: u64, tolerance_pct: f64) -> bool {
    if current == target {
        return false;
    }
    if current == 0 {
        return target != 0;
    }
    let diff = current.abs_diff(target) as f64;
    let change_ratio = diff / current as f64;
    change_ratio >= tolerance_pct
}

pub enum CheckStrategy {
    Absolute(u64),
    Relative(f64),
    Strict,
}

pub struct CachedFile {
    file: Option<File>,
    last_value: u64,
}

impl CachedFile {
    pub fn new(file: File, initial_value: u64) -> Self {
        Self {
            file: Some(file),
            last_value: initial_value,
        }
    }
    pub fn new_opt(file: Option<File>, initial_value: u64) -> Self {
        Self {
            file,
            last_value: initial_value,
        }
    }
    pub fn update(&mut self, new_value: u64, force: bool, strategy: CheckStrategy) {
        if let Some(ref mut file) = self.file {
            let needs_update = if force {
                true
            } else {
                match strategy {
                    CheckStrategy::Absolute(threshold) => check_absolute(self.last_value, new_value, threshold),
                    CheckStrategy::Relative(tolerance) => check_relative(self.last_value, new_value, tolerance),
                    CheckStrategy::Strict => self.last_value != new_value,
                }
            };
            if needs_update {
                if write_to_stream(file, new_value).is_ok() {
                    self.last_value = new_value;
                }
            }
        }
    }
}