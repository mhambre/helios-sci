use crate::util::errno;
use crate::util::syscall::{syscall1, syscall3};

// System call numbers for x86 Linux.
const OPEN: u32 = 5; /* open a file descriptor */
const READ: u32 = 3; /* read from a file descriptor */
const WRITE: u32 = 4; /* write to a file descriptor */
const CLOSE: u32 = 6; /* close a file descriptor */

// https://man7.org/linux/man-pages/man2/open.2.html
pub(crate) fn open(path: &[u8], flags: i32, mode: i32) -> Result<i32, i32> {
    if path.is_empty() || *path.last().unwrap_or(&0) != 0 {
        return Err(errno::EINVAL);
    }
    // SAFETY: `path` is NUL-terminated and pointer/args are passed using Linux i386 syscall ABI.
    unsafe { syscall3(OPEN, path.as_ptr() as usize as i32, flags, mode) }.map(|fd| fd as i32)
}

// https://man7.org/linux/man-pages/man2/read.2.html
pub(crate) fn read(fd: i32, buf: &mut [u8]) -> Result<usize, i32> {
    // SAFETY: `buf` points to writable memory for `buf.len()` bytes.
    unsafe { syscall3(READ, fd, buf.as_mut_ptr() as usize as i32, buf.len() as i32) }.map(|n| n as usize)
}

// https://man7.org/linux/man-pages/man2/write.2.html
pub(crate) fn write(fd: i32, buf: &[u8]) -> Result<usize, i32> {
    // SAFETY: `buf` points to readable memory for `buf.len()` bytes.
    unsafe { syscall3(WRITE, fd, buf.as_ptr() as usize as i32, buf.len() as i32) }.map(|n| n as usize)
}

// https://man7.org/linux/man-pages/man2/close.2.html
pub(crate) fn close(fd: i32) -> Result<(), i32> {
    // SAFETY: `CLOSE` number and integer argument follow Linux i386 syscall ABI.
    unsafe { syscall1(CLOSE, fd) }.map(|_| ())
}

#[cfg(all(test, target_arch = "x86", target_os = "linux"))]
mod tests {
    use super::{close, open, read};
    use crate::util::errno;

    #[test]
    fn open_nonexistent_path_returns_enoent() {
        let path = b"/__helios_sci_missing_file_for_test__\0";
        let err = open(path, 0, 0).expect_err("open on missing path should fail");
        assert_eq!(err, errno::ENOENT);
    }

    #[test]
    fn open_and_read_manifest_succeeds() {
        let mut path = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR should be set")
            .into_bytes();
        path.extend_from_slice(b"/Cargo.toml\0");

        let fd = open(&path, 0, 0).expect("open Cargo.toml should succeed");
        let mut buf = [0_u8; 32];
        let n = read(fd, &mut buf).expect("read should succeed");
        close(fd).expect("close should succeed");
        assert!(n > 0);
    }
}
