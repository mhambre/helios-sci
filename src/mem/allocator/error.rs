use std::i32;

/// Error types for possible allocator-related errors.
pub enum _AllocError {
    /// The allocator is not ready to allocate memory.
    NotReady,
    /// Init failed.
    InitFailed,
    /// The requested allocation size exceeds the maximum allowed size.
    SizeOverflow,
    /// The requested allocation size is zero.
    ZeroSize,
    /// The allocator has run out of memory.
    OutOfMemory,
    /// Unknown error type.
    UnknownError,
}

impl From<i32> for _AllocError {
    fn from(value: i32) -> Self {
        match value {
            0 => _AllocError::NotReady,
            1 => _AllocError::InitFailed,
            2 => _AllocError::SizeOverflow,
            3 => _AllocError::ZeroSize,
            4 => _AllocError::OutOfMemory,
            _ => _AllocError::UnknownError,
        }
    }
}
