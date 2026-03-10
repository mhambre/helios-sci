struct AtomicMutex {
    owner: AtomicUsize,
    depth: AtomicUsize,
}
