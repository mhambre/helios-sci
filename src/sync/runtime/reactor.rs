//! The polling system for our asynchronous runtime, how our executor
//! waits for events and wakes up tasks.

use crate::util::functions::{EpollEvent, epoll_create1, epoll_ctl};

// Helper function to get a file descriptor for an epoll instance that will
// be closed on exec.
fn create_fd() -> Result<i32, i32> {
    epoll_create1(libc::EPOLL_CLOEXEC)
}

// Helper function to add a file descriptor to the epoll instance with
// the specified events causing a reaction.
fn watch_fd(runtime_fd: i32, target_fd: i32, events: u32) -> Result<(), i32> {
    let event = EpollEvent {
        events,
        data: target_fd as u64,
    };
    epoll_ctl(runtime_fd, libc::EPOLL_CTL_ADD, target_fd, Some(&event))
}

fn create_runtime_descriptor() -> Result<i32, i32> {
    let fd = create_fd()?;

    Ok(fd)
}
