//! Memory allocation utilities for the Helios runtime.
#![allow(dead_code)]

use alloc::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;

const DEFAULT_HEAP_SIZE: usize = 1 << 20; // 1 MiB (before we call `mmap` to expand it)

/// A simple allocator that uses the `sbrk` and `mmap` system calls to allocate a
/// heap region.
pub struct HeliosAllocator {
    start: Cell<*mut u8>,
    end: Cell<*mut u8>,
}

impl HeliosAllocator {
    pub const fn new() -> Self {
        Self {
            start: Cell::new(0 as *mut u8),
            end: Cell::new(0 as *mut u8),
        }
    }

    unsafe fn init(&self) {
        unimplemented!()
    }
}

/// Implement the `GlobalAlloc` trait for `HeliosAllocator` to enable heap allocation
/// for an entire program.
///
/// To register as a program's global allocator, add the following to the program's main file:
/// ```rust
/// #[global_allocator]
/// static ALLOCATOR: HeliosAllocator = HeliosAllocator::new();
/// ```
unsafe impl GlobalAlloc for HeliosAllocator {
    /// Allocates memory to the heap.
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!()
    }

    /// Deallocates memory from the heap.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!()
    }
}
