use atomic_enum::atomic_enum;

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
