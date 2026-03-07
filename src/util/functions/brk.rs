use crate::util::errno;
use crate::util::syscall::syscall1;

// System call numbers for x86 Linux.
const BRK: u32 = 45;

/// https://man7.org/linux/man-pages/man2/brk.2.html
///
/// Set the program break to `addr` and return the resulting break.
///
/// Linux `sys_brk` semantics:
/// - Success: returns the new break value.
/// - Failure: returns the current break (i.e., lower than requested `addr`).
pub(crate) fn brk(addr: usize) -> Result<usize, i32> {
    let requested = addr as i32;
    // SAFETY: `BRK` number and integer argument follow Linux i386 syscall ABI.
    let result = unsafe { syscall1(BRK, requested) }? as usize;

    if addr != 0 && result < addr {
        Err(errno::ENOMEM)
    } else {
        Ok(result)
    }
}

/// https://man7.org/linux/man-pages/man2/sbrk.2.html
///
/// Change the program break by `increment` bytes.
///
/// Returns the previous break address on success.
pub(crate) fn sbrk(increment: isize) -> Result<*mut u8, i32> {
    let current = brk(0)?;
    let next = if increment >= 0 {
        current.checked_add(increment as usize)
    } else {
        current.checked_sub((-increment) as usize)
    }
    .ok_or(errno::ENOMEM)?;

    let applied = brk(next)?;
    if applied < next {
        return Err(errno::ENOMEM);
    }

    Ok(current as *mut u8)
}
