//! The polling system for our asynchronous runtime, how our executor
//! waits for events and wakes up tasks.

use std::os::fd;

use crate::util::functions::{EPOLL_CTL_ADD, epoll, epoll_create1, epoll_ctl, epoll_event, fcntl};

const RESERVED_FD: i32 = 0; // We reserve fd 0 for the runtime descriptor, which we use to track events.

// Helper function to get a file descriptor for an epoll instance that will
// be closed on exec.
fn create_fd() -> Result<i32, i32> {
    let fd = epoll_create1(0)?;
    let flags = fcntl(fd, fcntl::F_GETFL, 0)?;
    fcntl(fd, fcntl::F_SETFL, flags | fcntl::FD_CLOEXEC)?;
    Ok(fd)
}

// Helper function to add a file descriptor to the epoll instance with
// the specified events causing a reaction.
fn watch_fd(runtime_fd: i32, target_fd: i32, events: u32) -> Result<(), i32> {
    let event = epoll_event {
        events,
        data: target_fd as u64,
    };
    epoll_ctl(runtime_fd, EPOLL_CTL_ADD, target_fd, Some(&event))
}

fn create_runtime_descriptor() -> Result<i32, i32> {
    let fd = create_fd()?;

    Ok(fd)
}
