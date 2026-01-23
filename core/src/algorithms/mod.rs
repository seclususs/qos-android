pub mod cpu_math;
pub mod filter_math;
pub mod poll_math;
pub mod storage_math;
pub mod thermal_math;

#[inline(always)]
pub fn sanitize_to_u64(val: f32, fallback: u64) -> u64 {
    if !val.is_finite() {
        return fallback;
    }
    val.round() as u64
}

#[inline(always)]
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
