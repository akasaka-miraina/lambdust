//! Effect ordering manager for maintaining effect execution order.

use crate::effects::Effect;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use super::pending_effect::PendingEffect;
use super::ordering_constraint::{OrderingConstraint, ConstraintType};

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
        let start_time = std::time::SystemTime::now();
        
        for &dep_sequence in dependencies {
            loop {
                {
                    let pending = self.pending_effects.read().unwrap();
                    if !pending.contains_key(&dep_sequence) {
                        break; // Dependency completed
                    }
                }
                
                if start_time.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                    return Err(format!("Timeout waiting for dependency {dep_sequence}"));
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
        pending.get(&sequence).cloned()
    }
    
    /// Adds an ordering constraint.
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub fn add_ordering_constraint(&self, constraint: OrderingConstraint) {
        let mut constraints = self.ordering_constraints.write().unwrap();
        constraints.push(constraint);
        constraints.sort_by_key(|c| c.priority);
    }
}

impl Default for EffectOrderingManager {
    fn default() -> Self {
        Self::new()
    }
}