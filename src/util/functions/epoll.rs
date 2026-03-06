use core::ptr;

use crate::util::functions::fcntl;
use crate::util::syscall::{syscall1, syscall4};

// System call numbers for x86 Linux.
const EPOLL_CREATE1: u32 = 329;
const EPOLL_WAIT: u32 = 256;
const EPOLL_CTL: u32 = 255;

// System call numbers for x86 Linux.
pub(crate) const EPOLL_CTL_ADD: i32 = 1; // Register the target file descriptor on the epoll instance referred to by epfd and associate the event event with the internal file linked to fd.
pub(crate) const EPOLL_CTL_DEL: i32 = 2; // Remove the target file descriptor from the epoll instance referred to by epfd.
pub(crate) const EPOLL_CTL_MOD: i32 = 3; // Change the settings associated with the target file descriptor in the epoll instance referred to

pub(crate) const EPOLL_IN: u32 = 0x001; // The associated file is available for read operations.
pub(crate) const EPOLL_OUT: u32 = 0x004; // Writing is possible on the associated file descriptor.
pub(crate) const EPOLLERR: u32 = 0x008; // Error condition happened on the associated file descriptor.

pub(crate) const EPOLLONESHOT: u32 = 0x40000000; // Only trigger once, then remove from the set.

// https://docs.rs/libc/latest/libc/struct.epoll_event.html
#[repr(C, packed(1))]
pub(crate) struct epoll_event {
    pub events: u32,
    pub data: u64,
}

// https://man7.org/linux/man-pages/man2/epoll_create1.2.html
pub(crate) fn epoll_create1(flags: i32) -> Result<i32, i32> {
    syscall1(EPOLL_CREATE1, flags).map(|fd| fd as i32)
}

// https://man7.org/linux/man-pages/man2/epoll_ctl.2.html
pub(crate) fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: Option<&epoll_event>) -> Result<(), i32> {
    let event_ptr = event.map_or(ptr::null(), |ev| ev as *const epoll_event) as usize as i32;
    syscall4(EPOLL_CTL, epfd, op, fd, event_ptr).map(|_| ())
}

// https://man7.org/linux/man-pages/man2/epoll_wait.2.html
pub(crate) fn epoll_wait(epfd: i32, events: &mut [epoll_event], timeout: i32) -> Result<usize, i32> {
    syscall4(EPOLL_WAIT, epfd, events.as_mut_ptr() as usize as i32, events.len() as i32, timeout).map(|n| n as usize)
}
