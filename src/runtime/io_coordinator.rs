//! IO coordination system for multithreaded evaluation.
//!
//! This module provides distributed IO coordination to prevent race conditions
//! across threads while maintaining IO operation ordering and consistency.

use std::sync::{Arc, RwLock, Mutex, Condvar};
use std::thread::ThreadId;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{SystemTime, Duration};
use std::sync::atomic::{AtomicU64, Ordering};
use crossbeam::channel::{Sender, Receiver, unbounded};

/// Coordinates IO operations across multiple threads to prevent race conditions
/// and maintain operation ordering.
#[derive(Debug)]
pub struct IOCoordinator {
    /// Active IO operations by thread
    active_operations: Arc<RwLock<HashMap<ThreadId, Vec<IOOperation>>>>,
    /// IO resource locks to prevent conflicts
    resource_locks: Arc<RwLock<HashMap<String, IOResourceLock>>>,
    /// Operation ordering manager
    ordering_manager: Arc<IOOrderingManager>,
    /// Cross-thread IO coordination channels
    coordination_channels: Arc<RwLock<HashMap<ThreadId, IOChannel>>>,
    /// IO operation history for debugging
    operation_history: Arc<Mutex<VecDeque<IOOperationEvent>>>,
    /// Coordination policies
    policies: Arc<IOCoordinationPolicies>,
}

/// Manager for ordering IO operations across threads.
#[derive(Debug)]
pub struct IOOrderingManager {
    /// Operation sequence counter
    sequence_counter: AtomicU64,
    /// Pending operations ordered by sequence
    pending_operations: Arc<RwLock<BTreeMap<u64, PendingIOOperation>>>,
    /// Resource access order queues
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    resource_queues: Arc<RwLock<HashMap<String, VecDeque<u64>>>>,
    /// Completion notification system
    completion_notifier: Arc<(Mutex<HashMap<u64, bool>>, Condvar)>,
}

/// A pending IO operation waiting for coordination.
#[derive(Debug, Clone)]
pub struct PendingIOOperation {
    /// Operation sequence number
    pub sequence: u64,
    /// Thread that owns this operation
    pub thread_id: ThreadId,
    /// Type of IO operation
    pub operation_type: IOOperationType,
    /// Resource being accessed
    pub resource: String,
    /// Operation parameters
    pub parameters: IOParameters,
    /// Dependencies that must complete first
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    pub dependencies: Vec<u64>,
    /// Timestamp when operation was submitted
    pub submitted_at: SystemTime,
}

/// Cross-thread communication channel for IO coordination.
#[derive(Debug)]
pub struct IOChannel {
    /// Sender for IO coordination messages
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    pub sender: Sender<IOCoordinationMessage>,
    /// Receiver for IO coordination messages
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    pub receiver: Receiver<IOCoordinationMessage>,
}

/// A lock on an IO resource.
#[derive(Debug)]
pub struct IOResourceLock {
    /// Resource identifier
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    pub resource_id: String,
    /// Thread that holds the lock
    pub holder: ThreadId,
    /// Lock type (read/write)
    pub lock_type: LockType,
    /// When the lock was acquired
    pub acquired_at: SystemTime,
    /// Lock timeout
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    pub timeout: Duration,
    /// Queue of threads waiting for this resource
    pub wait_queue: VecDeque<(ThreadId, LockType, SystemTime)>,
}

/// Type of resource lock.
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    /// Read lock (multiple readers allowed)
    Read,
    /// Write lock (exclusive access)
    Write,
}

/// An IO operation being tracked.
#[derive(Debug, Clone)]
pub struct IOOperation {
    /// Unique operation ID
    pub id: u64,
    /// Type of operation
    pub operation_type: IOOperationType,
    /// Resource being accessed
    pub resource: String,
    /// Operation parameters
    pub parameters: IOParameters,
    /// When the operation started
    pub started_at: SystemTime,
    /// Current status
    pub status: IOOperationStatus,
}

/// Types of IO operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOOperationType {
    /// File read operation
    FileRead,
    /// File write operation
    FileWrite,
    /// File open operation
    FileOpen,
    /// File close operation
    FileClose,
    /// Directory operation
    Directory,
    /// Console output
    ConsoleOutput,
    /// Console input
    ConsoleInput,
    /// Network operation
    Network,
}

/// Parameters for IO operations.
#[derive(Debug, Clone, Default)]
pub struct IOParameters {
    /// File path for file operations
    pub file_path: Option<String>,
    /// Data being read/written
    pub data: Option<Vec<u8>>,
    /// Offset for seek operations
    pub offset: Option<u64>,
    /// Length for read operations
    pub length: Option<usize>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Status of an IO operation.
#[derive(Debug, Clone, PartialEq)]
pub enum IOOperationStatus {
    /// Operation is pending
    Pending,
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed(String),
    /// Operation was cancelled
    Cancelled,
}

/// Event recording an IO operation.
#[derive(Debug, Clone)]
pub struct IOOperationEvent {
    /// Thread that performed the operation
    pub thread_id: ThreadId,
    /// Timestamp of the event
    pub timestamp: SystemTime,
    /// Operation that was performed
    pub operation: IOOperation,
    /// Result of the operation
    pub result: Result<IOResult, String>,
    /// Sequence number for ordering
    pub sequence: u64,
}

/// Result of an IO operation.
#[derive(Debug, Clone)]
pub struct IOResult {
    /// Data returned by the operation
    pub data: Option<Vec<u8>>,
    /// Number of bytes processed
    pub bytes_processed: usize,
    /// Additional result metadata
    pub metadata: HashMap<String, String>,
}

/// Messages for cross-thread IO coordination.
#[derive(Debug, Clone)]
pub enum IOCoordinationMessage {
    /// Request to coordinate an IO operation
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    CoordinateIO {
        operation: IOOperation,
        dependencies: Vec<u64>,
    },
    /// Response to IO coordination request
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    CoordinationResponse {
        operation_id: u64,
        success: bool,
        error: Option<String>,
    },
    /// Notification that an IO operation completed
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    IOCompleted {
        operation_id: u64,
        result: Result<IOResult, String>,
    },
    /// Request a lock on a resource
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    RequestLock {
        resource: String,
        lock_type: LockType,
        timeout: Duration,
    },
    /// Response to lock request
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    LockResponse {
        resource: String,
        granted: bool,
        error: Option<String>,
    },
    /// Release a lock on a resource
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    ReleaseLock {
        resource: String,
    },
}

/// Policies for IO coordination.
#[derive(Debug)]
pub struct IOCoordinationPolicies {
    /// Whether to track IO operation history
    track_history: bool,
    /// Maximum size of operation history
    max_history_size: usize,
    /// Default timeout for IO operations
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    default_operation_timeout: Duration,
    /// Default timeout for resource locks
    default_lock_timeout: Duration,
    /// Whether to enforce strict ordering
    #[allow(dead_code)] // Part of Stage 3 IO coordination infrastructure
    enforce_strict_ordering: bool,
    /// Whether to allow concurrent reads
    allow_concurrent_reads: bool,
    /// Maximum concurrent operations per thread
    max_concurrent_operations_per_thread: usize,
}

impl IOCoordinator {
    /// Creates a new IO coordinator.
    pub fn new() -> Self {
        Self {
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            resource_locks: Arc::new(RwLock::new(HashMap::new())),
            ordering_manager: Arc::new(IOOrderingManager::new()),
            coordination_channels: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(Mutex::new(VecDeque::new())),
            policies: Arc::new(IOCoordinationPolicies::default()),
        }
    }

    /// Creates an IO coordinator with custom policies.
    pub fn with_policies(policies: IOCoordinationPolicies) -> Self {
        Self {
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            resource_locks: Arc::new(RwLock::new(HashMap::new())),
            ordering_manager: Arc::new(IOOrderingManager::new()),
            coordination_channels: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(Mutex::new(VecDeque::new())),
            policies: Arc::new(policies),
        }
    }

    /// Registers a thread with the IO coordinator.
    pub fn register_thread(&self, thread_id: ThreadId) {
        // Initialize active operations for this thread
        {
            let mut operations = self.active_operations.write().unwrap();
            operations.insert(thread_id, Vec::new());
        }

        // Create coordination channel for this thread
        let (sender, receiver) = unbounded();
        let channel = IOChannel { sender, receiver };

        let mut channels = self.coordination_channels.write().unwrap();
        channels.insert(thread_id, channel);
    }

    /// Unregisters a thread from the IO coordinator.
    pub fn unregister_thread(&self, thread_id: ThreadId) {
        // Remove active operations
        {
            let mut operations = self.active_operations.write().unwrap();
            operations.remove(&thread_id);
        }

        // Remove coordination channel
        {
            let mut channels = self.coordination_channels.write().unwrap();
            channels.remove(&thread_id);
        }

        // Release any locks held by this thread
        self.release_all_locks_for_thread(thread_id);

        // Cancel pending operations for this thread
        self.ordering_manager.cancel_operations_for_thread(thread_id);
    }

    /// Coordinates an IO operation across threads.
    pub fn coordinate_io_operation(
        &self,
        thread_id: ThreadId,
        operation_type: IOOperationType,
        resource: String,
        parameters: IOParameters,
    ) -> Result<u64, String> {
        // Check if thread has too many concurrent operations
        {
            let operations = self.active_operations.read().unwrap();
            if let Some(thread_ops) = operations.get(&thread_id) {
                if thread_ops.len() >= self.policies.max_concurrent_operations_per_thread {
                    return Err("Thread has reached maximum concurrent IO operations limit".to_string());
                }
            }
        }

        // Create the operation
        let operation_id = self.ordering_manager.next_sequence();
        let operation = IOOperation {
            id: operation_id,
            operation_type: operation_type.clone(),
            resource: resource.clone(),
            parameters,
            started_at: SystemTime::now(),
            status: IOOperationStatus::Pending,
        };

        // Determine dependencies based on resource and operation type
        let dependencies = self.ordering_manager.compute_dependencies(&operation)?;

        // Request resource lock if needed
        let lock_type = match operation_type {
            IOOperationType::FileRead | IOOperationType::ConsoleInput => LockType::Read,
            IOOperationType::FileWrite | IOOperationType::FileOpen | 
            IOOperationType::FileClose | IOOperationType::ConsoleOutput => LockType::Write,
            _ => LockType::Read,
        };

        self.request_resource_lock(&resource, lock_type, thread_id)?;

        // Add to pending operations
        let pending_operation = PendingIOOperation {
            sequence: operation_id,
            thread_id,
            operation_type,
            resource: resource.clone(),
            parameters: operation.parameters.clone(),
            dependencies,
            submitted_at: SystemTime::now(),
        };

        self.ordering_manager.add_pending_operation(pending_operation)?;

        // Add to active operations
        {
            let mut operations = self.active_operations.write().unwrap();
            if let Some(thread_ops) = operations.get_mut(&thread_id) {
                thread_ops.push(operation.clone());
            }
        }

        // Record the operation event
        if self.policies.track_history {
            self.record_operation_event(thread_id, operation, Ok(IOResult {
                data: None,
                bytes_processed: 0,
                metadata: HashMap::new(),
            }), operation_id);
        }

        Ok(operation_id)
    }

    /// Completes an IO operation.
    pub fn complete_io_operation(
        &self,
        operation_id: u64,
        result: Result<IOResult, String>,
    ) -> Result<(), String> {
        // Remove from pending operations
        let pending_op = self.ordering_manager.complete_operation(operation_id)?;

        // Release resource lock
        self.release_resource_lock(&pending_op.resource, pending_op.thread_id);

        // Remove from active operations
        {
            let mut operations = self.active_operations.write().unwrap();
            if let Some(thread_ops) = operations.get_mut(&pending_op.thread_id) {
                thread_ops.retain(|op| op.id != operation_id);
            }
        }

        // Notify ordering manager of completion
        self.ordering_manager.notify_operation_completion(operation_id);

        // Record completion event
        if self.policies.track_history {
            let operation = IOOperation {
                id: operation_id,
                operation_type: pending_op.operation_type,
                resource: pending_op.resource,
                parameters: pending_op.parameters,
                started_at: pending_op.submitted_at,
                status: match result {
                    Ok(_) => IOOperationStatus::Completed,
                    Err(_) => IOOperationStatus::Failed("Operation failed".to_string()),
                },
            };

            self.record_operation_event(pending_op.thread_id, operation, result, operation_id);
        }

        Ok(())
    }

    /// Requests a lock on a resource.
    fn request_resource_lock(
        &self,
        resource: &str,
        lock_type: LockType,
        thread_id: ThreadId,
    ) -> Result<(), String> {
        let mut locks = self.resource_locks.write().unwrap();

        // Check if resource is already locked
        if let Some(existing_lock) = locks.get_mut(resource) {
            // Check if we can grant the lock
            match (&existing_lock.lock_type, &lock_type) {
                (LockType::Read, LockType::Read) if self.policies.allow_concurrent_reads => {
                    // Allow concurrent reads
                    return Ok(());
                }
                _ => {
                    // Add to wait queue
                    existing_lock.wait_queue.push_back((thread_id, lock_type, SystemTime::now()));
                    return Err(format!("Resource {resource} is locked, added to wait queue"));
                }
            }
        } else {
            // Grant the lock
            let lock = IOResourceLock {
                resource_id: resource.to_string(),
                holder: thread_id,
                lock_type,
                acquired_at: SystemTime::now(),
                timeout: self.policies.default_lock_timeout,
                wait_queue: VecDeque::new(),
            };
            locks.insert(resource.to_string(), lock);
        }

        Ok(())
    }

    /// Releases a lock on a resource.
    fn release_resource_lock(&self, resource: &str, thread_id: ThreadId) {
        let mut locks = self.resource_locks.write().unwrap();

        if let Some(lock) = locks.get_mut(resource) {
            if lock.holder == thread_id {
                // Check if anyone is waiting
                if let Some((next_thread, next_lock_type, _)) = lock.wait_queue.pop_front() {
                    // Grant lock to next thread
                    lock.holder = next_thread;
                    lock.lock_type = next_lock_type;
                    lock.acquired_at = SystemTime::now();
                } else {
                    // Remove the lock entirely
                    locks.remove(resource);
                }
            }
        }
    }

    /// Releases all locks held by a thread.
    fn release_all_locks_for_thread(&self, thread_id: ThreadId) {
        let mut locks = self.resource_locks.write().unwrap();
        let resources_to_release: Vec<String> = locks
            .iter()
            .filter(|(_, lock)| lock.holder == thread_id)
            .map(|(resource, _)| resource.clone())
            .collect();

        for resource in resources_to_release {
            if let Some(mut lock) = locks.remove(&resource) {
                // Grant lock to next thread if any
                if let Some((next_thread, next_lock_type, _)) = lock.wait_queue.pop_front() {
                    lock.holder = next_thread;
                    lock.lock_type = next_lock_type;
                    lock.acquired_at = SystemTime::now();
                    locks.insert(resource, lock);
                }
            }
        }
    }

    /// Records an IO operation event.
    fn record_operation_event(
        &self,
        thread_id: ThreadId,
        operation: IOOperation,
        result: Result<IOResult, String>,
        sequence: u64,
    ) {
        let event = IOOperationEvent {
            thread_id,
            timestamp: SystemTime::now(),
            operation,
            result,
            sequence,
        };

        let mut history = self.operation_history.lock().unwrap();
        history.push_back(event);

        // Trim history if it gets too large
        if history.len() > self.policies.max_history_size {
            history.pop_front();
        }
    }

    /// Gets IO operation statistics.
    pub fn get_io_statistics(&self) -> IOStatistics {
        let operations = self.active_operations.read().unwrap();
        let locks = self.resource_locks.read().unwrap();
        let history = self.operation_history.lock().unwrap();

        let mut stats = IOStatistics {
            active_threads: operations.len(),
            total_active_operations: 0,
            operations_by_type: HashMap::new(),
            active_locks: locks.len(),
            total_historical_operations: history.len(),
            recent_operations: 0,
        };

        // Count active operations
        for thread_ops in operations.values() {
            stats.total_active_operations += thread_ops.len();
            for op in thread_ops {
                *stats.operations_by_type.entry(op.operation_type.clone()).or_insert(0) += 1;
            }
        }

        // Count recent operations (last minute)
        let one_minute_ago = SystemTime::now() - Duration::from_secs(60);
        stats.recent_operations = history
            .iter()
            .filter(|event| event.timestamp > one_minute_ago)
            .count();

        stats
    }

    /// Gets the operation history.
    pub fn get_operation_history(&self) -> Vec<IOOperationEvent> {
        let history = self.operation_history.lock().unwrap();
        history.iter().cloned().collect()
    }

    /// Clears the operation history.
    pub fn clear_operation_history(&self) {
        let mut history = self.operation_history.lock().unwrap();
        history.clear();
    }
}

/// Statistics about IO operations.
#[derive(Debug, Clone)]
pub struct IOStatistics {
    /// Number of active threads
    pub active_threads: usize,
    /// Total number of active operations across all threads
    pub total_active_operations: usize,
    /// Count of operations by type
    pub operations_by_type: HashMap<IOOperationType, usize>,
    /// Number of active resource locks
    pub active_locks: usize,
    /// Total number of historical operations
    pub total_historical_operations: usize,
    /// Number of operations in the last minute
    pub recent_operations: usize,
}

impl IOOrderingManager {
    /// Creates a new IO ordering manager.
    pub fn new() -> Self {
        Self {
            sequence_counter: AtomicU64::new(0),
            pending_operations: Arc::new(RwLock::new(BTreeMap::new())),
            resource_queues: Arc::new(RwLock::new(HashMap::new())),
            completion_notifier: Arc::new((Mutex::new(HashMap::new()), Condvar::new())),
        }
    }

    /// Gets the next sequence number.
    pub fn next_sequence(&self) -> u64 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Computes dependencies for an operation.
    pub fn compute_dependencies(&self, operation: &IOOperation) -> Result<Vec<u64>, String> {
        let pending = self.pending_operations.read().unwrap();
        let mut dependencies = Vec::new();

        // Find operations on the same resource that must complete first
        for (seq, pending_op) in pending.iter() {
            if pending_op.resource == operation.resource && *seq < operation.id {
                // For write operations, must wait for all previous operations
                // For read operations, only wait for write operations
                match (&pending_op.operation_type, &operation.operation_type) {
                    (_, IOOperationType::FileWrite) |
                    (_, IOOperationType::FileOpen) |
                    (_, IOOperationType::FileClose) |
                    (IOOperationType::FileWrite, _) |
                    (IOOperationType::FileOpen, _) |
                    (IOOperationType::FileClose, _) => {
                        dependencies.push(*seq);
                    }
                    _ => {
                        // Read operations can proceed in parallel
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Adds a pending operation.
    pub fn add_pending_operation(&self, operation: PendingIOOperation) -> Result<(), String> {
        let mut pending = self.pending_operations.write().unwrap();
        pending.insert(operation.sequence, operation);
        Ok(())
    }

    /// Completes an operation.
    pub fn complete_operation(&self, sequence: u64) -> Result<PendingIOOperation, String> {
        let mut pending = self.pending_operations.write().unwrap();
        pending.remove(&sequence).ok_or_else(|| {
            format!("Operation {sequence} not found in pending operations")
        })
    }

    /// Notifies of operation completion.
    pub fn notify_operation_completion(&self, sequence: u64) {
        let (lock, cvar) = &*self.completion_notifier;
        let mut completed = lock.lock().unwrap();
        completed.insert(sequence, true);
        cvar.notify_all();
    }

    /// Cancels operations for a thread.
    pub fn cancel_operations_for_thread(&self, thread_id: ThreadId) {
        let mut pending = self.pending_operations.write().unwrap();
        pending.retain(|_, op| op.thread_id != thread_id);
    }
}

impl Default for IOCoordinationPolicies {
    fn default() -> Self {
        Self {
            track_history: true,
            max_history_size: 1000,
            default_operation_timeout: Duration::from_secs(30),
            default_lock_timeout: Duration::from_secs(10),
            enforce_strict_ordering: true,
            allow_concurrent_reads: true,
            max_concurrent_operations_per_thread: 10,
        }
    }
}

impl IOCoordinationPolicies {
    /// Creates policies with minimal overhead.
    pub fn minimal() -> Self {
        Self {
            track_history: false,
            max_history_size: 100,
            default_operation_timeout: Duration::from_secs(5),
            default_lock_timeout: Duration::from_secs(1),
            enforce_strict_ordering: false,
            allow_concurrent_reads: true,
            max_concurrent_operations_per_thread: 5,
        }
    }

    /// Creates policies optimized for high throughput.
    pub fn high_throughput() -> Self {
        Self {
            track_history: false,
            max_history_size: 500,
            default_operation_timeout: Duration::from_secs(60),
            default_lock_timeout: Duration::from_secs(5),
            enforce_strict_ordering: false,
            allow_concurrent_reads: true,
            max_concurrent_operations_per_thread: 50,
        }
    }
}


impl Clone for IOChannel {
    fn clone(&self) -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

impl Default for IOCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for IOCoordinator {
    fn clone(&self) -> Self {
        Self {
            active_operations: self.active_operations.clone(),
            resource_locks: self.resource_locks.clone(),
            ordering_manager: self.ordering_manager.clone(),
            coordination_channels: self.coordination_channels.clone(),
            operation_history: self.operation_history.clone(),
            policies: self.policies.clone(),
        }
    }
}