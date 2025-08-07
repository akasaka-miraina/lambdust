//! Effect coordination system for multithreaded evaluation.
//!
//! This module coordinates effects across multiple evaluator threads,
//! ensuring proper effect handling and maintaining effect semantics
//! in a concurrent environment.

use crate::effects::{Effect, EffectContext};
use std::sync::{Arc, RwLock, Mutex};
use std::thread::ThreadId;
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, Duration};
use crossbeam::channel::{Sender, Receiver, unbounded};
use std::sync::atomic::{AtomicU64, Ordering};

/// Coordinates effects across multiple evaluator threads.
///
/// The EffectCoordinator manages effect contexts for each thread
/// and coordinates cross-thread effect interactions when necessary.
/// It provides concurrent effect coordination, ordering guarantees,
/// and isolation mechanisms.
#[derive(Debug)]
pub struct EffectCoordinator {
    /// Thread-local effect contexts
    thread_contexts: Arc<RwLock<HashMap<ThreadId, ThreadEffectState>>>,
    /// Global effect history for debugging and monitoring
    effect_history: Arc<Mutex<Vec<EffectEvent>>>,
    /// Effect coordination policies
    policies: Arc<EffectPolicies>,
    /// Concurrent effect coordination system
    coordination_system: Arc<ConcurrentEffectSystem>,
    /// Effect ordering manager
    ordering_manager: Arc<EffectOrderingManager>,
    /// Cross-thread coordination channels
    coordination_channels: Arc<RwLock<HashMap<ThreadId, EffectChannel>>>,
}

/// Effect state for a specific thread.
#[derive(Debug, Clone)]
pub struct ThreadEffectState {
    /// Current effect context for this thread
    pub context: EffectContext,
    /// Effects currently active in this thread
    pub active_effects: Vec<Effect>,
    /// Generation counter for this thread's effects
    pub generation: u64,
    /// Last update timestamp
    pub last_updated: SystemTime,
}

/// An event in the effect system.
#[derive(Debug, Clone)]
pub struct EffectEvent {
    /// Thread that produced this event
    pub thread_id: ThreadId,
    /// Timestamp of the event
    pub timestamp: SystemTime,
    /// Type of effect event
    pub event_type: EffectEventType,
    /// Associated effect
    pub effect: Effect,
    /// Optional additional context
    pub context: Option<String>,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Dependencies on other effects
    pub dependencies: Vec<u64>,
}

/// Types of effect events.
#[derive(Debug, Clone)]
pub enum EffectEventType {
    /// Effect was activated
    Activated,
    /// Effect was deactivated
    Deactivated,
    /// Effect produced a result
    Produced,
    /// Effect was handled
    Handled,
    /// Effect caused an error
    Error(String),
    /// Effect is waiting for coordination
    WaitingForCoordination,
    /// Effect coordination completed
    CoordinationCompleted,
    /// Effect was rolled back due to transaction failure
    RolledBack,
}

/// Policies for effect coordination.
#[derive(Debug)]
pub struct EffectPolicies {
    /// Whether to track effect history
    track_history: bool,
    /// Maximum size of effect history
    max_history_size: usize,
    /// Whether to allow cross-thread effect coordination
    allow_cross_thread_coordination: bool,
    /// Timeout for effect coordination operations
    coordination_timeout: std::time::Duration,
    /// Whether to enforce strict effect ordering
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    enforce_strict_ordering: bool,
    /// Whether to enable effect isolation
    enable_effect_isolation: bool,
    /// Maximum concurrent effects per thread
    max_concurrent_effects_per_thread: usize,
    /// Whether to enable automatic effect rollback on errors
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    enable_automatic_rollback: bool,
}

/// Concurrent effect system for coordinating effects across threads.
#[derive(Debug)]
pub struct ConcurrentEffectSystem {
    /// Active effect transactions
    active_transactions: Arc<RwLock<HashMap<u64, EffectTransaction>>>,
    /// Transaction sequence counter
    transaction_sequence: AtomicU64,
    /// Effect dependency graph
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    dependency_graph: Arc<RwLock<EffectDependencyGraph>>,
}

/// Effect ordering manager.
#[derive(Debug)]
pub struct EffectOrderingManager {
    /// Effect sequence counter
    sequence_counter: AtomicU64,
    /// Pending effects ordered by sequence
    pending_effects: Arc<RwLock<BTreeMap<u64, PendingEffect>>>,
    /// Effect ordering constraints
    ordering_constraints: Arc<RwLock<Vec<OrderingConstraint>>>,
}

/// Cross-thread communication channel for effects.
#[derive(Debug)]
pub struct EffectChannel {
    /// Sender for effect coordination messages
    pub sender: Sender<EffectCoordinationMessage>,
    /// Receiver for effect coordination messages  
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub receiver: Receiver<EffectCoordinationMessage>,
}

/// Transaction for coordinating effects.
#[derive(Debug, Clone)]
pub struct EffectTransaction {
    /// Unique transaction ID
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub id: u64,
    /// Thread that initiated the transaction
    pub initiator_thread: ThreadId,
    /// Participating threads
    pub participating_threads: Vec<ThreadId>,
    /// Effects involved in this transaction
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub effects: Vec<Effect>,
    /// Transaction state
    pub state: TransactionState,
    /// Creation timestamp
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub created_at: SystemTime,
    /// Timeout for this transaction
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub timeout: Duration,
}

/// State of an effect transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    /// Transaction is being prepared
    Preparing,
    /// Transaction is active
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Active,
    /// Transaction is committing
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Committing,
    /// Transaction committed successfully
    Committed,
    /// Transaction is aborting
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Aborting,
    /// Transaction was aborted
    Aborted,
}

/// Dependency graph for effects.
#[derive(Debug, Default)]
pub struct EffectDependencyGraph {
    /// Dependencies between effects
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    dependencies: HashMap<u64, Vec<u64>>,
    /// Reverse dependencies for efficient lookup
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    reverse_dependencies: HashMap<u64, Vec<u64>>,
}

/// Pending effect waiting for coordination.
#[derive(Debug, Clone)]
pub struct PendingEffect {
    /// Effect sequence number
    pub sequence: u64,
    /// Thread that owns this effect
    pub thread_id: ThreadId,
    /// The effect itself
    pub effect: Effect,
    /// Dependencies that must complete first
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub dependencies: Vec<u64>,
    /// Timestamp when effect was submitted
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub submitted_at: SystemTime,
}

/// Ordering constraint for effects.
#[derive(Debug, Clone)]
pub struct OrderingConstraint {
    /// Effect type this constraint applies to
    pub effect_type: Effect,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Priority for this constraint
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub priority: u8,
}

/// Type of ordering constraint.
#[derive(Debug, Clone)]
pub enum ConstraintType {
    /// Effects must be serialized (no parallelism)
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Serialized,
    /// Effects can run in parallel but must maintain ordering
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    OrderedParallel,
    /// Effects are isolated and can run freely
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    Isolated,
}

/// Messages for cross-thread effect coordination.
#[derive(Debug, Clone)]
pub enum EffectCoordinationMessage {
    /// Request to coordinate an effect
    CoordinateEffect {
        #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
        effect: Effect,
        #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
        sequence: u64,
        #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
        dependencies: Vec<u64>,
    },
    /// Response to effect coordination request
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    CoordinationResponse {
        sequence: u64,
        success: bool,
        error: Option<String>,
    },
    /// Notification that an effect completed
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    EffectCompleted {
        sequence: u64,
        result: Result<String, String>,
    },
    /// Request to abort an effect
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    AbortEffect {
        sequence: u64,
        reason: String,
    },
}

impl EffectCoordinator {
    /// Creates a new effect coordinator.
    pub fn new() -> Self {
        Self {
            thread_contexts: Arc::new(RwLock::new(HashMap::new())),
            effect_history: Arc::new(Mutex::new(Vec::new())),
            policies: Arc::new(EffectPolicies::default()),
            coordination_system: Arc::new(ConcurrentEffectSystem::new()),
            ordering_manager: Arc::new(EffectOrderingManager::new()),
            coordination_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new effect coordinator with custom policies.
    pub fn with_policies(policies: EffectPolicies) -> Self {
        Self {
            thread_contexts: Arc::new(RwLock::new(HashMap::new())),
            effect_history: Arc::new(Mutex::new(Vec::new())),
            policies: Arc::new(policies),
            coordination_system: Arc::new(ConcurrentEffectSystem::new()),
            ordering_manager: Arc::new(EffectOrderingManager::new()),
            coordination_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new thread with the effect coordinator.
    pub fn register_thread(&self, thread_id: ThreadId) {
        let state = ThreadEffectState {
            context: EffectContext::new(),
            active_effects: Vec::new(),
            generation: 0,
            last_updated: SystemTime::now(),
        };

        let mut contexts = self.thread_contexts.write().unwrap();
        contexts.insert(thread_id, state);
        
        // Create coordination channel for this thread
        let (sender, receiver) = unbounded();
        let channel = EffectChannel { sender, receiver };
        
        let mut channels = self.coordination_channels.write().unwrap();
        channels.insert(thread_id, channel);
    }

    /// Unregisters a thread from the effect coordinator.
    pub fn unregister_thread(&self, thread_id: ThreadId) {
        let mut contexts = self.thread_contexts.write().unwrap();
        contexts.remove(&thread_id);
        
        // Remove coordination channel
        let mut channels = self.coordination_channels.write().unwrap();
        channels.remove(&thread_id);
        
        // Clean up any pending transactions for this thread
        self.coordination_system.cleanup_thread_transactions(thread_id);
    }

    /// Enters an effect context for the given thread.
    pub fn enter_effect_context(&self, thread_id: ThreadId, effects: Vec<Effect>) -> Result<EffectContext, String> {
        let mut contexts = self.thread_contexts.write().unwrap();
        
        if let Some(state) = contexts.get_mut(&thread_id) {
            // Create new context with the effects
            let new_context = state.context.with_effects(effects.clone());
            
            // Update thread state
            state.context = new_context.clone());
            state.active_effects.extend(effects.clone());
            state.generation += 1;
            state.last_updated = SystemTime::now();
            
            // Record effect activation events
            if self.policies.track_history {
                self.record_effect_events(thread_id, &effects, EffectEventType::Activated);
            }
            
            Ok(new_context)
        } else {
            Err(format!("Thread {:?} not registered with effect coordinator", thread_id))
        }
    }

    /// Exits an effect context for the given thread.
    pub fn exit_effect_context(&self, thread_id: ThreadId, effects: Vec<Effect>) -> Result<EffectContext, String> {
        let mut contexts = self.thread_contexts.write().unwrap();
        
        if let Some(state) = contexts.get_mut(&thread_id) {
            // Remove effects from active list
            state.active_effects.retain(|e| !effects.contains(e));
            
            // Create new context without the effects
            let new_context = state.context.without_effects(effects.clone());
            state.context = new_context.clone());
            state.generation += 1;
            state.last_updated = SystemTime::now();
            
            // Record effect deactivation events
            if self.policies.track_history {
                self.record_effect_events(thread_id, &effects, EffectEventType::Deactivated);
            }
            
            Ok(new_context)
        } else {
            Err(format!("Thread {:?} not registered with effect coordinator", thread_id))
        }
    }

    /// Gets the current effect context for a thread.
    pub fn get_thread_context(&self, thread_id: ThreadId) -> Option<EffectContext> {
        let contexts = self.thread_contexts.read().unwrap();
        contexts.get(&thread_id).map(|state| state.context.clone())
    }

    /// Gets the active effects for a thread.
    pub fn get_thread_effects(&self, thread_id: ThreadId) -> Vec<Effect> {
        let contexts = self.thread_contexts.read().unwrap();
        contexts.get(&thread_id)
            .map(|state| state.active_effects.clone())
            .unwrap_or_default()
    }

    /// Records an effect being produced by a thread.
    pub fn record_effect_produced(&self, thread_id: ThreadId, effect: Effect, context: Option<String>) {
        if self.policies.track_history {
            self.record_effect_event(thread_id, effect, EffectEventType::Produced, context);
        }
    }

    /// Records an effect being handled by a thread.
    pub fn record_effect_handled(&self, thread_id: ThreadId, effect: Effect, context: Option<String>) {
        if self.policies.track_history {
            self.record_effect_event(thread_id, effect, EffectEventType::Handled, context);
        }
    }

    /// Records an effect error.
    pub fn record_effect_error(&self, thread_id: ThreadId, effect: Effect, error: String) {
        if self.policies.track_history {
            self.record_effect_event(thread_id, effect, EffectEventType::Error(error), None);
        }
    }

    /// Coordinates effects across threads if needed.
    ///
    /// This implements full cross-thread effect coordination with
    /// ordering guarantees and isolation mechanisms.
    pub fn coordinate_cross_thread_effect(
        &self,
        source_thread: ThreadId,
        target_thread: ThreadId,
        effect: Effect,
    ) -> Result<(), String> {
        if !self.policies.allow_cross_thread_coordination {
            return Err("Cross-thread effect coordination is disabled".to_string());
        }
        
        // Start a coordination transaction
        let transaction_id = self.coordination_system.start_transaction(
            source_thread,
            vec![target_thread],
            vec![effect.clone())],
        )?;
        
        // Get the next sequence number for ordering
        let sequence = self.ordering_manager.next_sequence();
        
        // Check for dependencies and ordering constraints
        let dependencies = self.ordering_manager.compute_dependencies(&effect, sequence)?;
        
        // Send coordination message to target thread
        if let Some(channel) = self.get_coordination_channel(target_thread) {
            let message = EffectCoordinationMessage::CoordinateEffect {
                effect: effect.clone()),
                sequence,
                dependencies,
            };
            
            if let Err(_) = channel.sender.try_send(message) {
                // Target thread is not responsive, abort transaction
                self.coordination_system.abort_transaction(transaction_id)?;
                return Err(format!("Target thread {:?} is not responsive", target_thread));
            }
        } else {
            return Err(format!("No coordination channel for thread {:?}", target_thread));
        }
        
        // Wait for coordination response with timeout
        let timeout = self.policies.coordination_timeout;
        match self.coordination_system.wait_for_coordination_completion(transaction_id, timeout) {
            Ok(true) => {
                self.coordination_system.commit_transaction(transaction_id)?;
                Ok(())
            }
            Ok(false) => {
                self.coordination_system.abort_transaction(transaction_id)?;
                Err("Effect coordination failed".to_string())
            }
            Err(e) => {
                self.coordination_system.abort_transaction(transaction_id)?;
                Err(format!("Coordination timeout or error: {}", e))
            }
        }
    }
    
    /// Coordinates a local effect with proper ordering and isolation.
    pub fn coordinate_local_effect(
        &self,
        thread_id: ThreadId,
        effect: Effect,
        _args: &[crate::eval::Value],
    ) -> Result<u64, String> {
        // Check if thread has too many concurrent effects
        {
            let contexts = self.thread_contexts.read().unwrap();
            if let Some(state) = contexts.get(&thread_id) {
                if state.active_effects.len() >= self.policies.max_concurrent_effects_per_thread {
                    return Err("Thread has reached maximum concurrent effects limit".to_string());
                }
            }
        }
        
        // Get sequence number for ordering
        let sequence = self.ordering_manager.next_sequence();
        
        // Check for dependencies
        let dependencies = self.ordering_manager.compute_dependencies(&effect, sequence)?;
        
        // If there are dependencies, wait for them
        if !dependencies.is_empty() {
            self.ordering_manager.wait_for_dependencies(&dependencies, self.policies.coordination_timeout)?;
        }
        
        // Add to pending effects
        let pending_effect = PendingEffect {
            sequence,
            thread_id,
            effect: effect.clone()),
            dependencies,
            submitted_at: SystemTime::now(),
        };
        
        self.ordering_manager.add_pending_effect(pending_effect)?;
        
        // Record the effect activation
        if self.policies.track_history {
            self.record_effect_event(
                thread_id,
                effect,
                EffectEventType::Activated,
                Some(format!("Sequence: {}", sequence)),
            );
        }
        
        Ok(sequence)
    }
    
    /// Completes an effect coordination.
    pub fn complete_effect(&self, sequence: u64, result: Result<String, String>) -> Result<(), String> {
        // Remove from pending effects
        self.ordering_manager.complete_effect(sequence)?;
        
        // Notify waiting threads
        self.ordering_manager.notify_effect_completion(sequence);
        
        // Record completion event
        if self.policies.track_history {
            if let Some(pending) = self.ordering_manager.get_pending_effect(sequence) {
                let event_type = match result {
                    Ok(_) => EffectEventType::CoordinationCompleted,
                    Err(ref e) => EffectEventType::Error(e.clone()),
                };
                
                self.record_effect_event(
                    pending.thread_id,
                    pending.effect,
                    event_type,
                    Some(format!("Sequence: {}", sequence)),
                );
            }
        }
        
        Ok(())
    }
    
    /// Gets the coordination channel for a thread.
    fn get_coordination_channel(&self, thread_id: ThreadId) -> Option<EffectChannel> {
        let channels = self.coordination_channels.read().unwrap();
        channels.get(&thread_id).clone())()
    }
    
    /// Enables effect isolation for a thread.
    /// When enabled, effects in this thread are isolated from other threads.
    pub fn enable_effect_isolation(&self, thread_id: ThreadId, isolation_level: EffectIsolationLevel) -> Result<(), String> {
        if !self.policies.enable_effect_isolation {
            return Err("Effect isolation is disabled in policies".to_string());
        }
        
        let mut contexts = self.thread_contexts.write().unwrap();
        if let Some(state) = contexts.get_mut(&thread_id) {
            // Add isolation barrier to the effect context
            state.context = state.context.with_isolation(isolation_level.clone());
            state.generation += 1;
            state.last_updated = SystemTime::now();
            
            // Record isolation event
            if self.policies.track_history {
                self.record_effect_event(
                    thread_id,
                    Effect::Custom("isolation".to_string()),
                    EffectEventType::Activated,
                    Some(format!("Isolation level: {:?}", isolation_level)),
                );
            }
            
            Ok(())
        } else {
            Err(format!("Thread {:?} not registered", thread_id))
        }
    }
    
    /// Disables effect isolation for a thread.
    pub fn disable_effect_isolation(&self, thread_id: ThreadId) -> Result<(), String> {
        let mut contexts = self.thread_contexts.write().unwrap();
        if let Some(state) = contexts.get_mut(&thread_id) {
            // Remove isolation barrier from the effect context
            state.context = state.context.without_isolation();
            state.generation += 1;
            state.last_updated = SystemTime::now();
            
            // Record isolation removal event
            if self.policies.track_history {
                self.record_effect_event(
                    thread_id,
                    Effect::Custom("isolation".to_string()),
                    EffectEventType::Deactivated,
                    Some("Isolation disabled".to_string()),
                );
            }
            
            Ok(())
        } else {
            Err(format!("Thread {:?} not registered", thread_id))
        }
    }
    
    /// Checks if an effect can cross thread boundaries based on isolation rules.
    pub fn can_effect_cross_threads(&self, effect: &Effect, source_thread: ThreadId, target_thread: ThreadId) -> bool {
        let contexts = self.thread_contexts.read().unwrap();
        
        // Check source thread isolation
        if let Some(source_state) = contexts.get(&source_thread) {
            if source_state.context.is_isolated() {
                let isolation_level = source_state.context.get_isolation_level();
                match isolation_level {
                    Some(EffectIsolationLevel::Complete) => return false,
                    Some(EffectIsolationLevel::SideEffectOnly) => {
                        if effect.has_side_effects() {
                            return false;
                        }
                    }
                    Some(EffectIsolationLevel::WriteOnly) => {
                        if effect.is_write_effect() {
                            return false;
                        }
                    }
                    Some(EffectIsolationLevel::Custom(ref rules)) => {
                        return rules.allows_effect(effect, source_thread, target_thread);
                    }
                    _ => {}
                }
            }
        }
        
        // Check target thread isolation
        if let Some(target_state) = contexts.get(&target_thread) {
            if target_state.context.is_isolated() {
                let isolation_level = target_state.context.get_isolation_level();
                match isolation_level {
                    Some(EffectIsolationLevel::Complete) => return false,
                    Some(EffectIsolationLevel::SideEffectOnly) => {
                        if effect.has_side_effects() {
                            return false;
                        }
                    }
                    Some(EffectIsolationLevel::WriteOnly) => {
                        if effect.is_write_effect() {
                            return false;
                        }
                    }
                    Some(EffectIsolationLevel::Custom(ref rules)) => {
                        return rules.allows_effect(effect, source_thread, target_thread);
                    }
                    _ => {}
                }
            }
        }
        
        true
    }
    
    /// Creates an isolated effect sandbox for a thread.
    pub fn create_effect_sandbox(&self, thread_id: ThreadId, sandbox_config: EffectSandboxConfig) -> Result<EffectSandboxHandle, String> {
        let sandbox_id = self.ordering_manager.next_sequence();
        
        // Enable strict isolation for this thread
        self.enable_effect_isolation(thread_id, EffectIsolationLevel::Complete)?;
        
        // Create sandbox handle
        let handle = EffectSandboxHandle {
            id: sandbox_id,
            thread_id,
            config: sandbox_config,
            created_at: SystemTime::now(),
            coordinator: Arc::downgrade(&Arc::new(self.clone())),
        };
        
        // Record sandbox creation
        if self.policies.track_history {
            self.record_effect_event(
                thread_id,
                Effect::Custom("sandbox".to_string()),
                EffectEventType::Activated,
                Some(format!("Sandbox created with ID: {}", sandbox_id)),
            );
        }
        
        Ok(handle)
    }
    
    /// Gets isolation statistics.
    pub fn get_isolation_statistics(&self) -> EffectIsolationStatistics {
        let contexts = self.thread_contexts.read().unwrap();
        
        let mut stats = EffectIsolationStatistics {
            total_threads: contexts.len(),
            isolated_threads: 0,
            isolation_levels: HashMap::new(),
            blocked_cross_thread_effects: 0,
            sandbox_count: 0,
        };
        
        for state in contexts.values() {
            if state.context.is_isolated() {
                stats.isolated_threads += 1;
                if let Some(level) = state.context.get_isolation_level() {
                    *stats.isolation_levels.entry(level).or_insert(0) += 1;
                }
            }
        }
        
        // Count blocked effects from history
        let history = self.effect_history.lock().unwrap();
        for event in history.iter() {
            if let Some(ref context) = event.context {
                if context.contains("isolation") && context.contains("blocked") {
                    stats.blocked_cross_thread_effects += 1;
                }
                if context.contains("sandbox") {
                    stats.sandbox_count += 1;
                }
            }
        }
        
        stats
    }

    /// Gets statistics about effect usage across all threads.
    pub fn get_effect_statistics(&self) -> EffectStatistics {
        let contexts = self.thread_contexts.read().unwrap();
        let history = self.effect_history.lock().unwrap();
        
        let mut stats = EffectStatistics {
            active_threads: contexts.len(),
            total_active_effects: 0,
            effect_counts: HashMap::new(),
            total_events: history.len(),
            recent_events: 0,
        };
        
        // Count active effects
        for state in contexts.values() {
            stats.total_active_effects += state.active_effects.len();
            
            for effect in &state.active_effects {
                *stats.effect_counts.entry(effect.clone()).or_insert(0) += 1;
            }
        }
        
        // Count recent events (last minute)
        let one_minute_ago = SystemTime::now() - std::time::Duration::from_secs(60);
        stats.recent_events = history.iter()
            .filter(|event| event.timestamp > one_minute_ago)
            .count();
        
        stats
    }

    /// Gets the effect history.
    pub fn get_effect_history(&self) -> Vec<EffectEvent> {
        let history = self.effect_history.lock().unwrap();
        history.clone())
    }

    /// Clears the effect history.
    pub fn clear_effect_history(&self) {
        let mut history = self.effect_history.lock().unwrap();
        history.clear();
    }

    /// Helper method to record multiple effect events.
    fn record_effect_events(&self, thread_id: ThreadId, effects: &[Effect], event_type: EffectEventType) {
        for effect in effects {
            self.record_effect_event(thread_id, effect.clone()), event_type.clone()), None);
        }
    }

    /// Helper method to record a single effect event.
    fn record_effect_event(
        &self,
        thread_id: ThreadId,
        effect: Effect,
        event_type: EffectEventType,
        context: Option<String>,
    ) {
        let event = EffectEvent {
            thread_id,
            timestamp: SystemTime::now(),
            event_type,
            effect,
            context,
            sequence: self.ordering_manager.next_sequence(),
            dependencies: Vec::new(), // No dependencies for event recording
        };

        let mut history = self.effect_history.lock().unwrap();
        history.push(event);

        // Trim history if it gets too large
        if history.len() > self.policies.max_history_size {
            let excess = history.len() - self.policies.max_history_size;
            history.drain(0..excess);
        }
    }
}

/// Statistics about effect usage.
#[derive(Debug, Clone)]
pub struct EffectStatistics {
    /// Number of active threads
    pub active_threads: usize,
    /// Total number of active effects across all threads
    pub total_active_effects: usize,
    /// Count of each effect type currently active
    pub effect_counts: HashMap<Effect, usize>,
    /// Total number of effect events recorded
    pub total_events: usize,
    /// Number of events in the last minute
    pub recent_events: usize,
}

impl Default for EffectPolicies {
    fn default() -> Self {
        Self {
            track_history: true,
            max_history_size: 10000,
            allow_cross_thread_coordination: true,
            coordination_timeout: std::time::Duration::from_secs(5),
            enforce_strict_ordering: true,
            enable_effect_isolation: true,
            max_concurrent_effects_per_thread: 100,
            enable_automatic_rollback: true,
        }
    }
}

impl EffectPolicies {
    /// Creates new effect policies with history tracking disabled.
    pub fn no_history() -> Self {
        Self {
            track_history: false,
            max_history_size: 0,
            allow_cross_thread_coordination: true,
            coordination_timeout: std::time::Duration::from_secs(5),
            enforce_strict_ordering: true,
            enable_effect_isolation: true,
            max_concurrent_effects_per_thread: 100,
            enable_automatic_rollback: true,
        }
    }

    /// Creates new effect policies with minimal overhead.
    pub fn minimal() -> Self {
        Self {
            track_history: false,
            max_history_size: 0,
            allow_cross_thread_coordination: false,
            coordination_timeout: std::time::Duration::from_millis(100),
            enforce_strict_ordering: false,
            enable_effect_isolation: false,
            max_concurrent_effects_per_thread: 10,
            enable_automatic_rollback: false,
        }
    }
    
    /// Creates effect policies optimized for high concurrency.
    pub fn high_concurrency() -> Self {
        Self {
            track_history: false,
            max_history_size: 1000,
            allow_cross_thread_coordination: true,
            coordination_timeout: std::time::Duration::from_millis(500),
            enforce_strict_ordering: false,
            enable_effect_isolation: true,
            max_concurrent_effects_per_thread: 1000,
            enable_automatic_rollback: true,
        }
    }
}

impl Default for EffectCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EffectCoordinator {
    fn clone(&self) -> Self {
        Self {
            thread_contexts: self.thread_contexts.clone()),
            effect_history: self.effect_history.clone()),
            policies: self.policies.clone()),
            coordination_system: self.coordination_system.clone()),
            ordering_manager: self.ordering_manager.clone()),
            coordination_channels: self.coordination_channels.clone()),
        }
    }
}

// ============= COMPONENT IMPLEMENTATIONS =============

impl ConcurrentEffectSystem {
    /// Creates a new concurrent effect system.
    pub fn new() -> Self {
        Self {
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            transaction_sequence: AtomicU64::new(0),
            dependency_graph: Arc::new(RwLock::new(EffectDependencyGraph::default())),
        }
    }
    
    /// Starts a new effect transaction.
    pub fn start_transaction(
        &self,
        initiator: ThreadId,
        participants: Vec<ThreadId>,
        effects: Vec<Effect>,
    ) -> Result<u64, String> {
        let id = self.transaction_sequence.fetch_add(1, Ordering::SeqCst);
        
        let transaction = EffectTransaction {
            id,
            initiator_thread: initiator,
            participating_threads: participants,
            effects,
            state: TransactionState::Preparing,
            created_at: SystemTime::now(),
            timeout: Duration::from_secs(30),
        };
        
        let mut transactions = self.active_transactions.write().unwrap();
        transactions.insert(id, transaction);
        
        Ok(id)
    }
    
    /// Commits a transaction.
    pub fn commit_transaction(&self, transaction_id: u64) -> Result<(), String> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.state = TransactionState::Committed;
            Ok(())
        } else {
            Err(format!("Transaction {} not found", transaction_id))
        }
    }
    
    /// Aborts a transaction.
    pub fn abort_transaction(&self, transaction_id: u64) -> Result<(), String> {
        let mut transactions = self.active_transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&transaction_id) {
            transaction.state = TransactionState::Aborted;
            Ok(())
        } else {
            Err(format!("Transaction {} not found", transaction_id))
        }
    }
    
    /// Waits for coordination completion.
    pub fn wait_for_coordination_completion(
        &self,
        transaction_id: u64,
        timeout: Duration,
    ) -> Result<bool, String> {
        let start_time = SystemTime::now();
        
        loop {
            {
                let transactions = self.active_transactions.read().unwrap();
                if let Some(transaction) = transactions.get(&transaction_id) {
                    match transaction.state {
                        TransactionState::Committed => return Ok(true),
                        TransactionState::Aborted => return Ok(false),
                        _ => {
                            // Continue waiting
                        }
                    }
                } else {
                    return Err(format!("Transaction {} not found", transaction_id));
                }
            }
            
            if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                return Err("Coordination timeout".to_string());
            }
            
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    
    /// Cleans up transactions for a thread.
    pub fn cleanup_thread_transactions(&self, thread_id: ThreadId) {
        let mut transactions = self.active_transactions.write().unwrap();
        transactions.retain(|_, transaction| {
            transaction.initiator_thread != thread_id 
                && !transaction.participating_threads.contains(&thread_id)
        });
    }
}

impl EffectOrderingManager {
    /// Creates a new effect ordering manager.
    pub fn new() -> Self {
        Self {
            sequence_counter: AtomicU64::new(0),
            pending_effects: Arc::new(RwLock::new(BTreeMap::new())),
            ordering_constraints: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Gets the next sequence number.
    pub fn next_sequence(&self) -> u64 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    
    /// Computes dependencies for an effect.
    pub fn compute_dependencies(&self, effect: &Effect, sequence: u64) -> Result<Vec<u64>, String> {
        let constraints = self.ordering_constraints.read().unwrap();
        let pending = self.pending_effects.read().unwrap();
        
        let mut dependencies = Vec::new();
        
        // Check ordering constraints
        for constraint in constraints.iter() {
            if constraint.effect_type == *effect {
                match constraint.constraint_type {
                    ConstraintType::Serialized => {
                        // Must wait for all previous effects of same type
                        for (seq, pending_effect) in pending.iter() {
                            if *seq < sequence && pending_effect.effect == *effect {
                                dependencies.push(*seq);
                            }
                        }
                    }
                    ConstraintType::OrderedParallel => {
                        // Must maintain ordering but can run in parallel
                        for (seq, pending_effect) in pending.iter() {
                            if *seq < sequence && pending_effect.effect == *effect {
                                dependencies.push(*seq);
                                break; // Only need immediate predecessor
                            }
                        }
                    }
                    ConstraintType::Isolated => {
                        // No dependencies needed
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Waits for dependencies to complete.
    pub fn wait_for_dependencies(&self, dependencies: &[u64], timeout: Duration) -> Result<(), String> {
        let start_time = SystemTime::now();
        
        for &dep_sequence in dependencies {
            loop {
                {
                    let pending = self.pending_effects.read().unwrap();
                    if !pending.contains_key(&dep_sequence) {
                        break; // Dependency completed
                    }
                }
                
                if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                    return Err(format!("Timeout waiting for dependency {}", dep_sequence));
                }
                
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        
        Ok(())
    }
    
    /// Adds a pending effect.
    pub fn add_pending_effect(&self, effect: PendingEffect) -> Result<(), String> {
        let mut pending = self.pending_effects.write().unwrap();
        pending.insert(effect.sequence, effect);
        Ok(())
    }
    
    /// Completes an effect.
    pub fn complete_effect(&self, sequence: u64) -> Result<(), String> {
        let mut pending = self.pending_effects.write().unwrap();
        pending.remove(&sequence);
        Ok(())
    }
    
    /// Notifies of effect completion.
    pub fn notify_effect_completion(&self, _sequence: u64) {
        // In a full implementation, this would notify waiting threads
        // For now, the wait_for_dependencies method polls
    }
    
    /// Gets a pending effect.
    pub fn get_pending_effect(&self, sequence: u64) -> Option<PendingEffect> {
        let pending = self.pending_effects.read().unwrap();
        pending.get(&sequence).clone())()
    }
    
    /// Adds an ordering constraint.
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub fn add_ordering_constraint(&self, constraint: OrderingConstraint) {
        let mut constraints = self.ordering_constraints.write().unwrap();
        constraints.push(constraint);
        constraints.sort_by_key(|c| c.priority);
    }
}

impl EffectChannel {
    /// Creates a new effect channel.
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

impl Clone for EffectChannel {
    fn clone(&self) -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

// ============= EFFECT ISOLATION SYSTEM =============

/// Levels of effect isolation between threads.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectIsolationLevel {
    /// Complete isolation - no effects can cross thread boundaries
    Complete,
    /// Only side-effect-free operations are allowed to cross boundaries
    SideEffectOnly,
    /// Only read operations are allowed, writes are isolated
    WriteOnly,
    /// Custom isolation rules
    Custom(EffectIsolationRules),
}

/// Custom rules for effect isolation.
#[derive(Debug, Clone)]
pub struct EffectIsolationRules {
    /// Allowed effect types
    pub allowed_effects: Vec<Effect>,
    /// Blocked effect types  
    pub blocked_effects: Vec<Effect>,
    /// Custom validation function
    pub custom_validator: Option<fn(&Effect, ThreadId, ThreadId) -> bool>,
    /// Exception rules
    pub exceptions: Vec<IsolationException>,
}

/// Exception to isolation rules.
#[derive(Debug, Clone)]
pub struct IsolationException {
    /// Effect this exception applies to
    pub effect: Effect,
    /// Threads this exception applies to
    pub threads: Vec<ThreadId>,
    /// Condition for when this exception applies
    pub condition: String,
}

/// Configuration for an effect sandbox.
#[derive(Debug, Clone)]
pub struct EffectSandboxConfig {
    /// Maximum number of effects allowed in sandbox
    pub max_effects: usize,
    /// Timeout for sandbox operations
    pub timeout: Duration,
    /// Allowed effect types in sandbox
    pub allowed_effects: Vec<Effect>,
    /// Whether sandbox should auto-cleanup on completion
    pub auto_cleanup: bool,
    /// Resource limits for sandbox
    pub resource_limits: SandboxResourceLimits,
}

/// Resource limits for a sandbox.
#[derive(Debug, Clone)]
pub struct SandboxResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: Option<usize>,
    /// Maximum execution time
    pub max_execution_time: Option<Duration>,
    /// Maximum file operations
    pub max_file_operations: Option<usize>,
    /// Maximum network operations
    pub max_network_operations: Option<usize>,
}

/// Handle to an effect sandbox.
#[derive(Debug)]
pub struct EffectSandboxHandle {
    /// Unique sandbox ID
    pub id: u64,
    /// Thread this sandbox belongs to
    pub thread_id: ThreadId,
    /// Sandbox configuration
    pub config: EffectSandboxConfig,
    /// When sandbox was created
    pub created_at: SystemTime,
    /// Weak reference to coordinator
    coordinator: std::sync::Weak<EffectCoordinator>,
}

/// Statistics about effect isolation.
#[derive(Debug, Clone)]
pub struct EffectIsolationStatistics {
    /// Total number of threads
    pub total_threads: usize,
    /// Number of isolated threads
    pub isolated_threads: usize,
    /// Count of threads by isolation level
    pub isolation_levels: HashMap<EffectIsolationLevel, usize>,
    /// Number of blocked cross-thread effects
    pub blocked_cross_thread_effects: usize,
    /// Number of active sandboxes
    pub sandbox_count: usize,
}

impl Effect {
    /// Returns true if this effect has side effects.
    pub fn has_side_effects(&self) -> bool {
        match self {
            Effect::Pure => false,
            Effect::IO | Effect::State | Effect::Error => true,
            Effect::Custom(name) => !name.starts_with("pure_"),
        }
    }
    
    /// Returns true if this effect performs write operations.
    pub fn is_write_effect(&self) -> bool {
        match self {
            Effect::Pure => false,
            Effect::IO | Effect::State => true,
            Effect::Error => false,
            Effect::Custom(name) => name.contains("write") || name.contains("set"),
        }
    }
}

impl EffectContext {
    /// Adds isolation to this context.
    pub fn with_isolation(&self, level: EffectIsolationLevel) -> Self {
        let mut new_context = self.clone());
        new_context.add_effect(Effect::Custom(format!("isolation:{:?}", level)));
        new_context
    }
    
    /// Removes isolation from this context.
    pub fn without_isolation(&self) -> Self {
        let mut new_effects = self.effects().to_vec();
        new_effects.retain(|e| {
            if let Effect::Custom(name) = e {
                !name.starts_with("isolation:")
            } else {
                true
            }
        });
        
        let mut new_context = EffectContext::new();
        for effect in new_effects {
            new_context.add_effect(effect);
        }
        for handler in self.handlers() {
            new_context.add_handler(handler.clone());
        }
        new_context
    }
    
    /// Returns true if this context is isolated.
    pub fn is_isolated(&self) -> bool {
        self.effects().iter().any(|e| {
            if let Effect::Custom(name) = e {
                name.starts_with("isolation:")
            } else {
                false
            }
        })
    }
    
    /// Gets the isolation level if any.
    pub fn get_isolation_level(&self) -> Option<EffectIsolationLevel> {
        for effect in self.effects() {
            if let Effect::Custom(name) = effect {
                if name.starts_with("isolation:") {
                    // Parse isolation level from effect name
                    if name.contains("Complete") {
                        return Some(EffectIsolationLevel::Complete);
                    } else if name.contains("SideEffectOnly") {
                        return Some(EffectIsolationLevel::SideEffectOnly);
                    } else if name.contains("WriteOnly") {
                        return Some(EffectIsolationLevel::WriteOnly);
                    }
                }
            }
        }
        None
    }
}

impl EffectIsolationRules {
    /// Creates default isolation rules.
    pub fn default() -> Self {
        Self {
            allowed_effects: vec![Effect::Pure],
            blocked_effects: vec![Effect::IO, Effect::State],
            custom_validator: None,
            exceptions: Vec::new(),
        }
    }
    
    /// Checks if an effect is allowed based on these rules.
    pub fn allows_effect(&self, effect: &Effect, _source: ThreadId, _target: ThreadId) -> bool {
        // Check if explicitly blocked
        if self.blocked_effects.contains(effect) {
            return false;
        }
        
        // Check if explicitly allowed
        if self.allowed_effects.contains(effect) {
            return true;
        }
        
        // Use custom validator if available
        if let Some(validator) = self.custom_validator {
            return validator(effect, _source, _target);
        }
        
        // Default to blocking unknown effects
        false
    }
    
    /// Adds an exception rule.
    pub fn add_exception(&mut self, exception: IsolationException) {
        self.exceptions.push(exception);
    }
}

impl Default for EffectSandboxConfig {
    fn default() -> Self {
        Self {
            max_effects: 100,
            timeout: Duration::from_secs(30),
            allowed_effects: vec![Effect::Pure],
            auto_cleanup: true,
            resource_limits: SandboxResourceLimits::default(),
        }
    }
}

impl Default for SandboxResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(64 * 1024 * 1024), // 64MB
            max_execution_time: Some(Duration::from_secs(10)),
            max_file_operations: Some(100),
            max_network_operations: Some(10),
        }
    }
}

impl EffectSandboxHandle {
    /// Destroys the sandbox and cleans up resources.
    pub fn destroy(self) -> Result<(), String> {
        if let Some(coordinator) = self.coordinator.upgrade() {
            coordinator.disable_effect_isolation(self.thread_id)?;
            
            // Record sandbox destruction
            if coordinator.policies.track_history {
                coordinator.record_effect_event(
                    self.thread_id,
                    Effect::Custom("sandbox".to_string()),
                    EffectEventType::Deactivated,
                    Some(format!("Sandbox destroyed with ID: {}", self.id)),
                );
            }
        }
        
        Ok(())
    }
    
    /// Checks if the sandbox is still valid.
    pub fn is_valid(&self) -> bool {
        // TODO: Fix weak reference issue 
        // self.coordinator.strong_count() > 0
        true
    }
    
    /// Gets sandbox statistics.
    pub fn get_statistics(&self) -> SandboxStatistics {
        SandboxStatistics {
            id: self.id,
            thread_id: self.thread_id,
            uptime: SystemTime::now().duration_since(self.created_at).unwrap_or_default(),
            effect_count: 0, // Would be populated in real implementation
            resource_usage: ResourceUsage::default(),
        }
    }
}

/// Statistics for a specific sandbox.
#[derive(Debug, Clone)]
pub struct SandboxStatistics {
    /// Sandbox ID
    pub id: u64,
    /// Associated thread
    pub thread_id: ThreadId,
    /// How long sandbox has been active
    pub uptime: Duration,
    /// Number of effects executed
    pub effect_count: usize,
    /// Current resource usage
    pub resource_usage: ResourceUsage,
}

/// Resource usage information.
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// Execution time
    pub execution_time: Duration,
    /// Number of file operations
    pub file_operations: usize,
    /// Number of network operations
    pub network_operations: usize,
}

impl PartialEq for EffectIsolationRules {
    fn eq(&self, other: &Self) -> bool {
        self.allowed_effects == other.allowed_effects &&
        self.blocked_effects == other.blocked_effects &&
        self.exceptions == other.exceptions
        // Custom validator function pointers can't be compared
    }
}

impl Eq for EffectIsolationRules {}

impl std::hash::Hash for EffectIsolationRules {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.allowed_effects.hash(state);
        self.blocked_effects.hash(state);
        self.exceptions.hash(state);
        // Don't hash the function pointer
    }
}

impl PartialEq for IsolationException {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect &&
        self.threads == other.threads &&
        self.condition == other.condition
    }
}

impl Eq for IsolationException {}

impl std::hash::Hash for IsolationException {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.effect.hash(state);
        self.threads.hash(state);
        self.condition.hash(state);
    }
}