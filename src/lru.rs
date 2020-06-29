use linked_hash_map::LinkedHashMap;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::{hash::Hash, sync::atomic::Ordering};

/// A simple and approximate concurrent lru list of values of type `T`. This is
/// typically a `DatabaseKeyIndex`.
#[derive(Debug)]
pub(crate) struct Lru<T: Copy + Hash + Eq + Debug> {
    capacity: AtomicUsize,
    data: Mutex<LinkedHashMap<T, ()>>,
}

impl<T: Copy + Hash + Eq + Debug> Lru<T> {
    /// Creates a new LRU list where LRU caching is disabled.
    pub fn new() -> Self {
        Self {
            capacity: AtomicUsize::new(0),
            data: Mutex::new(LinkedHashMap::new()),
        }
    }

    /// Adjust the total number of nodes permitted to have a value at once.  If
    /// `len` is zero, this disables LRU caching completely. This is expected to
    /// be used in the beginning of execution. If you reduce capacity during execution,
    /// evictions will occur gradually as things are used.
    pub fn set_lru_capacity(&self, len: usize) {
        let mut data = self.data.lock();
        self.capacity.store(len, Ordering::SeqCst);
        if len == 0 {
            *data = LinkedHashMap::new();
        }
    }

    /// Records that `node` was used. This may displace an old node (if the LRU limits are
    pub fn record_use(&self, node: T) -> Option<T> {
        log::debug!("record_use(node={:?})", node);

        let capacity = self.capacity.load(Ordering::SeqCst);
        if capacity > 0 {
            let mut data = self.data.lock();
            data.insert(node, ());
            if data.len() > capacity {
                return data.pop_back().map(|v| v.0);
            }
        }

        None
    }
}
