//! Author: [Seclususs](https://github.com/seclususs)

use std::fs;
use std::process::Command;

fn validate_path(path: &str) -> bool {
    let allowed_prefixes = [
        "/proc/", 
        "/sys/", 
        "/dev/input/"
    ];
    let has_valid_prefix = allowed_prefixes.iter().any(|&prefix| path.starts_with(prefix));
    let no_traversal = !path.contains("..");
    let safe_chars = path.chars().all(|c| 
        c.is_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.'
    );
    has_valid_prefix && no_traversal && safe_chars
}

fn validate_prop_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn validate_value(value: &str) -> bool {
    value.chars().all(|c| 
        c.is_alphanumeric() || 
        c == '.' || c == ',' || c == '-' || c == '_' || c == '=' || c == ' '
    )
}

pub fn write_to_file(path: &str, value: &str) -> bool {
    if !validate_path(path) {
        error!("Security Violation: Attempt to write to invalid path: {}", path);
        return false;
    }
    if !validate_value(value) {
        error!("Security Violation: Invalid characters in value for {}: {}", path, value);
        return false;
    }
    match fs::write(path, value) {
        Ok(_) => true,
        Err(e) => {
            debug!("Failed to write '{}' to {}: {}", value, path, e);
            false
        }
    }
}

pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    if !validate_path(path) {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Path validation failed"));
    }
    fs::read_to_string(path)
}

pub fn set_system_prop(key: &str, value: &str) {
    if !validate_prop_key(key) {
        error!("Security Violation: Invalid system prop key: {}", key);
        return;
    }
    if !validate_value(value) {
        error!("Security Violation: Potential shell injection in prop value: {}", value);
        return;
    }
    let status = Command::new("setprop")
        .arg(key)
        .arg(value)
        .status();
    match status {
        Ok(s) if s.success() => {},
        _ => error!("Failed to set prop {} = {}", key, value),
    }
}

pub fn set_android_setting(table: &str, key: &str, value: &str) {
    if !["system", "global", "secure"].contains(&table) {
        error!("Security Violation: Invalid settings table: {}", table);
        return;
    }
    if !validate_prop_key(key) {
         error!("Security Violation: Invalid setting key: {}", key);
         return;
    }
    if !validate_value(value) {
         error!("Security Violation: Invalid setting value: {}", value);
         return;
    }
    let status = Command::new("cmd")
        .arg("settings")
        .arg("put")
        .arg(table)
        .arg(key)
        .arg(value)
        .status();
    match status {
        Ok(s) if s.success() => {},
        _ => error!("Failed to set setting {}/{} = {}", table, key, value),
    }
}

pub fn parse_psi_avg10(path: &str) -> f64 {
    let content = match read_file(path) {
        Ok(c) => c,
        Err(_) => return 0.0,
    };
    for part in content.split_whitespace() {
        if part.starts_with("avg10=") {
            if let Some(val_str) = part.split('=').nth(1) {
                return val_str.parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    0.0
}