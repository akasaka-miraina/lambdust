//! Main effect coordinator for multithreaded evaluation.
//!
//! This module contains the core EffectCoordinator that manages effects 
//! across multiple evaluator threads, ensuring proper effect handling 
//! and maintaining effect semantics in a concurrent environment.

use crate::effects::{Effect, EffectContext};
use std::sync::{Arc, RwLock, Mutex};
use std::thread::ThreadId;
use std::collections::HashMap;
use std::time::SystemTime;
use crossbeam::channel::{Sender, Receiver, unbounded};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::thread_effect_state::ThreadEffectState;
use super::effect_event::{EffectEvent, EffectEventType};
use super::effect_policies::EffectPolicies;
use super::concurrent_effect_system::ConcurrentEffectSystem;
use super::effect_ordering_manager::EffectOrderingManager;
use super::effect_channel::EffectChannel;
use super::effect_statistics::EffectStatistics;
use super::effect_isolation_level::EffectIsolationLevel;
use super::effect_sandbox_config::{EffectSandboxConfig, EffectSandboxHandle};
use super::effect_isolation_statistics::EffectIsolationStatistics;

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
            state.context = new_context.clone();
            state.active_effects.extend(effects.clone());
            state.generation += 1;
            state.last_updated = SystemTime::now();
            
            // Record effect activation events
            if self.policies.track_history {
                self.record_effect_events(thread_id, &effects, EffectEventType::Activated);
            }
            
            Ok(new_context)
        } else {
            Err(format!("Thread {thread_id:?} not registered with effect coordinator"))
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
            state.context = new_context.clone();
            state.generation += 1;
            state.last_updated = SystemTime::now();
            
            // Record effect deactivation events
            if self.policies.track_history {
                self.record_effect_events(thread_id, &effects, EffectEventType::Deactivated);
            }
            
            Ok(new_context)
        } else {
            Err(format!("Thread {thread_id:?} not registered with effect coordinator"))
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
            vec![effect.clone()],
        )?;
        
        // Get the next sequence number for ordering
        let sequence = self.ordering_manager.next_sequence();
        
        // Check for dependencies and ordering constraints
        let dependencies = self.ordering_manager.compute_dependencies(&effect, sequence)?;
        
        // Send coordination message to target thread
        if let Some(channel) = self.get_coordination_channel(target_thread) {
            let message = super::effect_coordination_message::EffectCoordinationMessage::CoordinateEffect {
                effect: effect.clone(),
                sequence,
                dependencies,
            };
            
            if channel.sender.try_send(message).is_err() {
                // Target thread is not responsive, abort transaction
                self.coordination_system.abort_transaction(transaction_id)?;
                return Err(format!("Target thread {target_thread:?} is not responsive"));
            }
        } else {
            return Err(format!("No coordination channel for thread {target_thread:?}"));
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
                Err(format!("Coordination timeout or error: {e}"))
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
        let pending_effect = super::pending_effect::PendingEffect {
            sequence,
            thread_id,
            effect: effect.clone(),
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
                Some(format!("Sequence: {sequence}")),
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
                    Some(format!("Sequence: {sequence}")),
                );
            }
        }
        
        Ok(())
    }
    
    /// Gets the coordination channel for a thread.
    fn get_coordination_channel(&self, thread_id: ThreadId) -> Option<EffectChannel> {
        let channels = self.coordination_channels.read().unwrap();
        channels.get(&thread_id).cloned()
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
                    Some(format!("Isolation level: {isolation_level:?}")),
                );
            }
            
            Ok(())
        } else {
            Err(format!("Thread {thread_id:?} not registered"))
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
            Err(format!("Thread {thread_id:?} not registered"))
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
            coordinator: std::sync::Weak::new(), // Simplified to avoid circular dependency
        };
        
        // Record sandbox creation
        if self.policies.track_history {
            self.record_effect_event(
                thread_id,
                Effect::Custom("sandbox".to_string()),
                EffectEventType::Activated,
                Some(format!("Sandbox created with ID: {sandbox_id}")),
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
        history.clone()
    }

    /// Clears the effect history.
    pub fn clear_effect_history(&self) {
        let mut history = self.effect_history.lock().unwrap();
        history.clear();
    }

    /// Helper method to record multiple effect events.
    fn record_effect_events(&self, thread_id: ThreadId, effects: &[Effect], event_type: EffectEventType) {
        for effect in effects {
            self.record_effect_event(thread_id, effect.clone(), event_type.clone(), None);
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

impl Default for EffectCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EffectCoordinator {
    fn clone(&self) -> Self {
        Self {
            thread_contexts: self.thread_contexts.clone(),
            effect_history: self.effect_history.clone(),
            policies: self.policies.clone(),
            coordination_system: self.coordination_system.clone(),
            ordering_manager: self.ordering_manager.clone(),
            coordination_channels: self.coordination_channels.clone(),
        }
    }
}