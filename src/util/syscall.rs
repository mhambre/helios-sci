//! Linux and Helios x86 shared system call abstractions, using the int 0x80 syscall ABI
//! for maximum compatibility with older kernels and Helios itself.
//!
//! https://x86.syscall.sh/
use core::arch::asm;

#[inline(always)]
fn decode_ret(ret: i32) -> Result<usize, i32> {
    if (-4095..=-1).contains(&ret) {
        Err(-ret)
    } else {
        Ok(ret as usize)
    }
}

#[inline(always)]
pub(crate) unsafe fn syscall0(num: u32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Register constraints follow x86 int 0x80 syscall ABI.
    unsafe {
        asm!("int 0x80", inlateout("eax") num as i32 => ret);
    }
    decode_ret(ret)
}

#[inline(always)]
pub(crate) unsafe fn syscall1(num: u32, a1: i32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Preserve ebx because LLVM may reserve it for PIC/GOT.
    unsafe {
        asm!(
            "push ebx",
            "mov ebx, {a1:e}",
            "int 0x80",
            "pop ebx",
            a1 = in(reg) a1 as i32,
            inlateout("eax") num as i32 => ret,
        );
    }
    decode_ret(ret)
}

#[inline(always)]
pub(crate) unsafe fn syscall2(num: u32, a1: i32, a2: i32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Preserve ebx because LLVM may reserve it for PIC/GOT.
    unsafe {
        asm!(
            "push ebx",
            "mov ebx, {a1:e}",
            "mov ecx, {a2:e}",
            "int 0x80",
            "pop ebx",
            a1 = in(reg) a1 as i32,
            a2 = in(reg) a2 as i32,
            inlateout("eax") num as i32 => ret,
            lateout("ecx") _,
        );
    }
    decode_ret(ret)
}

#[inline(always)]
pub(crate) unsafe fn syscall3(num: u32, a1: i32, a2: i32, a3: i32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Preserve ebx because LLVM may reserve it for PIC/GOT.
    unsafe {
        asm!(
            "push ebx",
            "mov ebx, {a1:e}",
            "mov ecx, {a2:e}",
            "mov edx, {a3:e}",
            "int 0x80",
            "pop ebx",
            a1 = in(reg) a1 as i32,
            a2 = in(reg) a2 as i32,
            a3 = in(reg) a3 as i32,
            inlateout("eax") num as i32 => ret,
            lateout("ecx") _,
            lateout("edx") _,
        );
    }
    decode_ret(ret)
}

#[inline(always)]
pub(crate) unsafe fn syscall4(num: u32, a1: i32, a2: i32, a3: i32, a4: i32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Preserve ebx/esi because LLVM may reserve them for generated code.
    unsafe {
        asm!(
            "push ebx",
            "push esi",
            "mov ebx, {a1:e}",
            "mov ecx, {a2:e}",
            "mov edx, {a3:e}",
            "mov esi, {a4:e}",
            "int 0x80",
            "pop esi",
            "pop ebx",
            a1 = in(reg) a1 as i32,
            a2 = in(reg) a2 as i32,
            a3 = in(reg) a3 as i32,
            a4 = in(reg) a4 as i32,
            inlateout("eax") num as i32 => ret,
            lateout("ecx") _,
            lateout("edx") _,
        );
    }
    decode_ret(ret)
}

#[inline(always)]
pub(crate) unsafe fn syscall5(num: u32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Preserve ebx/esi/edi because LLVM may reserve them for generated code.
    unsafe {
        asm!(
            "push ebx",
            "push esi",
            "push edi",
            "mov ebx, {a1:e}",
            "mov ecx, {a2:e}",
            "mov edx, {a3:e}",
            "mov esi, {a4:e}",
            "mov edi, {a5:e}",
            "int 0x80",
            "pop edi",
            "pop esi",
            "pop ebx",
            a1 = in(reg) a1 as i32,
            a2 = in(reg) a2 as i32,
            a3 = in(reg) a3 as i32,
            a4 = in(reg) a4 as i32,
            a5 = in(reg) a5 as i32,
            inlateout("eax") num as i32 => ret,
            lateout("ecx") _,
            lateout("edx") _,
        );
    }
    decode_ret(ret)
}

#[inline(always)]
pub(crate) unsafe fn syscall6(num: u32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) -> Result<usize, i32> {
    let ret: i32;
    // SAFETY: Preserve ebx/esi/edi/ebp because LLVM may reserve them for generated code.
    unsafe {
        asm!(
            "push ebx",
            "push esi",
            "push edi",
            "push ebp",
            "mov ebx, {a1:e}",
            "mov ecx, {a2:e}",
            "mov edx, {a3:e}",
            "mov esi, {a4:e}",
            "mov edi, {a5:e}",
            "mov ebp, {a6:e}",
            "int 0x80",
            "pop ebp",
            "pop edi",
            "pop esi",
            "pop ebx",
            a1 = in(reg) a1 as i32,
            a2 = in(reg) a2 as i32,
            a3 = in(reg) a3 as i32,
            a4 = in(reg) a4 as i32,
            a5 = in(reg) a5 as i32,
            a6 = in(reg) a6 as i32,
            inlateout("eax") num as i32 => ret,
            lateout("ecx") _,
            lateout("edx") _,
        );
    }
    decode_ret(ret)
}
