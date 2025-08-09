//! Comprehensive synchronization primitives for concurrent programming.
//!
//! This module provides thread-safe synchronization primitives including
//! mutexes, semaphores, condition variables, barriers, and lock-free data structures.

use crate::eval::Value;
use crate::diagnostics::{Error, Result, error::helpers};
use super::ConcurrencyError;
use std::sync::{Arc, Mutex as StdMutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicI64, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock, Semaphore, Notify};
use crossbeam::queue::{ArrayQueue, SegQueue};
use crossbeam::epoch::{self, Atomic, Owned, Shared};
use std::collections::HashMap;

/// Mutual exclusion lock for protecting shared data.
#[derive(Debug, Clone)]
pub struct Mutex {
    inner: Arc<AsyncMutex<Value>>,
    name: Option<String>,
}

impl Mutex {
    /// Creates a new mutex with an initial value.
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(value)),
            name: None,
        }
    }

    /// Creates a new named mutex.
    pub fn with_name(value: Value, name: String) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(value)),
            name: Some(name),
        }
    }

    /// Locks the mutex and returns a guard.
    pub async fn lock(&self) -> MutexGuard<'_> {
        let guard = self.inner.lock().await;
        MutexGuard { guard }
    }

    /// Attempts to lock the mutex without blocking.
    pub fn try_lock(&self) -> Result<MutexGuard<'_>> {
        match self.inner.try_lock() {
            Ok(guard) => Ok(MutexGuard { guard }),
            Err(_) => Err(Box::new(Error::runtime_error("Mutex is locked".to_string(), None))),
        }
    }

    /// Locks the mutex with a timeout.
    pub async fn lock_timeout(&self, duration: Duration) -> Result<MutexGuard<'_>> {
        match tokio::time::timeout(duration, self.lock()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }

    /// Gets the name of the mutex.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// RAII guard for mutex locks.
pub struct MutexGuard<'a> {
    guard: tokio::sync::MutexGuard<'a, Value>,
}

impl<'a> MutexGuard<'a> {
    /// Gets a reference to the protected value.
    pub fn get(&self) -> &Value {
        &self.guard
    }

    /// Gets a mutable reference to the protected value.
    pub fn get_mut(&mut self) -> &mut Value {
        &mut self.guard
    }

    /// Sets the protected value.
    pub fn set(&mut self, value: Value) {
        *self.guard = value;
    }
}

/// Reader-writer lock for shared data with concurrent reads.
#[derive(Debug, Clone)]
pub struct RwLock {
    inner: Arc<AsyncRwLock<Value>>,
    name: Option<String>,
}

impl RwLock {
    /// Creates a new RwLock with an initial value.
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(AsyncRwLock::new(value)),
            name: None,
        }
    }

    /// Creates a new named RwLock.
    pub fn with_name(value: Value, name: String) -> Self {
        Self {
            inner: Arc::new(AsyncRwLock::new(value)),
            name: Some(name),
        }
    }

    /// Acquires a read lock.
    pub async fn read(&self) -> ReadGuard<'_> {
        let guard = self.inner.read().await;
        ReadGuard { guard }
    }

    /// Acquires a write lock.
    pub async fn write(&self) -> WriteGuard<'_> {
        let guard = self.inner.write().await;
        WriteGuard { guard }
    }

    /// Attempts to acquire a read lock without blocking.
    pub fn try_read(&self) -> Result<ReadGuard<'_>> {
        match self.inner.try_read() {
            Ok(guard) => Ok(ReadGuard { guard }),
            Err(_) => Err(Box::new(Error::runtime_error("RwLock is write-locked".to_string(), None))),
        }
    }

    /// Attempts to acquire a write lock without blocking.
    pub fn try_write(&self) -> Result<WriteGuard<'_>> {
        match self.inner.try_write() {
            Ok(guard) => Ok(WriteGuard { guard }),
            Err(_) => Err(Box::new(Error::runtime_error("RwLock is locked".to_string(), None))),
        }
    }

    /// Gets the name of the RwLock.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// RAII guard for read locks.
pub struct ReadGuard<'a> {
    guard: tokio::sync::RwLockReadGuard<'a, Value>,
}

impl<'a> ReadGuard<'a> {
    /// Gets a reference to the protected value.
    pub fn get(&self) -> &Value {
        &self.guard
    }
}

/// RAII guard for write locks.
pub struct WriteGuard<'a> {
    guard: tokio::sync::RwLockWriteGuard<'a, Value>,
}

impl<'a> WriteGuard<'a> {
    /// Gets a reference to the protected value.
    pub fn get(&self) -> &Value {
        &self.guard
    }

    /// Gets a mutable reference to the protected value.
    pub fn get_mut(&mut self) -> &mut Value {
        &mut self.guard
    }

    /// Sets the protected value.
    pub fn set(&mut self, value: Value) {
        *self.guard = value;
    }
}

/// Semaphore for controlling access to a limited resource.
#[derive(Debug, Clone)]
pub struct SemaphoreSync {
    inner: Arc<Semaphore>,
    name: Option<String>,
}

impl SemaphoreSync {
    /// Creates a new semaphore with the given number of permits.
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(Semaphore::new(permits)),
            name: None,
        }
    }

    /// Creates a new named semaphore.
    pub fn with_name(permits: usize, name: String) -> Self {
        Self {
            inner: Arc::new(Semaphore::new(permits)),
            name: Some(name),
        }
    }

    /// Acquires a permit from the semaphore.
    pub async fn acquire(&self) -> Result<SemaphorePermit<'_>> {
        let permit = self.inner.acquire().await
            .map_err(|_| ConcurrencyError::ChannelClosed)?;
        Ok(SemaphorePermit { permit })
    }

    /// Attempts to acquire a permit without blocking.
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>> {
        let permit = self.inner.try_acquire()
            .map_err(|_| Error::runtime_error("No permits available".to_string(), None))?;
        Ok(SemaphorePermit { permit })
    }

    /// Acquires multiple permits.
    pub async fn acquire_many(&self, permits: u32) -> Result<SemaphorePermit<'_>> {
        let permit = self.inner.acquire_many(permits).await
            .map_err(|_| ConcurrencyError::ChannelClosed)?;
        Ok(SemaphorePermit { permit })
    }

    /// Gets the current number of available permits.
    pub fn available_permits(&self) -> usize {
        self.inner.available_permits()
    }

    /// Adds permits to the semaphore.
    pub fn add_permits(&self, n: usize) {
        self.inner.add_permits(n);
    }

    /// Gets the name of the semaphore.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// RAII guard for semaphore permits.
pub struct SemaphorePermit<'a> {
    permit: tokio::sync::SemaphorePermit<'a>,
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {
        // Permit is automatically returned when dropped
    }
}

/// Condition variable for thread coordination.
#[derive(Debug, Clone)]
pub struct CondVar {
    notify: Arc<Notify>,
    name: Option<String>,
}

impl CondVar {
    /// Creates a new condition variable.
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            name: None,
        }
    }

    /// Creates a new named condition variable.
    pub fn with_name(name: String) -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            name: Some(name),
        }
    }

    /// Waits for a notification.
    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    /// Waits for a notification with a timeout.
    pub async fn wait_timeout(&self, duration: Duration) -> Result<()> {
        match tokio::time::timeout(duration, self.wait()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }

    /// Notifies one waiting task.
    pub fn notify_one(&self) {
        self.notify.notify_one();
    }

    /// Notifies all waiting tasks.
    pub fn notify_all(&self) {
        self.notify.notify_waiters();
    }

    /// Gets the name of the condition variable.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Default for CondVar {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Synchronization primitives registry for managing named primitives.
#[derive(Debug)]
pub struct SyncRegistry {
    mutexes: StdMutex<HashMap<String, Mutex>>,
    rwlocks: StdMutex<HashMap<String, RwLock>>,
    semaphores: StdMutex<HashMap<String, SemaphoreSync>>,
    condvars: StdMutex<HashMap<String, CondVar>>,
    counters: StdMutex<HashMap<String, AtomicCounter>>,
    flags: StdMutex<HashMap<String, AtomicFlag>>,
}

impl SyncRegistry {
    /// Creates a new synchronization registry.
    pub fn new() -> Self {
        Self {
            mutexes: StdMutex::new(HashMap::new()),
            rwlocks: StdMutex::new(HashMap::new()),
            semaphores: StdMutex::new(HashMap::new()),
            condvars: StdMutex::new(HashMap::new()),
            counters: StdMutex::new(HashMap::new()),
            flags: StdMutex::new(HashMap::new()),
        }
    }

    /// Registers a named mutex.
    pub fn register_mutex(&self, name: String, mutex: Mutex) -> Result<()> {
        let mut mutexes = self.mutexes.lock()
            .map_err(|_| Error::runtime_error("Failed to lock mutex registry".to_string(), None))?;
        mutexes.insert(name, mutex);
        Ok(())
    }

    /// Gets a named mutex.
    pub fn get_mutex(&self, name: &str) -> Result<Mutex> {
        let mutexes = self.mutexes.lock()
            .map_err(|_| helpers::runtime_error_simple("Failed to lock mutex registry"))?;
        mutexes.get(name)
            .cloned()
            .ok_or_else(|| helpers::runtime_error_simple(format!("Mutex '{name}' not found")))
    }

    // Similar methods for other primitives...
    // (Implementation would be similar for RwLock, Semaphore, etc.)
}

impl Default for SyncRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global synchronization primitives registry.
static SYNC_REGISTRY: std::sync::OnceLock<Arc<SyncRegistry>> = std::sync::OnceLock::new();

/// Gets the global synchronization registry.
pub fn global_sync_registry() -> Arc<SyncRegistry> {
    SYNC_REGISTRY.get_or_init(|| Arc::new(SyncRegistry::new())).clone()
}