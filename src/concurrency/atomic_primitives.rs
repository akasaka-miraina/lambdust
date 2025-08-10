//! Atomic primitives for high-performance concurrent programming.
//!
//! This module provides atomic counter and flag operations with
//! optional naming for debugging and monitoring purposes.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

/// Atomic counter for high-performance counting.
#[derive(Debug, Clone)]
pub struct AtomicCounter {
    inner: Arc<AtomicI64>,
    name: Option<String>,
}

impl AtomicCounter {
    /// Creates a new atomic counter.
    pub fn new(initial: i64) -> Self {
        Self {
            inner: Arc::new(AtomicI64::new(initial)),
            name: None,
        }
    }

    /// Creates a new named atomic counter.
    pub fn with_name(initial: i64, name: String) -> Self {
        Self {
            inner: Arc::new(AtomicI64::new(initial)),
            name: Some(name),
        }
    }

    /// Gets the current value.
    pub fn get(&self) -> i64 {
        self.inner.load(Ordering::SeqCst)
    }

    /// Sets the value.
    pub fn set(&self, value: i64) {
        self.inner.store(value, Ordering::SeqCst);
    }

    /// Increments the counter and returns the new value.
    pub fn increment(&self) -> i64 {
        self.inner.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Decrements the counter and returns the new value.
    pub fn decrement(&self) -> i64 {
        self.inner.fetch_sub(1, Ordering::SeqCst) - 1
    }

    /// Adds a value to the counter and returns the new value.
    pub fn add(&self, value: i64) -> i64 {
        self.inner.fetch_add(value, Ordering::SeqCst) + value
    }

    /// Subtracts a value from the counter and returns the new value.
    pub fn sub(&self, value: i64) -> i64 {
        self.inner.fetch_sub(value, Ordering::SeqCst) - value
    }

    /// Compares and swaps the value.
    pub fn compare_and_swap(&self, current: i64, new: i64) -> i64 {
        self.inner.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|x| x)
    }

    /// Gets the name of the counter.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Atomic flag for simple boolean synchronization.
#[derive(Debug, Clone)]
pub struct AtomicFlag {
    inner: Arc<AtomicBool>,
    name: Option<String>,
}

impl AtomicFlag {
    /// Creates a new atomic flag.
    pub fn new(initial: bool) -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(initial)),
            name: None,
        }
    }

    /// Creates a new named atomic flag.
    pub fn with_name(initial: bool, name: String) -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(initial)),
            name: Some(name),
        }
    }

    /// Gets the current value.
    pub fn get(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }

    /// Sets the value.
    pub fn set(&self, value: bool) {
        self.inner.store(value, Ordering::SeqCst);
    }

    /// Sets the flag to true and returns the previous value.
    pub fn set_true(&self) -> bool {
        self.inner.swap(true, Ordering::SeqCst)
    }

    /// Sets the flag to false and returns the previous value.
    pub fn set_false(&self) -> bool {
        self.inner.swap(false, Ordering::SeqCst)
    }

    /// Compares and swaps the value.
    pub fn compare_and_swap(&self, current: bool, new: bool) -> bool {
        self.inner.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|x| x)
    }

    /// Gets the name of the flag.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}