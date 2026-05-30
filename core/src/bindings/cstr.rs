//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types;

pub fn with_cstr<F, R>(s: &str, f: F) -> Result<R, types::QosError>
where
    F: FnOnce(*const core::ffi::c_char) -> R,
{
    let bytes = s.as_bytes();
    if bytes.len() >= 256 {
        return Err(types::QosError::InvalidInput(
            "String too long for FFI boundary".into(),
        ));
    }

    let mut buffer = [0u8; 256];
    buffer[..bytes.len()].copy_from_slice(bytes);
    buffer[bytes.len()] = b'\0';

    Ok(f(buffer.as_ptr().cast::<core::ffi::c_char>()))
}

#[inline]
pub fn validate_value(value: &str) -> bool {
    value.bytes().all(|c| {
        c.is_ascii_alphanumeric() || c == b'.' || c == b'-' || c == b'_' || c == b'=' || c == b' '
    })
}
