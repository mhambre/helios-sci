//! Two-Level Segregate Fit Allocator - Helios Userspace Implementation
//
// Note, account for offset later

use core::alloc::{GlobalAlloc, Layout};
use std::ptr::null_mut;

const FL_OFFSET: usize = 6; // 64 bytes/blocks is the first reasonable block size
const FIRST_LEVEL_SIZE: usize = 1 << 6; // 1 word; 64 bytes/blocks
const SECOND_LEVEL_SHIFT: usize = 4; // Number of bits to represent the second level index; 16 blocks per first level block
const SECOND_LEVEL_SIZE: usize = 1 << SECOND_LEVEL_SHIFT; // 1/4 word, 16 bytes/blocks
const HEAP_CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB heap allocated from the kernel at a time

// A free block in the TLSF allocator. The `size` field includes the size
// of the block itself, and the `next` pointer is used to link free blocks
// together in a free list.
pub struct FreeBlock {
    /// The size of the free block, including the size of the block itself
    size: usize,
    /// The next free block in the free list
    next: *mut FreeBlock,
    /// The previous free block in the free list
    prev: *mut FreeBlock,
    /// The previous physical block in memory
    prev_physical: usize,
}

const MIN_FREE_BLOCK_SIZE: usize = core::mem::size_of::<FreeBlock>();

// Helper to map a size to an expected bitmap index
pub fn get_optimal_free_list_index(size: usize) -> (usize, usize) {
    // If the size is smaller than the first level offset
    if size < (1 << FL_OFFSET) {
        return (0, 0);
    }

    // The first level index is determined by the position of the highest set bit in the size to the next
    // power of two
    let l1_index = (usize::BITS - size.leading_zeros() - 1) as usize;
    // The second level index is determined by the actual value of the next 4 bits after the highest set bit
    let l2_index = (size >> (l1_index - SECOND_LEVEL_SHIFT)) & (SECOND_LEVEL_SIZE - 1);
    (l1_index - FL_OFFSET, l2_index)
}

// The TLSF allocator struct, which maintains the bitmaps and free lists
// for the two-level segregate fit algorithm.
pub struct TlsfAllocator {
    l1_bitmap: usize,
    l2_bitmap: [usize; FIRST_LEVEL_SIZE],
    free_lists: [[*mut FreeBlock; SECOND_LEVEL_SIZE]; FIRST_LEVEL_SIZE],
}

impl TlsfAllocator {
    /// Creates a new TLSF allocator with the given memory region and size.
    pub fn new() -> Self {
        Self {
            l1_bitmap: 0,
            l2_bitmap: [0; FIRST_LEVEL_SIZE],
            free_lists: [[null_mut(); SECOND_LEVEL_SIZE]; FIRST_LEVEL_SIZE],
        }
    }

    // Mapps the first available free block to its bitmap index. This is used to find the next
    // free block to allocate from.
    pub fn get_next_available_free(&self, size: usize) -> (usize, usize) {
        let (optimal_l1_index, optimal_l2_index) = get_optimal_free_list_index(size);
        let l1_index = (self.l1_bitmap & ((1 << optimal_l1_index) - 1)).trailing_zeros() as usize;
        let l2_index = (self.l2_bitmap[l1_index] & ((1 << optimal_l2_index) - 1)).trailing_zeros() as usize;
        (l1_index, l2_index)
    }
}

impl Default for TlsfAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for TlsfAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!()
    }
}
