use core::ptr;

use crate::util::syscall::{syscall1, syscall4};

// System call numbers for x86_64 Linux.
const EPOLL_CREATE1: u64 = 291;
const EPOLL_WAIT: u64 = 232;
const EPOLL_CTL: u64 = 233;

// https://docs.rs/libc/latest/libc/struct.epoll_event.html
#[repr(C, packed(1))]
pub(crate) struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

#[inline]
fn arg_i32(value: i32) -> usize {
    value as isize as usize
}

// https://man7.org/linux/man-pages/man2/epoll_create1.2.html
pub(crate) fn epoll_create1(flags: i32) -> Result<i32, i32> {
    // SAFETY: `EPOLL_CREATE1` number and integer arguments match Linux x86_64 syscall ABI.
    unsafe { syscall1(EPOLL_CREATE1, arg_i32(flags)) }.map(|fd| fd as i32)
}

// https://man7.org/linux/man-pages/man2/epoll_ctl.2.html
pub(crate) fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: Option<&EpollEvent>) -> Result<(), i32> {
    let event_ptr = event.map_or(ptr::null(), |ev| ev as *const EpollEvent) as usize;
    // SAFETY: `EPOLL_CTL` argument layout matches Linux x86_64 syscall ABI.
    unsafe { syscall4(EPOLL_CTL, arg_i32(epfd), arg_i32(op), arg_i32(fd), event_ptr) }.map(|_| ())
}

// https://man7.org/linux/man-pages/man2/epoll_wait.2.html
pub(crate) fn epoll_wait(epfd: i32, events: &mut [EpollEvent], timeout: i32) -> Result<usize, i32> {
    // SAFETY: `events` points to writable memory for `events.len()` entries.
    unsafe { syscall4(EPOLL_WAIT, arg_i32(epfd), events.as_mut_ptr() as usize, events.len(), arg_i32(timeout)) }
}

#[cfg(all(test, target_arch = "x86_64", target_os = "linux"))]
mod tests {
    use super::epoll_create1;
    use crate::util::functions::close;

    #[test]
    fn epoll_create_and_close_succeeds() {
        let epfd = epoll_create1(libc::EPOLL_CLOEXEC).expect("epoll_create1 should succeed");
        close(epfd).expect("close epoll fd should succeed");
    }
}
