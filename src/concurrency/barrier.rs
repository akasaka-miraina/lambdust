//! Barrier for synchronizing multiple tasks.
//!
//! This module provides barrier synchronization that allows multiple tasks
//! to wait until all tasks reach the barrier point before proceeding.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Notify;

/// Barrier for synchronizing multiple tasks.
#[derive(Debug, Clone)]
pub struct Barrier {
    inner: Arc<BarrierInner>,
}

#[derive(Debug)]
struct BarrierInner {
    count: AtomicUsize,
    total: usize,
    generation: AtomicUsize,
    notify: Notify,
}

impl Barrier {
    /// Creates a new barrier for the given number of tasks.
    pub fn new(n: usize) -> Self {
        Self {
            inner: Arc::new(BarrierInner {
                count: AtomicUsize::new(0),
                total: n,
                generation: AtomicUsize::new(0),
                notify: Notify::new(),
            }),
        }
    }

    /// Waits for all tasks to reach the barrier.
    pub async fn wait(&self) -> BarrierWaitResult {
        let current_generation = self.inner.generation.load(Ordering::SeqCst);
        let count = self.inner.count.fetch_add(1, Ordering::SeqCst) + 1;

        if count == self.inner.total {
            // Last task to arrive - reset and notify others
            self.inner.count.store(0, Ordering::SeqCst);
            self.inner.generation.fetch_add(1, Ordering::SeqCst);
            self.inner.notify.notify_waiters();
            BarrierWaitResult { is_leader: true }
        } else {
            // Wait for the barrier to be released
            loop {
                self.inner.notify.notified().await;
                if self.inner.generation.load(Ordering::SeqCst) != current_generation {
                    break;
                }
            }
            BarrierWaitResult { is_leader: false }
        }
    }
}

/// Result of waiting on a barrier.
pub struct BarrierWaitResult {
    /// True if this task was the last to reach the barrier.
    pub is_leader: bool,
}