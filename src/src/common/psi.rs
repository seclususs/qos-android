//! Author: [Seclususs](https://github.com/seclususs)

use crate::common::error::QosError;
use crate::common::fs::open_file_for_read;
use std::io::Read;

const BUFFER_SIZE: usize = 512;

pub struct PsiMonitor {
    path: String,
    buffer: [u8; BUFFER_SIZE],
}

impl PsiMonitor {
    pub fn new(path: &str) -> Result<Self, QosError> {
        let _ = open_file_for_read(path)?;
        Ok(Self {
            path: path.to_string(),
            buffer: [0u8; BUFFER_SIZE],
        })
    }
    pub fn read_avg10(&mut self) -> Result<f64, QosError> {
        let mut file = open_file_for_read(&self.path)?;
        let bytes_read = match file.read(&mut self.buffer) {
            Ok(n) => n,
            Err(e) => return Err(QosError::IoError(e)),
        };
        if bytes_read == 0 {
            return Err(QosError::PsiParseError("Empty PSI file".to_string()));
        }
        let content = std::str::from_utf8(&self.buffer[..bytes_read])
            .map_err(|_| QosError::PsiParseError("Invalid UTF-8 in PSI file".to_string()))?;
        for line in content.lines() {
            if line.starts_with("some ") {
                for token in line.split_whitespace() {
                    if let Some(value_str) = token.strip_prefix("avg10=") {
                        return value_str.parse::<f64>().map_err(|_| {
                            QosError::PsiParseError(format!("Invalid float format: '{}'", value_str))
                        });
                    }
                }
            }
        }
        log::warn!("PSI Parse Fail. Raw content: {:?}", content);
        Err(QosError::PsiParseError("avg10 keyword not found in 'some' line".to_string()))
    }
}