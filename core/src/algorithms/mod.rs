pub mod cpu_math;
pub mod memory_math;
pub mod storage_math;
pub mod poll_math;
pub mod thermal_math;

#[inline(always)]
pub fn sanitize_to_u64(val: f64, fallback: u64) -> u64 {
    if !val.is_finite() {
        return fallback;
    }
    val.round() as u64
}