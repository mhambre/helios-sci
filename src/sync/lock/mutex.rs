/// A simple mutex implementation for synchronizing access to shared data.
pub struct Mutex<T> {
    /// The data protected by the mutex.
    data: T,
    /// The state of the mutex (locked or unlocked).
    state: core::sync::atomic::AtomicBool,
}

/// A RAII guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.state.store(false, core::sync::atomic::Ordering::Release);
    }
}

impl<T> Mutex<T> {
    /// Creates a new mutex with the given data.
    pub fn new(data: T) -> Self {
        Self {
            data,
            state: core::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Locks the mutex, blocking until it is available.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Spin until we can acquire the lock (move to futex next)
        while self
            .state
            .compare_exchange(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed)
            .is_err()
        {}
        MutexGuard { mutex: self }
    }
}
