//! Lock-free queues for high-performance message passing.
//!
//! This module provides both unbounded and bounded lock-free queue
//! implementations using crossbeam's efficient queue structures.

use std::sync::Arc;
use crossbeam::queue::{ArrayQueue, SegQueue};

/// Lock-free queue for high-performance message passing.
#[derive(Debug, Clone)]
pub struct LockFreeQueue<T> {
    inner: Arc<SegQueue<T>>,
}

impl<T> LockFreeQueue<T> {
    /// Creates a new lock-free queue.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SegQueue::new()),
        }
    }

    /// Pushes a value to the queue.
    pub fn push(&self, value: T) {
        self.inner.push(value);
    }

    /// Pops a value from the queue.
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets the approximate length of the queue.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T> Default for LockFreeQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock-free bounded queue with fixed capacity.
#[derive(Debug, Clone)]
pub struct BoundedLockFreeQueue<T> {
    inner: Arc<ArrayQueue<T>>,
}

impl<T> BoundedLockFreeQueue<T> {
    /// Creates a new bounded lock-free queue.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    /// Pushes a value to the queue.
    pub fn push(&self, value: T) -> std::result::Result<(), T> {
        self.inner.push(value)
    }

    /// Pops a value from the queue.
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Checks if the queue is full.
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Gets the current length of the queue.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Gets the capacity of the queue.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}