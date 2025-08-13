//! Atomic reference for lock-free programming.
//!
//! This module provides lock-free atomic reference functionality using
//! crossbeam's epoch-based memory reclamation for safe memory management.

use crossbeam::epoch::{self, Atomic, Owned, Shared};
use std::sync::atomic::Ordering;

/// Atomic reference for lock-free programming.
#[derive(Debug)]
pub struct AtomicRef<T> {
    inner: Atomic<T>,
}

impl<T> AtomicRef<T> {
    /// Creates a new atomic reference.
    pub fn new(value: T) -> Self {
        Self {
            inner: Atomic::new(value),
        }
    }

    /// Loads the current value.
    pub fn load<'g>(&self, guard: &'g epoch::Guard) -> Shared<'g, T> {
        self.inner.load(Ordering::SeqCst, guard)
    }

    /// Stores a new value.
    pub fn store(&self, value: T) {
        let guard = &epoch::pin();
        let new = Owned::new(value);
        let old = self.inner.swap(new, Ordering::SeqCst, guard);
        unsafe {
            guard.defer_destroy(old);
        }
    }

    /// Compares and swaps the value.
    pub fn compare_and_swap(&self, current: Shared<'_, T>, new: T) -> std::result::Result<T, T>
    where
        T: Clone,
    {
        let guard = epoch::pin();
        let new_owned = Owned::new(new.clone());
        match self.inner.compare_exchange(current, new_owned, Ordering::SeqCst, Ordering::SeqCst, &guard) {
            Ok(_) => unsafe { Ok((*current.as_raw()).clone()) },
            Err(_) => Err(new),
        }
    }
}