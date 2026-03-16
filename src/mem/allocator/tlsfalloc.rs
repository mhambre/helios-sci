//! Two-Level Segregate Fit Allocator - Helios Userspace Implementation
//
// Note, account for offset later

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::Ordering;

use crate::mem::allocator::shared::{AtomicAllocState, DEFAULT_HEAP_SIZE, PAGE_SIZE, request_heap_chunk};
use crate::sync::lock::{Mutex, MutexGuard};
use crate::util::numbers::align_up;

const FL_OFFSET: usize = 6; // 64 bytes/blocks is the first reasonable block size
const FIRST_LEVEL_SIZE: usize = 1 << 6; // 1 word; 64 bytes/blocks
const SECOND_LEVEL_SHIFT: usize = 4; // Number of bits to represent the second level index; 16 blocks per first level block
const SECOND_LEVEL_SIZE: usize = 1 << SECOND_LEVEL_SHIFT; // 1/4 word, 16 bytes/blocks

type InnerLock<'a> = MutexGuard<'a, TlsfAllocatorInner>;

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

/// Metadata header for allocated blocks so we can track the size and previous physical block for coalescing on free
pub struct AllocHeader {
    /// The size of the allocated block, including the size of the block itself
    size: usize,
    /// The previous physical block in memory
    prev_physical: usize,
}

// Minimum block sizes to ensure we can fit the necessary metadata to put the block back in the free lists on free
const ALLOC_HEADER_SIZE: usize = core::mem::size_of::<AllocHeader>();

// We need to ensure that the minimum free block size can fit the free block metadata, and we never have blocks smaller than
// the first level offset to keep mappings valid.
// Note: If `FreeBlock` ever grows larger than `MIN_FREE_BLOCK_SIZE`, we will need to make sure we can still fit the metadata
// in the free block (we 32 bytes of slack though)
const MIN_FREE_BLOCK_SIZE: usize = 1 << FL_OFFSET;

// Helper to map a size to an expected bitmap index
fn get_optimal_free_list_index(size: usize) -> (usize, usize) {
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

// Helper function to get the next available power of two block size for a given size
// in a bitmap
fn next_power_of_two_block_size(index: usize, bitmap: usize) -> usize {
    (!((1 << index) - 1) & bitmap).trailing_zeros() as usize
}

// The TLSF allocator struct, which maintains the bitmaps and free lists
// for the two-level segregate fit algorithm.
pub struct TlsfAllocator {
    // Above will go in a Mutex
    state: AtomicAllocState,
    // Inner data structure stores free blocks and is thus protected by a mutex to allow for safe concurrent access
    inner: Mutex<TlsfAllocatorInner>,
}

// Inner TLSF allocator data structure
pub struct TlsfAllocatorInner {
    l1_bitmap: usize,
    l2_bitmap: [usize; FIRST_LEVEL_SIZE],
    free_lists: [[*mut FreeBlock; SECOND_LEVEL_SIZE]; FIRST_LEVEL_SIZE],
}

// The allocator itself can be safely shared across threads, as the internal mutex ensures
// safe concurrent access to the free lists and bitmaps.
unsafe impl Send for TlsfAllocator {}
unsafe impl Sync for TlsfAllocator {}

impl TlsfAllocator {
    /// Creates a new TLSF allocator with the given memory region and size.
    pub const fn new() -> Self {
        // This ensures we can always insert an AllocHeader where a FreeBlock started
        // which'll also ensure the reverse holds for coalescing on free
        assert!(
            core::mem::align_of::<FreeBlock>() >= core::mem::align_of::<AllocHeader>(),
            "AllocHeader alignment must be less than or equal to FreeBlock alignment"
        );

        Self {
            inner: Mutex::new(TlsfAllocatorInner {
                l1_bitmap: 0,
                l2_bitmap: [0; FIRST_LEVEL_SIZE],
                free_lists: [[null_mut(); SECOND_LEVEL_SIZE]; FIRST_LEVEL_SIZE],
            }),
            state: AtomicAllocState::new(super::shared::AllocState::Uninitialized),
        }
    }

    /// Initializes the allocator with a start free block
    pub fn init(&self) {
        if self.state.load(Ordering::Acquire) == super::shared::AllocState::Ready {
            return;
        }

        if self.add_pool(DEFAULT_HEAP_SIZE).is_null() {
            panic!("Failed to initialize TLSF allocator: unable to allocate initial pool");
        }

        self.state.store(super::shared::AllocState::Ready, Ordering::Release);
    }

    /// Requests a new contiguous block of memory from the kernel and adds it to the free lists
    fn add_pool(&self, size: usize) -> *mut FreeBlock {
        let mut inner_lock = self.inner.lock();
        let sentinel_size = 2 * MIN_FREE_BLOCK_SIZE;
        let request_size: usize = align_up(core::cmp::max(size, DEFAULT_HEAP_SIZE), PAGE_SIZE) + sentinel_size;
        let block = unsafe { request_heap_chunk(Some(request_size)) as *mut FreeBlock };

        // Likely an OOM error if we can't get a new pool
        if block.is_null() {
            return null_mut();
        }

        let pool = block as usize + MIN_FREE_BLOCK_SIZE;
        let end = block as usize + request_size - MIN_FREE_BLOCK_SIZE;

        unsafe {
            // Our actual free block of memory
            let pool_block = pool as *mut FreeBlock;
            (*pool_block).size = request_size - sentinel_size;
            (*pool_block).next = null_mut();
            (*pool_block).prev = null_mut();
            (*pool_block).prev_physical = block as usize;

            // Front sentinel block
            (*block).size = sentinel_size;
            (*block).next = null_mut();
            (*block).prev = null_mut();
            (*block).prev_physical = 0;

            // Rear sentinel block
            let end_block = end as *mut FreeBlock;
            (*end_block).size = sentinel_size;
            (*end_block).next = null_mut();
            (*end_block).prev = null_mut();
            (*end_block).prev_physical = pool_block as usize;

            // Add the new pool block to the free lists
            // Sentinels are ignored to
            let (l1_index, l2_index) = get_optimal_free_list_index((*pool_block).size);
            inner_lock.l1_bitmap |= 1 << l1_index;
            inner_lock.l2_bitmap[l1_index] |= 1 << l2_index;
            inner_lock.free_lists[l1_index][l2_index] = pool_block;

            pool_block
        }
    }

    // Mapps the first available free block to its bitmap index. This is used to find the next
    // free block to allocate from.
    fn get_next_available_free(&self, inner_lock: &mut InnerLock, size: usize) -> (usize, usize) {
        let (optimal_l1_index, optimal_l2_index) = get_optimal_free_list_index(size);
        let l1_index = next_power_of_two_block_size(optimal_l1_index, inner_lock.l1_bitmap);
        let l2_index = if l1_index == optimal_l1_index {
            // Next available same l2 bitmap, so we need to find the next greater available block in the bitmap
            next_power_of_two_block_size(optimal_l2_index, inner_lock.l2_bitmap[l1_index])
        } else {
            // Has to be a greater block, so we take the first available block in the bitmap
            inner_lock.l2_bitmap[l1_index].trailing_zeros() as usize
        };

        (l1_index, l2_index)
    }

    /// Splits a free block into an allocated block, and the remaining (optional) into a smaller free block,
    /// which is added back to the free lists. Returns a pointer to the allocated block's usable memory
    /// (after the AllocHeader)
    fn split_free_block(&self, inner_lock: &mut InnerLock, block: *mut FreeBlock, needed: usize) -> *mut u8 {
        let (original_size, original_prev_physical) = unsafe { ((*block).size, (*block).prev_physical) };
        let block_start = block as usize;
        let free_block_start = block_start + needed;
        let alloc_block_ptr = block_start as *mut AllocHeader;

        unsafe {
            if (block_start + original_size).saturating_sub(free_block_start) <= MIN_FREE_BLOCK_SIZE {
                // Consume the entire block, so we just allocate it without splitting
                (*alloc_block_ptr).size = original_size;
                (*alloc_block_ptr).prev_physical = original_prev_physical;
            } else {
                // Split the block, and insert the free block back into the free lists
                let free_block_ptr = free_block_start as *mut FreeBlock;

                // Make the allocated block look like a valid allocated block by writing the AllocHeader
                (*alloc_block_ptr).size = needed;
                (*alloc_block_ptr).prev_physical = original_prev_physical;

                // Make the free block look like a valid free block
                (*free_block_ptr).size = original_size - needed;
                (*free_block_ptr).prev_physical = block_start;
                (*free_block_ptr).next = null_mut();
                (*free_block_ptr).prev = null_mut();

                // Insert the new free block into the free lists
                let (l1_index, l2_index) = get_optimal_free_list_index((*free_block_ptr).size);
                inner_lock.l1_bitmap |= 1 << l1_index;
                inner_lock.l2_bitmap[l1_index] |= 1 << l2_index;
                (*free_block_ptr).next = inner_lock.free_lists[l1_index][l2_index];
                if !inner_lock.free_lists[l1_index][l2_index].is_null() {
                    (*inner_lock.free_lists[l1_index][l2_index]).prev = free_block_ptr;
                }
                inner_lock.free_lists[l1_index][l2_index] = free_block_ptr;
            }
        }

        (alloc_block_ptr as usize + ALLOC_HEADER_SIZE) as *mut u8
    }

    /// Gets or allocates a free block of at least the given size and alignment, removing it from the free
    /// lists and updating the bitmaps
    fn get_free_block(&self, inner_lock: &mut InnerLock, size: usize) -> *mut FreeBlock {
        let (mut l1_index, mut l2_index) = get_optimal_free_list_index(size);
        let mut block = inner_lock.free_lists[l1_index][l2_index];

        // No free block available, we should find the next available free block in the bitmaps
        if block.is_null() {
            (l1_index, l2_index) = self.get_next_available_free(inner_lock, size);

            // We definitely can't go backwards, so 0 means no available block
            // https://doc.rust-lang.org/std/primitive.u32.html#method.trailing_zeros
            if l1_index == 0 as usize {
                // Add pool and get the next available free block again (it)
                let pool = self.add_pool(size);
                (l1_index, l2_index) = if unsafe { (*pool).size == size } {
                    get_optimal_free_list_index(size)
                } else {
                    self.get_next_available_free(inner_lock, size)
                };

                // If we still don't have a block, then we are out of memory
                if l1_index == 0 as usize {
                    return null_mut();
                }
            }

            block = inner_lock.free_lists[l1_index][l2_index];
        }

        // We got a free block, so we need to remove it from the free list and update the bitmaps
        let next = unsafe { (*block).next };
        inner_lock.free_lists[l1_index][l2_index] = next;

        // If the free list is now empty, update the bitmaps
        if next.is_null() {
            inner_lock.l2_bitmap[l1_index] &= !(1 << l2_index);
            if inner_lock.l2_bitmap[l1_index] == 0 {
                inner_lock.l1_bitmap &= !(1 << l1_index);
            }
        }

        block
    }

    /// Removes a free block from the free lists and updates the bitmaps accordingly
    /// will be used for allocation
    fn pop_free_block(&self, size: usize, align: usize) -> *mut u8 {
        let mut inner_lock = self.inner.lock();
        let required_size = align_up(core::cmp::max(ALLOC_HEADER_SIZE + size, MIN_FREE_BLOCK_SIZE), align);

        // Get a valid free block of at least the required size
        let block = self.get_free_block(&mut inner_lock, required_size);
        if block.is_null() {
            return null_mut();
        }

        // Split the free block if it's larger than the required size, add the remaining free block back to the
        // free list, make this block look like an allocated block by writing the AllocHeader, and return a pointer
        // to the usable memory
        self.split_free_block(&mut inner_lock, block, required_size)
    }
}

impl Default for TlsfAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for TlsfAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.init();
        let ptr = self.pop_free_block(layout.size(), layout.align());
        if ptr.is_null() { null_mut() } else { ptr as *mut u8 }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        self.init();
    }
}

mod tests {
    #[test]
    fn test_get_optimal_free_list_index() {
        assert_eq!(get_optimal_free_list_index(0), (0, 0));
        assert_eq!(get_optimal_free_list_index(1), (0, 0));
        assert_eq!(get_optimal_free_list_index(63), (0, 0));
        assert_eq!(get_optimal_free_list_index(64), (0, 0));
        assert_eq!(get_optimal_free_list_index(80), (0, 4));
        assert_eq!(get_optimal_free_list_index(127), (0, 15));
        assert_eq!(get_optimal_free_list_index(128), (1, 0));
        assert_eq!(get_optimal_free_list_index(32768), (9, 0));
        assert_eq!(get_optimal_free_list_index(65536), (10, 0));
        assert_eq!(get_optimal_free_list_index(460), (2, 12));
        assert_eq!(get_optimal_free_list_index(480), (2, 14));
        assert_eq!(get_optimal_free_list_index(72), (0, 2));
        assert_eq!(get_optimal_free_list_index(96), (0, 8));
        assert_eq!(get_optimal_free_list_index(200), (1, 9));
        assert_eq!(get_optimal_free_list_index(1000), (3, 15));
    }
}
