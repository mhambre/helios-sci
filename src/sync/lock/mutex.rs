use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

use super::LockState;
use crate::util::functions::{futex_wait, futex_wake};

/// A RAII guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    _not_send: PhantomData<*mut ()>,
}

// When the guard is dropped, we set the state back to unlocked and wake one waiting thread.
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.state.store(LockState::Unlocked as u32, Ordering::Release);
        futex_wake(&self.mutex.state, LockState::Unlocked as u32);
    }
}

// The guard allows access to the protected data through dereferencing.
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: we hold the lock, so no other thread can access data.
        unsafe { &*self.mutex.data.get() }
    }
}

// The guard allows mutable access to the protected data through dereferencing.
impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: we hold the lock exclusively.
        unsafe { &mut *self.mutex.data.get() }
    }
}

/// A simple mutex implementation for synchronizing access to shared data.
pub struct Mutex<T> {
    /// The data protected by the mutex.
    data: UnsafeCell<T>,
    /// The futex state of the mutex.
    state: AtomicU32,
}

/// Data itself must be `Send` to be safely shared across threads, and the mutex can be `Sync`
/// because it ensures safe concurrent access to the data.
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex with the given data.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            state: AtomicU32::new(LockState::Unlocked as u32),
        }
    }

    /// Locks the mutex, blocking until it is available.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            let prev = self
                .state
                .compare_exchange(
                    LockState::Unlocked as u32,
                    LockState::Locked as u32,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_ok();

            if prev {
                return MutexGuard {
                    mutex: self,
                    _not_send: PhantomData,
                };
            }

            futex_wait(&self.state, LockState::Unlocked as u32);
        }
    }
}
