use crate::util::syscall::syscall3;

// System call numbers for x86 Linux.
const FCNTL: u32 = 55;

pub(crate) const F_DUPFD: i32 = 0; /* Duplicate file descriptor.  */
pub(crate) const F_GETFD: i32 = 1; /* Get file descriptor flags.  */
pub(crate) const F_SETFD: i32 = 2; /* Set file descriptor flags.  */
pub(crate) const F_GETFL: i32 = 3; /* Get file status flags.  */
pub(crate) const F_SETFL: i32 = 4; /* Set file status flags.  */

pub(crate) const FD_CLOEXEC: i32 = 1; /* Close the fd on exec.  */

// https://man7.org/linux/man-pages/man2/fcntl.2.html
pub(crate) fn fcntl(fd: i32, cmd: i32, arg: i32) -> Result<i32, i32> {
    // SAFETY: `FCNTL` number and integer arguments follow Linux i386 syscall ABI.
    unsafe { syscall3(FCNTL, fd, cmd, arg) }.map(|res| res as i32)
}

#[cfg(all(test, target_arch = "x86", target_os = "linux"))]
mod tests {
    use super::{F_GETFD, fcntl};
    use crate::util::errno;

    #[test]
    fn fcntl_invalid_fd_returns_ebadf() {
        let err = fcntl(-1, F_GETFD, 0).expect_err("fcntl on invalid fd should fail");
        assert_eq!(err, errno::EBADF);
    }

    #[test]
    fn fcntl_getfd_stdout_succeeds() {
        let flags = fcntl(1, F_GETFD, 0).expect("fcntl(F_GETFD) on stdout should succeed");
        assert!(flags >= 0);
    }
}
