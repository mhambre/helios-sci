//! Memory allocation utilities for the Helios runtime.
#![allow(dead_code)]

use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::util::functions;
use crate::util::functions::mmap::{self, MAP_ANON, MAP_PRIVATE, PROT_READ, PROT_WRITE};

const DEFAULT_HEAP_SIZE: usize = 1 << 20; // 1 MiB (before we call `mmap` to expand it)

/// Different allocation strategies for our free list allocator
pub enum AllocateStrategy {
    /// Finds next contiguous set of blocks >= the size of block we need to allocate.
    /// Quicker but more prone to fragmentation.
    NextAvailable,
    /// Finds best contiguous set of blocks to efficiently fit the block we need to allocate.
    /// Slower but less prone to fragmentation.
    BestFit,
}

/// A simple allocator that uses the `sbrk` and `mmap` system calls to allocate a
/// heap region.
pub struct Allocator {
    start: AtomicUsize,
    end: AtomicUsize,
    strategy: AllocateStrategy,
}

/// A singular node in our free list, contains a pointer to
/// our next free block in the free list
pub struct FreeNode {
    next: *mut FreeNode,
}

impl Allocator {
    /// Create a new allocator with the given allocation strategy.
    pub const fn new(strategy: AllocateStrategy) -> Self {
        Self {
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
            strategy,
        }
    }

    /// Initialize an instance of the allocator
    unsafe fn init(&self) -> Result<(), i32> {
        // We've already initialized the allocator
        if self.start.load(Ordering::SeqCst) != 0 {
            return Ok(());
        }

        let block_size = mem::size_of::<FreeNode>();
        let heap = functions::brk::sbrk(DEFAULT_HEAP_SIZE as isize)? as *mut u8;
        let num_blocks = DEFAULT_HEAP_SIZE / block_size;

        // Bootstrap our heap blocks out of the new region
        unsafe {
            for i in 0..num_blocks - 1 {
                let curr = heap.add(i * block_size) as *mut FreeNode;
                let next = heap.add((i + 1) * block_size) as *mut FreeNode;
                (*curr).next = next;
            }
        }

        self.start.store(heap as usize, Ordering::SeqCst);
        self.end.store(heap as usize + DEFAULT_HEAP_SIZE, Ordering::SeqCst);

        Ok(())
    }

    /// Internal function to grow our heap used by the global
    /// allocator implementation.
    unsafe fn extend(&self, size: usize) -> *mut u8 {
        let ptr = mmap::mmap(0, size, PROT_WRITE | PROT_READ, MAP_PRIVATE | MAP_ANON, -1, 0).unwrap_or(null_mut());

        if ptr == null_mut() {
            return null_mut();
        }

        let block_size = mem::size_of::<FreeNode>();
        let num_blocks = (size) / block_size;

        // Bootstrap our heap blocks out of the new region
        unsafe {
            for i in 0..num_blocks - 1 {
                let curr = ptr.add(i * block_size) as *mut FreeNode;
                let next = ptr.add((i + 1) * block_size) as *mut FreeNode;
                (*curr).next = next
            }
        }

        ptr
    }

    // Internal function to find the next available block in the free list that can fit the requested layout.
    unsafe fn try_insert_greedy(mut curr: *mut FreeNode, end: *mut FreeNode, size: usize) -> *mut u8 {
        // Find next available block in the free list that can fit the requested layout
        let block_size = core::mem::size_of::<FreeNode>();
        while curr != end {
            let next = unsafe { (*curr).next };
            let available = next as usize - curr as usize;

            if available >= size {
                // Split the block if there's enough room left over
                let remaining = available - size;
                if remaining >= block_size {
                    // Disappear it from our view
                    let split = (curr as usize + size) as *mut FreeNode;
                    unsafe {
                        (*split).next = next;
                        (*curr).next = split;
                    }
                }
                return curr as *mut u8;
            }

            unsafe { curr = (*curr).next };
        }

        null_mut()
    }
}

/// This is a lie for now but it allows us to use AtomicUsize.
unsafe impl Sync for Allocator {}

/// Implement the `GlobalAlloc` trait for `Allocator` to enable heap allocation
/// for an entire program.
///
/// To register as a program's global allocator, add the following to the program's main file:
/// ```rust
/// use helios_sci::allocate::Allocator;
///
/// #[global_allocator]
/// static ALLOCATOR: Allocator = Allocator::new();
/// ```
unsafe impl GlobalAlloc for Allocator {
    /// Allocates memory to the heap.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Initialize the allocator if it hasn't been already
        if let Err(_) = unsafe { self.init() } {
            return null_mut();
        }

        let curr = self.start.load(Ordering::SeqCst) as *mut FreeNode;
        let end = self.end.load(Ordering::SeqCst) as *mut FreeNode;

        // For now, we only support the next available strategy, but we can add best fit later (probably not though)
        if matches!(self.strategy, AllocateStrategy::BestFit) {
            unimplemented!("Best fit allocation strategy is not implemented yet");
        }

        // Try to find a block in the free list that can fit the requested layout
        let mut ptr = unsafe { Self::try_insert_greedy(curr, end, layout.size()) };
        if ptr != null_mut() {
            return ptr;
        }

        // We couldn't find a block in the free list, so we need to grow our heap
        ptr = unsafe { self.extend(layout.size()) };
        if !ptr.is_null() {
            self.end.store(ptr as usize + layout.size(), Ordering::SeqCst);
            return ptr;
        }

        core::ptr::null_mut()
    }

    /// Deallocates memory from the heap.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // Initialize the allocator if it hasn't been already
        if let Err(_) = unsafe { self.init() } {
            return;
        }

        let mut curr = self.start.load(Ordering::SeqCst) as *mut FreeNode;
        let end = self.end.load(Ordering::SeqCst) as *mut FreeNode;

        // Pointer is out of bounds of our heap, ignore it
        if ptr < curr as *mut u8 || ptr >= end as *mut u8 {
            return;
        }

        // Find which block in the free list the pointer belongs to and link it back into the free list
        while curr != end {
            let next = unsafe { (*curr).next };

            if curr as *mut u8 <= ptr && ptr < next as *mut u8 {
                // We found that area in the free list, link it back in
                unsafe {
                    (*curr).next = ptr as *mut FreeNode;
                    (*ptr.cast::<FreeNode>()).next = next;
                }
                return;
            }

            curr = unsafe { (*curr).next };
        }
    }
}
