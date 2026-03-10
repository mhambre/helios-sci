use libc::EINVAL;

use crate::util::syscall::{syscall2, syscall6};

// System call numbers for x86_64 Linux.
const MMAP: u64 = 9;
const MUNMAP: u64 = 11;

// Mapping errors.
pub(crate) const MAP_FAILED: *mut u8 = !0 as *mut u8; // (void *) -1

#[inline]
fn arg_i32(value: i32) -> usize {
    value as isize as usize
}

/// https://man7.org/linux/man-pages/man2/mmap.2.html
///
/// Map `length` bytes starting at `addr` with
/// protection `prot` and flags `flags` on file descriptor
/// `fd` at offset `offset`.
pub(crate) fn mmap(addr: usize, length: usize, prot: i32, flags: i32, fd: i32, offset: usize) -> Result<*mut u8, i32> {
    if (offset & 0xFFF) != 0 {
        return Err(EINVAL);
    }

    // SAFETY: Raw `mmap` syscall forwarding; args match Linux x86_64 mmap ABI.
    let result = unsafe { syscall6(MMAP, addr, length, arg_i32(prot), arg_i32(flags), arg_i32(fd), offset) }?;
    Ok(result as *mut u8)
}

/// https://man7.org/linux/man-pages/man2/munmap.2.html
///
/// Unmaps the mapping starting at `addr` of length `length`.
///
/// # Safety
///
/// The caller must ensure that `addr` and `length` correspond to a valid mapping created by `mmap`, and that
/// the mapping is not currently in use by any threads.
pub(crate) unsafe fn munmap(addr: *mut u8, length: usize) -> Result<(), i32> {
    // SAFETY: Raw `munmap` syscall forwarding; caller upholds `munmap` preconditions.
    unsafe { syscall2(MUNMAP, addr as usize, length) }.map(|_| ())
}
