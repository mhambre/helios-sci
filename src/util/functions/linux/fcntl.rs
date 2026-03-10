use crate::util::numbers::arg_i32;
use crate::util::syscall::syscall3;

// System call numbers for x86_64 Linux.
const FCNTL: u64 = 72;

// https://man7.org/linux/man-pages/man2/fcntl.2.html
pub(crate) fn fcntl(fd: i32, cmd: i32, arg: i32) -> Result<i32, i32> {
    // SAFETY: `FCNTL` number and integer arguments follow Linux x86_64 syscall ABI.
    unsafe { syscall3(FCNTL, arg_i32(fd), arg_i32(cmd), arg_i32(arg)) }.map(|res| res as i32)
}

#[cfg(all(test, target_arch = "x86_64", target_os = "linux"))]
mod tests {
    use super::fcntl;

    #[test]
    fn fcntl_invalid_fd_returns_ebadf() {
        let err = fcntl(-1, libc::F_GETFD, 0).expect_err("fcntl on invalid fd should fail");
        assert_eq!(err, libc::EBADF);
    }

    #[test]
    fn fcntl_getfd_stdout_succeeds() {
        let flags = fcntl(1, libc::F_GETFD, 0).expect("fcntl(F_GETFD) on stdout should succeed");
        assert!(flags >= 0);
    }
}
