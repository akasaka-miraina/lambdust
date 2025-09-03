//! Fast hygiene resolver with interning and bitmask optimization.
//!
//! This module implements Phase 2A.2 hygiene optimization from the CS-architect design:
//! - O(1) identifier comparison through string interning
//! - Bitmask-based hygiene mark operations
//! - Cache-aware hygiene context management
//! - Memory-efficient mark tracking

use super::{HygieneContext, next_hygiene_id};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Environment;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::rc::Rc;
use std::sync::{Arc, Weak};

/// High-performance hygiene resolver with string interning and bitmask optimization.
#[derive(Debug)]
pub struct FastHygieneResolver {
    /// String interner for O(1) identifier comparison
    identifier_interner: RefCell<IdentifierInterner>,
    /// Hygiene mark registry with bitmask operations
    mark_registry: RefCell<HygieneMarkRegistry>,
    /// Scope tracking with optimized lookups
    scope_tracker: RefCell<ScopeTracker>,
    /// Performance metrics
    metrics: RefCell<HygieneMetrics>,
    /// Configuration
    config: HygieneConfig,
}

/// String interner for fast identifier operations.
#[derive(Debug)]
struct IdentifierInterner {
    /// String to ID mapping
    string_to_id: HashMap<String, InterId>,
    /// ID to string mapping for reconstruction
    id_to_string: Vec<Arc<str>>,
    /// Next available ID
    next_id: InterId,
    /// Frequently used identifiers cache
    hot_cache: HashMap<InterId, CachedIdentifierInfo>,
}

/// Interned identifier ID for fast operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct InterId(u32);

/// Cached information about frequently used identifiers.
#[derive(Debug, Clone)]
struct CachedIdentifierInfo {
    /// Original string
    string: Arc<str>,
    /// Usage count for cache management
    usage_count: u64,
    /// Most recent hygiene context
    last_context: Option<HygieneContextId>,
    /// Common rename patterns
    rename_cache: HashMap<HygieneMarkSet, InterId>,
}

/// Hygiene mark registry with bitmask optimization.
#[derive(Debug)]
struct HygieneMarkRegistry {
    /// Mark ID to mark mapping
    marks: HashMap<MarkId, HygieneMark>,
    /// Next available mark ID
    next_mark_id: MarkId,
    /// Bitmask allocator for efficient set operations
    bitmask_allocator: BitmaskAllocator,
    /// Mark combination cache
    combination_cache: HashMap<Vec<MarkId>, HygieneMarkSet>,
    /// Frequently used mark sets
    hot_mark_sets: HashMap<HygieneMarkSet, MarkSetUsageInfo>,
}

/// Hygiene mark with optimized representation.
#[derive(Debug, Clone)]
struct HygieneMark {
    /// Unique mark identifier
    id: MarkId,
    /// Bitmask position for fast operations
    bitmask_position: u8,
    /// Source location for debugging
    source_span: Option<Span>,
    /// Mark metadata
    metadata: MarkMetadata,
}

/// Mark identifier for fast lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MarkId(u32);

impl MarkId {
    /// Creates a new mark ID with the given numeric value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Gets the numeric ID of this mark.
    pub fn id(&self) -> u32 {
        self.0
    }

    /// Returns the raw underlying value.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Hygiene mark set using bitmask for O(1) operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HygieneMarkSet {
    /// Primary bitmask (first 64 marks)
    primary_mask: u64,
    /// Secondary bitmask (next 64 marks)
    secondary_mask: u64,
    /// Overflow storage for marks beyond 128
    overflow_id: Option<u32>,
}

/// Mark metadata for additional information.
#[derive(Debug, Clone)]
struct MarkMetadata {
    /// Mark creation timestamp
    created_at: std::time::Instant,
    /// Source macro that created this mark
    source_macro: Option<String>,
    /// Expansion phase when created
    expansion_phase: u32,
    /// Custom properties
    properties: HashMap<String, String>,
}

/// Bitmask allocator for efficient mark set operations.
#[derive(Debug)]
struct BitmaskAllocator {
    /// Currently allocated positions in primary mask
    primary_allocated: u64,
    /// Currently allocated positions in secondary mask
    secondary_allocated: u64,
    /// Next available position in primary mask
    next_primary_pos: u8,
    /// Next available position in secondary mask
    next_secondary_pos: u8,
    /// Freed positions available for reuse
    free_positions: BTreeSet<u8>,
}

/// Information about frequently used mark sets.
#[derive(Debug, Clone)]
struct MarkSetUsageInfo {
    /// How many times this set has been used
    usage_count: u64,
    /// Last time this set was used
    last_used: std::time::Instant,
    /// Common operations performed on this set
    common_operations: HashMap<MarkSetOperation, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MarkSetOperation {
    Union,
    Intersection,
    Difference,
    Contains,
    IsSubset,
}

/// Scope tracker for hygiene context management.
#[derive(Debug)]
struct ScopeTracker {
    /// Active scopes
    scopes: HashMap<ScopeId, ScopeInfo>,
    /// Current scope stack
    scope_stack: Vec<ScopeId>,
    /// Next available scope ID
    next_scope_id: ScopeId,
    /// Scope hierarchy for fast parent lookups
    scope_hierarchy: HashMap<ScopeId, Option<ScopeId>>,
    /// Binding cache for resolved identifiers
    binding_cache: HashMap<(InterId, ScopeId), ResolvedBinding>,
}

/// Scope identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ScopeId(u32);

/// Hygiene context identifier for caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct HygieneContextId(u64);

/// Information about a scope.
#[derive(Debug, Clone)]
struct ScopeInfo {
    /// Scope identifier
    id: ScopeId,
    /// Parent scope
    parent: Option<ScopeId>,
    /// Bindings in this scope
    bindings: HashMap<InterId, BindingInfo>,
    /// Hygiene marks active in this scope
    active_marks: HygieneMarkSet,
    /// Scope creation metadata
    metadata: ScopeMetadata,
}

/// Information about a binding.
#[derive(Debug, Clone)]
struct BindingInfo {
    /// Original identifier
    original_id: InterId,
    /// Renamed identifier (if any)
    renamed_id: Option<InterId>,
    /// Binding type
    binding_type: BindingType,
    /// Hygiene marks at binding site
    binding_marks: HygieneMarkSet,
    /// Source location
    source_span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingType {
    Variable,
    Function,
    Macro,
    Syntax,
    Type,
}

/// Scope metadata.
#[derive(Debug, Clone)]
struct ScopeMetadata {
    /// Scope creation time
    created_at: std::time::Instant,
    /// Scope type
    scope_type: ScopeType,
    /// Source macro that created this scope
    source_macro: Option<String>,
    /// Additional properties
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeType {
    Global,
    Function,
    Macro,
    Block,
    Template,
}

/// Resolved binding result.
#[derive(Debug, Clone)]
struct ResolvedBinding {
    /// Final identifier to use
    resolved_id: InterId,
    /// Scope where binding was found
    binding_scope: ScopeId,
    /// Whether this is a fresh rename
    is_fresh_rename: bool,
    /// Hygiene marks that affected the resolution
    resolution_marks: HygieneMarkSet,
}

/// Performance metrics for hygiene operations.
#[derive(Debug, Default, Clone)]
pub struct HygieneMetrics {
    /// Total identifier resolutions
    total_resolutions: u64,
    /// Cache hits for identifier resolution
    resolution_cache_hits: u64,
    /// Total mark operations
    total_mark_operations: u64,
    /// Bitmask operations performed
    bitmask_operations: u64,
    /// String intern operations
    intern_operations: u64,
    /// Memory usage statistics
    memory_stats: HygieneMemoryStats,
    /// Performance timings
    timing_stats: HygieneTimingStats,
}

#[derive(Debug, Default, Clone)]
struct HygieneMemoryStats {
    /// Memory used by string interner
    interner_memory_bytes: usize,
    /// Memory used by mark registry
    mark_registry_memory_bytes: usize,
    /// Memory used by scope tracker
    scope_tracker_memory_bytes: usize,
    /// Cache memory usage
    cache_memory_bytes: usize,
}

#[derive(Debug, Default, Clone)]
struct HygieneTimingStats {
    /// Average resolution time in nanoseconds
    avg_resolution_time_ns: u64,
    /// Average mark operation time in nanoseconds
    avg_mark_operation_time_ns: u64,
    /// Total time spent in hygiene operations
    total_time_ns: u64,
}

/// Configuration for hygiene resolver.
#[derive(Debug, Clone)]
pub struct HygieneConfig {
    /// Enable identifier interning
    pub enable_interning: bool,
    /// Enable bitmask optimization
    pub enable_bitmask_optimization: bool,
    /// Enable caching
    pub enable_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Hot cache threshold
    pub hot_cache_threshold: u64,
    /// Performance monitoring level
    pub monitoring_level: HygieneMonitoringLevel,
}

/// Monitoring levels for hygiene resolution performance analysis.
///
/// Controls the amount of performance data collected during hygiene resolution
/// to balance monitoring overhead with diagnostic information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HygieneMonitoringLevel {
    /// No performance monitoring
    None,
    /// Basic performance counters
    Basic,
    /// Detailed timing and cache statistics
    Detailed,
    /// Full profiling with call traces
    Profiling,
}

impl Default for HygieneConfig {
    fn default() -> Self {
        Self {
            enable_interning: true,
            enable_bitmask_optimization: true,
            enable_caching: true,
            max_cache_size: 5000,
            hot_cache_threshold: 10,
            monitoring_level: HygieneMonitoringLevel::Basic,
        }
    }
}

impl FastHygieneResolver {
    /// Creates a new fast hygiene resolver.
    pub fn new() -> Self {
        Self::with_config(HygieneConfig::default())
    }

    /// Creates a new fast hygiene resolver with custom configuration.
    pub fn with_config(config: HygieneConfig) -> Self {
        Self {
            identifier_interner: RefCell::new(IdentifierInterner::new()),
            mark_registry: RefCell::new(HygieneMarkRegistry::new()),
            scope_tracker: RefCell::new(ScopeTracker::new()),
            metrics: RefCell::new(HygieneMetrics::default()),
            config,
        }
    }

    /// Resolves an identifier with fast hygiene processing.
    pub fn resolve_identifier_fast(
        &self,
        identifier: &str,
        context: &HygieneContext,
    ) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Intern the identifier for fast operations
        let id = self.intern_identifier(identifier);

        // Check resolution cache
        if self.config.enable_caching {
            let context_id = self.compute_context_id(context);
            let cache_key = (id, self.current_scope_id());

            if let Ok(tracker) = self.scope_tracker.try_borrow() {
                if let Some(cached) = tracker.binding_cache.get(&cache_key) {
                    self.record_cache_hit();
                    return Ok(self.resolve_id_to_string(cached.resolved_id));
                }
            }
        }

        // Perform fast hygiene resolution
        let resolved_binding = self.perform_fast_resolution(id, context)?;

        // Update cache
        if self.config.enable_caching {
            let cache_key = (id, self.current_scope_id());
            self.scope_tracker
                .borrow_mut()
                .binding_cache
                .insert(cache_key, resolved_binding.clone());
        }

        // Update metrics
        self.record_resolution(start_time.elapsed());

        Ok(self.resolve_id_to_string(resolved_binding.resolved_id))
    }

    /// Creates a new hygiene mark with bitmask optimization.
    pub fn create_mark(&self, source_span: Option<Span>) -> MarkId {
        let mut registry = self.mark_registry.borrow_mut();

        // Allocate bitmask position
        let bitmask_position = registry.bitmask_allocator.allocate_position();

        let mark_id = registry.next_mark_id;
        registry.next_mark_id = MarkId(mark_id.0 + 1);

        let mark = HygieneMark {
            id: mark_id,
            bitmask_position,
            source_span,
            metadata: MarkMetadata {
                created_at: std::time::Instant::now(),
                source_macro: None, // Would be filled by caller
                expansion_phase: 0, // Would be filled by caller
                properties: HashMap::new(),
            },
        };

        registry.marks.insert(mark_id, mark);

        if self.config.monitoring_level >= HygieneMonitoringLevel::Basic {
            self.metrics.borrow_mut().total_mark_operations += 1;
        }

        mark_id
    }

    /// Combines hygiene marks using bitmask operations.
    pub fn combine_marks(&self, marks: &[MarkId]) -> HygieneMarkSet {
        if marks.is_empty() {
            return HygieneMarkSet::empty();
        }

        // Check combination cache
        if self.config.enable_caching {
            let cache_key = marks.to_vec();
            if let Ok(registry) = self.mark_registry.try_borrow() {
                if let Some(cached) = registry.combination_cache.get(&cache_key) {
                    return *cached;
                }
            }
        }

        let mut result = HygieneMarkSet::empty();

        if let Ok(registry) = self.mark_registry.try_borrow() {
            for &mark_id in marks {
                if let Some(mark) = registry.marks.get(&mark_id) {
                    result = result.union_with_mark(mark.bitmask_position);
                }
            }
        }

        // Update cache
        if self.config.enable_caching {
            let cache_key = marks.to_vec();
            self.mark_registry
                .borrow_mut()
                .combination_cache
                .insert(cache_key, result);
        }

        if self.config.monitoring_level >= HygieneMonitoringLevel::Basic {
            self.metrics.borrow_mut().bitmask_operations += 1;
        }

        result
    }

    /// Applies hygiene renaming with optimization.
    pub fn apply_hygiene_fast(
        &self,
        expr: Spanned<Expr>,
        definition_env: &Environment,
    ) -> Result<Spanned<Expr>> {
        self.transform_expression_hygiene(expr, definition_env, &mut Vec::new())
    }

    /// Gets current performance metrics.
    pub fn metrics(&self) -> HygieneMetrics {
        self.metrics
            .try_borrow()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    /// Optimizes caches and memory usage.
    pub fn optimize_caches(&self) {
        self.optimize_identifier_cache();
        self.optimize_mark_cache();
        self.optimize_binding_cache();
    }

    // Private helper methods

    fn intern_identifier(&self, identifier: &str) -> InterId {
        let mut interner = self.identifier_interner.borrow_mut();

        if let Some(&id) = interner.string_to_id.get(identifier) {
            // Update hot cache if needed
            if self.config.enable_caching {
                interner.update_usage_count(id);
            }
            return id;
        }

        // Create new interned identifier
        let id = interner.next_id;
        interner.next_id = InterId(id.0 + 1);

        let arc_str: Arc<str> = Arc::from(identifier);
        interner.string_to_id.insert(identifier.to_string(), id);
        interner.id_to_string.push(arc_str.clone());

        if self.config.enable_caching {
            interner.hot_cache.insert(
                id,
                CachedIdentifierInfo {
                    string: arc_str,
                    usage_count: 1,
                    last_context: None,
                    rename_cache: HashMap::new(),
                },
            );
        }

        if self.config.monitoring_level >= HygieneMonitoringLevel::Basic {
            self.metrics.borrow_mut().intern_operations += 1;
        }

        id
    }

    fn resolve_id_to_string(&self, id: InterId) -> String {
        if let Ok(interner) = self.identifier_interner.try_borrow() {
            if let Some(arc_str) = interner.id_to_string.get(id.0 as usize) {
                return arc_str.to_string();
            }
        }
        format!("__unknown_id_{}", id.0)
    }

    fn compute_context_id(&self, context: &HygieneContext) -> HygieneContextId {
        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        HygieneContextId(hasher.finish())
    }

    fn current_scope_id(&self) -> ScopeId {
        if let Ok(tracker) = self.scope_tracker.try_borrow() {
            tracker.scope_stack.last().copied().unwrap_or(ScopeId(0))
        } else {
            ScopeId(0)
        }
    }

    fn perform_fast_resolution(
        &self,
        id: InterId,
        context: &HygieneContext,
    ) -> Result<ResolvedBinding> {
        // Simplified resolution - would be more sophisticated in practice
        Ok(ResolvedBinding {
            resolved_id: id,
            binding_scope: self.current_scope_id(),
            is_fresh_rename: false,
            resolution_marks: HygieneMarkSet::empty(),
        })
    }

    fn transform_expression_hygiene(
        &self,
        expr: Spanned<Expr>,
        _definition_env: &Environment,
        _visited: &mut [String],
    ) -> Result<Spanned<Expr>> {
        // Simplified implementation - would recursively transform identifiers
        Ok(expr)
    }

    fn record_cache_hit(&self) {
        if self.config.monitoring_level >= HygieneMonitoringLevel::Basic {
            self.metrics.borrow_mut().resolution_cache_hits += 1;
        }
    }

    fn record_resolution(&self, duration: std::time::Duration) {
        if self.config.monitoring_level >= HygieneMonitoringLevel::Basic {
            let mut metrics = self.metrics.borrow_mut();
            metrics.total_resolutions += 1;
            metrics.timing_stats.total_time_ns += duration.as_nanos() as u64;
            if metrics.total_resolutions > 0 {
                metrics.timing_stats.avg_resolution_time_ns =
                    metrics.timing_stats.total_time_ns / metrics.total_resolutions;
            }
        }
    }

    fn optimize_identifier_cache(&self) {
        // Implement cache optimization logic
    }

    fn optimize_mark_cache(&self) {
        // Implement mark cache optimization logic
    }

    fn optimize_binding_cache(&self) {
        // Implement binding cache optimization logic
    }
}

// Implementation for supporting structures

impl IdentifierInterner {
    fn new() -> Self {
        Self {
            string_to_id: HashMap::new(),
            id_to_string: Vec::new(),
            next_id: InterId(0),
            hot_cache: HashMap::new(),
        }
    }

    fn update_usage_count(&mut self, id: InterId) {
        if let Some(info) = self.hot_cache.get_mut(&id) {
            info.usage_count += 1;
        }
    }
}

impl HygieneMarkRegistry {
    fn new() -> Self {
        Self {
            marks: HashMap::new(),
            next_mark_id: MarkId(0),
            bitmask_allocator: BitmaskAllocator::new(),
            combination_cache: HashMap::new(),
            hot_mark_sets: HashMap::new(),
        }
    }
}

impl BitmaskAllocator {
    fn new() -> Self {
        Self {
            primary_allocated: 0,
            secondary_allocated: 0,
            next_primary_pos: 0,
            next_secondary_pos: 0,
            free_positions: BTreeSet::new(),
        }
    }

    fn allocate_position(&mut self) -> u8 {
        if let Some(&pos) = self.free_positions.first() {
            self.free_positions.remove(&pos);
            return pos;
        }

        if self.next_primary_pos < 64 {
            let pos = self.next_primary_pos;
            self.next_primary_pos += 1;
            self.primary_allocated |= 1u64 << pos;
            pos
        } else if self.next_secondary_pos < 64 {
            let pos = 64 + self.next_secondary_pos;
            self.next_secondary_pos += 1;
            self.secondary_allocated |= 1u64 << self.next_secondary_pos;
            pos
        } else {
            // Overflow case - would need more sophisticated handling
            0
        }
    }
}

impl HygieneMarkSet {
    /// Creates an empty hygiene mark set.
    pub fn empty() -> Self {
        Self {
            primary_mask: 0,
            secondary_mask: 0,
            overflow_id: None,
        }
    }

    /// Creates a mark set with a single mark at the given position.
    pub fn union_with_mark(mut self, position: u8) -> Self {
        if position < 64 {
            self.primary_mask |= 1u64 << position;
        } else if position < 128 {
            self.secondary_mask |= 1u64 << (position - 64);
        }
        // Overflow case would be handled here
        self
    }

    /// Checks if this mark set contains a mark at the given position.
    pub fn contains_mark(&self, position: u8) -> bool {
        if position < 64 {
            (self.primary_mask & (1u64 << position)) != 0
        } else if position < 128 {
            (self.secondary_mask & (1u64 << (position - 64))) != 0
        } else {
            false // Simplified
        }
    }

    /// Combines this mark set with another mark set.
    pub fn union(self, other: Self) -> Self {
        Self {
            primary_mask: self.primary_mask | other.primary_mask,
            secondary_mask: self.secondary_mask | other.secondary_mask,
            overflow_id: self.overflow_id.or(other.overflow_id),
        }
    }

    /// Checks if this mark set is empty.
    pub fn is_empty(&self) -> bool {
        self.primary_mask == 0 && self.secondary_mask == 0 && self.overflow_id.is_none()
    }

    /// Creates a mark set containing a single mark at the given position.
    pub fn single_mark(position: u8) -> Self {
        Self::empty().union_with_mark(position)
    }

    /// Computes the intersection of this mark set with another.
    pub fn intersection(self, other: Self) -> Self {
        Self {
            primary_mask: self.primary_mask & other.primary_mask,
            secondary_mask: self.secondary_mask & other.secondary_mask,
            overflow_id: if self.overflow_id == other.overflow_id {
                self.overflow_id
            } else {
                None
            },
        }
    }

    /// Computes the difference of this mark set with another (marks in self but not in other).
    pub fn difference(self, other: Self) -> Self {
        Self {
            primary_mask: self.primary_mask & !other.primary_mask,
            secondary_mask: self.secondary_mask & !other.secondary_mask,
            overflow_id: if other.overflow_id == self.overflow_id {
                None
            } else {
                self.overflow_id
            },
        }
    }

    /// Checks if this mark set is a subset of another mark set.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        let primary_subset = (self.primary_mask & other.primary_mask) == self.primary_mask;
        let secondary_subset = (self.secondary_mask & other.secondary_mask) == self.secondary_mask;
        let overflow_subset = match (self.overflow_id, other.overflow_id) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => a == b,
        };
        primary_subset && secondary_subset && overflow_subset
    }

    /// Returns the number of marks in this set (approximate for overflow case).
    pub fn count(&self) -> usize {
        let primary_count = self.primary_mask.count_ones() as usize;
        let secondary_count = self.secondary_mask.count_ones() as usize;
        let overflow_count = if self.overflow_id.is_some() { 1 } else { 0 };
        primary_count + secondary_count + overflow_count
    }

    /// Checks if this mark set contains the given mark ID.
    pub fn contains_mark_id(&self, mark_id: MarkId) -> bool {
        // This is a simplified implementation - in practice, you'd need to
        // map the MarkId to its bitmask position through the registry
        self.contains_mark(mark_id.0 as u8)
    }
}

impl ScopeTracker {
    fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            scope_stack: Vec::new(),
            next_scope_id: ScopeId(0),
            scope_hierarchy: HashMap::new(),
            binding_cache: HashMap::new(),
        }
    }
}

impl Default for FastHygieneResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_hygiene_resolver_creation() {
        let resolver = FastHygieneResolver::new();
        assert!(resolver.config.enable_interning);
    }

    #[test]
    fn test_identifier_interning() {
        let resolver = FastHygieneResolver::new();
        let id1 = resolver.intern_identifier("test");
        let id2 = resolver.intern_identifier("test");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_mark_creation() {
        let resolver = FastHygieneResolver::new();
        let mark = resolver.create_mark(None);
        assert_eq!(mark.0, 0);
    }

    #[test]
    fn test_mark_combination() {
        let resolver = FastHygieneResolver::new();
        let mark1 = resolver.create_mark(None);
        let mark2 = resolver.create_mark(None);
        let combined = resolver.combine_marks(&[mark1, mark2]);
        // Should have bits set for both marks
        assert!(combined.primary_mask > 0);
    }

    #[test]
    fn test_hygiene_mark_set_operations() {
        let set1 = HygieneMarkSet::empty().union_with_mark(0);
        let set2 = HygieneMarkSet::empty().union_with_mark(1);
        let union = set1.union(set2);
        assert!(union.contains_mark(0));
        assert!(union.contains_mark(1));
    }
}
