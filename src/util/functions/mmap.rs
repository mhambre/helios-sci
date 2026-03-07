use crate::util::errno;
use crate::util::syscall::{syscall2, syscall6};

// System call numbers for x86 Linux.
const MMAP2: u32 = 192;
const MUNMAP: u32 = 91;

// File protections for memory map region.
pub(crate) const PROT_READ: i32 = 0x1; // Page can be read.
pub(crate) const PROT_WRITE: i32 = 0x2; // Page can be written.
pub(crate) const PROT_EXEC: i32 = 0x4; // Page can be executed.
pub(crate) const PROT_NONE: i32 = 0x0; // Page can not be accessed.
pub(crate) const PROT_GROWSDOWN: i32 = 0x0100_0000; // mprotect-only growsdown hint.
pub(crate) const PROT_GROWSUP: i32 = 0x0200_0000; // mprotect-only growsup hint.

// Mapping flags.
pub(crate) const MAP_FILE: i32 = 0;
pub(crate) const MAP_SHARED: i32 = 0x01;
pub(crate) const MAP_PRIVATE: i32 = 0x02;
pub(crate) const MAP_FIXED: i32 = 0x10;
pub(crate) const MAP_ANONYMOUS: i32 = 0x20; // Don't use a file.
pub(crate) const MAP_ANON: i32 = MAP_ANONYMOUS;
pub(crate) const MAP_32BIT: i32 = 0x40;
pub(crate) const MAP_GROWSDOWN: i32 = 0x00100; // Stack-like segment.
pub(crate) const MAP_DENYWRITE: i32 = 0x00800;
pub(crate) const MAP_EXECUTABLE: i32 = 0x01000;
pub(crate) const MAP_LOCKED: i32 = 0x02000;
pub(crate) const MAP_NORESERVE: i32 = 0x04000;
pub(crate) const MAP_POPULATE: i32 = 0x08000;
pub(crate) const MAP_NONBLOCK: i32 = 0x10000;
pub(crate) const MAP_STACK: i32 = 0x20000;

// Mapping errors.
pub(crate) const MAP_FAILED: *mut u8 = !0 as *mut u8; // (void *) -1

/// https://man7.org/linux/man-pages/man2/mmap.2.html
///
/// Map `length` bytes starting at `addr` with
/// protection `prot` and flags `flags` on file descriptor
/// `fd` at offset `offset`.
pub(crate) fn mmap(addr: usize, length: usize, prot: i32, flags: i32, fd: i32, offset: usize) -> Result<*mut u8, i32> {
    if (offset & 0xFFF) != 0 {
        return Err(errno::EINVAL);
    }

    let pgoff = (offset >> 12) as i32;

    // SAFETY: Raw `mmap2` syscall forwarding; args match Linux i386 mmap2 ABI.
    let result = unsafe { syscall6(MMAP2, addr as i32, length as i32, prot, flags, fd, pgoff) }?;
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
