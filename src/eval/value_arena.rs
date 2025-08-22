//! Arena-based memory management for runtime values.
//!
//! This module provides efficient memory allocation for Lambdust runtime values,
//! reducing heap fragmentation and improving performance through:
//!
//! - **Bulk Allocation**: Large contiguous memory regions instead of many small allocations
//! - **Cache Locality**: Related values are stored near each other in memory
//! - **Reduced Overhead**: Lightweight references instead of `Arc<T>` for common cases
//! - **Automatic Deduplication**: Common values are shared automatically
//! - **Fast Collection**: Arena-scoped garbage collection
//!
//! # Design Philosophy
//!
//! The arena system distinguishes between different allocation patterns:
//!
//! 1. **Short-lived values**: Expression evaluation, temporary computations
//! 2. **Medium-lived values**: Function arguments, local variables
//! 3. **Long-lived values**: Global definitions, cached results
//!
//! Each category uses different allocation strategies optimized for their lifetimes.

use crate::ast::Literal;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::value::{CaseLambdaProcedure, Continuation, Frame, Procedure, Value};
use crate::utils::SymbolId;
use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// A lightweight reference to a value stored in the arena.
///
/// ValueRefs are only 8 bytes and provide:
/// - Bounds checking and generation-based safety
/// - Efficient equality comparisons by index
/// - Optional type tagging for faster dispatch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueRef {
    /// Index into the arena storage
    index: u32,
    /// Generation counter for safety (detects use-after-free)
    generation: u32,
}

impl ValueRef {
    /// Create a new ValueRef for placeholder/testing purposes
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

/// Arena-allocated storage for runtime values.
///
/// This structure manages memory-efficient storage of Lambdust runtime values
/// using multiple specialized arenas for different allocation patterns.
#[derive(Debug)]
pub struct ValueArena {
    /// Short-lived values (expression evaluation)
    short_arena: Bump,
    /// Medium-lived values (function calls, local variables)
    medium_arena: Bump,
    /// Long-lived values (global definitions)
    long_arena: Bump,

    /// Value storage indices
    value_storage: RefCell<Vec<ValueEntry>>,
    /// Generation counter for safety
    generation: AtomicU32,
    /// Allocation counter for statistics
    allocation_count: AtomicU64,

    /// Deduplication caches for common values
    literal_cache: RefCell<HashMap<LiteralHash, ValueRef>>,
    symbol_cache: RefCell<HashMap<SymbolId, ValueRef>>,
    pair_cache: RefCell<HashMap<PairHash, ValueRef>>,

    /// Arena configuration
    config: ArenaConfig,
}

/// Configuration for arena allocation strategies
#[derive(Debug, Clone)]
pub struct ArenaConfig {
    /// Enable value deduplication (trades memory for CPU)
    pub enable_deduplication: bool,
    /// Maximum cache size for each type
    pub max_cache_size: usize,
    /// Short arena capacity (KB)
    pub short_arena_capacity: usize,
    /// Medium arena capacity (KB)
    pub medium_arena_capacity: usize,
    /// Long arena capacity (KB)
    pub long_arena_capacity: usize,
    /// Enable arena compaction when utilization drops below threshold
    pub enable_compaction: bool,
    /// Compaction threshold (0.0 to 1.0)
    pub compaction_threshold: f64,
}

/// Internal storage entry for values
#[derive(Debug)]
struct ValueEntry {
    /// The actual value data stored in the arena
    data: &'static ArenaValue,
    /// Generation when this entry was created
    generation: u32,
    /// Which arena this value is stored in
    arena_type: ArenaType,
    /// Whether this entry is still valid
    valid: bool,
    /// Reference count for arena compaction
    ref_count: u32,
}

/// Type of arena used for allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArenaType {
    Short,
    Medium,
    Long,
}

/// Arena-optimized value representation.
///
/// This enum uses lightweight references instead of `Arc<T>` for heap-allocated
/// components, enabling better cache locality and reduced memory overhead.
#[derive(Debug, Clone, PartialEq)]
pub enum ArenaValue {
    // ============= PRIMITIVE VALUES =============
    /// Literal values (optimized storage)
    Literal(Literal),

    /// Symbols (direct storage, no allocation)
    Symbol(SymbolId),

    /// Keywords (interned strings)
    Keyword(String),

    /// The empty list
    Nil,

    /// Unspecified value
    Unspecified,

    // ============= COMPOUND VALUES =============
    /// Cons pair with arena references
    Pair(ValueRef, ValueRef),

    /// Vector with arena-allocated elements
    Vector(Vec<ValueRef>),

    /// Procedure with arena-allocated environment
    Procedure {
        /// Formal parameters for the procedure
        formals: crate::ast::Formals,
        /// Procedure body expressions
        body: Vec<crate::diagnostics::Spanned<crate::ast::Expr>>,
        /// Reference to the closure environment
        environment_ref: ValueRef,
        /// Optional procedure name for debugging
        name: Option<String>,
    },

    /// Continuation with arena-allocated stack
    Continuation {
        /// Reference to the continuation stack
        stack_ref: ValueRef,
        /// Reference to the continuation environment
        environment_ref: ValueRef,
        /// Unique continuation identifier
        id: u64,
    },

    // ============= ARENA-SPECIFIC VALUES =============
    /// Reference to original Value for gradual migration
    ValueRef(Arc<Value>),

    /// Large object stored externally (fallback)
    External(Arc<Value>),
}

/// Hash key for literal deduplication
#[derive(Debug, Hash, PartialEq, Eq)]
struct LiteralHash {
    discriminant: u8,
    content_hash: u64,
}

/// Hash key for pair deduplication
#[derive(Debug, Hash, PartialEq, Eq)]
struct PairHash {
    car_ref: ValueRef,
    cdr_ref: ValueRef,
}

impl ValueArena {
    /// Create a new value arena with default configuration
    pub fn new() -> Self {
        Self::with_config(ArenaConfig::default())
    }

    /// Create a new arena with specific configuration
    pub fn with_config(config: ArenaConfig) -> Self {
        Self {
            short_arena: Bump::with_capacity(config.short_arena_capacity * 1024),
            medium_arena: Bump::with_capacity(config.medium_arena_capacity * 1024),
            long_arena: Bump::with_capacity(config.long_arena_capacity * 1024),
            value_storage: RefCell::new(Vec::new()),
            generation: AtomicU32::new(0),
            allocation_count: AtomicU64::new(0),
            literal_cache: RefCell::new(HashMap::new()),
            symbol_cache: RefCell::new(HashMap::new()),
            pair_cache: RefCell::new(HashMap::new()),
            config,
        }
    }

    /// Allocate a short-lived value (expression evaluation)
    pub fn alloc_short(&self, value: ArenaValue) -> Result<ValueRef> {
        self.alloc_value(value, ArenaType::Short)
    }

    /// Allocate a medium-lived value (function calls)
    pub fn alloc_medium(&self, value: ArenaValue) -> Result<ValueRef> {
        self.alloc_value(value, ArenaType::Medium)
    }

    /// Allocate a long-lived value (global definitions)
    pub fn alloc_long(&self, value: ArenaValue) -> Result<ValueRef> {
        self.alloc_value(value, ArenaType::Long)
    }

    /// Smart allocation based on value type
    pub fn alloc_smart(&self, value: ArenaValue) -> Result<ValueRef> {
        let arena_type = match &value {
            // Short-lived: temporary computation results
            ArenaValue::Literal(_) | ArenaValue::Nil | ArenaValue::Unspecified => ArenaType::Short,
            // Medium-lived: function arguments, local bindings
            ArenaValue::Pair(_, _) | ArenaValue::Vector(_) => ArenaType::Medium,
            // Long-lived: procedures, global state
            ArenaValue::Procedure { .. } | ArenaValue::Continuation { .. } => ArenaType::Long,
            // Default to medium for unknown types
            _ => ArenaType::Medium,
        };

        self.alloc_value(value, arena_type)
    }

    /// Internal allocation implementation
    fn alloc_value(&self, value: ArenaValue, arena_type: ArenaType) -> Result<ValueRef> {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        // Check deduplication cache if enabled
        if self.config.enable_deduplication {
            if let Some(cached_ref) = self.check_cache(&value) {
                if self.is_valid_ref(cached_ref) {
                    return Ok(cached_ref);
                }
            }
        }

        // Select appropriate arena
        let arena_data: &'static ArenaValue = unsafe {
            let arena = match arena_type {
                ArenaType::Short => &self.short_arena,
                ArenaType::Medium => &self.medium_arena,
                ArenaType::Long => &self.long_arena,
            };

            // Safety: The arena outlives all references to this data
            std::mem::transmute(arena.alloc(value.clone()))
        };

        let current_gen = self.generation.fetch_add(1, Ordering::SeqCst);
        let mut storage = self.value_storage.borrow_mut();

        let index = storage.len() as u32;
        let value_ref = ValueRef {
            index,
            generation: current_gen,
        };

        storage.push(ValueEntry {
            data: arena_data,
            generation: current_gen,
            arena_type,
            valid: true,
            ref_count: 1,
        });

        // Update caches if enabled
        if self.config.enable_deduplication {
            self.update_cache(&value, value_ref);
        }

        Ok(value_ref)
    }

    /// Resolve a ValueRef to its data
    pub fn resolve(&self, value_ref: ValueRef) -> Result<&ArenaValue> {
        let storage = self.value_storage.try_borrow().map_err(|_| {
            Box::new(Error::runtime_error(
                "Cannot borrow value storage".to_string(),
                Some(Span::new(0, 0)),
            ))
        })?;

        if value_ref.index as usize >= storage.len() {
            return Err(Box::new(Error::runtime_error(
                format!(
                    "Invalid value reference: index {} out of bounds",
                    value_ref.index
                ),
                Some(Span::new(0, 0)),
            )));
        }

        let entry = &storage[value_ref.index as usize];

        if !entry.valid || entry.generation != value_ref.generation {
            return Err(Box::new(Error::runtime_error(
                "Invalid value reference: generation mismatch or invalidated".to_string(),
                Some(Span::new(0, 0)),
            )));
        }

        Ok(entry.data)
    }

    /// Check if a ValueRef is still valid
    pub fn is_valid_ref(&self, value_ref: ValueRef) -> bool {
        let storage = match self.value_storage.try_borrow() {
            Ok(storage) => storage,
            Err(_) => return false,
        };
        if value_ref.index as usize >= storage.len() {
            return false;
        }

        let entry = &storage[value_ref.index as usize];
        entry.valid && entry.generation == value_ref.generation
    }

    /// Increment reference count for a value
    pub fn ref_inc(&self, value_ref: ValueRef) {
        if let Ok(mut storage) = self.value_storage.try_borrow_mut() {
            if let Some(entry) = storage.get_mut(value_ref.index as usize) {
                if entry.valid && entry.generation == value_ref.generation {
                    entry.ref_count = entry.ref_count.saturating_add(1);
                }
            }
        }
    }

    /// Decrement reference count for a value
    pub fn ref_dec(&self, value_ref: ValueRef) {
        if let Ok(mut storage) = self.value_storage.try_borrow_mut() {
            if let Some(entry) = storage.get_mut(value_ref.index as usize) {
                if entry.valid && entry.generation == value_ref.generation {
                    entry.ref_count = entry.ref_count.saturating_sub(1);
                }
            }
        }
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> Option<ArenaMemoryStats> {
        let storage = self.value_storage.try_borrow().ok()?;
        let literal_cache = self.literal_cache.try_borrow().ok()?;
        let symbol_cache = self.symbol_cache.try_borrow().ok()?;
        let pair_cache = self.pair_cache.try_borrow().ok()?;

        let mut short_count = 0;
        let mut medium_count = 0;
        let mut long_count = 0;
        let mut valid_count = 0;

        for entry in storage.iter() {
            if entry.valid {
                valid_count += 1;
                match entry.arena_type {
                    ArenaType::Short => short_count += 1,
                    ArenaType::Medium => medium_count += 1,
                    ArenaType::Long => long_count += 1,
                }
            }
        }

        Some(ArenaMemoryStats {
            short_count,
            medium_count,
            long_count,
            valid_count,
            total_allocated: storage.len(),
            short_memory: self.short_arena.allocated_bytes(),
            medium_memory: self.medium_arena.allocated_bytes(),
            long_memory: self.long_arena.allocated_bytes(),
            literal_cache_size: literal_cache.len(),
            symbol_cache_size: symbol_cache.len(),
            pair_cache_size: pair_cache.len(),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
        })
    }

    /// Clear short-lived arena (expression evaluation cleanup)
    pub fn clear_short(&mut self) {
        self.short_arena.reset();
        self.invalidate_arena_entries(ArenaType::Short);
    }

    /// Clear medium-lived arena (function call cleanup)
    pub fn clear_medium(&mut self) {
        self.medium_arena.reset();
        self.invalidate_arena_entries(ArenaType::Medium);
    }

    /// Clear all arenas
    pub fn clear_all(&mut self) {
        self.short_arena.reset();
        self.medium_arena.reset();
        self.long_arena.reset();
        self.value_storage.borrow_mut().clear();
        self.literal_cache.borrow_mut().clear();
        self.symbol_cache.borrow_mut().clear();
        self.pair_cache.borrow_mut().clear();
        self.generation.store(0, Ordering::SeqCst);
        self.allocation_count.store(0, Ordering::SeqCst);
    }

    /// Perform arena compaction to reclaim unused space
    pub fn compact(&mut self) -> Result<CompactionStats> {
        if !self.config.enable_compaction {
            return Ok(CompactionStats::default());
        }

        let stats = self.memory_stats().unwrap();
        let utilization = stats.valid_count as f64 / stats.total_allocated as f64;

        if utilization >= self.config.compaction_threshold {
            return Ok(CompactionStats::default());
        }

        // TODO: Implement sophisticated compaction algorithm
        // For now, just clear invalid entries
        let mut storage = self.value_storage.borrow_mut();
        let before_count = storage.len();
        storage.retain(|entry| entry.valid);
        let after_count = storage.len();

        Ok(CompactionStats {
            entries_removed: before_count - after_count,
            memory_reclaimed: 0,   // TODO: Calculate actual memory reclaimed
            compaction_time_ms: 0, // TODO: Measure compaction time
        })
    }

    /// Convert ArenaValue to standard Value for compatibility
    pub fn to_standard_value(&self, arena_value: &ArenaValue) -> Result<Value> {
        match arena_value {
            ArenaValue::Literal(lit) => Ok(Value::Literal(lit.clone())),
            ArenaValue::Symbol(id) => Ok(Value::Symbol(*id)),
            ArenaValue::Keyword(k) => Ok(Value::Keyword(k.clone())),
            ArenaValue::Nil => Ok(Value::Nil),
            ArenaValue::Unspecified => Ok(Value::Unspecified),
            ArenaValue::Pair(car_ref, cdr_ref) => {
                let car = self.to_standard_value(self.resolve(*car_ref)?)?;
                let cdr = self.to_standard_value(self.resolve(*cdr_ref)?)?;
                Ok(Value::pair(car, cdr))
            }
            ArenaValue::Vector(refs) => {
                let mut values = Vec::with_capacity(refs.len());
                for value_ref in refs {
                    values.push(self.to_standard_value(self.resolve(*value_ref)?)?);
                }
                Ok(Value::vector(values))
            }
            ArenaValue::ValueRef(value) => Ok((**value).clone()),
            ArenaValue::External(value) => Ok((**value).clone()),
            _ => {
                // For complex types, fall back to external storage
                Err(Box::new(Error::runtime_error(
                    "Cannot convert complex arena value to standard value".to_string(),
                    Some(Span::new(0, 0)),
                )))
            }
        }
    }

    /// Convert standard Value to ArenaValue for arena allocation
    pub fn from_standard_value(&self, value: &Value) -> ArenaValue {
        match value {
            Value::Literal(lit) => ArenaValue::Literal(lit.clone()),
            Value::Symbol(id) => ArenaValue::Symbol(*id),
            Value::Keyword(k) => ArenaValue::Keyword(k.clone()),
            Value::Nil => ArenaValue::Nil,
            Value::Unspecified => ArenaValue::Unspecified,
            // For complex types, store as external reference during migration
            _ => ArenaValue::External(Arc::new(value.clone())),
        }
    }

    // ============= PRIVATE HELPER METHODS =============

    /// Check deduplication cache for existing value
    fn check_cache(&self, value: &ArenaValue) -> Option<ValueRef> {
        match value {
            ArenaValue::Literal(lit) => {
                let hash = self.hash_literal(lit);
                self.literal_cache.try_borrow().ok()?.get(&hash).copied()
            }
            ArenaValue::Symbol(id) => self.symbol_cache.try_borrow().ok()?.get(id).copied(),
            ArenaValue::Pair(car, cdr) => {
                let hash = PairHash {
                    car_ref: *car,
                    cdr_ref: *cdr,
                };
                self.pair_cache.try_borrow().ok()?.get(&hash).copied()
            }
            _ => None,
        }
    }

    /// Update deduplication cache with new value
    fn update_cache(&self, value: &ArenaValue, value_ref: ValueRef) {
        if let Ok(cache) = self.literal_cache.try_borrow() {
            if cache.len() >= self.config.max_cache_size {
                return; // Cache is full
            }
        } else {
            return; // Can't access cache
        }

        match value {
            ArenaValue::Literal(lit) => {
                let hash = self.hash_literal(lit);
                self.literal_cache.borrow_mut().insert(hash, value_ref);
            }
            ArenaValue::Symbol(id) => {
                self.symbol_cache.borrow_mut().insert(*id, value_ref);
            }
            ArenaValue::Pair(car, cdr) => {
                let hash = PairHash {
                    car_ref: *car,
                    cdr_ref: *cdr,
                };
                self.pair_cache.borrow_mut().insert(hash, value_ref);
            }
            _ => {}
        }
    }

    /// Create hash for literal deduplication
    fn hash_literal(&self, literal: &Literal) -> LiteralHash {
        use std::collections::hash_map::DefaultHasher;

        let discriminant = match literal {
            Literal::ExactInteger(_) | Literal::Integer(_) => 0,
            Literal::InexactReal(_) => 1,
            Literal::Number(_) => 2,
            Literal::Rational(_) => 3,
            Literal::Complex(_) => 4,
            Literal::String(_) => 5,
            Literal::InternedString(_) => 6,
            Literal::Character(_) => 7,
            Literal::Boolean(_) => 8,
            Literal::Bytevector(_) => 9,
            Literal::Nil => 10,
            Literal::Unspecified => 11,
        };

        let mut hasher = DefaultHasher::new();
        literal.hash(&mut hasher);
        let content_hash = hasher.finish();

        LiteralHash {
            discriminant,
            content_hash,
        }
    }

    /// Invalidate entries from specific arena type
    fn invalidate_arena_entries(&self, arena_type: ArenaType) {
        if let Ok(mut storage) = self.value_storage.try_borrow_mut() {
            for entry in storage.iter_mut() {
                if entry.arena_type == arena_type {
                    entry.valid = false;
                }
            }
        }
    }
}

/// Memory usage statistics for the value arena
#[derive(Debug, Clone)]
pub struct ArenaMemoryStats {
    /// Count of short-lived values (< 1KB)
    pub short_count: usize,
    /// Count of medium-lived values (1KB - 10KB)
    pub medium_count: usize,
    /// Count of long-lived values (> 10KB)
    pub long_count: usize,
    /// Count of currently valid entries
    pub valid_count: usize,
    /// Total number of allocated values
    pub total_allocated: usize,
    /// Memory used by short-lived values in bytes
    pub short_memory: usize,
    /// Memory used by medium-lived values in bytes
    pub medium_memory: usize,
    /// Memory used by long-lived values in bytes
    pub long_memory: usize,
    /// Size of the literal value cache
    pub literal_cache_size: usize,
    /// Size of the symbol cache
    pub symbol_cache_size: usize,
    /// Size of the pair cache
    pub pair_cache_size: usize,
    /// Total allocation count since startup
    pub allocation_count: u64,
}

/// Statistics from arena compaction
#[derive(Debug, Clone, Default)]
pub struct CompactionStats {
    /// Number of invalid entries removed during compaction
    pub entries_removed: usize,
    /// Amount of memory reclaimed in bytes
    pub memory_reclaimed: usize,
    /// Time taken for compaction in milliseconds
    pub compaction_time_ms: u64,
}

impl ArenaMemoryStats {
    /// Total memory used by all arenas
    pub fn total_memory(&self) -> usize {
        self.short_memory + self.medium_memory + self.long_memory
    }

    /// Memory utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.total_allocated == 0 {
            0.0
        } else {
            self.valid_count as f64 / self.total_allocated as f64
        }
    }

    /// Average memory per value
    pub fn avg_memory_per_value(&self) -> f64 {
        if self.valid_count == 0 {
            0.0
        } else {
            self.total_memory() as f64 / self.valid_count as f64
        }
    }
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            enable_deduplication: true,
            max_cache_size: 10_000,
            short_arena_capacity: 256,   // 256KB
            medium_arena_capacity: 1024, // 1MB
            long_arena_capacity: 4096,   // 4MB
            enable_compaction: true,
            compaction_threshold: 0.7,
        }
    }
}

impl Default for ValueArena {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ArenaMemoryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Arena Stats: {} values ({:.1}KB total), {:.1}% utilization, {} allocations\n\
             Short: {} values ({:.1}KB), Medium: {} values ({:.1}KB), Long: {} values ({:.1}KB)\n\
             Caches: {} literals, {} symbols, {} pairs",
            self.valid_count,
            self.total_memory() as f64 / 1024.0,
            self.utilization() * 100.0,
            self.allocation_count,
            self.short_count,
            self.short_memory as f64 / 1024.0,
            self.medium_count,
            self.medium_memory as f64 / 1024.0,
            self.long_count,
            self.long_memory as f64 / 1024.0,
            self.literal_cache_size,
            self.symbol_cache_size,
            self.pair_cache_size
        )
    }
}

impl fmt::Display for CompactionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Compaction: {} entries removed, {:.1}KB reclaimed, {}ms",
            self.entries_removed,
            self.memory_reclaimed as f64 / 1024.0,
            self.compaction_time_ms
        )
    }
}

// Thread safety markers
unsafe impl Send for ValueArena {}
unsafe impl Sync for ValueArena {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_allocation() {
        let arena = ValueArena::new();

        let nil_value = ArenaValue::Nil;
        let value_ref = arena.alloc_short(nil_value.clone()).unwrap();

        let resolved = arena.resolve(value_ref).unwrap();
        assert_eq!(resolved, &nil_value);
    }

    #[test]
    fn test_arena_deduplication() {
        let mut config = ArenaConfig::default();
        config.enable_deduplication = true;
        let arena = ValueArena::with_config(config);

        let symbol_id = SymbolId::new(42);
        let symbol_value = ArenaValue::Symbol(symbol_id);

        let ref1 = arena.alloc_short(symbol_value.clone()).unwrap();
        let ref2 = arena.alloc_short(symbol_value).unwrap();

        // Should return the same reference due to deduplication
        assert_eq!(ref1, ref2);
    }

    #[test]
    fn test_arena_smart_allocation() {
        let arena = ValueArena::new();

        // Literals should go to short arena
        let literal_ref = arena
            .alloc_smart(ArenaValue::Literal(Literal::integer(42)))
            .unwrap();
        assert!(arena.is_valid_ref(literal_ref));

        // Procedures should go to long arena
        let proc_ref = arena
            .alloc_smart(ArenaValue::Procedure {
                formals: crate::ast::Formals::Fixed(vec![]),
                body: vec![],
                environment_ref: literal_ref,
                name: Some("test".to_string()),
            })
            .unwrap();
        assert!(arena.is_valid_ref(proc_ref));
    }

    #[test]
    fn test_arena_memory_stats() {
        let arena = ValueArena::new();

        let initial_stats = arena.memory_stats().unwrap();
        assert_eq!(initial_stats.valid_count, 0);

        let _ref1 = arena.alloc_short(ArenaValue::Nil).unwrap();
        let _ref2 = arena.alloc_medium(ArenaValue::Unspecified).unwrap();

        let stats = arena.memory_stats().unwrap();
        assert_eq!(stats.valid_count, 2);
        assert_eq!(stats.short_count, 1);
        assert_eq!(stats.medium_count, 1);
    }

    #[test]
    fn test_value_conversion() {
        let arena = ValueArena::new();

        // Test conversion from standard Value to ArenaValue
        let standard_value = Value::integer(42);
        let arena_value = arena.from_standard_value(&standard_value);

        // Test conversion back to standard Value
        let converted_back = arena.to_standard_value(&arena_value).unwrap();
        assert_eq!(standard_value, converted_back);
    }

    #[test]
    fn test_invalid_reference_detection() {
        let arena = ValueArena::new();

        let invalid_ref = ValueRef {
            index: 999,
            generation: 0,
        };

        assert!(!arena.is_valid_ref(invalid_ref));
        assert!(arena.resolve(invalid_ref).is_err());
    }
}
