use std::sync::atomic::{AtomicU32, Ordering};

use crate::util::numbers::arg_i32;
use crate::util::syscall::syscall6;

// System call numbers for x86_64 Linux.
const FUTEX: u64 = 202;

// Bug 1: The addr parameter type was `*mut *const u32` (pointer to pointer),
// but futex expects a plain `*mut u32`. This would pass the wrong address to the kernel.
unsafe fn futex(
    addr: *mut u32,
    op: i32,
    val: u32,
    timeout: *const u8,
    addr2: *const u32,
    val3: u32,
) -> Result<i32, i32> {
    unsafe {
        syscall6(FUTEX, addr as usize, arg_i32(op), val as usize, timeout as usize, addr2 as usize, val3 as usize)
    }
    .map(|res| res as i32)
}

/// https://man7.org/linux/man-pages/man2/futex.2.html
///
/// This function will block the current thread until the value at `addr` is not equal to `expected`.
pub(crate) fn futex_wait(addr: &AtomicU32, expected: u32) {
    loop {
        if addr.load(Ordering::Acquire) != expected {
            return;
        }

        let res = unsafe {
            futex(
                addr as *const AtomicU32 as *mut u32,
                libc::FUTEX_WAIT,
                expected,
                core::ptr::null(),
                core::ptr::null(),
                0,
            )
        };

        if let Err(err) = res {
            if err == libc::EAGAIN || err == libc::EINTR {
                continue;
            }

            panic!("futex_wait failed: {}", err);
        }
    }
}

/// https://man7.org/linux/man-pages/man2/futex.2.html
///
/// This function will wake up threads waiting on the futex at `addr`.
pub(crate) fn futex_wake(addr: &AtomicU32, count: u32) -> i32 {
    let res = unsafe {
        futex(
            addr as *const AtomicU32 as *mut u32,
            libc::FUTEX_WAKE,
            count,
            core::ptr::null(),
            core::ptr::null(),
            0,
        )
    };

    if let Err(err) = res {
        panic!("futex_wake failed with error: {}", err);
    }

    res.unwrap()
}
