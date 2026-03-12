use atomic_enum::atomic_enum;
use core::ptr::null_mut;

/// Allocator state values multi-threaded coordination.
#[derive(PartialEq)]
#[atomic_enum]
pub(super) enum AllocState {
    Uninitialized,
    InProgress,
    Ready,
}

pub(super) const DEFAULT_HEAP_SIZE: usize = 1 << 20; // 1 MiB
pub(super) const PAGE_SIZE: usize = 1 << 12; // 4 KiB

// Requests a new chunk of memory from the kernel to be used as the heap for the allocator
pub(super) unsafe fn request_heap_chunk(size: Option<usize>) -> *const u8 {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux", target_arch = "x86_64")] {
            crate::util::functions::mmap(0, size.unwrap_or(DEFAULT_HEAP_SIZE), libc::PROT_WRITE | libc::PROT_READ, libc::MAP_PRIVATE | libc::MAP_ANON, -1, 0).unwrap_or(null_mut()) as *const u8
        } else {
            compile_error!("Unsupported Target OS")
        }
    }
}
