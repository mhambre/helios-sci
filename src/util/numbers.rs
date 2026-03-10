//! Simple number utilities.

/// Converts an `i32` to a `usize` for syscall arguments, ensuring the value is correctly
/// sign-extended
#[inline]
pub(crate) fn arg_i32(value: i32) -> usize {
    value as isize as usize
}

/// Aligns `addr` upwards to the nearest multiple of `align`, which must be a power of two.
#[inline]
pub(crate) fn align_up(addr: usize, align: usize) -> usize {
    let mask = align - 1;
    (addr + mask) & !mask
}
