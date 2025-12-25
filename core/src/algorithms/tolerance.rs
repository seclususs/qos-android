//! Author: [Seclususs](https://github.com/seclususs)

#[inline(always)]
pub fn check_absolute(current: u64, target: u64, threshold: u64) -> bool {
    if current == target {
        return false;
    }
    current.abs_diff(target) >= threshold
}

#[inline(always)]
pub fn check_relative(current: u64, target: u64, tolerance_pct: f64) -> bool {
    if current == target {
        return false;
    }
    if current == 0 {
        return target != 0;
    }
    let diff = current.abs_diff(target) as f64;
    let change_ratio = diff / current as f64;
    change_ratio >= tolerance_pct
}