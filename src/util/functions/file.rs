use crate::util::syscall::{syscall1, syscall3};

// System call numbers for x86 Linux.
const OPEN: u32 = 5; /* open a file descriptor */
const READ: u32 = 3; /* read from a file descriptor */
const WRITE: u32 = 4; /* write to a file descriptor */
const CLOSE: u32 = 6; /* close a file descriptor */

// https://man7.org/linux/man-pages/man2/open.2.html
fn open(path: &[u8], flags: i32, mode: i32) -> Result<i32, i32> {
    syscall3(OPEN, path.as_ptr() as usize as i32, flags, mode).map(|fd| fd as i32)
}

// https://man7.org/linux/man-pages/man2/read.2.html
fn read(fd: i32, buf: &mut [u8]) -> Result<usize, i32> {
    syscall3(READ, fd, buf.as_mut_ptr() as usize as i32, buf.len() as i32).map(|n| n as usize)
}

// https://man7.org/linux/man-pages/man2/write.2.html
fn write(fd: i32, buf: &[u8]) -> Result<usize, i32> {
    syscall3(WRITE, fd, buf.as_ptr() as usize as i32, buf.len() as i32).map(|n| n as usize)
}

// https://man7.org/linux/man-pages/man2/close.2.html
fn close(fd: i32) -> Result<(), i32> {
    syscall1(CLOSE, fd).map(|_| ())
}
