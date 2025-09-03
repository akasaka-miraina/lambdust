//! SRFI-21 Real-time Multithreading Support
//!
//! This module implements SRFI-21 as a wrapper over Lambdust's existing
//! concurrency infrastructure, providing R7RS-compliant threading primitives
//! with real-time scheduling capabilities.

use crate::diagnostics::{Error, Result};
use crate::eval::Value;
use crate::utils::SymbolId;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::sync::{Condvar as StdCondVar, Mutex as StdMutex};
use std::thread::{self, JoinHandle, ThreadId as StdThreadId};
use std::time::{Duration, Instant};

/// Unique identifier for SRFI-21 threads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(pub u64);

impl ThreadId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        ThreadId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Thread priority with inheritance support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreadPriority {
    /// Base priority assigned to thread (0-255)
    pub base_priority: u8,
    /// Boosted priority from priority inheritance
    pub boosted_priority: u8,
    /// Effective priority (max of base and boosted)
    pub effective_priority: u8,
    /// Whether priority inheritance is currently active
    pub boosted_flag: bool,
}

impl ThreadPriority {
    pub fn new(base_priority: u8) -> Self {
        Self {
            base_priority,
            boosted_priority: 0,
            effective_priority: base_priority,
            boosted_flag: false,
        }
    }

    pub fn inherit_priority(&mut self, inherited: u8) {
        if inherited > self.base_priority {
            self.boosted_priority = inherited;
            self.effective_priority = inherited;
            self.boosted_flag = true;
        }
    }

    pub fn release_inheritance(&mut self) {
        self.boosted_priority = 0;
        self.effective_priority = self.base_priority;
        self.boosted_flag = false;
    }
}

impl PartialOrd for ThreadPriority {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for ThreadPriority {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.effective_priority.cmp(&other.effective_priority)
    }
}

/// Thread execution model - hybrid approach
#[derive(Debug)]
pub enum ThreadExecutionModel {
    /// Native OS thread for compute-intensive, real-time threads
    NativeThread(Option<JoinHandle<Result<Value>>>),
    /// Async task for I/O-bound threads
    #[cfg(feature = "async-runtime")]
    AsyncTask(tokio::task::JoinHandle<Result<Value>>),
    /// Pinned async task for priority threads on dedicated cores
    #[cfg(feature = "async-runtime")]
    PinnedAsync(tokio::task::JoinHandle<Result<Value>>),
}

/// Thread state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    /// Thread created but not started
    New,
    /// Thread is ready to run or running
    Runnable,
    /// Thread is blocked on synchronization
    Blocked,
    /// Thread has terminated normally
    Terminated,
    /// Thread was forcibly terminated
    Killed,
}

/// SRFI-21 compliant thread structure
#[derive(Debug)]
pub struct Srfi21Thread {
    /// Unique thread identifier
    pub id: ThreadId,
    /// Thread name (optional)
    pub name: Option<String>,
    /// Execution model (native vs async)
    pub model: ThreadExecutionModel,
    /// Thread priority with inheritance
    pub priority: Arc<RwLock<ThreadPriority>>,
    /// Current thread state
    pub state: Arc<RwLock<ThreadState>>,
    /// Quantum time slice (optional for real-time threads)
    pub quantum: Option<Duration>,
    /// Thread-specific storage
    pub specific_storage: Arc<RwLock<HashMap<String, Value>>>,
    /// Dynamic environment bindings
    pub dynamic_env: Arc<RwLock<DynamicEnvironment>>,
    /// Start time for performance monitoring
    pub start_time: Option<Instant>,
    /// Thread termination flag
    pub terminate_flag: Arc<AtomicBool>,
}

/// Dynamic environment for thread-local bindings and dynamic-wind
#[derive(Debug, Clone)]
pub struct DynamicEnvironment {
    /// Stack of binding frames for nested dynamic scopes
    pub bindings: Vec<HashMap<String, Value>>,
    /// Dynamic-wind stack for cleanup on exit
    pub wind_stack: Vec<WindFrame>,
}

/// Frame for dynamic-wind cleanup tracking
#[derive(Debug, Clone)]
pub struct WindFrame {
    /// Cleanup thunk to execute on exit
    pub after_thunk: Value,
    /// Unique identifier for this frame
    pub frame_id: u64,
}

impl DynamicEnvironment {
    pub fn new() -> Self {
        Self {
            bindings: vec![HashMap::new()],
            wind_stack: Vec::new(),
        }
    }

    pub fn push_frame(&mut self) {
        self.bindings.push(HashMap::new());
    }

    pub fn pop_frame(&mut self) {
        if self.bindings.len() > 1 {
            self.bindings.pop();
        }
    }

    pub fn bind(&mut self, name: String, value: Value) {
        if let Some(frame) = self.bindings.last_mut() {
            frame.insert(name, value);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        for frame in self.bindings.iter().rev() {
            if let Some(value) = frame.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
}

impl Default for DynamicEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// SRFI-21 compliant mutex with priority inheritance
#[derive(Debug)]
pub struct Srfi21Mutex {
    /// Underlying standard mutex
    inner: StdMutex<()>,
    /// Current owner thread (if any)
    owner: Arc<RwLock<Option<ThreadId>>>,
    /// Whether priority inheritance is enabled
    priority_inheritance: bool,
    /// Mutex name for debugging
    name: Option<String>,
    /// Specific timeout value (None = wait forever)
    timeout: Option<Duration>,
}

/// SRFI-21 compliant condition variable with priority queuing
#[derive(Debug)]
pub struct Srfi21CondVar {
    /// Underlying standard condition variable
    inner: StdCondVar,
    /// Priority queue for waiting threads (highest priority first)
    priority_queue: Arc<RwLock<BinaryHeap<WaitingThread>>>,
    /// Condition variable name for debugging
    name: Option<String>,
}

/// Waiting thread entry for priority queuing
#[derive(Debug, Clone)]
pub struct WaitingThread {
    /// Thread identifier
    pub thread_id: ThreadId,
    /// Thread priority at wait time
    pub priority: ThreadPriority,
    /// Time when thread started waiting
    pub wait_start: Instant,
}

impl PartialEq for WaitingThread {
    fn eq(&self, other: &Self) -> bool {
        self.thread_id == other.thread_id
    }
}

impl Eq for WaitingThread {}

impl PartialOrd for WaitingThread {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for WaitingThread {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Higher priority threads come first (reverse order for max-heap)
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.wait_start.cmp(&other.wait_start))
    }
}

/// Priority inheritance graph for managing priority relationships
#[derive(Debug)]
pub struct PriorityInheritanceGraph {
    /// Dependencies between threads (holder -> waiters)
    dependencies: HashMap<ThreadId, HashSet<ThreadId>>,
    /// Current effective priorities for all threads
    effective_priorities: HashMap<ThreadId, ThreadPriority>,
}

/// Global thread registry for SRFI-21 threads
#[derive(Debug)]
pub struct ThreadRegistry {
    /// All active threads
    threads: RwLock<HashMap<ThreadId, Arc<Srfi21Thread>>>,
    /// Priority inheritance graph
    priority_graph: RwLock<PriorityInheritanceGraph>,
    /// Current thread mapping (simplified for now)
    current_thread: Arc<RwLock<Option<ThreadId>>>,
}

/// Time representation for SRFI-21 (nanoseconds since epoch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Srfi21Time {
    pub nanos: u64,
}

impl Srfi21Time {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0));
        Self {
            nanos: duration.as_nanos() as u64,
        }
    }

    pub fn to_seconds(&self) -> f64 {
        self.nanos as f64 / 1_000_000_000.0
    }

    pub fn from_seconds(seconds: f64) -> Self {
        Self {
            nanos: (seconds * 1_000_000_000.0) as u64,
        }
    }

    pub fn add_duration(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos + duration.as_nanos() as u64,
        }
    }

    pub fn sub_duration(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_sub(duration.as_nanos() as u64),
        }
    }
}

impl Srfi21Thread {
    /// Create a new SRFI-21 thread with specified priority and quantum
    pub fn new(name: Option<String>, priority: u8, quantum: Option<Duration>) -> Self {
        Self {
            id: ThreadId::new(),
            name,
            model: ThreadExecutionModel::NativeThread(None),
            priority: Arc::new(RwLock::new(ThreadPriority::new(priority))),
            state: Arc::new(RwLock::new(ThreadState::New)),
            quantum,
            specific_storage: Arc::new(RwLock::new(HashMap::new())),
            dynamic_env: Arc::new(RwLock::new(DynamicEnvironment::new())),
            start_time: None,
            terminate_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get thread identifier
    pub fn id(&self) -> ThreadId {
        self.id
    }

    /// Get thread name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get current thread state
    pub fn state(&self) -> ThreadState {
        *self.state.read().unwrap()
    }

    /// Set thread state
    pub fn set_state(&self, new_state: ThreadState) {
        *self.state.write().unwrap() = new_state;
    }

    /// Get effective priority
    pub fn effective_priority(&self) -> u8 {
        self.priority.read().unwrap().effective_priority
    }

    /// Check if thread should terminate
    pub fn should_terminate(&self) -> bool {
        self.terminate_flag.load(Ordering::Acquire)
    }

    /// Request thread termination
    pub fn request_termination(&self) {
        self.terminate_flag.store(true, Ordering::Release);
    }
}

impl Srfi21Mutex {
    /// Create a new SRFI-21 mutex
    pub fn new(name: Option<String>, priority_inheritance: bool) -> Self {
        Self {
            inner: StdMutex::new(()),
            owner: Arc::new(RwLock::new(None)),
            priority_inheritance,
            name,
            timeout: None,
        }
    }

    /// Create a timed mutex with timeout
    pub fn new_timed(name: Option<String>, priority_inheritance: bool, timeout: Duration) -> Self {
        Self {
            inner: StdMutex::new(()),
            owner: Arc::new(RwLock::new(None)),
            priority_inheritance,
            name,
            timeout: Some(timeout),
        }
    }

    /// Get current owner (if any)
    pub fn owner(&self) -> Option<ThreadId> {
        *self.owner.read().unwrap()
    }

    /// Check if mutex supports priority inheritance
    pub fn has_priority_inheritance(&self) -> bool {
        self.priority_inheritance
    }

    /// Get mutex name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Srfi21CondVar {
    /// Create a new SRFI-21 condition variable
    pub fn new(name: Option<String>) -> Self {
        Self {
            inner: StdCondVar::new(),
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            name,
        }
    }

    /// Get number of waiting threads
    pub fn waiting_count(&self) -> usize {
        self.priority_queue.read().unwrap().len()
    }

    /// Get condition variable name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl PriorityInheritanceGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            effective_priorities: HashMap::new(),
        }
    }

    /// Record that waiter is blocked on holder's resource
    pub fn inherit_priority(&mut self, holder: ThreadId, waiter: ThreadId) -> Result<()> {
        self.dependencies
            .entry(holder)
            .or_insert_with(HashSet::new)
            .insert(waiter);
        // Recalculate effective priorities
        self.recalculate_priorities();
        Ok(())
    }

    /// Remove priority inheritance relationship
    pub fn release_priority(&mut self, holder: ThreadId, waiter: ThreadId) -> Result<()> {
        if let Some(waiters) = self.dependencies.get_mut(&holder) {
            waiters.remove(&waiter);
            if waiters.is_empty() {
                self.dependencies.remove(&holder);
            }
        }
        // Recalculate effective priorities
        self.recalculate_priorities();
        Ok(())
    }

    /// Recalculate all effective priorities
    fn recalculate_priorities(&mut self) {
        // This is a simplified implementation - in production, would use
        // a more sophisticated graph algorithm to handle priority chains
        for (&holder, waiters) in &self.dependencies {
            // First, calculate the max waiter priority without mutable borrow
            let max_waiter_priority = waiters
                .iter()
                .filter_map(|&waiter| self.effective_priorities.get(&waiter))
                .map(|p| p.effective_priority)
                .max();

            // Then, get mutable access to holder priority
            if let Some(holder_priority) = self.effective_priorities.get_mut(&holder) {
                let base_priority = holder_priority.base_priority;
                let max_waiter_priority = max_waiter_priority.unwrap_or(base_priority);

                if max_waiter_priority > base_priority {
                    holder_priority.inherit_priority(max_waiter_priority);
                } else {
                    holder_priority.release_inheritance();
                }
            }
        }
    }

    /// Get effective priority for a thread
    pub fn effective_priority(&self, thread_id: ThreadId) -> Option<ThreadPriority> {
        self.effective_priorities.get(&thread_id).cloned()
    }
}

impl Default for PriorityInheritanceGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadRegistry {
    pub fn new() -> Self {
        Self {
            threads: RwLock::new(HashMap::new()),
            priority_graph: RwLock::new(PriorityInheritanceGraph::new()),
            current_thread: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a new thread
    pub fn register_thread(&self, thread: Arc<Srfi21Thread>) -> Result<()> {
        let thread_id = thread.id();
        self.threads.write().unwrap().insert(thread_id, thread);
        Ok(())
    }

    /// Unregister a thread
    pub fn unregister_thread(&self, thread_id: ThreadId) -> Result<()> {
        self.threads.write().unwrap().remove(&thread_id);
        Ok(())
    }

    /// Get thread by ID
    pub fn get_thread(&self, thread_id: ThreadId) -> Option<Arc<Srfi21Thread>> {
        self.threads.read().unwrap().get(&thread_id).cloned()
    }

    /// Get current thread ID
    pub fn current_thread_id(&self) -> Option<ThreadId> {
        *self.current_thread.read().unwrap()
    }

    /// Set current thread ID (called when thread starts)
    pub fn set_current_thread(&self, thread_id: ThreadId) {
        *self.current_thread.write().unwrap() = Some(thread_id);
    }
}

impl Default for ThreadRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global thread registry instance
pub static THREAD_REGISTRY: std::sync::LazyLock<ThreadRegistry> =
    std::sync::LazyLock::new(ThreadRegistry::new);

/// Error types specific to SRFI-21 operations
#[derive(Debug)]
pub enum Srfi21Error {
    /// Thread creation failed
    ThreadCreationFailed(String),
    /// Thread not found
    ThreadNotFound(ThreadId),
    /// Thread already started
    ThreadAlreadyStarted(ThreadId),
    /// Thread not started
    ThreadNotStarted(ThreadId),
    /// Mutex lock timeout
    MutexTimeout,
    /// Condition variable timeout
    CondVarTimeout,
    /// Invalid priority value
    InvalidPriority(u8),
    /// Deadlock detected
    DeadlockDetected,
    /// Thread terminated
    ThreadTerminated(ThreadId),
}

impl std::fmt::Display for Srfi21Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThreadCreationFailed(msg) => write!(f, "Thread creation failed: {msg}"),
            Self::ThreadNotFound(id) => write!(f, "Thread not found: {:?}", id),
            Self::ThreadAlreadyStarted(id) => write!(f, "Thread already started: {:?}", id),
            Self::ThreadNotStarted(id) => write!(f, "Thread not started: {:?}", id),
            Self::MutexTimeout => write!(f, "Mutex lock timeout"),
            Self::CondVarTimeout => write!(f, "Condition variable timeout"),
            Self::InvalidPriority(p) => write!(f, "Invalid priority: {p}"),
            Self::DeadlockDetected => write!(f, "Deadlock detected"),
            Self::ThreadTerminated(id) => write!(f, "Thread terminated: {:?}", id),
        }
    }
}

impl std::error::Error for Srfi21Error {}

impl From<Srfi21Error> for Error {
    fn from(err: Srfi21Error) -> Self {
        Error::runtime_error(err.to_string(), None)
    }
}

// Note: ConcurrencyError is not available without async-runtime feature
// This conversion is removed for now
