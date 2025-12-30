//! Author: [Seclususs](https://github.com/seclususs)

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct ThermalSensor {
    file: Option<File>,
    buffer: [u8; 16],
    default_val: f64,
}

impl ThermalSensor {
    pub fn new(path: &str, default_val: f64) -> Self {
        let file = File::open(path).ok();
        Self {
            file,
            buffer: [0u8; 16],
            default_val,
        }
    }
    pub fn read(&mut self) -> f64 {
        let file = match self.file.as_mut() {
            Some(f) => f,
            None => return self.default_val,
        };
        if file.seek(SeekFrom::Start(0)).is_err() {
            return self.default_val;
        }
        match file.read(&mut self.buffer) {
            Ok(n) if n > 0 => {
                let s = match std::str::from_utf8(&self.buffer[..n]) {
                    Ok(v) => v.trim(),
                    Err(_) => return self.default_val,
                };
                match s.parse::<f64>() {
                    Ok(val) => {
                        let abs = val.abs();
                        if abs >= 10_000.0 {
                            val / 1000.0
                        } else if abs >= 100.0 {
                            val / 10.0
                        } else {
                            val
                        }
                    }
                    Err(_) => self.default_val,
                }
            }
            _ => self.default_val,
        }
    }
}