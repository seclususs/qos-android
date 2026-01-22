pub mod battery;
pub mod bridge;
pub mod cached_file;
pub mod filesystem;
pub mod kernel;
pub mod monitored_file;
pub mod properties;
pub mod surface_flinger;
pub mod thermal;
pub mod traversal;

pub fn validate_value(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '=' || c == ' ')
}
