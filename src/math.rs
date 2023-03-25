pub fn log2_ceil(n: u64) -> u64 {
    (n as f32).log2().ceil() as u64
}

pub fn ceil_div(a: u64, b: u64) -> u64 {
    (a + b - 1) / b
}

pub fn div_with_remainder(a: u64, b: u64) -> (u64, u64) {
    let div = a / b;
    let rem = a % b;
    (div, rem)
}

// https://eugene-babichenko.github.io/blog/2019/11/13/rust-popcount-intrinsics/
#[inline(never)]
#[cfg_attr(target_arch = "x86_64", target_feature(enable = "popcnt"))]
/// Count the number of ones in the binary representation of the target integer
///
/// # Safety
///
/// This library depends on a single popcnt instruction to support constant time
/// rank queries. Use of this library on a machine that does not include the instruction
/// is undefined behavior
///
pub unsafe fn popcount(x: u64) -> u32 {
    x.count_ones()
}
