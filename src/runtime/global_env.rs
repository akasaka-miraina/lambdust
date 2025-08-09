//! Global environment manager for multithreaded evaluation.
//!
//! This module manages the shared global environment state that is accessible
//! across all evaluator threads, while also managing thread-local environment
//! extensions.

use crate::eval::{Value, ThreadSafeEnvironment, Generation};
use crate::diagnostics::Result;
use std::sync::{Arc, RwLock};
use std::thread::ThreadId;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use std::sync::atomic::{AtomicU64, Ordering};

/// Manages global environment state across multiple evaluator threads.
///
/// This manager maintains the root global environment that contains
/// standard library bindings, and coordinates thread-local environment
/// extensions for each evaluator thread. It now includes transaction-based
/// state management with rollback capabilities.
#[derive(Debug)]
pub struct GlobalEnvironmentManager {
    /// The root global environment with standard bindings
    root_environment: Arc<ThreadSafeEnvironment>,
    /// Thread-local environment extensions
    thread_local_envs: Arc<RwLock<HashMap<ThreadId, Arc<ThreadSafeEnvironment>>>>,
    /// Global variable definitions that can be accessed by all threads
    global_definitions: Arc<RwLock<HashMap<String, Value>>>,
    /// Generation counter for the global environment
    global_generation: Arc<AtomicU64>,
    /// Transaction management system
    transaction_manager: Arc<TransactionManager>,
    /// State snapshot manager for rollbacks
    snapshot_manager: Arc<StateSnapshotManager>,
}

/// Transaction manager for coordinating state changes across threads.
#[derive(Debug)]
pub struct TransactionManager {
    /// Active transactions
    active_transactions: Arc<RwLock<HashMap<u64, StateTransaction>>>,
    /// Transaction sequence counter
    transaction_sequence: AtomicU64,
    /// Transaction timeout duration
    default_timeout: Duration,
}

/// State snapshot manager for rollback capabilities.
#[derive(Debug)]
pub struct StateSnapshotManager {
    /// Environment snapshots indexed by generation
    environment_snapshots: Arc<RwLock<HashMap<Generation, EnvironmentSnapshot>>>,
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
    /// Snapshot creation policy
    snapshot_policy: SnapshotPolicy,
}

/// A transaction for coordinating state changes.
#[derive(Debug, Clone)]
pub struct StateTransaction {
    /// Unique transaction ID
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub id: u64,
    /// Thread that initiated the transaction
    pub initiator_thread: ThreadId,
    /// Participating threads
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub participating_threads: Vec<ThreadId>,
    /// Transaction state
    pub state: TransactionState,
    /// Changes made in this transaction
    pub changes: Vec<StateChange>,
    /// Snapshot generation at transaction start
    pub snapshot_generation: Generation,
    /// Creation timestamp
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub created_at: SystemTime,
    /// Timeout for this transaction
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub timeout: Duration,
}

/// State of a transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    /// Transaction is being prepared
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    Preparing,
    /// Transaction is active
    Active,
    /// Transaction is committing
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    Committing,
    /// Transaction committed successfully
    Committed,
    /// Transaction is aborting
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    Aborting,
    /// Transaction was aborted
    Aborted,
    /// Transaction rolled back
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    RolledBack,
}

/// A change made within a transaction.
#[derive(Debug, Clone)]
pub struct StateChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Variable name affected
    pub variable_name: String,
    /// Old value (for rollback)
    pub old_value: Option<Value>,
    /// New value
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub new_value: Option<Value>,
    /// Generation when change was made
    pub generation: Generation,
    /// Thread that made the change
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub thread_id: ThreadId,
}

/// Type of state change.
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// Variable was defined
    Define,
    /// Variable was updated
    Update,
    /// Variable was deleted
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    Delete,
}

/// Snapshot of environment state.
#[derive(Debug, Clone)]
pub struct EnvironmentSnapshot {
    /// Generation of this snapshot
    pub generation: Generation,
    /// Global variable definitions at snapshot time
    pub global_definitions: HashMap<String, Value>,
    /// Thread-local environments at snapshot time
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    pub thread_local_envs: HashMap<ThreadId, HashMap<String, Value>>,
    /// Timestamp when snapshot was created
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    pub created_at: SystemTime,
}

/// Policy for when to create snapshots.
#[derive(Debug, Clone)]
pub enum SnapshotPolicy {
    /// Create snapshot on every generation change
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    EveryGeneration,
    /// Create snapshot every N generations
    EveryNGenerations(u64),
    /// Create snapshot on demand only
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    OnDemand,
    /// Create snapshot before every transaction
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    BeforeTransaction,
}

impl GlobalEnvironmentManager {
    /// Creates a new global environment manager.
    pub fn new() -> Self {
        let root_environment = Self::create_root_environment();
        
        Self {
            root_environment,
            thread_local_envs: Arc::new(RwLock::new(HashMap::new())),
            global_definitions: Arc::new(RwLock::new(HashMap::new())),
            global_generation: Arc::new(AtomicU64::new(0)),
            transaction_manager: Arc::new(TransactionManager::new()),
            snapshot_manager: Arc::new(StateSnapshotManager::new()),
        }
    }

    /// Creates the root environment with standard library bindings.
    fn create_root_environment() -> Arc<ThreadSafeEnvironment> {
        // Start with an empty environment
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        
        // Add basic values
        env.define("true".to_string(), Value::t());
        env.define("false".to_string(), Value::f());
        env.define("null".to_string(), Value::Nil);

        // Populate with standard library (includes system functions)
        let stdlib = crate::stdlib::StandardLibrary::new();
        stdlib.populate_environment(&env);
        
        env
    }

    /// Creates a thread-local environment for the given thread.
    ///
    /// This environment extends the root environment and can have
    /// thread-specific bindings.
    pub fn create_thread_local_env(&self, thread_id: ThreadId) -> Arc<ThreadSafeEnvironment> {
        let generation = self.next_generation();
        let local_env = self.root_environment.extend(generation);
        
        // Store the thread-local environment
        {
            let mut thread_envs = self.thread_local_envs.write().unwrap();
            thread_envs.insert(thread_id, local_env.clone());
        }
        
        local_env
    }

    /// Gets the thread-local environment for the given thread.
    pub fn get_thread_local_env(&self, thread_id: ThreadId) -> Option<Arc<ThreadSafeEnvironment>> {
        let thread_envs = self.thread_local_envs.read().unwrap();
        thread_envs.get(&thread_id).cloned()
    }

    /// Defines a global variable that is accessible to all threads.
    /// This operation is now transactional and can be rolled back.
    pub fn define_global(&self, name: String, value: Value) -> Result<()> {
        let thread_id = std::thread::current().id();
        self.define_global_transactional(name, value, thread_id)
    }
    
    /// Defines a global variable within a transaction context.
    pub fn define_global_transactional(
        &self, 
        name: String, 
        value: Value, 
        thread_id: ThreadId
    ) -> Result<()> {
        // Start a transaction if none is active for this thread
        let transaction_id = self.transaction_manager.get_or_start_transaction(thread_id)?;
        
        // Create snapshot if needed
        if self.snapshot_manager.should_create_snapshot() {
            self.create_environment_snapshot()?;
        }
        
        // Record the old value for potential rollback
        let old_value = {
            let globals = self.global_definitions.read().unwrap();
            globals.get(&name).cloned()
        };
        
        // Make the change
        {
            let mut globals = self.global_definitions.write().unwrap();
            globals.insert(name.clone(), value.clone());
        }
        
        // Record the change in the transaction
        let change = StateChange {
            change_type: if old_value.is_some() { ChangeType::Update } else { ChangeType::Define },
            variable_name: name,
            old_value,
            new_value: Some(value),
            generation: self.current_generation(),
            thread_id,
        };
        
        self.transaction_manager.add_change(transaction_id, change)?;
        
        Ok(())
    }

    /// Looks up a global variable.
    pub fn lookup_global(&self, name: &str) -> Option<Value> {
        // First check global definitions
        {
            let globals = self.global_definitions.read().unwrap();
            if let Some(value) = globals.get(name) {
                return Some(value.clone());
            }
        }
        
        // Then check root environment
        self.root_environment.lookup(name)
    }

    /// Gets the root environment.
    pub fn root_environment(&self) -> Arc<ThreadSafeEnvironment> {
        self.root_environment.clone()
    }

    /// Gets the current global generation.
    pub fn current_generation(&self) -> Generation {
        self.global_generation.load(Ordering::SeqCst)
    }

    /// Increments and returns the next global generation.
    pub fn next_generation(&self) -> Generation {
        let new_gen = self.global_generation.fetch_add(1, Ordering::SeqCst) + 1;
        
        // Check if we should create a snapshot
        if self.snapshot_manager.should_create_snapshot_for_generation(new_gen) {
            let _ = self.create_environment_snapshot();
        }
        
        new_gen
    }

    /// Removes a thread's local environment (called when thread shuts down).
    pub fn remove_thread_local_env(&self, thread_id: ThreadId) {
        let mut thread_envs = self.thread_local_envs.write().unwrap();
        thread_envs.remove(&thread_id);
    }

    /// Gets the number of active thread-local environments.
    pub fn active_thread_count(&self) -> usize {
        let thread_envs = self.thread_local_envs.read().unwrap();
        thread_envs.len()
    }

    /// Lists all global variable names.
    pub fn global_variable_names(&self) -> Vec<String> {
        let globals = self.global_definitions.read().unwrap();
        globals.keys().cloned().collect()
    }

    /// Gets a snapshot of all global variables.
    pub fn global_variables_snapshot(&self) -> HashMap<String, Value> {
        let globals = self.global_definitions.read().unwrap();
        globals.clone()
    }

    /// Clears all global variable definitions (but not root environment).
    pub fn clear_global_definitions(&self) {
        let mut globals = self.global_definitions.write().unwrap();
        globals.clear();
        
        // Create a snapshot after clearing
        let _ = self.create_environment_snapshot();
    }
    
    /// Starts a new transaction for coordinated state changes.
    pub fn start_transaction(&self, thread_id: ThreadId) -> Result<u64> {
        self.transaction_manager.start_transaction(thread_id, Vec::new())
    }
    
    /// Commits a transaction, making all changes permanent.
    pub fn commit_transaction(&self, transaction_id: u64) -> Result<()> {
        self.transaction_manager.commit_transaction(transaction_id)
    }
    
    /// Aborts a transaction, rolling back all changes.
    pub fn abort_transaction(&self, transaction_id: u64) -> Result<()> {
        // Get the transaction to find rollback information
        let transaction = self.transaction_manager.get_transaction(transaction_id)?;
        
        // Rollback all changes in reverse order
        for change in transaction.changes.iter().rev() {
            self.rollback_change(change)?;
        }
        
        self.transaction_manager.abort_transaction(transaction_id)
    }
    
    /// Rolls back to a specific generation.
    pub fn rollback_to_generation(&self, target_generation: Generation) -> Result<()> {
        self.snapshot_manager.rollback_to_generation(target_generation, self)
    }
    
    /// Creates a snapshot of the current environment state.
    pub fn create_environment_snapshot(&self) -> Result<Generation> {
        let generation = self.current_generation();
        
        let global_definitions = {
            let globals = self.global_definitions.read().unwrap();
            globals.clone()
        };
        
        let thread_local_envs = {
            let envs = self.thread_local_envs.read().unwrap();
            let mut snapshot_envs = HashMap::new();
            
            for (thread_id, _env) in envs.iter() {
                // For simplicity, we'll store an empty HashMap for thread-local envs
                // In a full implementation, you'd extract the actual bindings
                snapshot_envs.insert(*thread_id, HashMap::new());
            }
            
            snapshot_envs
        };
        
        let snapshot = EnvironmentSnapshot {
            generation,
            global_definitions,
            thread_local_envs,
            created_at: SystemTime::now(),
        };
        
        self.snapshot_manager.store_snapshot(snapshot);
        
        Ok(generation)
    }
    
    /// Rolls back a single state change.
    fn rollback_change(&self, change: &StateChange) -> Result<()> {
        match change.change_type {
            ChangeType::Define => {
                // Remove the variable
                let mut globals = self.global_definitions.write().unwrap();
                globals.remove(&change.variable_name);
            }
            ChangeType::Update => {
                // Restore the old value
                if let Some(ref old_value) = change.old_value {
                    let mut globals = self.global_definitions.write().unwrap();
                    globals.insert(change.variable_name.clone(), old_value.clone());
                } else {
                    // Old value was None, so remove the variable
                    let mut globals = self.global_definitions.write().unwrap();
                    globals.remove(&change.variable_name);
                }
            }
            ChangeType::Delete => {
                // Restore the deleted value
                if let Some(ref old_value) = change.old_value {
                    let mut globals = self.global_definitions.write().unwrap();
                    globals.insert(change.variable_name.clone(), old_value.clone());
                }
            }
        }
        
        Ok(())
    }
}

impl Default for GlobalEnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalEnvironmentManager {
    fn clone(&self) -> Self {
        Self {
            root_environment: self.root_environment.clone(),
            thread_local_envs: self.thread_local_envs.clone(),
            global_definitions: self.global_definitions.clone(),
            global_generation: self.global_generation.clone(),
            transaction_manager: self.transaction_manager.clone(),
            snapshot_manager: self.snapshot_manager.clone(),
        }
    }
}

// ============= TRANSACTION MANAGER IMPLEMENTATION =============

impl TransactionManager {
    /// Creates a new transaction manager.
    pub fn new() -> Self {
        Self {
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            transaction_sequence: AtomicU64::new(0),
            default_timeout: Duration::from_secs(30),
        }
    }
    
    /// Starts a new transaction.
    pub fn start_transaction(
        &self,
        initiator: ThreadId,
        participants: Vec<ThreadId>,
    ) -> Result<u64> {
        let id = self.transaction_sequence.fetch_add(1, Ordering::SeqCst);
        
        let transaction = StateTransaction {
            id,
            initiator_thread: initiator,
            participating_threads: participants,
            state: TransactionState::Active,
            changes: Vec::new(),
            snapshot_generation: 0, // Will be set when first change is made
            created_at: SystemTime::now(),
            timeout: self.default_timeout,
        };
        
        let mut transactions = self.active_transactions.write().unwrap();
        transactions.insert(id, transaction);
        
        Ok(id)
    }
    
    /// Gets or starts a transaction for a thread.
    pub fn get_or_start_transaction(&self, thread_id: ThreadId) -> Result<u64> {
        // Check if thread already has an active transaction
        {
            let transactions = self.active_transactions.read().unwrap();
            for (id, transaction) in transactions.iter() {
                if transaction.initiator_thread == thread_id 
                    && matches!(transaction.state, TransactionState::Active) {
                    return Ok(*id);
                }
            }
        }
        
        // Start a new transaction
        self.start_transaction(thread_id, Vec::new())
    }
    
    /// Gets a transaction by ID.
    pub fn get_transaction(&self, transaction_id: u64) -> Result<StateTransaction> {
        let transactions = self.active_transactions.read().unwrap();
        transactions.get(&transaction_id)
            .cloned()
            .ok_or_else(|| crate::diagnostics::Error::runtime_error(
                format!("Transaction {transaction_id} not found"),
                None
            ).boxed())
    }
    
    /// Adds a change to a transaction.
    pub fn add_change(&self, transaction_id: u64, change: StateChange) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            if transaction.changes.is_empty() {
                // Set snapshot generation on first change
                transaction.snapshot_generation = change.generation;
            }
            transaction.changes.push(change);
            Ok(())
        } else {
            Err(crate::diagnostics::Error::runtime_error(
                format!("Transaction {transaction_id} not found"),
                None
            ).boxed())
        }
    }
    
    /// Commits a transaction.
    pub fn commit_transaction(&self, transaction_id: u64) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.state = TransactionState::Committed;
            Ok(())
        } else {
            Err(crate::diagnostics::Error::runtime_error(
                format!("Transaction {transaction_id} not found"),
                None
            ).boxed())
        }
    }
    
    /// Aborts a transaction.
    pub fn abort_transaction(&self, transaction_id: u64) -> Result<()> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.state = TransactionState::Aborted;
            Ok(())
        } else {
            Err(crate::diagnostics::Error::runtime_error(
                format!("Transaction {transaction_id} not found"),
                None
            ).boxed())
        }
    }
    
    /// Cleans up completed transactions.
    #[allow(dead_code)] // Part of Stage 3 transaction infrastructure
    pub fn cleanup_completed_transactions(&self) {
        let mut transactions = self.active_transactions.write().unwrap();
        transactions.retain(|_, transaction| {
            !matches!(transaction.state, TransactionState::Committed | TransactionState::Aborted)
        });
    }
}

// ============= SNAPSHOT MANAGER IMPLEMENTATION =============

impl StateSnapshotManager {
    /// Creates a new snapshot manager.
    pub fn new() -> Self {
        Self {
            environment_snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots: 100,
            snapshot_policy: SnapshotPolicy::EveryNGenerations(10),
        }
    }
    
    /// Creates a snapshot manager with custom policy.
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    pub fn with_policy(policy: SnapshotPolicy, max_snapshots: usize) -> Self {
        Self {
            environment_snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots,
            snapshot_policy: policy,
        }
    }
    
    /// Checks if a snapshot should be created.
    pub fn should_create_snapshot(&self) -> bool {
        matches!(self.snapshot_policy, SnapshotPolicy::BeforeTransaction)
    }
    
    /// Checks if a snapshot should be created for a specific generation.
    pub fn should_create_snapshot_for_generation(&self, generation: Generation) -> bool {
        match self.snapshot_policy {
            SnapshotPolicy::EveryGeneration => true,
            SnapshotPolicy::EveryNGenerations(n) => generation % n == 0,
            SnapshotPolicy::OnDemand => false,
            SnapshotPolicy::BeforeTransaction => false,
        }
    }
    
    /// Stores a snapshot.
    pub fn store_snapshot(&self, snapshot: EnvironmentSnapshot) {
        let mut snapshots = self.environment_snapshots.write().unwrap();
        snapshots.insert(snapshot.generation, snapshot);
        
        // Clean up old snapshots if we exceed the limit
        if snapshots.len() > self.max_snapshots {
            let oldest_generations: Vec<_> = {
                let mut generations: Vec<_> = snapshots.keys().copied().collect();
                generations.sort();
                generations.into_iter().take(snapshots.len() - self.max_snapshots).collect()
            };
            
            for generation in oldest_generations {
                snapshots.remove(&generation);
            }
        }
    }
    
    /// Gets a snapshot by generation.
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    pub fn get_snapshot(&self, generation: Generation) -> Option<EnvironmentSnapshot> {
        let snapshots = self.environment_snapshots.read().unwrap();
        snapshots.get(&generation).cloned()
    }
    
    /// Finds the latest snapshot at or before the given generation.
    pub fn find_snapshot_before(&self, generation: Generation) -> Option<EnvironmentSnapshot> {
        let snapshots = self.environment_snapshots.read().unwrap();
        let mut best_generation = None;
        
        for &snap_generation in snapshots.keys() {
            if snap_generation <= generation && (best_generation.is_none() || snap_generation > best_generation.unwrap()) {
                best_generation = Some(snap_generation);
            }
        }
        
        best_generation.and_then(|generation| snapshots.get(&generation).cloned())
    }
    
    /// Rolls back to a specific generation.
    pub fn rollback_to_generation(
        &self,
        target_generation: Generation,
        env_manager: &GlobalEnvironmentManager,
    ) -> Result<()> {
        // Find the appropriate snapshot
        let snapshot = self.find_snapshot_before(target_generation)
            .ok_or_else(|| crate::diagnostics::Error::runtime_error(
                format!("No snapshot found for generation {target_generation}"),
                None
            ))?;
        
        // Restore global definitions
        {
            let mut globals = env_manager.global_definitions.write().unwrap();
            globals.clear();
            for (name, value) in snapshot.global_definitions {
                globals.insert(name, value);
            }
        }
        
        // Restore generation counter
        env_manager.global_generation.store(target_generation, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Lists all available snapshots.
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    pub fn list_snapshots(&self) -> Vec<Generation> {
        let snapshots = self.environment_snapshots.read().unwrap();
        let mut generations: Vec<_> = snapshots.keys().copied().collect();
        generations.sort();
        generations
    }
    
    /// Clears all snapshots.
    #[allow(dead_code)] // Part of Stage 3 snapshot infrastructure
    pub fn clear_snapshots(&self) {
        let mut snapshots = self.environment_snapshots.write().unwrap();
        snapshots.clear();
    }
}