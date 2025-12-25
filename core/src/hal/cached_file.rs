//! Author: [Seclususs](https://github.com/seclususs)

use std::fs::File;
use crate::hal::filesystem::write_to_stream;
use crate::algorithms::tolerance::{check_absolute, check_relative};

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
                if write_to_stream(file, &new_value.to_string()).is_ok() {
                    self.last_value = new_value;
                }
            }
        }
    }
    pub fn set_cache(&mut self, value: u64) {
        self.last_value = value;
    }
    pub fn get_cache(&self) -> u64 {
        self.last_value
    }
}