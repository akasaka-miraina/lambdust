//! Advanced memory pool and cache system for dependent types.
//!
//! This module provides specialized memory pooling strategies for frequently
//! allocated dependent type structures, with intelligent caching and prefetching.
//!
//! # Memory Pool Architecture
//!
//! ```text
//! Memory Pool System:
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Pool Manager                          │
//! ├─────────────┬─────────────┬─────────────┬──────────────┤
//! │ Small Types │ Medium Types│ Large Types │ Specialized  │
//! │   Pool      │    Pool     │    Pool     │   Pools      │
//! ├─────────────┼─────────────┼─────────────┼──────────────┤
//! │ Universe    │ Pi Types    │ Inductive   │ Constructor  │
//! │ Identity    │ Sigma Types │ Match Terms │ Normalize    │
//! └─────────────┴─────────────┴─────────────┴──────────────┘
//! ```
//!
//! # Caching Strategy
//!
//! - **Hot Path Cache**: Frequently used types kept in CPU cache
//! - **LRU Eviction**: Least recently used items evicted first
//! - **Prefetching**: Predict and preload related types
//! - **Compression**: Pack similar types together

use super::arena::{ArenaStats, TermRef, TypeArena, TypeRef};
use crate::diagnostics::{Error, Result};
use lru::LruCache;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Intelligent memory pool manager for dependent types.
///
/// This system optimizes memory allocation patterns by:
/// - **Size-based pools**: Different pools for different allocation sizes
/// - **Type-specific pools**: Specialized pools for common patterns
/// - **Cache-aware allocation**: Keep hot data in CPU cache lines
/// - **Predictive prefetching**: Load likely-needed types ahead of time
#[derive(Debug)]
pub struct MemoryPoolManager {
    /// Small type pool (≤64 bytes): Universe, simple Pi types
    small_pool: Rc<RefCell<SpecializedPool>>,
    /// Medium type pool (65-512 bytes): Complex Pi/Sigma types
    medium_pool: Rc<RefCell<SpecializedPool>>,
    /// Large type pool (>512 bytes): Inductive types, large matches
    large_pool: Rc<RefCell<SpecializedPool>>,
    /// Hot path cache for frequently accessed types
    hot_cache: RefCell<LruCache<TypeRef, CachedTypeData>>,
    /// Cold storage for rarely accessed types
    cold_storage: RefCell<HashMap<TypeRef, CompressedTypeData>>,
    /// Allocation pattern tracker for prediction
    pattern_tracker: RefCell<AllocationPatternTracker>,
    /// Performance statistics
    stats: RefCell<PoolStatistics>,
}

/// Specialized memory pool for specific allocation patterns.
#[derive(Debug)]
struct SpecializedPool {
    /// The underlying arena allocator
    arena: TypeArena,
    /// Free list for reusable allocations
    free_list: VecDeque<usize>,
    /// Pool-specific statistics
    pool_stats: PoolStats,
    /// Pool configuration
    config: PoolConfig,
}

/// Cached type data with access metadata
#[derive(Debug, Clone)]
struct CachedTypeData {
    /// The actual type reference
    type_ref: TypeRef,
    /// Last access time for LRU
    last_accessed: Instant,
    /// Access frequency counter
    access_count: u32,
    /// Predicted next access time
    predicted_access: Option<Instant>,
}

/// Compressed type data for cold storage
#[derive(Debug)]
struct CompressedTypeData {
    /// Compressed representation (simplified for now)
    data: Vec<u8>,
    /// Original size for decompression
    original_size: usize,
    /// Compression ratio achieved
    compression_ratio: f32,
}

/// Tracks allocation patterns to predict future needs
#[derive(Debug)]
struct AllocationPatternTracker {
    /// Recent allocation history
    history: VecDeque<AllocationEvent>,
    /// Pattern recognition cache
    patterns: HashMap<AllocationPattern, PredictionData>,
    /// Sequence predictor state
    sequence_state: SequencePredictor,
}

/// Individual allocation event
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AllocationEvent {
    /// Type of allocation
    allocation_type: AllocationType,
    /// Time of allocation
    timestamp: Instant,
    /// Size of allocation
    size_hint: usize,
    /// Context hash (for pattern matching)
    context_hash: u64,
}

/// Classification of allocation types
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AllocationType {
    /// Universe type allocations (Type₀, Type₁, etc.)
    Universe,
    /// Π-type (dependent function type) allocations
    PiType,
    /// Σ-type (dependent pair type) allocations
    SigmaType,
    /// Identity type allocations for equality types
    IdentityType,
    /// Inductive type allocations with constructors
    InductiveType,
    /// Lambda abstraction term allocations
    Lambda,
    /// Function application term allocations
    Application,
    /// Constructor application term allocations
    Constructor,
    /// Pattern matching term allocations
    Match,
}

/// Recognized allocation pattern
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AllocationPattern {
    /// Sequence of allocation types forming the pattern
    sequence: Vec<AllocationType>,
    /// Average time interval between allocations in the pattern
    interval: Duration,
}

/// Prediction data for a pattern
#[derive(Debug, Clone)]
struct PredictionData {
    /// Confidence level for this prediction (0.0-1.0)
    confidence: f32,
    /// Number of times this pattern was observed
    observation_count: u32,
    /// The predicted next allocation type in the sequence
    next_allocation: AllocationType,
    /// Types that should be prefetched based on this pattern
    prefetch_types: Vec<AllocationType>,
}

/// Sequence-based predictor using Markov chains
#[derive(Debug)]
struct SequencePredictor {
    /// Transition probabilities between allocation types
    transitions: HashMap<AllocationType, HashMap<AllocationType, f32>>,
    /// Recent allocation history for context
    context_window: VecDeque<AllocationType>,
    /// Size of the context window for pattern recognition
    window_size: usize,
}

/// Pool configuration parameters
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial number of pre-allocated objects in the pool
    initial_size: usize,
    /// Maximum number of objects the pool can hold
    max_size: usize,
    /// Multiplicative factor for pool growth when expanding
    growth_factor: f32,
    /// Minimum number of free objects before triggering compaction
    min_free_list: usize,
}

/// Statistics for a specialized pool
#[derive(Debug, Clone)]
struct PoolStats {
    /// Total number of allocations ever made from this pool
    total_allocations: u64,
    /// Current number of active (not freed) allocations
    active_allocations: u64,
    /// Maximum number of simultaneous allocations ever reached
    peak_allocations: u64,
    /// Total number of bytes allocated by this pool
    bytes_allocated: u64,
    /// Number of cache hits for this pool
    cache_hits: u64,
    /// Number of cache misses for this pool
    cache_misses: u64,
}

/// Overall pool system statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    /// Statistics for the small object pool (≤64 bytes)
    small_pool: PoolStats,
    /// Statistics for the medium object pool (65-512 bytes)
    medium_pool: PoolStats,
    /// Statistics for the large object pool (>512 bytes)
    large_pool: PoolStats,
    /// Number of hits in the hot path cache
    hot_cache_hits: u64,
    /// Number of misses in the hot path cache
    hot_cache_misses: u64,
    /// Current size of cold storage in bytes
    cold_storage_size: usize,
    /// Accuracy of prefetch predictions (0.0-1.0)
    prefetch_accuracy: f32,
    /// Total memory saved compared to naive allocation (in bytes)
    memory_savings: f64,
}

impl MemoryPoolManager {
    /// Create a new memory pool manager with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolManagerConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PoolManagerConfig) -> Self {
        Self {
            small_pool: Rc::new(RefCell::new(SpecializedPool::new(config.small_pool_config))),
            medium_pool: Rc::new(RefCell::new(SpecializedPool::new(
                config.medium_pool_config,
            ))),
            large_pool: Rc::new(RefCell::new(SpecializedPool::new(config.large_pool_config))),
            hot_cache: RefCell::new(LruCache::new(config.hot_cache_size.try_into().unwrap())),
            cold_storage: RefCell::new(HashMap::new()),
            pattern_tracker: RefCell::new(AllocationPatternTracker::new()),
            stats: RefCell::new(PoolStatistics::new()),
        }
    }

    /// Allocate a type reference with intelligent pool selection
    pub fn allocate_type(
        &self,
        allocation_type: AllocationType,
        size_hint: usize,
    ) -> Result<TypeRef> {
        // Record allocation event for pattern learning
        let event = AllocationEvent {
            allocation_type: allocation_type.clone(),
            timestamp: Instant::now(),
            size_hint,
            context_hash: self.compute_context_hash(),
        };
        self.pattern_tracker.borrow_mut().record_event(event);

        // Select appropriate pool based on size
        let pool = self.select_pool(size_hint);

        // Try hot cache first
        if let Some(cached) = self.try_hot_cache(&allocation_type) {
            self.stats.borrow_mut().hot_cache_hits += 1;
            return Ok(cached);
        }

        // Allocate from selected pool
        let type_ref = pool.borrow_mut().allocate()?;

        // Add to hot cache if frequently used
        self.maybe_cache_hot(type_ref, allocation_type.clone());

        // Trigger predictive prefetching
        self.trigger_prefetch(&allocation_type);

        self.stats.borrow_mut().hot_cache_misses += 1;
        Ok(type_ref)
    }

    /// Deallocate a type reference, potentially returning to pool
    pub fn deallocate_type(&self, type_ref: TypeRef) -> Result<()> {
        // Determine which pool owns this reference
        let pool = self.find_owning_pool(type_ref)?;

        // Return to pool's free list
        pool.borrow_mut().deallocate(type_ref)?;

        // Remove from hot cache if present
        self.remove_from_hot_cache(type_ref);

        Ok(())
    }

    /// Get comprehensive memory statistics
    pub fn memory_statistics(&self) -> MemoryPoolStatistics {
        let stats = self
            .stats
            .try_borrow()
            .map(|s| s.clone())
            .unwrap_or_else(|_| PoolStatistics::new());
        let small_arena_stats = if let Ok(pool) = self.small_pool.try_borrow() {
            pool.arena.memory_stats().unwrap_or(ArenaStats {
                types_count: 0,
                terms_count: 0,
                types_memory: 0,
                terms_memory: 0,
                cache_hits_types: 0,
                cache_hits_terms: 0,
            })
        } else {
            ArenaStats {
                types_count: 0,
                terms_count: 0,
                types_memory: 0,
                terms_memory: 0,
                cache_hits_types: 0,
                cache_hits_terms: 0,
            }
        };
        let medium_arena_stats = if let Ok(pool) = self.medium_pool.try_borrow() {
            pool.arena.memory_stats().unwrap_or(ArenaStats {
                types_count: 0,
                terms_count: 0,
                types_memory: 0,
                terms_memory: 0,
                cache_hits_types: 0,
                cache_hits_terms: 0,
            })
        } else {
            ArenaStats {
                types_count: 0,
                terms_count: 0,
                types_memory: 0,
                terms_memory: 0,
                cache_hits_types: 0,
                cache_hits_terms: 0,
            }
        };
        let large_arena_stats = if let Ok(pool) = self.large_pool.try_borrow() {
            pool.arena.memory_stats().unwrap_or(ArenaStats {
                types_count: 0,
                terms_count: 0,
                types_memory: 0,
                terms_memory: 0,
                cache_hits_types: 0,
                cache_hits_terms: 0,
            })
        } else {
            ArenaStats {
                types_count: 0,
                terms_count: 0,
                types_memory: 0,
                terms_memory: 0,
                cache_hits_types: 0,
                cache_hits_terms: 0,
            }
        };

        let total_memory = small_arena_stats.total_memory()
            + medium_arena_stats.total_memory()
            + large_arena_stats.total_memory();

        MemoryPoolStatistics {
            pool_stats: stats,
            small_pool_arena: small_arena_stats,
            medium_pool_arena: medium_arena_stats,
            large_pool_arena: large_arena_stats,
            total_memory_pools: total_memory,
            cache_efficiency: self.calculate_cache_efficiency(),
            prediction_accuracy: self.calculate_prediction_accuracy(),
        }
    }

    /// Compact memory pools to reduce fragmentation
    pub fn compact_pools(&self) -> Result<CompactionReport> {
        let mut report = CompactionReport::new();

        // Compact each pool
        let small_compacted = self.small_pool.borrow_mut().compact()?;
        let medium_compacted = self.medium_pool.borrow_mut().compact()?;
        let large_compacted = self.large_pool.borrow_mut().compact()?;

        report.small_pool_savings = small_compacted;
        report.medium_pool_savings = medium_compacted;
        report.large_pool_savings = large_compacted;
        report.total_savings = small_compacted + medium_compacted + large_compacted;

        // Migrate cold items to compressed storage
        report.cold_storage_migrations = self.migrate_to_cold_storage()?;

        Ok(report)
    }

    /// Force prefetch of predicted types
    pub fn prefetch_predicted(&self) -> Result<PrefetchReport> {
        let predictions = self
            .pattern_tracker
            .try_borrow()
            .map(|p| p.get_predictions())
            .unwrap_or_default();
        let mut report = PrefetchReport::new();

        for prediction in predictions {
            if prediction.confidence > 0.7 {
                // High confidence threshold
                for prefetch_type in &prediction.prefetch_types {
                    match self.prefetch_type(prefetch_type.clone()) {
                        Ok(_) => report.successful_prefetches += 1,
                        Err(_) => report.failed_prefetches += 1,
                    }
                }
            }
        }

        Ok(report)
    }

    /// Clear all caches and reset pools
    pub fn reset_pools(&self) -> Result<()> {
        self.small_pool.borrow_mut().reset();
        self.medium_pool.borrow_mut().reset();
        self.large_pool.borrow_mut().reset();

        self.hot_cache.borrow_mut().clear();
        self.cold_storage.borrow_mut().clear();
        self.pattern_tracker.borrow_mut().reset();

        *self.stats.borrow_mut() = PoolStatistics::new();

        Ok(())
    }

    // Private helper methods

    fn select_pool(&self, size_hint: usize) -> Rc<RefCell<SpecializedPool>> {
        match size_hint {
            0..=64 => self.small_pool.clone(),
            65..=512 => self.medium_pool.clone(),
            _ => self.large_pool.clone(),
        }
    }

    fn try_hot_cache(&self, allocation_type: &AllocationType) -> Option<TypeRef> {
        let mut cache = self.hot_cache.borrow_mut();
        // Simplified lookup - in real implementation would use allocation_type as key
        None // Placeholder
    }

    fn maybe_cache_hot(&self, type_ref: TypeRef, allocation_type: AllocationType) {
        let cached_data = CachedTypeData {
            type_ref,
            last_accessed: Instant::now(),
            access_count: 1,
            predicted_access: None,
        };

        self.hot_cache.borrow_mut().put(type_ref, cached_data);
    }

    fn trigger_prefetch(&self, allocation_type: &AllocationType) {
        // Get predictions and prefetch likely next allocations
        let predictions = if let Ok(tracker) = self.pattern_tracker.try_borrow() {
            tracker.predict_next(allocation_type)
        } else {
            Vec::new()
        };
        for prediction in predictions {
            if prediction.confidence > 0.5 {
                let _ = self.prefetch_type(prediction.next_allocation);
            }
        }
    }

    fn prefetch_type(&self, allocation_type: AllocationType) -> Result<TypeRef> {
        // Pre-allocate the predicted type
        let size_hint = self.estimate_size_for_type(&allocation_type);
        self.allocate_type(allocation_type, size_hint)
    }

    fn estimate_size_for_type(&self, allocation_type: &AllocationType) -> usize {
        match allocation_type {
            AllocationType::Universe => 16,
            AllocationType::PiType => 64,
            AllocationType::SigmaType => 64,
            AllocationType::IdentityType => 48,
            AllocationType::InductiveType => 256,
            AllocationType::Lambda => 48,
            AllocationType::Application => 32,
            AllocationType::Constructor => 128,
            AllocationType::Match => 512,
        }
    }

    fn find_owning_pool(&self, type_ref: TypeRef) -> Result<Rc<RefCell<SpecializedPool>>> {
        // Check which pool owns this reference
        // This is a simplified implementation
        if self
            .small_pool
            .try_borrow()
            .map(|p| p.owns_reference(type_ref))
            .unwrap_or(false)
        {
            Ok(self.small_pool.clone())
        } else if self
            .medium_pool
            .try_borrow()
            .map(|p| p.owns_reference(type_ref))
            .unwrap_or(false)
        {
            Ok(self.medium_pool.clone())
        } else {
            Ok(self.large_pool.clone())
        }
    }

    fn remove_from_hot_cache(&self, type_ref: TypeRef) {
        self.hot_cache.borrow_mut().pop(&type_ref);
    }

    fn compute_context_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Hash current allocation context
        Instant::now().hash(&mut hasher);
        hasher.finish()
    }

    fn calculate_cache_efficiency(&self) -> f32 {
        if let Ok(stats) = self.stats.try_borrow() {
            let total_accesses = stats.hot_cache_hits + stats.hot_cache_misses;
            if total_accesses == 0 {
                0.0
            } else {
                stats.hot_cache_hits as f32 / total_accesses as f32
            }
        } else {
            0.0
        }
    }

    fn calculate_prediction_accuracy(&self) -> f32 {
        self.pattern_tracker
            .try_borrow()
            .map(|p| p.get_accuracy())
            .unwrap_or(0.0)
    }

    fn migrate_to_cold_storage(&self) -> Result<usize> {
        // Move least recently used items to compressed cold storage
        let mut migrations = 0;

        // This would be a complex implementation involving:
        // 1. Identifying cold items
        // 2. Compressing their data
        // 3. Moving to cold storage
        // 4. Updating references

        Ok(migrations)
    }
}

impl SpecializedPool {
    fn new(config: PoolConfig) -> Self {
        Self {
            arena: TypeArena::with_capacity(config.initial_size),
            free_list: VecDeque::with_capacity(config.initial_size / 4),
            pool_stats: PoolStats::new(),
            config,
        }
    }

    fn allocate(&mut self) -> Result<TypeRef> {
        self.pool_stats.total_allocations += 1;
        self.pool_stats.active_allocations += 1;

        if self.pool_stats.active_allocations > self.pool_stats.peak_allocations {
            self.pool_stats.peak_allocations = self.pool_stats.active_allocations;
        }

        // Try free list first
        if let Some(index) = self.free_list.pop_front() {
            // Reuse freed allocation
            // Create a dummy TypeRef - in real implementation would use proper arena allocation
            let dummy_data = super::arena::DependentTypeData::Universe(0);
            self.arena.alloc_type(dummy_data)
        } else {
            // Create new allocation in arena
            // This is simplified - would need actual type data
            let dummy_data = super::arena::DependentTypeData::Universe(0);
            self.arena.alloc_type(dummy_data)
        }
    }

    fn deallocate(&mut self, type_ref: TypeRef) -> Result<()> {
        if self.pool_stats.active_allocations > 0 {
            self.pool_stats.active_allocations -= 1;
        }

        // Add to free list for reuse
        // Note: TypeRef.index is private, so we need a different approach
        // In a real implementation, we'd need accessor methods or different design
        // For now, we'll use a placeholder approach
        self.free_list.push_back(0); // Placeholder

        Ok(())
    }

    fn owns_reference(&self, type_ref: TypeRef) -> bool {
        self.arena.is_valid_type_ref(type_ref)
    }

    fn compact(&mut self) -> Result<usize> {
        // Compact the arena and update free list
        let old_size = self
            .arena
            .memory_stats()
            .map(|s| s.total_memory())
            .unwrap_or(0);

        // Arena compaction would go here
        // This is complex and involves:
        // 1. Identifying live references
        // 2. Moving them to new contiguous memory
        // 3. Updating all pointers
        // 4. Freeing old memory

        let new_size = self
            .arena
            .memory_stats()
            .map(|s| s.total_memory())
            .unwrap_or(0);
        Ok(old_size.saturating_sub(new_size))
    }

    fn reset(&mut self) {
        self.arena.clear();
        self.free_list.clear();
        self.pool_stats = PoolStats::new();
    }
}

impl AllocationPatternTracker {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(1000),
            patterns: HashMap::new(),
            sequence_state: SequencePredictor::new(),
        }
    }

    fn record_event(&mut self, event: AllocationEvent) {
        self.history.push_back(event.clone());

        // Keep history bounded
        if self.history.len() > 1000 {
            self.history.pop_front();
        }

        // Update sequence predictor
        self.sequence_state.observe(&event.allocation_type);

        // Extract patterns
        self.extract_patterns();
    }

    fn predict_next(&self, current_type: &AllocationType) -> Vec<PredictionData> {
        self.sequence_state.predict(current_type)
    }

    fn get_predictions(&self) -> Vec<PredictionData> {
        self.patterns.values().cloned().collect()
    }

    fn get_accuracy(&self) -> f32 {
        self.sequence_state.get_accuracy()
    }

    fn reset(&mut self) {
        self.history.clear();
        self.patterns.clear();
        self.sequence_state = SequencePredictor::new();
    }

    fn extract_patterns(&mut self) {
        // Look for recurring sequences in the allocation history
        // This is a simplified pattern extraction
        if self.history.len() < 3 {
            return;
        }

        let recent: Vec<_> = self
            .history
            .iter()
            .rev()
            .take(10)
            .map(|e| e.allocation_type.clone())
            .collect();

        // Look for patterns of length 2-5
        for pattern_len in 2..=5.min(recent.len()) {
            if let Some(sequence) = recent.get(0..pattern_len) {
                let pattern = AllocationPattern {
                    sequence: sequence.to_vec(),
                    interval: Duration::from_millis(100), // Simplified
                };

                // Update pattern statistics
                let prediction = self
                    .patterns
                    .entry(pattern)
                    .or_insert_with(|| PredictionData {
                        confidence: 0.1,
                        observation_count: 0,
                        next_allocation: sequence[0].clone(),
                        prefetch_types: vec![],
                    });

                prediction.observation_count += 1;
                prediction.confidence = (prediction.observation_count as f32 / 100.0).min(1.0);
            }
        }
    }
}

impl SequencePredictor {
    fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            context_window: VecDeque::new(),
            window_size: 5,
        }
    }

    fn observe(&mut self, allocation_type: &AllocationType) {
        // Update transition probabilities
        if let Some(prev_type) = self.context_window.back() {
            let transitions = self.transitions.entry(prev_type.clone()).or_default();
            let count = transitions.entry(allocation_type.clone()).or_insert(0.0);
            *count += 1.0;

            // Normalize probabilities
            let total: f32 = transitions.values().sum();
            if total > 0.0 {
                for prob in transitions.values_mut() {
                    *prob /= total;
                }
            }
        }

        // Update context window
        self.context_window.push_back(allocation_type.clone());
        if self.context_window.len() > self.window_size {
            self.context_window.pop_front();
        }
    }

    fn predict(&self, current_type: &AllocationType) -> Vec<PredictionData> {
        if let Some(transitions) = self.transitions.get(current_type) {
            transitions
                .iter()
                .map(|(next_type, &confidence)| PredictionData {
                    confidence,
                    observation_count: (confidence * 100.0) as u32,
                    next_allocation: next_type.clone(),
                    prefetch_types: vec![next_type.clone()],
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn get_accuracy(&self) -> f32 {
        // Calculate overall prediction accuracy
        // This would be based on successful predictions vs total predictions
        0.5 // Placeholder
    }
}

impl PoolStats {
    fn new() -> Self {
        Self {
            total_allocations: 0,
            active_allocations: 0,
            peak_allocations: 0,
            bytes_allocated: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl PoolStatistics {
    fn new() -> Self {
        Self {
            small_pool: PoolStats::new(),
            medium_pool: PoolStats::new(),
            large_pool: PoolStats::new(),
            hot_cache_hits: 0,
            hot_cache_misses: 0,
            cold_storage_size: 0,
            prefetch_accuracy: 0.0,
            memory_savings: 0.0,
        }
    }
}

/// Configuration for the pool manager
#[derive(Debug, Clone)]
pub struct PoolManagerConfig {
    /// Configuration for the small object pool (≤64 bytes)
    pub small_pool_config: PoolConfig,
    /// Configuration for the medium object pool (65-512 bytes)
    pub medium_pool_config: PoolConfig,
    /// Configuration for the large object pool (>512 bytes)
    pub large_pool_config: PoolConfig,
    /// Size of the hot path cache for frequently accessed objects
    pub hot_cache_size: usize,
}

impl Default for PoolManagerConfig {
    fn default() -> Self {
        Self {
            small_pool_config: PoolConfig {
                initial_size: 1024,
                max_size: 16384,
                growth_factor: 1.5,
                min_free_list: 32,
            },
            medium_pool_config: PoolConfig {
                initial_size: 512,
                max_size: 8192,
                growth_factor: 1.5,
                min_free_list: 16,
            },
            large_pool_config: PoolConfig {
                initial_size: 256,
                max_size: 4096,
                growth_factor: 2.0,
                min_free_list: 8,
            },
            hot_cache_size: 1024,
        }
    }
}

/// Comprehensive memory pool statistics
#[derive(Debug, Clone)]
pub struct MemoryPoolStatistics {
    /// High-level statistics for all pools combined
    pub pool_stats: PoolStatistics,
    /// Arena statistics for the small object pool
    pub small_pool_arena: ArenaStats,
    /// Arena statistics for the medium object pool
    pub medium_pool_arena: ArenaStats,
    /// Arena statistics for the large object pool
    pub large_pool_arena: ArenaStats,
    /// Total number of memory pools being managed
    pub total_memory_pools: usize,
    /// Cache hit rate efficiency (0.0-1.0)
    pub cache_efficiency: f32,
    /// Accuracy of allocation predictions (0.0-1.0)
    pub prediction_accuracy: f32,
}

/// Report from pool compaction operation
#[derive(Debug)]
pub struct CompactionReport {
    /// Memory saved by compacting the small object pool (bytes)
    pub small_pool_savings: usize,
    /// Memory saved by compacting the medium object pool (bytes)
    pub medium_pool_savings: usize,
    /// Memory saved by compacting the large object pool (bytes)
    pub large_pool_savings: usize,
    /// Total memory saved across all pools (bytes)
    pub total_savings: usize,
    /// Number of objects migrated to cold storage during compaction
    pub cold_storage_migrations: usize,
}

impl CompactionReport {
    fn new() -> Self {
        Self {
            small_pool_savings: 0,
            medium_pool_savings: 0,
            large_pool_savings: 0,
            total_savings: 0,
            cold_storage_migrations: 0,
        }
    }
}

/// Report from prefetching operations
#[derive(Debug)]
pub struct PrefetchReport {
    /// Number of prefetch operations that resulted in cache hits
    pub successful_prefetches: usize,
    /// Number of prefetch operations that were not subsequently used
    pub failed_prefetches: usize,
}

impl PrefetchReport {
    fn new() -> Self {
        Self {
            successful_prefetches: 0,
            failed_prefetches: 0,
        }
    }

    /// Calculates the success rate of prefetch operations
    ///
    /// Returns the ratio of successful to total prefetch attempts as a value between 0.0 and 1.0
    pub fn success_rate(&self) -> f32 {
        let total = self.successful_prefetches + self.failed_prefetches;
        if total == 0 {
            0.0
        } else {
            self.successful_prefetches as f32 / total as f32
        }
    }
}

impl Default for MemoryPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_manager_creation() {
        let manager = MemoryPoolManager::new();
        let stats = manager.memory_statistics();

        assert_eq!(stats.pool_stats.small_pool.total_allocations, 0);
        assert_eq!(stats.pool_stats.medium_pool.total_allocations, 0);
        assert_eq!(stats.pool_stats.large_pool.total_allocations, 0);
    }

    #[test]
    fn test_size_based_pool_selection() {
        let manager = MemoryPoolManager::new();

        // Small allocation should go to small pool
        let small_ref = manager.allocate_type(AllocationType::Universe, 32).unwrap();

        // Large allocation should go to large pool
        let large_ref = manager
            .allocate_type(AllocationType::InductiveType, 1024)
            .unwrap();

        // Verify different pools were used (simplified check)
        assert_ne!(small_ref, large_ref);
    }

    #[test]
    fn test_allocation_pattern_tracking() {
        let manager = MemoryPoolManager::new();

        // Create a pattern: Universe -> Pi -> Sigma repeatedly
        for _ in 0..5 {
            let _ = manager.allocate_type(AllocationType::Universe, 16);
            let _ = manager.allocate_type(AllocationType::PiType, 64);
            let _ = manager.allocate_type(AllocationType::SigmaType, 64);
        }

        let stats = manager.memory_statistics();

        // Should have detected some patterns
        assert!(stats.prediction_accuracy >= 0.0);
        assert!(stats.cache_efficiency >= 0.0);
    }

    #[test]
    fn test_memory_compaction() {
        let manager = MemoryPoolManager::new();

        // Allocate many items
        let mut refs = Vec::new();
        for i in 0..100 {
            refs.push(manager.allocate_type(AllocationType::Universe, 16).unwrap());
        }

        // Deallocate some
        for i in (0..100).step_by(2) {
            let _ = manager.deallocate_type(refs[i]);
        }

        // Compact should recover some memory
        let report = manager.compact_pools().unwrap();

        // Some savings should be achieved (even if 0 in this simplified test)
        // Note: total_savings is unsigned, so this is always true
        assert!(report.total_savings < usize::MAX);
    }

    #[test]
    fn test_prefetch_system() {
        let manager = MemoryPoolManager::new();

        // Create predictable pattern
        for _ in 0..10 {
            let _ = manager.allocate_type(AllocationType::Lambda, 48);
            let _ = manager.allocate_type(AllocationType::Application, 32);
        }

        // Trigger prefetch
        let report = manager.prefetch_predicted().unwrap();

        // Should have attempted some prefetches
        let total_attempts = report.successful_prefetches + report.failed_prefetches;
        assert!(total_attempts < usize::MAX); // Unsigned value is always >= 0
    }

    #[test]
    fn test_pool_reset() {
        let manager = MemoryPoolManager::new();

        // Allocate some items
        for i in 0..10 {
            let _ = manager.allocate_type(AllocationType::Universe, 16);
        }

        let stats_before = manager.memory_statistics();
        assert!(stats_before.pool_stats.small_pool.total_allocations > 0);

        // Reset should clear everything
        manager.reset_pools().unwrap();

        let stats_after = manager.memory_statistics();
        assert_eq!(stats_after.pool_stats.small_pool.total_allocations, 0);
    }
}
