//! Generational environment system for handling mutations.
//!
//! This module implements a generational approach to environment management
//! where mutations create new environment generations rather than modifying
//! existing ones in place. This preserves referential transparency while
//! allowing efficient mutation through Rc-based sharing.

#![allow(missing_docs)]

use super::{Effect};
use crate::eval::value::{ThreadSafeEnvironment, Value, Generation};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Weak, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global generation counter for creating unique generation IDs.
static GENERATION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a unique generation ID.
fn next_generation() -> Generation {
    GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Manager for generational environments.
///
/// This coordinates the creation, tracking, and cleanup of environment generations,
/// ensuring that mutations are handled efficiently while preserving immutability
/// guarantees for pure computations.
#[derive(Debug, Clone)]
pub struct GenerationalEnvManager {
    /// Current active generation
    current_generation: Generation,
    /// History of generations for garbage collection
    generation_history: VecDeque<GenerationRecord>,
    /// Maximum number of generations to keep in history
    max_history: usize,
    /// Weak references to environments for cleanup (thread-safe)
    environment_registry: Arc<RwLock<HashMap<Generation, Vec<Weak<GenerationalEnvironment>>>>>,
}

/// Record of a generation for tracking and cleanup.
#[derive(Debug, Clone)]
pub struct GenerationRecord {
    /// Generation number
    generation: Generation,
    /// Number of environments in this generation
    env_count: usize,
    /// Effects that caused this generation to be created
    causing_effects: Vec<Effect>,
    /// Timestamp when this generation was created
    #[allow(dead_code)]
    created_at: std::time::Instant,
    /// Parent generation (if any)
    #[allow(dead_code)]
    parent_generation: Option<Generation>,
}

/// A generational environment that tracks its generation and provides
/// immutable mutation semantics (thread-safe).
#[derive(Debug)]
pub struct GenerationalEnvironment {
    /// The underlying thread-safe environment
    inner: Arc<ThreadSafeEnvironment>,
    /// Generation this environment belongs to
    generation: Generation,
    /// Parent generation environment (if any)
    parent: Option<Arc<GenerationalEnvironment>>,
    /// Manager that created this environment
    manager: GenerationalEnvManager,
}

/// Configuration for the generational environment system.
#[derive(Debug, Clone)]
pub struct GenerationalConfig {
    /// Maximum number of generations to keep in history
    pub max_history: usize,
    /// Whether to automatically clean up old generations
    pub auto_cleanup: bool,
    /// Threshold for triggering cleanup (number of generations)
    pub cleanup_threshold: usize,
    /// Whether to track generation statistics
    pub track_stats: bool,
}

/// Statistics about generation usage.
#[derive(Debug, Clone)]
pub struct GenerationStats {
    /// Total number of generations created
    pub total_generations: u64,
    /// Number of active generations
    pub active_generations: usize,
    /// Number of environments in current generation
    pub current_env_count: usize,
    /// Memory usage estimate (simplified)
    pub estimated_memory_kb: usize,
}

impl GenerationalEnvManager {
    /// Creates a new generational environment manager.
    pub fn new() -> Self {
        Self {
            current_generation: 0,
            generation_history: VecDeque::new(),
            max_history: 100, // Default limit
            environment_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Creates a new manager with the given configuration.
    pub fn with_config(config: GenerationalConfig) -> Self {
        Self {
            current_generation: 0,
            generation_history: VecDeque::new(),
            max_history: config.max_history,
            environment_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Gets the current generation number.
    pub fn current_generation(&self) -> Generation {
        self.current_generation
    }
    
    /// Creates a new generation for a mutation.
    pub fn new_generation(&mut self, effects: Vec<Effect>) -> Generation {
        let new_gen = next_generation();
        
        // Record the new generation
        let record = GenerationRecord {
            generation: new_gen,
            env_count: 0,
            causing_effects: effects,
            created_at: std::time::Instant::now(),
            parent_generation: if self.current_generation == 0 { 
                None 
            } else { 
                Some(self.current_generation) 
            },
        };
        
        self.generation_history.push_back(record);
        
        // Cleanup old generations if necessary
        if self.generation_history.len() > self.max_history {
            self.cleanup_old_generations();
        }
        
        self.current_generation = new_gen;
        new_gen
    }
    
    /// Creates a new generational environment.
    pub fn create_environment(
        &mut self, 
        parent: Option<Arc<ThreadSafeEnvironment>>,
        effects: Vec<Effect>
    ) -> Arc<GenerationalEnvironment> {
        let generation = if effects.iter().any(|e| e.is_state()) {
            // State effects require a new generation
            self.new_generation(effects)
        } else {
            // Pure operations can use the current generation
            self.current_generation
        };
        
        let env = Arc::new(ThreadSafeEnvironment::new(parent, generation));
        let gen_env = Arc::new(GenerationalEnvironment {
            inner: env.clone(),
            generation,
            parent: None, // TODO: Set up parent chain
            manager: self.clone(),
        });
        
        // Register the environment for cleanup tracking
        {
            if let Ok(mut registry) = self.environment_registry.write() {
                registry.entry(generation)
                    .or_default()
                    .push(Arc::downgrade(&gen_env));
            }
        }

        // Update generation record
        if let Some(record) = self.generation_history.iter_mut().find(|r| r.generation == generation) {
            record.env_count += 1;
        }
        
        gen_env
    }
    
    /// Creates an environment for a state mutation.
    pub fn create_mutation_environment(
        &mut self,
        base_env: Arc<GenerationalEnvironment>,
        variable: String,
        value: Value
    ) -> Arc<GenerationalEnvironment> {
        let new_generation = self.new_generation(vec![Effect::State]);
        
        // Create new environment extending the base
        let new_env = base_env.inner.define_cow(variable, value);
        
        let gen_env = Arc::new(GenerationalEnvironment {
            inner: new_env,
            generation: new_generation,
            parent: Some(base_env),
            manager: self.clone(),
        });
        
        // Register the new environment
        {
            if let Ok(mut registry) = self.environment_registry.write() {
                registry.entry(new_generation)
                    .or_default()
                    .push(Arc::downgrade(&gen_env));
            }
        }
        
        gen_env
    }
    
    /// Gets statistics about generation usage.
    pub fn stats(&self) -> GenerationStats {
        let active_gens = self.generation_history.len();
        let current_env_count = self.generation_history
            .back()
            .map(|r| r.env_count)
            .unwrap_or(0);
        
        GenerationStats {
            total_generations: GENERATION_COUNTER.load(Ordering::SeqCst),
            active_generations: active_gens,
            current_env_count,
            estimated_memory_kb: active_gens * 10, // Rough estimate
        }
    }
    
    /// Performs cleanup of old generations.
    pub fn cleanup_old_generations(&mut self) {
        let cutoff = self.generation_history.len().saturating_sub(self.max_history);
        
        for _ in 0..cutoff {
            if let Some(record) = self.generation_history.pop_front() {
                self.cleanup_generation(record.generation);
            }
        }
    }
    
    /// Cleans up a specific generation.
    fn cleanup_generation(&self, generation: Generation) {
        {
            if let Ok(mut registry) = self.environment_registry.write() {
                if let Some(weak_refs) = registry.remove(&generation) {
                    // Check if any environments in this generation are still alive
                    let alive_count = weak_refs.iter().filter(|w| w.strong_count() > 0).count();
                    if alive_count == 0 {
                        // All environments in this generation have been dropped
                        // We can safely remove the registry entry
                    }
                }
            }
        }
    }
    
    /// Forces garbage collection of unreferenced generations.
    pub fn force_gc(&mut self) {
        {
            if let Ok(mut registry) = self.environment_registry.write() {
                registry.retain(|_gen, weak_refs| {
                    // Remove weak references that are no longer valid
                    weak_refs.retain(|w| w.strong_count() > 0);
                    // Keep the generation if it still has live environments
                    !weak_refs.is_empty()
                });
            }
        }
        
        // Update generation history to remove GC'd generations
        let live_generations: std::collections::HashSet<Generation> = {
            if let Ok(registry) = self.environment_registry.read() {
                registry.keys().cloned().collect()
            } else {
                std::collections::HashSet::new()
            }
        };
        
        self.generation_history.retain(|record| {
            live_generations.contains(&record.generation) || 
            record.generation == self.current_generation
        });
    }
    
    /// Gets the generation history.
    pub fn generation_history(&self) -> &VecDeque<GenerationRecord> {
        &self.generation_history
    }
    
    /// Checks if a generation is still active.
    pub fn is_generation_active(&self, generation: Generation) -> bool {
        if let Ok(registry) = self.environment_registry.read() {
            registry.get(&generation)
                .map(|weak_refs| weak_refs.iter().any(|w| w.strong_count() > 0))
                .unwrap_or(false)
        } else {
            false
        }
    }
}

impl GenerationalEnvironment {
    /// Gets the underlying environment.
    pub fn inner(&self) -> &Arc<ThreadSafeEnvironment> {
        &self.inner
    }
    
    /// Gets the generation this environment belongs to.
    pub fn generation(&self) -> Generation {
        self.generation
    }
    
    /// Gets the parent generation environment.
    pub fn parent(&self) -> Option<&Arc<GenerationalEnvironment>> {
        self.parent.as_ref()
    }
    
    /// Looks up a variable, searching through the generation chain.
    pub fn lookup(&self, name: &str) -> Option<Value> {
        // First check the current environment
        if let Some(value) = self.inner.lookup(name) {
            return Some(value);
        }
        
        // Then check parent generations
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
    
    /// Creates a new environment with the defined variable (pure operation).
    /// Returns a new GenerationalEnvironment due to COW semantics.
    pub fn define(&self, name: String, value: Value) -> Arc<GenerationalEnvironment> {
        let new_inner = self.inner.define_cow(name, value);
        Arc::new(GenerationalEnvironment {
            inner: new_inner,
            generation: self.generation,
            parent: self.parent.clone(),
            manager: self.manager.clone(),
        })
    }
    
    /// Creates a mutation of this environment (creates new generation).
    pub fn mutate(&self, name: String, value: Value) -> Arc<GenerationalEnvironment> {
        let mut manager = self.manager.clone();
        manager.create_mutation_environment(
            // We need to clone self into an Arc, which requires some restructuring
            // For now, create a new environment
            Arc::new(GenerationalEnvironment {
                inner: self.inner.clone(),
                generation: self.generation,
                parent: self.parent.clone(),
                manager: self.manager.clone(),
            }),
            name,
            value
        )
    }
    
    /// Gets all variable names accessible from this environment.
    pub fn accessible_variables(&self) -> Vec<String> {
        let mut vars = self.inner.variable_names();
        
        // Add variables from parent generations
        if let Some(parent) = &self.parent {
            vars.extend(parent.accessible_variables());
        }
        
        vars.sort();
        vars.dedup();
        vars
    }
    
    /// Checks if this environment can access a variable.
    pub fn can_access(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }
    
    /// Gets the generation chain (from current to root).
    pub fn generation_chain(&self) -> Vec<Generation> {
        let mut chain = vec![self.generation];
        
        if let Some(parent) = &self.parent {
            chain.extend(parent.generation_chain());
        }
        
        chain
    }
}

impl Default for GenerationalConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            auto_cleanup: true,
            cleanup_threshold: 50,
            track_stats: true,
        }
    }
}

impl GenerationalConfig {
    
    /// Creates a configuration optimized for memory usage.
    pub fn memory_optimized() -> Self {
        Self {
            max_history: 20,
            auto_cleanup: true,
            cleanup_threshold: 10,
            track_stats: false,
        }
    }
    
    /// Creates a configuration optimized for debugging.
    pub fn debug_optimized() -> Self {
        Self {
            max_history: 1000,
            auto_cleanup: false,
            cleanup_threshold: 500,
            track_stats: true,
        }
    }
}

impl Default for GenerationalEnvManager {
    fn default() -> Self {
        Self::new()
    }
}

// Thread safety markers for generational types
unsafe impl Send for GenerationalEnvManager {}
unsafe impl Sync for GenerationalEnvManager {}

unsafe impl Send for GenerationalEnvironment {}
unsafe impl Sync for GenerationalEnvironment {}

unsafe impl Send for GenerationRecord {}
unsafe impl Sync for GenerationRecord {}

unsafe impl Send for GenerationalConfig {}
unsafe impl Sync for GenerationalConfig {}

unsafe impl Send for GenerationStats {}
unsafe impl Sync for GenerationStats {}

impl Clone for GenerationalEnvironment {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            generation: self.generation,
            parent: self.parent.clone(),
            manager: self.manager.clone(),
        }
    }
}

impl fmt::Display for GenerationalEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenEnv(gen={}, vars={:?})", 
               self.generation, 
               self.inner.variable_names())
    }
}

impl fmt::Display for GenerationRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gen {} (envs={}, effects={:?})", 
               self.generation, 
               self.env_count, 
               self.causing_effects)
    }
}

impl fmt::Display for GenerationStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenStats(total={}, active={}, current_envs={}, mem={}KB)", 
               self.total_generations,
               self.active_generations,
               self.current_env_count,
               self.estimated_memory_kb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generation_creation() {
        let mut manager = GenerationalEnvManager::new();
        assert_eq!(manager.current_generation(), 0);
        
        let gen1 = manager.new_generation(vec![Effect::State]);
        assert!(gen1 > 0);
        assert_eq!(manager.current_generation(), gen1);
        
        let gen2 = manager.new_generation(vec![Effect::IO]);
        assert!(gen2 > gen1);
        assert_eq!(manager.current_generation(), gen2);
    }
    
    #[test]
    fn test_environment_creation() {
        let mut manager = GenerationalEnvManager::new();
        let env = manager.create_environment(None, vec![Effect::Pure]);
        
        assert_eq!(env.generation(), manager.current_generation());
        assert!(env.parent().is_none());
    }
    
    #[test]
    fn test_mutation_creates_new_generation() {
        let mut manager = GenerationalEnvManager::new();
        let base_env = manager.create_environment(None, vec![Effect::Pure]);
        let base_gen = base_env.generation();
        
        let mutated_env = manager.create_mutation_environment(
            base_env,
            "x".to_string(),
            Value::integer(42)
        );
        
        assert!(mutated_env.generation() > base_gen);
        assert_eq!(mutated_env.lookup("x"), Some(Value::integer(42)));
    }
}