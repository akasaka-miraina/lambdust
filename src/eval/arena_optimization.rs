//! Arena Allocator Integration with Optimization Framework
//!
//! This module integrates the existing arena allocation system with
//! the trait-based optimization framework, providing unified memory
//! management with intelligent allocation strategies.

use crate::eval::{
    ValueArena, ValueRef, ArenaConfig, ArenaMemoryStats, EnhancedNanBoxedValue,
    OptimizationHint, TraitOptimizedValue, Value, NanBoxedValue,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

// ============= UNIFIED ARENA SYSTEM =============

/// Unified value system with hierarchical arena allocation
pub struct UnifiedValueSystem {
    // Three-tier arena structure
    short_arena: NanBoxArena,     // Immediate values (8 bytes)
    medium_arena: CompoundArena,  // Compound data structures
    long_arena: ProcedureArena,   // Procedures and environments
    
    // Arena selection and optimization
    arena_selector: ArenaSelector,
    memory_tracker: MemoryTracker,
    
    // Statistics
    allocation_stats: AllocationStatistics,
}

impl UnifiedValueSystem {
    /// Create new unified value system
    pub fn new() -> Self {
        Self {
            short_arena: NanBoxArena::new(),
            medium_arena: CompoundArena::new(),
            long_arena: ProcedureArena::new(),
            arena_selector: ArenaSelector::new(),
            memory_tracker: MemoryTracker::new(),
            allocation_stats: AllocationStatistics::new(),
        }
    }
    
    /// Allocate value with optimization hints
    pub fn allocate_optimized(&mut self, value: Value, hint: OptimizationHint) -> UnifiedValueRef {
        // Select appropriate arena based on value type and hint
        let arena_choice = self.arena_selector.select_arena(&value, hint);
        
        match arena_choice {
            ArenaChoice::Short => {
                if let Some(nan_boxed) = self.try_nan_box(&value) {
                    let enhanced = EnhancedNanBoxedValue::from_nan_boxed(nan_boxed, hint);
                    let ref_id = self.short_arena.allocate(enhanced);
                    self.allocation_stats.record_short_allocation();
                    UnifiedValueRef::Short(ref_id)
                } else {
                    // Fall back to medium arena
                    self.allocate_in_medium(value, hint)
                }
            }
            ArenaChoice::Medium => self.allocate_in_medium(value, hint),
            ArenaChoice::Long => self.allocate_in_long(value, hint),
        }
    }
    
    /// Try to convert value to NaN-boxed representation
    fn try_nan_box(&self, value: &Value) -> Option<NanBoxedValue> {
        match value {
            Value::number(n) => Some(NanBoxedValue::from_number(*n)),
            Value::Boolean(b) => Some(NanBoxedValue::from_boolean(*b)),
            Value::Character(c) => Some(NanBoxedValue::from_char(*c)),
            Value::Nil => Some(NanBoxedValue::nil_value()),
            Value::Unspecified => Some(NanBoxedValue::unspecified_value()),
            _ => None,
        }
    }
    
    /// Allocate in medium arena (compound data structures)
    fn allocate_in_medium(&mut self, value: Value, hint: OptimizationHint) -> UnifiedValueRef {
        let ref_id = self.medium_arena.allocate(value);
        self.allocation_stats.record_medium_allocation();
        UnifiedValueRef::Medium(ref_id)
    }
    
    /// Allocate in long arena (procedures and environments)
    fn allocate_in_long(&mut self, value: Value, hint: OptimizationHint) -> UnifiedValueRef {
        let ref_id = self.long_arena.allocate(value);
        self.allocation_stats.record_long_allocation();
        UnifiedValueRef::Long(ref_id)
    }
    
    /// Dereference unified value reference
    pub fn dereference(&self, value_ref: UnifiedValueRef) -> Option<Value> {
        match value_ref {
            UnifiedValueRef::Short(ref_id) => {
                self.short_arena.get(ref_id)
                    .map(|enhanced| self.convert_from_enhanced(*enhanced))
            }
            UnifiedValueRef::Medium(ref_id) => {
                self.medium_arena.get(ref_id).cloned()
            }
            UnifiedValueRef::Long(ref_id) => {
                self.long_arena.get(ref_id).cloned()
            }
        }
    }
    
    /// Convert enhanced NaN-boxed value back to Value
    fn convert_from_enhanced(&self, enhanced: EnhancedNanBoxedValue) -> Value {
        let nan_boxed = enhanced.to_nan_boxed();
        
        if nan_boxed.is_number() {
            if let Some(n) = nan_boxed.as_number() {
                return Value::number(n);
            }
        }
        
        if nan_boxed.is_boolean() {
            if nan_boxed == NanBoxedValue::true_value() {
                return Value::boolean(true);
            } else if nan_boxed == NanBoxedValue::false_value() {
                return Value::boolean(false);
            }
        }
        
        if nan_boxed.is_nil() {
            return Value::nil();
        }
        
        Value::unspecified() // Fallback
    }
    
    /// Get comprehensive memory statistics
    pub fn get_memory_stats(&self) -> UnifiedMemoryStats {
        UnifiedMemoryStats {
            short_arena_stats: self.short_arena.get_stats(),
            medium_arena_stats: self.medium_arena.get_stats(),
            long_arena_stats: self.long_arena.get_stats(),
            allocation_stats: self.allocation_stats.clone(),
            total_memory_used: self.calculate_total_memory(),
            compression_ratio: self.calculate_compression_ratio(),
        }
    }
    
    fn calculate_total_memory(&self) -> usize {
        self.short_arena.memory_usage() +
        self.medium_arena.memory_usage() +
        self.long_arena.memory_usage()
    }
    
    fn calculate_compression_ratio(&self) -> f32 {
        let enhanced_size = self.allocation_stats.short_allocations * std::mem::size_of::<EnhancedNanBoxedValue>();
        let original_size = self.allocation_stats.short_allocations * std::mem::size_of::<Value>();
        
        if enhanced_size > 0 {
            original_size as f32 / enhanced_size as f32
        } else {
            1.0
        }
    }
}

// ============= ARENA IMPLEMENTATIONS =============

/// Specialized arena for NaN-boxed values (8 bytes each)
pub struct NanBoxArena {
    storage: Vec<EnhancedNanBoxedValue>,
    free_list: Vec<u32>,
    next_id: u32,
}

impl NanBoxArena {
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(1024), // Start with 8KB
            free_list: Vec::new(),
            next_id: 0,
        }
    }
    
    pub fn allocate(&mut self, value: EnhancedNanBoxedValue) -> u32 {
        if let Some(free_id) = self.free_list.pop() {
            self.storage[free_id as usize] = value;
            free_id
        } else {
            let id = self.next_id;
            self.storage.push(value);
            self.next_id += 1;
            id
        }
    }
    
    pub fn get(&self, id: u32) -> Option<&EnhancedNanBoxedValue> {
        self.storage.get(id as usize)
    }
    
    pub fn free(&mut self, id: u32) {
        if (id as usize) < self.storage.len() {
            self.free_list.push(id);
        }
    }
    
    pub fn memory_usage(&self) -> usize {
        self.storage.len() * std::mem::size_of::<EnhancedNanBoxedValue>()
    }
    
    pub fn get_stats(&self) -> NanBoxArenaStats {
        NanBoxArenaStats {
            total_slots: self.storage.len(),
            used_slots: self.storage.len() - self.free_list.len(),
            free_slots: self.free_list.len(),
            memory_usage: self.memory_usage(),
        }
    }
}

/// Arena for compound data structures (lists, vectors, hashtables)
pub struct CompoundArena {
    storage: Vec<Value>,
    free_list: Vec<u32>,
    next_id: u32,
}

impl CompoundArena {
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(256), // Start smaller for larger values
            free_list: Vec::new(),
            next_id: 0,
        }
    }
    
    pub fn allocate(&mut self, value: Value) -> u32 {
        if let Some(free_id) = self.free_list.pop() {
            self.storage[free_id as usize] = value;
            free_id
        } else {
            let id = self.next_id;
            self.storage.push(value);
            self.next_id += 1;
            id
        }
    }
    
    pub fn get(&self, id: u32) -> Option<&Value> {
        self.storage.get(id as usize)
    }
    
    pub fn memory_usage(&self) -> usize {
        self.storage.len() * std::mem::size_of::<Value>()
    }
    
    pub fn get_stats(&self) -> CompoundArenaStats {
        CompoundArenaStats {
            total_slots: self.storage.len(),
            used_slots: self.storage.len() - self.free_list.len(),
            memory_usage: self.memory_usage(),
        }
    }
}

/// Arena for long-lived values (procedures, environments)
pub struct ProcedureArena {
    storage: Vec<Value>,
    reference_counts: HashMap<u32, u32>,
    next_id: u32,
}

impl ProcedureArena {
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(64), // Smallest initial capacity
            reference_counts: HashMap::new(),
            next_id: 0,
        }
    }
    
    pub fn allocate(&mut self, value: Value) -> u32 {
        let id = self.next_id;
        self.storage.push(value);
        self.reference_counts.insert(id, 1);
        self.next_id += 1;
        id
    }
    
    pub fn get(&self, id: u32) -> Option<&Value> {
        self.storage.get(id as usize)
    }
    
    pub fn memory_usage(&self) -> usize {
        self.storage.len() * std::mem::size_of::<Value>()
    }
    
    pub fn get_stats(&self) -> ProcedureArenaStats {
        ProcedureArenaStats {
            total_procedures: self.storage.len(),
            total_references: self.reference_counts.values().sum(),
            memory_usage: self.memory_usage(),
        }
    }
}

// ============= ARENA SELECTION LOGIC =============

/// Intelligent arena selector based on value characteristics
pub struct ArenaSelector {
    allocation_history: HashMap<String, ArenaChoice>,
}

impl ArenaSelector {
    pub fn new() -> Self {
        Self {
            allocation_history: HashMap::new(),
        }
    }
    
    pub fn select_arena(&mut self, value: &Value, hint: OptimizationHint) -> ArenaChoice {
        // Primary selection based on value type
        let type_choice = match value {
            Value::number(_) | Value::Boolean(_) | Value::Character(_) |
            Value::Nil | Value::Unspecified => ArenaChoice::Short,
            
            Value::Pair(..) | Value::Vector(_) | Value::Hashtable(_) |
            Value::MutableString(_) => ArenaChoice::Medium,
            
            Value::Procedure(_) | Value::Continuation(_) => ArenaChoice::Long,
            
            _ => ArenaChoice::Medium, // Default for other types
        };
        
        // Refine based on optimization hint
        match hint {
            OptimizationHint::HotPath => {
                // Hot path values benefit from NaN boxing when possible
                match type_choice {
                    ArenaChoice::Medium => ArenaChoice::Short,
                    other => other,
                }
            }
            OptimizationHint::LongTerm => {
                // Long-term values prefer less memory pressure
                ArenaChoice::Long
            }
            OptimizationHint::Shared => {
                // Shared values benefit from reference counting
                ArenaChoice::Long
            }
            _ => type_choice,
        }
    }
}

/// Arena choice for value allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaChoice {
    Short,  // NaN-boxed values
    Medium, // Compound structures
    Long,   // Procedures and environments
}

// ============= UNIFIED VALUE REFERENCE =============

/// Unified reference to values in different arenas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnifiedValueRef {
    Short(u32),  // Reference to NaN-boxed arena
    Medium(u32), // Reference to compound arena
    Long(u32),   // Reference to procedure arena
}

impl UnifiedValueRef {
    /// Get the arena type this reference points to
    pub fn arena_type(&self) -> ArenaChoice {
        match self {
            UnifiedValueRef::Short(_) => ArenaChoice::Short,
            UnifiedValueRef::Medium(_) => ArenaChoice::Medium,
            UnifiedValueRef::Long(_) => ArenaChoice::Long,
        }
    }
    
    /// Check if this reference is to a NaN-boxed value
    pub fn is_nan_boxed(&self) -> bool {
        matches!(self, UnifiedValueRef::Short(_))
    }
}

// ============= MEMORY TRACKING =============

/// Memory usage tracker across all arenas
pub struct MemoryTracker {
    peak_usage: AtomicU64,
    current_usage: AtomicU64,
    allocation_count: AtomicU64,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            peak_usage: AtomicU64::new(0),
            current_usage: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
        }
    }
    
    pub fn track_allocation(&self, size: usize) {
        let current = self.current_usage.fetch_add(size as u64, Ordering::Relaxed) + size as u64;
        
        // Update peak usage
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
        
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn track_deallocation(&self, size: usize) {
        self.current_usage.fetch_sub(size as u64, Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> MemoryTrackerStats {
        MemoryTrackerStats {
            current_usage: self.current_usage.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
        }
    }
}

// ============= STATISTICS STRUCTURES =============

/// Statistics for allocation patterns
#[derive(Debug, Clone)]
pub struct AllocationStatistics {
    pub short_allocations: usize,
    pub medium_allocations: usize,
    pub long_allocations: usize,
}

impl AllocationStatistics {
    pub fn new() -> Self {
        Self {
            short_allocations: 0,
            medium_allocations: 0,
            long_allocations: 0,
        }
    }
    
    pub fn record_short_allocation(&mut self) {
        self.short_allocations += 1;
    }
    
    pub fn record_medium_allocation(&mut self) {
        self.medium_allocations += 1;
    }
    
    pub fn record_long_allocation(&mut self) {
        self.long_allocations += 1;
    }
}

/// Comprehensive memory statistics
#[derive(Debug, Clone)]
pub struct UnifiedMemoryStats {
    pub short_arena_stats: NanBoxArenaStats,
    pub medium_arena_stats: CompoundArenaStats,
    pub long_arena_stats: ProcedureArenaStats,
    pub allocation_stats: AllocationStatistics,
    pub total_memory_used: usize,
    pub compression_ratio: f32,
}

/// Statistics for NaN-boxed arena
#[derive(Debug, Clone)]
pub struct NanBoxArenaStats {
    pub total_slots: usize,
    pub used_slots: usize,
    pub free_slots: usize,
    pub memory_usage: usize,
}

/// Statistics for compound data arena
#[derive(Debug, Clone)]
pub struct CompoundArenaStats {
    pub total_slots: usize,
    pub used_slots: usize,
    pub memory_usage: usize,
}

/// Statistics for procedure arena
#[derive(Debug, Clone)]
pub struct ProcedureArenaStats {
    pub total_procedures: usize,
    pub total_references: u32,
    pub memory_usage: usize,
}

/// Memory tracker statistics
#[derive(Debug, Clone)]
pub struct MemoryTrackerStats {
    pub current_usage: u64,
    pub peak_usage: u64,
    pub allocation_count: u64,
}

// ============= TRAIT IMPLEMENTATIONS =============

impl TraitOptimizedValue for UnifiedValueRef {
    type Inner = Value;
    type MemoryStats = (); // Placeholder
    
    fn from_inner(_inner: Self::Inner, _hint: OptimizationHint) -> Self {
        // This would require access to the UnifiedValueSystem
        // In practice, this would be handled differently
        UnifiedValueRef::Medium(0) // Placeholder
    }
    
    fn into_inner(self) -> Self::Inner {
        // This would require access to the UnifiedValueSystem
        // In practice, this would be handled differently
        Value::unspecified() // Placeholder
    }
    
    fn memory_stats(&self) -> Self::MemoryStats {
        // Return unit type for now
        ()
    }
    
    fn optimize(&mut self) -> bool {
        // Optimization would be handled at the system level
        false
    }
}

// ============= TESTING FRAMEWORK =============

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_system() {
        let mut system = UnifiedValueSystem::new();
        
        // Test NaN-boxable value
        let number_ref = system.allocate_optimized(
            Value::number(42.0),
            OptimizationHint::Computational
        );
        assert!(number_ref.is_nan_boxed());
        
        // Verify retrieval
        let retrieved = system.dereference(number_ref).unwrap();
        assert!(matches!(retrieved, Value::number(_)));
    }
    
    #[test]
    fn test_arena_selection() {
        let mut selector = ArenaSelector::new();
        
        // Numbers should go to short arena
        let choice = selector.select_arena(&Value::number(3.14), OptimizationHint::Default);
        assert_eq!(choice, ArenaChoice::Short);
        
        // Procedures should go to long arena
        let proc_value = Value::unspecified(); // Placeholder for procedure
        let choice = selector.select_arena(&proc_value, OptimizationHint::LongTerm);
        assert_eq!(choice, ArenaChoice::Long);
    }
    
    #[test]
    fn test_memory_tracking() {
        let tracker = MemoryTracker::new();
        
        tracker.track_allocation(100);
        tracker.track_allocation(200);
        
        let stats = tracker.get_stats();
        assert_eq!(stats.current_usage, 300);
        assert_eq!(stats.peak_usage, 300);
        assert_eq!(stats.allocation_count, 2);
        
        tracker.track_deallocation(150);
        let stats = tracker.get_stats();
        assert_eq!(stats.current_usage, 150);
        assert_eq!(stats.peak_usage, 300); // Peak should remain
    }
    
    #[test]
    fn test_nan_box_arena() {
        let mut arena = NanBoxArena::new();
        
        let value = EnhancedNanBoxedValue::from_f64_optimized(3.14);
        let id = arena.allocate(value);
        
        let retrieved = arena.get(id).unwrap();
        assert_eq!(*retrieved, value);
        
        let stats = arena.get_stats();
        assert_eq!(stats.used_slots, 1);
        assert_eq!(stats.free_slots, 0);
    }
}