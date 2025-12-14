//! Author: [Seclususs](https://github.com/seclususs)

use std::fs::{self, File};
use std::io::Read;
use std::process::Command;
use std::path::Path;
use crate::error::QosError;

const ALLOWED_PREFIXES: [&str; 3] = ["/proc/", "/sys/", "/dev/input/"];

fn validate_path_secure(path_str: &str) -> Result<(), QosError> {
    let path = Path::new(path_str);
    let canonical_path = fs::canonicalize(path)
        .map_err(|_| QosError::InvalidPath(format!("Path not found: {}", path_str)))?;
    let canonical_str = canonical_path.to_str()
        .ok_or_else(|| QosError::InvalidPath("Non-UTF8 path".to_string()))?;
    if ALLOWED_PREFIXES.iter().any(|&prefix| canonical_str.starts_with(prefix)) {
        Ok(())
    } else {
        Err(QosError::PermissionDenied(format!("Access denied: {}", canonical_str)))
    }
}

fn validate_value(value: &str) -> bool {
    value.chars().all(|c| 
        c.is_alphanumeric() || 
        c == '.' || c == ',' || c == '-' || c == '_' || c == '=' || c == ' '
    )
}

pub fn write_to_file(path: &str, value: &str) -> Result<(), QosError> {
    validate_path_secure(path)?;
    if !validate_value(value) {
        return Err(QosError::SystemCheckFailed(format!("Invalid characters in value for {}: '{}'", path, value)));
    }
    fs::write(path, value).map_err(|e| {
        log::debug!("Write failed '{}' -> {}: {}", value, path, e);
        QosError::IoError(e)
    })
}

pub fn parse_psi_avg10(path: &str) -> Result<f64, QosError> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 128];
    let bytes_read = file.read(&mut buffer)?;
    let content = String::from_utf8_lossy(&buffer[..bytes_read]);
    for part in content.split_whitespace() {
        if let Some(val_str) = part.strip_prefix("avg10=") {
            return val_str.parse::<f64>()
                .map_err(|_| QosError::PsiParseError(format!("Invalid float: {}", val_str)));
        }
    }
    Err(QosError::PsiParseError("avg10 key not found".to_string()))
}

pub fn set_system_prop(key: &str, value: &str) {
    if !key.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') { return; }
    if !validate_value(value) { return; }
    let _ = Command::new("setprop").arg(key).arg(value).status();
}

pub fn set_android_setting(table: &str, key: &str, value: &str) {
    if !["system", "global", "secure"].contains(&table) { return; }
    if !validate_value(key) || !validate_value(value) { return; }
    let _ = Command::new("cmd").args(["settings", "put", table, key, value]).status();
}