//! Author: [Seclususs](https://github.com/seclususs)

use crate::hal::filesystem;

use std::fs;

#[inline]
fn check_absolute(current: u64, target: u64, threshold: u64) -> bool {
    if current == target {
        return false;
    }
    current.abs_diff(target) >= threshold
}

#[inline]
fn check_relative(current: u64, target: u64, tolerance_pct: f32) -> bool {
    if current == target {
        return false;
    }
    if current == 0 {
        return target != 0;
    }
    let diff = current.abs_diff(target) as f32;
    let threshold = (current as f32) * tolerance_pct;
    diff >= threshold
}

pub enum CheckStrategy {
    Absolute(u64),
    Relative(f32),
    Strict,
}

pub struct CachedFile {
    file: Option<fs::File>,
    last_value: u64,
}

impl CachedFile {
    pub fn new(file: fs::File, initial_value: u64) -> Self {
        Self {
            file: Some(file),
            last_value: initial_value,
        }
    }
    pub fn new_opt(file: Option<fs::File>, initial_value: u64) -> Self {
        Self {
            file,
            last_value: initial_value,
        }
    }
    pub fn is_active(&self) -> bool {
        self.file.is_some()
    }
    pub fn update(&mut self, new_value: u64, force: bool, strategy: &CheckStrategy) {
        if let Some(ref mut file) = self.file {
            let needs_update = if force {
                true
            } else {
                match strategy {
                    CheckStrategy::Absolute(threshold) => {
                        check_absolute(self.last_value, new_value, *threshold)
                    }
                    CheckStrategy::Relative(tolerance) => {
                        check_relative(self.last_value, new_value, *tolerance)
                    }
                    CheckStrategy::Strict => self.last_value != new_value,
                }
            };
            if needs_update && filesystem::write_to_stream(file, new_value).is_ok() {
                self.last_value = new_value;
            }
        }
    }
}
