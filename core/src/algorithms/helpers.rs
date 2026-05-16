//! Author: [Seclususs](https://github.com/seclususs)

#[inline]
pub fn sanitize_to_u64(val: f32, fallback: u64) -> u64 {
    if !val.is_finite() || val < 0.0 {
        return fallback;
    }
    (val + 0.5) as u64
}

#[inline]
pub fn sanitize_to_clean_u64(val: f32, fallback: u64, step: u64) -> u64 {
    let val_u64 = sanitize_to_u64(val, fallback);
    if step == 0 || step == 1 {
        return val_u64;
    }
    let remainder = val_u64 % step;
    if remainder >= step / 2 {
        val_u64 + step - remainder
    } else {
        val_u64 - remainder
    }
}
