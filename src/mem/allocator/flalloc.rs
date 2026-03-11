//! Free-List Allocator - Helios Userspace Implementation

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{cmp, mem};

use crate::mem::allocator::shared::{AllocState, AtomicAllocState, DEFAULT_HEAP_SIZE, PAGE_SIZE};
use crate::util::numbers::align_up;

const MAX_ITERATIONS: usize = 1_000_000; // Hard stop to prevent infinite loops in free list traversal

/// Available block of free memory in the allocator's free list.
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

/// Metadata header for allocated blocks, stored immediately before
/// the returned pointer so we always know the size we allocated without
/// relying on `alloc::Layout`.
struct AllocHeader {
    size: usize,
}

const MIN_FREE_BLOCK_SIZE: usize = mem::size_of::<FreeBlock>();
const ALLOC_HEADER_SIZE: usize = mem::size_of::<AllocHeader>();

/// A simple free-list heap allocator
pub struct FLAllocator {
    head: AtomicUsize,
    state: AtomicAllocState,
}

impl FLAllocator {
    /// Create a new allocator.
    pub const fn new() -> Self {
        Self {
            head: AtomicUsize::new(0),
            state: AtomicAllocState::new(AllocState::Uninitialized),
        }
    }

    /// Initialize allocator state exactly once, with retry-on-failure semantics.
    unsafe fn init(&self) -> Result<(), i32> {
        if self.state.load(Ordering::Acquire) == AllocState::Ready {
            return Ok(());
        }

        // OS-Specific heap initialization. On Linux, we use `mmap` to ask the kernel for a large initial heap region.
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux", target_arch = "x86_64")] {
                let heap = crate::util::functions::mmap(0, DEFAULT_HEAP_SIZE, libc::PROT_WRITE | libc::PROT_READ, libc::MAP_PRIVATE | libc::MAP_ANON, -1, 0).unwrap_or(null_mut());
            } else {
                compile_error!("Unsupported Target OS");
            }
        }

        // Allocate a massive initial heap region and add it to the free list.
        let block = heap as *mut FreeBlock;
        unsafe {
            (*block).size = DEFAULT_HEAP_SIZE;
            (*block).next = null_mut();
        }

        self.head.store(block as usize, Ordering::Release);
        self.state.store(AllocState::Ready, Ordering::Release);
        Ok(())
    }

    /// Extend the heap by at least `min_size` bytes and return a new free block for the extended region.
    unsafe fn extend(&self, min_size: usize) -> *mut FreeBlock {
        let size = align_up(cmp::max(min_size, DEFAULT_HEAP_SIZE), PAGE_SIZE);

        // OS-Specific heap extension. On Linux, we use `mmap` to ask the kernel for a large initial heap region.
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux", target_arch = "x86_64")] {
                let ptr = crate::util::functions::mmap(0, size, libc::PROT_WRITE | libc::PROT_READ, libc::MAP_PRIVATE | libc::MAP_ANON, -1, 0).unwrap_or(null_mut());
            } else {
                compile_error!("Unsupported Target OS");
            }
        }

        if ptr.is_null() {
            return null_mut();
        }

        // Prep the new block to go into the free list before returning it, so it can be used immediately
        let block = ptr as *mut FreeBlock;
        unsafe {
            (*block).size = size;
            (*block).next = null_mut();
        }
        block
    }

    /// Insert a free block into the free list.
    unsafe fn insert_free_block(&self, block: *mut FreeBlock) {
        let mut prev: *mut FreeBlock = null_mut();
        let mut curr = self.head.load(Ordering::Relaxed) as *mut FreeBlock;

        // Walk the free list to find the correct insertion point for the new block.
        // This is the first block with a memory address higher than the new block.
        while !curr.is_null() && (curr as usize) < (block as usize) {
            prev = curr;
            curr = unsafe { (*curr).next };
        }

        unsafe {
            (*block).next = curr;

            // Edge case for extending at the head of the list, otherwise link the previous
            // block to the new block. (i.e. first insertion is somehow > 1 MiB, or we extended the heap
            // and got a new block that's higher than the current head)
            if prev.is_null() {
                self.head.store(block as usize, Ordering::Relaxed);
            } else {
                (*prev).next = block;
            }

            // Combine with next block if adjacent in memory.
            if !curr.is_null() && (block as usize + (*block).size == curr as usize) {
                (*block).size += (*curr).size;
                (*block).next = (*curr).next;
            }

            // Combine with previous block if adjacent in memory.
            if !prev.is_null() && (prev as usize + (*prev).size == block as usize) {
                (*prev).size += (*block).size;
                (*prev).next = (*block).next;
            }
        }
    }

    unsafe fn alloc_from_list(&self, layout: Layout) -> *mut u8 {
        let size = cmp::max(layout.size(), 1);
        let align = cmp::max(layout.align(), mem::align_of::<usize>());

        let mut prev: *mut FreeBlock = null_mut();
        let mut curr = self.head.load(Ordering::Relaxed) as *mut FreeBlock;

        // Hard stop prevents infinite loops if the free list is corrupted.
        let mut iterations: usize = 0;
        while !curr.is_null() {
            // We don't want to loop indefinitely if the free list is somehow circularly linked or just extremely long
            // (major fragmentation)
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return null_mut();
            }

            let block_start = curr as usize;
            let block_size = unsafe { (*curr).size };
            let next = unsafe { (*curr).next };

            let mut payload = align_up(block_start + ALLOC_HEADER_SIZE, align);
            let mut header_addr = payload - ALLOC_HEADER_SIZE;
            let mut prefix = header_addr - block_start;

            // Bump the payload forward if the prefix is too small to be a free block,
            // until we have a large enough prefix or no prefix at all. We need to maintain the invariant that any free
            // blocks we create are large enough to hold a `FreeBlock`, otherwise we won't be able to store metadata for
            // them and they will be unusable.
            while prefix > 0 && prefix < MIN_FREE_BLOCK_SIZE {
                // Overflowing a memory address is a hard failure
                payload = match payload.checked_add(align) {
                    Some(v) => v,
                    None => return null_mut(),
                };
                header_addr = payload - ALLOC_HEADER_SIZE;
                prefix = header_addr - block_start;
            }

            // Overflowing a memory address is a hard failure
            let mut needed = match prefix.checked_add(ALLOC_HEADER_SIZE).and_then(|v| v.checked_add(size)) {
                Some(v) => v,
                None => return null_mut(),
            };

            // If the block is too small after adjusting for alignment and minimum free block sizes, skip it.
            if needed > block_size {
                prev = curr;
                curr = next;
                continue;
            }

            // If the remaining free memory after carving out the allocation is too small to be useful,
            // just give the caller the rest
            let mut suffix = block_size - needed;
            if suffix > 0 && suffix < MIN_FREE_BLOCK_SIZE {
                needed += suffix;
                suffix = 0;
            }

            // Free blocks must be aligned for `FreeBlock`. If the computed suffix address is
            // misaligned, absorb suffix into the allocation instead of creating invalid metadata.
            if suffix >= MIN_FREE_BLOCK_SIZE {
                let suffix_addr = block_start + needed;
                if !suffix_addr.is_multiple_of(mem::align_of::<FreeBlock>()) {
                    needed += suffix;
                    suffix = 0;
                }
            }

            unsafe {
                if prefix >= MIN_FREE_BLOCK_SIZE {
                    // We can carve out a prefix block to return to the free list
                    let prefix_block = curr;
                    (*prefix_block).size = prefix;

                    // We can also carve out a suffix block to return to the free list, so link the prefix block to the
                    // suffix block
                    if suffix >= MIN_FREE_BLOCK_SIZE {
                        let suffix_block = (block_start + needed) as *mut FreeBlock;
                        (*suffix_block).size = suffix;
                        (*suffix_block).next = next;
                        (*prefix_block).next = suffix_block;
                    } else {
                        (*prefix_block).next = next;
                    }

                    // Edge case for allocating at the head of the list,
                    // otherwise link the previous block to the prefix block.
                    if prev.is_null() {
                        self.head.store(prefix_block as usize, Ordering::Relaxed);
                    } else {
                        (*prev).next = prefix_block;
                    }
                } else if suffix >= MIN_FREE_BLOCK_SIZE {
                    // We can carve out at most a suffix block to return to the free list,
                    // so link the previous block to the suffix block
                    let suffix_block = (block_start + needed) as *mut FreeBlock;
                    (*suffix_block).size = suffix;
                    (*suffix_block).next = next;

                    if prev.is_null() {
                        self.head.store(suffix_block as usize, Ordering::Relaxed);
                    } else {
                        (*prev).next = suffix_block;
                    }
                } else if prev.is_null() {
                    self.head.store(next as usize, Ordering::Relaxed);
                } else {
                    (*prev).next = next;
                }

                // Write the allocation header just before the returned pointer so we know how big this
                // allocation is when it's deallocated by just subtracting the header size.
                let alloc_size = needed - prefix;
                let header = header_addr as *mut AllocHeader;
                (*header).size = alloc_size;
                return payload as *mut u8;
            }
        }

        null_mut()
    }
}

impl Default for FLAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for FLAllocator {}

/// Implement the `GlobalAlloc` trait for `Allocator` to enable heap allocation
/// for an entire program.
///
/// To register as a program's global allocator, add the following to the program's main file:
/// ```rust
/// use helios_sci::mem::allocator::FLAllocator;
///
/// #[global_allocator]
/// static ALLOCATOR: FLAllocator = FLAllocator::new();
/// ```
unsafe impl GlobalAlloc for FLAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if unsafe { self.init() }.is_err() {
            return null_mut();
        }
        let mut ptr = unsafe { self.alloc_from_list(layout) };

        // We couldn't find a suitable block in the free list, so we need to extend the heap and try again.
        if ptr.is_null() {
            // Get the minimum size we need to extend by, which is the requested size plus the allocation header
            let min = match layout
                .size()
                .checked_add(ALLOC_HEADER_SIZE)
                .and_then(|v| v.checked_add(MIN_FREE_BLOCK_SIZE))
            {
                Some(v) => cmp::max(v, DEFAULT_HEAP_SIZE),
                None => {
                    return null_mut();
                },
            };

            // Extend the heap and insert the new block into the free list, then try to allocate again.
            let block = unsafe { self.extend(min) };
            if !block.is_null() {
                unsafe { self.insert_free_block(block) };
                ptr = unsafe { self.alloc_from_list(layout) };
            }
        }

        ptr
    }

    /// Deallocate a block of memory previously allocated by this allocator.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        if unsafe { self.init() }.is_err() {
            return;
        }

        unsafe {
            let header = ptr.sub(ALLOC_HEADER_SIZE) as *mut AllocHeader;
            let size = (*header).size;

            // Only insert the block back into the free list if it's large enough to hold a `FreeBlock`, otherwise we
            // just accept it as a fragment that can't be reused for now.
            if size >= MIN_FREE_BLOCK_SIZE {
                let block = header as *mut FreeBlock;
                (*block).size = size;
                (*block).next = null_mut();
                self.insert_free_block(block);
            }
        }
    }
}

#[cfg(all(test, target_arch = "x86_64", target_os = "linux"))]
mod tests {
    use alloc::alloc::{GlobalAlloc, Layout};

    use super::FLAllocator;

    #[test]
    fn alloc_respects_alignment() {
        let allocator = FLAllocator::new();
        let layout = Layout::from_size_align(32, 64).expect("valid layout");

        // SAFETY: Layout is valid and dealloc uses the same layout.
        let ptr = unsafe { allocator.alloc(layout) };
        assert!(!ptr.is_null());
        assert_eq!((ptr as usize) % 64, 0);

        // SAFETY: Pointer came from allocator.alloc with matching layout.
        unsafe { allocator.dealloc(ptr, layout) };
    }

    #[test]
    fn alloc_dealloc_reuses_memory() {
        let allocator = FLAllocator::new();
        let layout = Layout::from_size_align(64, 8).expect("valid layout");

        // SAFETY: Layout is valid and dealloc uses the same layout.
        let p1 = unsafe { allocator.alloc(layout) };
        assert!(!p1.is_null());

        // SAFETY: Pointer came from allocator.alloc with matching layout.
        unsafe { allocator.dealloc(p1, layout) };

        // SAFETY: Layout is valid.
        let p2 = unsafe { allocator.alloc(layout) };
        assert!(!p2.is_null());
        assert_eq!(p1, p2);

        // SAFETY: Pointer came from allocator.alloc with matching layout.
        unsafe { allocator.dealloc(p2, layout) };
    }
}
