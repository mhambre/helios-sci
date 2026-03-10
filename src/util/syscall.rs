//! Linux and Helios x86_64 shared system call abstractions, using the `syscall`
//! instruction and the x86_64 syscall ABI.
//!
//! https://x64.syscall.sh/
use core::arch::asm;

#[inline(always)]
fn decode_ret(ret: isize) -> Result<usize, i32> {
    if (-4095..=-1).contains(&ret) {
        Err((-ret) as i32)
    } else {
        Ok(ret as usize)
    }
}

macro_rules! define_syscall {
    ($name:ident ( $($arg:ident),* ) [ $($operands:tt)* ]) => {
        #[inline(always)]
        pub(crate) unsafe fn $name(num: u64 $(, $arg: usize)*) -> Result<usize, i32> {
            let ret: isize;
            // SAFETY: Register constraints follow Linux x86_64 syscall ABI.
            unsafe {
                asm!(
                    "syscall",
                    inlateout("rax") num as usize => ret,
                    $($operands)*
                    lateout("rcx") _,
                    lateout("r11") _,
                );
            }
            decode_ret(ret)
        }
    };
}

define_syscall!(syscall0() []);
define_syscall!(syscall1(a1) [in("rdi") a1,]);
define_syscall!(syscall2(a1, a2) [in("rdi") a1, in("rsi") a2,]);
define_syscall!(syscall3(a1, a2, a3) [in("rdi") a1, in("rsi") a2, in("rdx") a3,]);
define_syscall!(syscall4(a1, a2, a3, a4) [in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4,]);
define_syscall!(syscall5(a1, a2, a3, a4, a5) [in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4, in("r8") a5,]);
define_syscall!(syscall6(a1, a2, a3, a4, a5, a6) [in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4, in("r8") a5, in("r9") a6,]);

#[cfg(all(test, target_arch = "x86_64", target_os = "linux"))]
mod tests {
    use super::{syscall0, syscall1};

    const GETPID: u64 = 39;
    const CLOSE: u64 = 3;

    #[test]
    fn syscall_getpid_succeeds() {
        // SAFETY: `GETPID` takes no arguments and returns current process id.
        let pid = unsafe { syscall0(GETPID) }.expect("getpid should succeed");
        assert!(pid > 0);
    }

    #[test]
    fn syscall_close_invalid_fd_returns_ebadf() {
        // SAFETY: `close(-1)` is valid input and should fail with EBADF.
        let err = unsafe { syscall1(CLOSE, (-1_i32) as isize as usize) }.expect_err("close(-1) should fail");
        assert_eq!(err, libc::EBADF);
    }
}
