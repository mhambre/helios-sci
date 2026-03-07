use crate::util::syscall::{syscall2, syscall6};

const MMAP: u32 = 192;
const MUNMAP: u32 = 91;

/// https://man7.org/linux/man-pages/man2/mmap.2.html
///
/// Map `length` bytes starting at `addr` with
/// protection `prot` and flags `flags` on file descriptor
/// `fd` at offset `offset`.
pub(crate) fn mmap(addr: usize, length: usize, prot: i32, flags: i32, fd: i32, offset: usize) -> Result<*mut u8, i32> {
    // SAFETY: Raw `mmap2` syscall forwarding; caller-supplied args are passed verbatim.
    let result = unsafe { syscall6(MMAP, addr as i32, length as i32, prot, flags, fd, offset as i32) }?;
    Ok(result as *mut u8)
}

/// https://man7.org/linux/man-pages/man2/munmap.2.html
///
/// Unmaps the mapping starting at `addr` of length `length`.
///
/// # Safety
///
/// The caller must ensure that `addr` and `length` correspond to a valid mapping created by `mmap`, and that
/// the mapping is not currently in use by any threads. Failure to
pub(crate) unsafe fn munmap(addr: *mut u8, length: usize) -> Result<(), i32> {
    // SAFETY: Raw `munmap` syscall forwarding; caller upholds `munmap` preconditions.
    unsafe { syscall2(MUNMAP, addr as usize as i32, length as i32) }.map(|_| ())
}
