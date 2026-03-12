//! Synchronization primitives for Helios in userspace.
mod mutex;

pub use mutex::{Mutex, MutexGuard};

#[repr(u32)]
pub(self) enum LockState {
    Unlocked = 0,
    Locked = 1,
}
