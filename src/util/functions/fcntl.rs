use crate::util::syscall::syscall3;

const FCNTL: u32 = 55; /* fcntl system call number for x86 Linux. */

pub(crate) const F_DUPFD: i32 = 0; /* Duplicate file descriptor.  */
pub(crate) const F_GETFD: i32 = 1; /* Get file descriptor flags.  */
pub(crate) const F_SETFD: i32 = 2; /* Set file descriptor flags.  */
pub(crate) const F_GETFL: i32 = 3; /* Get file status flags.  */
pub(crate) const F_SETFL: i32 = 4; /* Set file status flags.  */

pub(crate) const FD_CLOEXEC: i32 = 1; /* Close the fd on exec.  */

// https://man7.org/linux/man-pages/man2/fcntl.2.html
pub(crate) fn fcntl(fd: i32, cmd: i32, arg: i32) -> Result<i32, i32> {
    syscall3(FCNTL, fd, cmd, arg).map(|res| res as i32)
}
