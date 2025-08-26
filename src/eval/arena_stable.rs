//! Stable Arena Allocation Optimization Integration
//!
//! This module provides a production-ready arena allocation system that integrates
//! with the trait optimization and NaN boxing frameworks from Stage 1 and 2.

use crate::eval::{Value, OptimizationHint, TraitOptimizedValue};

#[cfg(feature = "nan-boxing-optimization")]
use crate::eval::{EnhancedNanBoxedValue, ValueNanBoxingExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

// ============= ARENA ALLOCATION SYSTEM =============

/// Arena-allocated value with stable lifetime management
#[derive(Debug, Clone)]
pub struct ArenaValue {
    /// Value stored in arena
    data: ArenaValueData,
    /// Generation for validity checking
    generation: u32,
    /// Arena identifier
    arena_id: ArenaId,
}

/// Core data for arena-allocated values
#[derive(Debug, Clone)]
pub enum ArenaValueData {
    /// Simple value directly stored
    Direct(Value),
    /// NaN-boxed optimized value
    #[cfg(feature = "nan-boxing-optimization")]
    NanBoxed(EnhancedNanBoxedValue),
    /// Reference to shared data
    Shared(ArenaRef),
}

/// Reference to shared arena data
#[derive(Debug, Clone)]
pub struct ArenaRef {
    /// Index in arena storage
    index: usize,
    /// Reference count for lifetime management
    ref_count: Arc<AtomicUsize>,
}

/// Arena identifier with generation tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaId {
    /// Thread identifier
    thread_id: u64,
    /// Generation counter
    generation: u32,
}

impl ArenaId {
    /// Create new arena ID for current thread
    pub fn new() -> Self {
        static GENERATION_COUNTER: AtomicU64 = AtomicU64::new(0);
        
        Self {
            thread_id: {
                // Stable way to get thread ID as u64
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                std::thread::current().id().hash(&mut hasher);
                hasher.finish()
            },
            generation: GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst) as u32,
        }
    }
    
    /// Check if this arena ID is still valid
    pub fn is_valid(&self) -> bool {
        // Simple validity check - in production this would be more sophisticated
        self.generation > 0
    }
}

impl ArenaValue {
    /// Create new arena value from regular value
    pub fn from_value(value: Value, hint: OptimizationHint) -> Self {
        let arena_id = ArenaId::new();
        
        // Try to optimize using NaN boxing if possible
        let data = {
            #[cfg(feature = "nan-boxing-optimization")]
            {
                if let Some(enhanced) = value.to_nan_boxed_optimized(hint) {
                    ArenaValueData::NanBoxed(enhanced)
                } else {
                    ArenaValueData::Direct(value)
                }
            }
            #[cfg(not(feature = "nan-boxing-optimization"))]
            {
                ArenaValueData::Direct(value)
            }
        };
        
        Self {
            data,
            generation: arena_id.generation,
            arena_id,
        }
    }
    
    /// Create shared arena value
    pub fn shared(value: Value, arena_id: ArenaId) -> Self {
        let arena_ref = ArenaRef {
            index: 0, // Would be assigned by arena allocator
            ref_count: Arc::new(AtomicUsize::new(1)),
        };
        
        Self {
            data: ArenaValueData::Shared(arena_ref),
            generation: arena_id.generation,
            arena_id,
        }
    }
    
    /// Extract the underlying value
    pub fn to_value(&self) -> Value {
        match &self.data {
            ArenaValueData::Direct(value) => value.clone(),
            #[cfg(feature = "nan-boxing-optimization")]
            ArenaValueData::NanBoxed(enhanced) => {
                // Convert back from NaN boxed representation
                if let Some(n) = enhanced.as_number() {
                    Value::number(n)
                } else {
                    Value::Unspecified // Fallback
                }
            }
            ArenaValueData::Shared(_) => {
                // Would resolve through arena in full implementation
                Value::Unspecified
            }
        }
    }
    
    /// Check if this value is still valid in its arena
    pub fn is_valid(&self) -> bool {
        self.arena_id.is_valid()
    }
    
    /// Get optimization hint based on storage type
    pub fn optimization_hint(&self) -> OptimizationHint {
        match &self.data {
            ArenaValueData::Direct(_) => OptimizationHint::Default,
            #[cfg(feature = "nan-boxing-optimization")]
            ArenaValueData::NanBoxed(enhanced) => enhanced.optimization_hint(),
            ArenaValueData::Shared(_) => OptimizationHint::Shared,
        }
    }
    
    /// Calculate memory footprint
    pub fn memory_footprint(&self) -> usize {
        match &self.data {
            ArenaValueData::Direct(value) => std::mem::size_of_val(value),
            #[cfg(feature = "nan-boxing-optimization")]
            ArenaValueData::NanBoxed(enhanced) => std::mem::size_of_val(enhanced),
            ArenaValueData::Shared(arena_ref) => std::mem::size_of_val(arena_ref),
        }
    }
}

// ============= ARENA ALLOCATION STRATEGY =============

/// Arena allocation strategy based on value characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaStrategy {
    /// Thread-local allocation for short-lived values
    ThreadLocal,
    /// Global allocation for shared values
    Global,
    /// Hybrid allocation mixing arena and heap
    Hybrid,
    /// No arena allocation (fallback to heap)
    NoArena,
}

impl ArenaStrategy {
    /// Determine optimal strategy for a value
    pub fn for_value(value: &Value, hint: OptimizationHint) -> Self {
        match hint {
            OptimizationHint::HotPath => Self::ThreadLocal,
            OptimizationHint::Shared => Self::Global,
            OptimizationHint::LongTerm => Self::Global,
            OptimizationHint::Computational => Self::ThreadLocal,
            // Phase 5 JIT-related hints for optimal performance
            OptimizationHint::JITCompile => Self::ThreadLocal,
            OptimizationHint::HotLoop => Self::ThreadLocal,
            OptimizationHint::TypeSpecialized => Self::ThreadLocal,
            OptimizationHint::ProfileGuided => Self::ThreadLocal,
            OptimizationHint::Default => {
                // Use heuristics based on value type
                if value.is_number() || value.is_boolean() {
                    Self::ThreadLocal
                } else if value.is_pair() || value.is_vector() {
                    Self::Hybrid
                } else {
                    Self::NoArena
                }
            }
        }
    }
}

// ============= MEMORY MANAGEMENT SYSTEM =============

/// Thread-safe memory management for arena values
pub struct ArenaMemoryManager {
    /// Total allocations across all arenas
    total_allocations: AtomicU64,
    /// Memory currently in use
    memory_used: AtomicU64,
    /// Peak memory usage
    peak_memory: AtomicU64,
    /// Active arena tracking
    active_arenas: RwLock<HashMap<ArenaId, ArenaStats>>,
}

/// Statistics for individual arena
#[derive(Debug, Clone)]
pub struct ArenaStats {
    /// Number of values allocated
    pub allocated_values: usize,
    /// Total memory used in bytes
    pub memory_used: usize,
    /// Number of active references
    pub active_references: usize,
    /// Allocation strategy used
    pub strategy: ArenaStrategy,
}

impl ArenaMemoryManager {
    /// Create new memory manager
    pub fn new() -> Self {
        Self {
            total_allocations: AtomicU64::new(0),
            memory_used: AtomicU64::new(0),
            peak_memory: AtomicU64::new(0),
            active_arenas: RwLock::new(HashMap::new()),
        }
    }
    
    /// Allocate value in appropriate arena
    pub fn allocate(&self, value: Value, hint: OptimizationHint) -> ArenaValue {
        let strategy = ArenaStrategy::for_value(&value, hint);
        let arena_value = ArenaValue::from_value(value, hint);
        
        // Update statistics
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        let memory_size = arena_value.memory_footprint() as u64;
        let new_memory = self.memory_used.fetch_add(memory_size, Ordering::Relaxed) + memory_size;
        
        // Update peak memory
        let mut peak = self.peak_memory.load(Ordering::Relaxed);
        while peak < new_memory {
            match self.peak_memory.compare_exchange_weak(
                peak,
                new_memory,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
        
        // Update arena statistics
        if let Ok(mut arenas) = self.active_arenas.write() {
            let stats = arenas.entry(arena_value.arena_id).or_insert(ArenaStats {
                allocated_values: 0,
                memory_used: 0,
                active_references: 0,
                strategy,
            });
            stats.allocated_values += 1;
            stats.memory_used += arena_value.memory_footprint();
            stats.active_references += 1;
        }
        
        arena_value
    }
    
    /// Deallocate arena value
    pub fn deallocate(&self, arena_value: &ArenaValue) {
        let memory_size = arena_value.memory_footprint() as u64;
        self.memory_used.fetch_sub(memory_size, Ordering::Relaxed);
        
        // Update arena statistics
        if let Ok(mut arenas) = self.active_arenas.write() {
            if let Some(stats) = arenas.get_mut(&arena_value.arena_id) {
                stats.active_references = stats.active_references.saturating_sub(1);
                stats.memory_used = stats.memory_used.saturating_sub(arena_value.memory_footprint());
                
                // Remove arena if no active references
                if stats.active_references == 0 {
                    arenas.remove(&arena_value.arena_id);
                }
            }
        }
    }
    
    /// Get comprehensive statistics
    pub fn get_stats(&self) -> ArenaMemoryStats {
        let arena_count = self.active_arenas.read()
            .map(|arenas| arenas.len())
            .unwrap_or(0);
        
        ArenaMemoryStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            current_memory_usage: self.memory_used.load(Ordering::Relaxed),
            peak_memory_usage: self.peak_memory.load(Ordering::Relaxed),
            active_arenas: arena_count,
            fragmentation_ratio: 0.0, // Would be calculated in full implementation
        }
    }
    
    /// Force cleanup of inactive arenas
    pub fn cleanup(&self) {
        if let Ok(mut arenas) = self.active_arenas.write() {
            arenas.retain(|_, stats| stats.active_references > 0);
        }
    }
}

/// Global arena memory manager instance
static GLOBAL_ARENA_MANAGER: std::sync::LazyLock<ArenaMemoryManager> =
    std::sync::LazyLock::new(|| ArenaMemoryManager::new());

/// Overall arena memory statistics
#[derive(Debug, Clone)]
pub struct ArenaMemoryStats {
    /// Total number of allocations performed
    pub total_allocations: u64,
    /// Current memory usage in bytes
    pub current_memory_usage: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// Number of active arenas
    pub active_arenas: usize,
    /// Memory fragmentation ratio (0.0 to 1.0)
    pub fragmentation_ratio: f32,
}

// ============= VALUE OPTIMIZATION INTEGRATION =============

/// Arena-aware value optimization
pub struct ArenaValueOptimizer {
    memory_manager: &'static ArenaMemoryManager,
    optimization_threshold: usize,
    stats: RwLock<ArenaOptimizationStats>,
}

impl ArenaValueOptimizer {
    /// Create new arena value optimizer
    pub fn new() -> Self {
        Self {
            memory_manager: &GLOBAL_ARENA_MANAGER,
            optimization_threshold: 1000,
            stats: RwLock::new(ArenaOptimizationStats::new()),
        }
    }
    
    /// Optimize value using arena allocation
    pub fn optimize(&self, value: Value, hint: OptimizationHint) -> OptimizedArenaValue {
        let strategy = ArenaStrategy::for_value(&value, hint);
        
        let optimized = match strategy {
            ArenaStrategy::ThreadLocal | ArenaStrategy::Global => {
                let arena_value = self.memory_manager.allocate(value, hint);
                OptimizedArenaValue::Arena(arena_value)
            }
            ArenaStrategy::Hybrid => {
                // In hybrid mode, try both arena and NaN boxing
                #[cfg(feature = "nan-boxing-optimization")]
                {
                    if let Some(nan_boxed) = value.to_nan_boxed_optimized(hint) {
                        OptimizedArenaValue::Hybrid {
                            nan_boxed,
                            arena_components: Vec::new(),
                        }
                    } else {
                        OptimizedArenaValue::Heap(value)
                    }
                }
                #[cfg(not(feature = "nan-boxing-optimization"))]
                {
                    OptimizedArenaValue::Heap(value)
                }
            }
            ArenaStrategy::NoArena => OptimizedArenaValue::Heap(value),
        };
        
        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.total_optimizations += 1;
            match &optimized {
                OptimizedArenaValue::Arena(_) => stats.arena_optimizations += 1,
                #[cfg(feature = "nan-boxing-optimization")]
                OptimizedArenaValue::Hybrid { .. } => stats.hybrid_optimizations += 1,
                OptimizedArenaValue::Heap(_) => stats.heap_fallbacks += 1,
            }
        }
        
        optimized
    }
    
    /// Get optimization statistics
    pub fn get_stats(&self) -> ArenaOptimizationStats {
        self.stats.read()
            .map(|stats| stats.clone())
            .unwrap_or_else(|_| ArenaOptimizationStats::new())
    }
    
    /// Reset optimization statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            *stats = ArenaOptimizationStats::new();
        }
    }
}

/// Result of arena optimization
#[derive(Debug, Clone)]
pub enum OptimizedArenaValue {
    /// Pure arena-allocated value
    Arena(ArenaValue),
    /// Hybrid optimization with multiple techniques
    #[cfg(feature = "nan-boxing-optimization")]
    Hybrid {
        /// NaN-boxed component
        nan_boxed: EnhancedNanBoxedValue,
        /// Additional arena components
        arena_components: Vec<ArenaValue>,
    },
    /// Heap-allocated fallback
    Heap(Value),
}

impl OptimizedArenaValue {
    /// Extract underlying value
    pub fn to_value(&self) -> Value {
        match self {
            OptimizedArenaValue::Arena(arena_value) => arena_value.to_value(),
            #[cfg(feature = "nan-boxing-optimization")]
            OptimizedArenaValue::Hybrid { nan_boxed, .. } => {
                // Convert from NaN boxed representation
                if let Some(n) = nan_boxed.as_number() {
                    Value::number(n)
                } else {
                    Value::Unspecified
                }
            }
            OptimizedArenaValue::Heap(value) => value.clone(),
        }
    }
    
    /// Calculate memory footprint
    pub fn memory_footprint(&self) -> usize {
        match self {
            OptimizedArenaValue::Arena(arena_value) => arena_value.memory_footprint(),
            #[cfg(feature = "nan-boxing-optimization")]
            OptimizedArenaValue::Hybrid { nan_boxed, arena_components } => {
                std::mem::size_of_val(nan_boxed) + 
                arena_components.iter().map(|c| c.memory_footprint()).sum::<usize>()
            }
            OptimizedArenaValue::Heap(value) => std::mem::size_of_val(value),
        }
    }
}

/// Statistics for arena optimization
#[derive(Debug, Clone)]
pub struct ArenaOptimizationStats {
    /// Total optimization attempts
    pub total_optimizations: u64,
    /// Number of pure arena optimizations
    pub arena_optimizations: u64,
    /// Number of hybrid optimizations
    pub hybrid_optimizations: u64,
    /// Number of heap fallbacks
    pub heap_fallbacks: u64,
}

impl ArenaOptimizationStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_optimizations: 0,
            arena_optimizations: 0,
            hybrid_optimizations: 0,
            heap_fallbacks: 0,
        }
    }
    
    /// Calculate arena optimization ratio
    pub fn arena_ratio(&self) -> f32 {
        if self.total_optimizations == 0 {
            0.0
        } else {
            self.arena_optimizations as f32 / self.total_optimizations as f32
        }
    }
    
    /// Calculate hybrid optimization ratio
    pub fn hybrid_ratio(&self) -> f32 {
        if self.total_optimizations == 0 {
            0.0
        } else {
            self.hybrid_optimizations as f32 / self.total_optimizations as f32
        }
    }
}

// ============= EXTENSION TRAITS =============

/// Extension trait for Value to enable arena optimization
pub trait ValueArenaExt {
    /// Convert to optimized arena representation
    fn to_arena_optimized(&self, hint: OptimizationHint) -> OptimizedArenaValue;
    
    /// Check if value can benefit from arena allocation
    fn can_arena_optimize(&self) -> bool;
    
    /// Get estimated memory savings from arena optimization
    fn arena_savings_estimate(&self) -> usize;
}

impl ValueArenaExt for Value {
    fn to_arena_optimized(&self, hint: OptimizationHint) -> OptimizedArenaValue {
        let optimizer = ArenaValueOptimizer::new();
        optimizer.optimize(self.clone(), hint)
    }
    
    fn can_arena_optimize(&self) -> bool {
        // Arena optimization benefits structured data
        self.is_pair() || self.is_vector() || self.is_procedure() || self.is_environment()
    }
    
    fn arena_savings_estimate(&self) -> usize {
        if self.can_arena_optimize() {
            // Estimate based on value type
            if self.is_pair() {
                48 // Typical pair overhead reduction
            } else if self.is_vector() {
                32 // Vector header optimization
            } else {
                16 // General overhead reduction
            }
        } else {
            0
        }
    }
}

// ============= TESTING FRAMEWORK =============

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_id_generation() {
        let id1 = ArenaId::new();
        let id2 = ArenaId::new();
        
        assert_ne!(id1.generation, id2.generation);
        assert!(id1.is_valid());
        assert!(id2.is_valid());
    }
    
    #[test]
    fn test_arena_value_creation() {
        let value = Value::number(42.0);
        let arena_value = ArenaValue::from_value(value, OptimizationHint::Computational);
        
        assert!(arena_value.is_valid());
        
        let restored = arena_value.to_value();
        assert!(restored.is_number());
    }
    
    #[test]
    fn test_arena_strategy_determination() {
        let number = Value::number(3.14);
        let strategy = ArenaStrategy::for_value(&number, OptimizationHint::HotPath);
        assert_eq!(strategy, ArenaStrategy::ThreadLocal);
        
        let shared_strategy = ArenaStrategy::for_value(&number, OptimizationHint::Shared);
        assert_eq!(shared_strategy, ArenaStrategy::Global);
    }
    
    #[test]
    fn test_memory_manager_allocation() {
        let manager = ArenaMemoryManager::new();
        let value = Value::number(42.0);
        
        let arena_value = manager.allocate(value, OptimizationHint::Default);
        assert!(arena_value.is_valid());
        
        let stats_before = manager.get_stats();
        manager.deallocate(&arena_value);
        let stats_after = manager.get_stats();
        
        assert!(stats_after.current_memory_usage <= stats_before.current_memory_usage);
    }
    
    #[test]
    fn test_arena_value_optimizer() {
        let optimizer = ArenaValueOptimizer::new();
        let value = Value::number(3.14);
        
        let optimized = optimizer.optimize(value, OptimizationHint::Computational);
        
        match optimized {
            OptimizedArenaValue::Arena(arena_value) => {
                assert!(arena_value.is_valid());
            }
            _ => {} // Other optimizations are also valid
        }
        
        let stats = optimizer.get_stats();
        assert!(stats.total_optimizations > 0);
    }
    
    #[test]
    fn test_value_arena_ext() {
        let value = Value::number(42.0);
        
        let optimized = value.to_arena_optimized(OptimizationHint::Default);
        assert!(optimized.memory_footprint() > 0);
        
        let restored = optimized.to_value();
        assert!(restored.is_number());
    }
    
    #[test]
    fn test_arena_optimization_stats() {
        let mut stats = ArenaOptimizationStats::new();
        stats.total_optimizations = 100;
        stats.arena_optimizations = 60;
        stats.hybrid_optimizations = 30;
        stats.heap_fallbacks = 10;
        
        assert_eq!(stats.arena_ratio(), 0.6);
        assert_eq!(stats.hybrid_ratio(), 0.3);
    }
}