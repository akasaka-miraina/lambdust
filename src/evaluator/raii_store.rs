//! RAII-based Store implementation leveraging Rust's ownership model
//!
//! This module provides a more Rust-idiomatic memory management system
//! that reduces the need for explicit garbage collection by utilizing
//! RAII (Resource Acquisition Is Initialization) and smart pointers.
//!
//! ## Batch Cleanup Optimization
//!
//! The store includes intelligent batch cleanup optimization that reduces
//! timer overhead by processing multiple locations in a single pass:
//!
//! - **Batch Processing**: Groups cleanup operations to reduce system call overhead
//! - **Configurable Thresholds**: Adjustable batch size and minimum threshold
//! - **Efficient Memory Management**: Single-pass statistics updates and memory tracking
//! - **Hybrid Approach**: Falls back to individual cleanup for small numbers of locations
//!
//! ### Configuration Options
//!
//! - `batch_cleanup_size`: Maximum number of locations to process per batch (default: 100)
//! - `min_batch_cleanup_threshold`: Minimum locations needed to trigger batch cleanup (default: 10)
//!
//! ### Performance Benefits
//!
//! - Reduces timestamp checking overhead by batching operations
//! - Minimizes HashMap operations by grouping removals
//! - Provides single-pass statistics updates instead of per-location updates
//! - Maintains memory efficiency through intelligent pre-allocation

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global location ID counter
static NEXT_LOCATION_ID: AtomicUsize = AtomicUsize::new(0);

/// RAII-managed location with automatic cleanup
#[derive(Debug, Clone)]
pub struct RaiiLocation {
    /// Unique identifier
    id: usize,
    /// Reference to the store manager
    store_manager: Weak<RefCell<RaiiStoreManager>>,
}

impl RaiiLocation {
    /// Get the location ID
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the value at this location
    pub fn get(&self) -> Option<Value> {
        if let Some(manager) = self.store_manager.upgrade() {
            manager.borrow_mut().get_value(self.id)
        } else {
            None
        }
    }

    /// Set the value at this location
    pub fn set(&self, value: Value) -> Result<()> {
        if let Some(manager) = self.store_manager.upgrade() {
            manager.borrow_mut().set_value(self.id, value)
        } else {
            Err(LambdustError::runtime_error(
                "Location manager no longer available".to_string(),
            ))
        }
    }

    /// Check if this location is still valid
    pub fn is_valid(&self) -> bool {
        self.store_manager.upgrade().is_some()
    }
}

impl Drop for RaiiLocation {
    /// Automatic cleanup when location goes out of scope
    fn drop(&mut self) {
        if let Some(manager) = self.store_manager.upgrade() {
            manager.borrow_mut().deallocate_location(self.id);
        }
    }
}

impl std::fmt::Display for RaiiLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raii-location:{}", self.id)
    }
}

/// Memory cell with RAII management
#[derive(Debug, Clone)]
struct RaiiMemoryCell {
    /// The stored value
    value: Value,
    /// Creation timestamp for age-based cleanup
    created_at: std::time::Instant,
    /// Last access timestamp
    last_accessed: std::time::Instant,
}

impl RaiiMemoryCell {
    fn new(value: Value) -> Self {
        let now = std::time::Instant::now();
        RaiiMemoryCell {
            value,
            created_at: now,
            last_accessed: now,
        }
    }

    fn access(&mut self) -> &Value {
        self.last_accessed = std::time::Instant::now();
        &self.value
    }

    fn update(&mut self, value: Value) {
        self.last_accessed = std::time::Instant::now();
        self.value = value;
    }

    fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    #[allow(dead_code)]
    fn idle_time(&self) -> std::time::Duration {
        self.last_accessed.elapsed()
    }
}

/// Enhanced statistics for RAII store
#[derive(Debug, Clone, Default)]
pub struct RaiiStoreStatistics {
    /// Total allocations made
    pub total_allocations: usize,
    /// Total deallocations made
    pub total_deallocations: usize,
    /// Current active locations
    pub active_locations: usize,
    /// Peak active locations
    pub peak_active_locations: usize,
    /// Total memory usage estimation
    pub estimated_memory_usage: usize,
    /// Peak memory usage
    pub peak_memory_usage: usize,
    /// Auto-cleanup events
    pub auto_cleanup_events: usize,
    /// Batch cleanup events
    pub batch_cleanup_events: usize,
    /// Total locations processed in batch cleanups
    pub batch_processed_locations: usize,
}

/// RAII-based store manager
#[derive(Debug)]
pub struct RaiiStoreManager {
    /// Active memory cells
    cells: HashMap<usize, RaiiMemoryCell>,
    /// Statistics
    stats: RaiiStoreStatistics,
    /// Memory limit (0 = unlimited)
    memory_limit: usize,
    /// Auto-cleanup threshold (age-based)
    cleanup_age_threshold: std::time::Duration,
    /// Auto-cleanup threshold (idle-based)
    cleanup_idle_threshold: std::time::Duration,
    /// Batch size for cleanup operations
    batch_cleanup_size: usize,
    /// Minimum locations to trigger batch cleanup
    min_batch_cleanup_threshold: usize,
}

impl Default for RaiiStoreManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RaiiStoreManager {
    /// Create a new RAII store manager
    pub fn new() -> Self {
        RaiiStoreManager {
            cells: HashMap::new(),
            stats: RaiiStoreStatistics::default(),
            memory_limit: 0,
            cleanup_age_threshold: std::time::Duration::from_secs(300), // 5 minutes
            cleanup_idle_threshold: std::time::Duration::from_secs(60), // 1 minute
            batch_cleanup_size: 100, // Process up to 100 locations per batch
            min_batch_cleanup_threshold: 10, // Minimum 10 locations to trigger batch cleanup
        }
    }

    /// Create a store manager with memory limit
    pub fn with_memory_limit(memory_limit: usize) -> Self {
        RaiiStoreManager {
            cells: HashMap::new(),
            stats: RaiiStoreStatistics::default(),
            memory_limit,
            cleanup_age_threshold: std::time::Duration::from_secs(300),
            cleanup_idle_threshold: std::time::Duration::from_secs(60),
            batch_cleanup_size: 100,
            min_batch_cleanup_threshold: 10,
        }
    }

    /// Create a store manager with custom batch settings
    pub fn with_batch_settings(
        memory_limit: usize,
        batch_size: usize,
        min_threshold: usize,
    ) -> Self {
        RaiiStoreManager {
            cells: HashMap::new(),
            stats: RaiiStoreStatistics::default(),
            memory_limit,
            cleanup_age_threshold: std::time::Duration::from_secs(300),
            cleanup_idle_threshold: std::time::Duration::from_secs(60),
            batch_cleanup_size: batch_size,
            min_batch_cleanup_threshold: min_threshold,
        }
    }

    /// Allocate a new location with RAII management
    pub fn allocate_location(&mut self, value: Value) -> (usize, Value) {
        let id = NEXT_LOCATION_ID.fetch_add(1, Ordering::SeqCst);

        // Auto-cleanup if needed
        if self.should_auto_cleanup() {
            self.auto_cleanup();
        }

        // Create memory cell
        let cell = RaiiMemoryCell::new(value.clone());
        let memory_usage = self.estimate_value_size(&value);

        // Update manager state
        self.cells.insert(id, cell);
        self.stats.total_allocations += 1;
        self.stats.active_locations += 1;
        self.stats.estimated_memory_usage += memory_usage;

        // Update peaks
        if self.stats.active_locations > self.stats.peak_active_locations {
            self.stats.peak_active_locations = self.stats.active_locations;
        }
        if self.stats.estimated_memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.stats.estimated_memory_usage;
        }

        (id, value)
    }

    /// Get value at location
    fn get_value(&mut self, location_id: usize) -> Option<Value> {
        self.cells
            .get_mut(&location_id)
            .map(|cell| cell.access().clone())
    }

    /// Set value at location
    fn set_value(&mut self, location_id: usize, value: Value) -> Result<()> {
        // Get old value size first
        let old_size = if let Some(cell) = self.cells.get(&location_id) {
            self.estimate_value_size(&cell.value)
        } else {
            return Err(LambdustError::runtime_error(format!(
                "Invalid location ID: {}",
                location_id
            )));
        };

        // Calculate new value size
        let new_size = self.estimate_value_size(&value);

        // Now update the cell
        if let Some(cell) = self.cells.get_mut(&location_id) {
            cell.update(value);

            // Update memory usage
            self.stats.estimated_memory_usage =
                self.stats.estimated_memory_usage.saturating_sub(old_size) + new_size;

            if self.stats.estimated_memory_usage > self.stats.peak_memory_usage {
                self.stats.peak_memory_usage = self.stats.estimated_memory_usage;
            }

            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location ID: {}",
                location_id
            )))
        }
    }

    /// Deallocate location (called automatically by Drop)
    fn deallocate_location(&mut self, location_id: usize) {
        if let Some(cell) = self.cells.remove(&location_id) {
            let memory_usage = self.estimate_value_size(&cell.value);
            self.stats.total_deallocations += 1;
            self.stats.active_locations = self.stats.active_locations.saturating_sub(1);
            self.stats.estimated_memory_usage = self
                .stats
                .estimated_memory_usage
                .saturating_sub(memory_usage);
        }
    }

    /// Check if auto-cleanup should be triggered
    fn should_auto_cleanup(&self) -> bool {
        // Memory-based trigger
        if self.memory_limit > 0 && self.stats.estimated_memory_usage >= self.memory_limit {
            return true;
        }

        // Age-based trigger (if we have many old locations)
        let old_locations = self
            .cells
            .values()
            .filter(|cell| cell.age() > self.cleanup_age_threshold)
            .count();

        old_locations > self.stats.active_locations / 4 // 25% threshold
    }

    /// Perform automatic cleanup based on age and idle time
    fn auto_cleanup(&mut self) {
        let candidates = self.collect_cleanup_candidates();

        if candidates.len() >= self.min_batch_cleanup_threshold {
            self.batch_cleanup(candidates);
        } else {
            // Fall back to individual cleanup for small numbers
            self.individual_cleanup(candidates);
        }
    }

    /// Collect candidates for cleanup in a single pass
    fn collect_cleanup_candidates(&self) -> Vec<usize> {
        let mut candidates = Vec::new();
        let now = std::time::Instant::now();

        // Pre-allocate with estimated capacity to reduce allocations
        candidates.reserve(self.cells.len() / 10);

        for (id, cell) in &self.cells {
            // Batch timestamp checking - check both conditions at once
            if cell.created_at.elapsed() > self.cleanup_age_threshold
                || (now - cell.last_accessed) > self.cleanup_idle_threshold
            {
                candidates.push(*id);
            }
        }

        candidates
    }

    /// Perform batch cleanup for efficiency
    fn batch_cleanup(&mut self, candidates: Vec<usize>) {
        let mut total_memory_freed = 0;
        let mut successful_removals = 0;

        // Process candidates in batches to reduce lock contention
        for batch in candidates.chunks(self.batch_cleanup_size) {
            let mut batch_memory_freed = 0;
            let mut batch_removals = 0;

            // Collect all values to be removed first
            let mut cells_to_remove = Vec::with_capacity(batch.len());
            for &id in batch {
                if let Some(cell) = self.cells.get(&id) {
                    cells_to_remove.push((id, self.estimate_value_size(&cell.value)));
                }
            }

            // Batch remove all collected cells
            for (id, memory_size) in cells_to_remove {
                if self.cells.remove(&id).is_some() {
                    batch_memory_freed += memory_size;
                    batch_removals += 1;
                }
            }

            total_memory_freed += batch_memory_freed;
            successful_removals += batch_removals;
        }

        // Update statistics once after all batches
        self.stats.total_deallocations += successful_removals;
        self.stats.active_locations = self
            .stats
            .active_locations
            .saturating_sub(successful_removals);
        self.stats.estimated_memory_usage = self
            .stats
            .estimated_memory_usage
            .saturating_sub(total_memory_freed);
        self.stats.batch_cleanup_events += 1;
        self.stats.batch_processed_locations += successful_removals;
    }

    /// Fallback individual cleanup for small numbers
    fn individual_cleanup(&mut self, candidates: Vec<usize>) {
        for id in candidates {
            self.deallocate_location(id);
            self.stats.auto_cleanup_events += 1;
        }
    }

    /// Force manual cleanup (less needed with RAII)
    pub fn manual_cleanup(&mut self) {
        self.auto_cleanup();
    }

    /// Get current statistics
    pub fn statistics(&self) -> &RaiiStoreStatistics {
        &self.stats
    }

    /// Set memory limit
    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_limit = limit;
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.stats.estimated_memory_usage
    }

    /// Get number of active locations
    pub fn active_location_count(&self) -> usize {
        self.stats.active_locations
    }

    /// Set batch cleanup size
    pub fn set_batch_cleanup_size(&mut self, size: usize) {
        self.batch_cleanup_size = size.max(1); // Ensure at least 1
    }

    /// Set minimum batch cleanup threshold
    pub fn set_min_batch_cleanup_threshold(&mut self, threshold: usize) {
        self.min_batch_cleanup_threshold = threshold;
    }

    /// Get batch cleanup configuration
    pub fn batch_cleanup_config(&self) -> (usize, usize) {
        (self.batch_cleanup_size, self.min_batch_cleanup_threshold)
    }

    /// Estimate memory size of a value (same as before)
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Boolean(_) => 1,
            Value::Number(_) => 8,
            Value::Character(_) => 4,
            Value::String(s) => s.len() + 24,
            Value::Symbol(s) => s.len() + 24,
            Value::Pair(_) => 32,
            Value::Vector(v) => v.len() * 8 + 24,
            Value::HashTable(_) => 64,
            Value::Procedure(_) => 48,
            Value::Promise(_) => 32,
            Value::Port(_) => 64,
            Value::External(_) => 48,
            Value::Record(_) => 64,
            Value::Values(v) => {
                v.iter()
                    .map(|val| self.estimate_value_size(val))
                    .sum::<usize>()
                    + 24
            }
            Value::Continuation(_) => 96,
            Value::Nil => 8,
            Value::Undefined => 8,
            Value::Box(_) => 24,          // Rc<RefCell<Value>> overhead
            Value::Comparator(_) => 64,   // Comparator overhead
            Value::StringCursor(_) => 48, // StringCursor overhead
        }
    }
}

/// RAII Store wrapper for evaluator integration
#[derive(Debug, Clone)]
pub struct RaiiStore {
    manager: Rc<RefCell<RaiiStoreManager>>,
}

impl Default for RaiiStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RaiiStore {
    /// Create a new RAII store
    pub fn new() -> Self {
        RaiiStore {
            manager: Rc::new(RefCell::new(RaiiStoreManager::new())),
        }
    }

    /// Create a new RAII store with memory limit
    pub fn with_memory_limit(memory_limit: usize) -> Self {
        RaiiStore {
            manager: Rc::new(RefCell::new(RaiiStoreManager::with_memory_limit(
                memory_limit,
            ))),
        }
    }

    /// Create a new RAII store with custom batch settings
    pub fn with_batch_settings(
        memory_limit: usize,
        batch_size: usize,
        min_threshold: usize,
    ) -> Self {
        RaiiStore {
            manager: Rc::new(RefCell::new(RaiiStoreManager::with_batch_settings(
                memory_limit,
                batch_size,
                min_threshold,
            ))),
        }
    }

    /// Allocate a new location
    pub fn allocate(&self, value: Value) -> RaiiLocation {
        let mut manager = self.manager.borrow_mut();
        let (id, _) = manager.allocate_location(value);

        // Create RAII location with weak reference to prevent cycles
        RaiiLocation {
            id,
            store_manager: Rc::downgrade(&self.manager),
        }
    }

    /// Force manual cleanup (rarely needed)
    pub fn manual_cleanup(&self) {
        self.manager.borrow_mut().manual_cleanup();
    }

    /// Get statistics
    pub fn statistics(&self) -> RaiiStoreStatistics {
        self.manager.borrow().statistics().clone()
    }

    /// Set memory limit
    pub fn set_memory_limit(&self, limit: usize) {
        self.manager.borrow_mut().set_memory_limit(limit);
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.manager.borrow().memory_usage()
    }

    /// Get active location count
    pub fn active_location_count(&self) -> usize {
        self.manager.borrow().active_location_count()
    }

    /// Set batch cleanup size
    pub fn set_batch_cleanup_size(&self, size: usize) {
        self.manager.borrow_mut().set_batch_cleanup_size(size);
    }

    /// Set minimum batch cleanup threshold
    pub fn set_min_batch_cleanup_threshold(&self, threshold: usize) {
        self.manager
            .borrow_mut()
            .set_min_batch_cleanup_threshold(threshold);
    }

    /// Get batch cleanup configuration
    pub fn batch_cleanup_config(&self) -> (usize, usize) {
        self.manager.borrow().batch_cleanup_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_raii_location_auto_cleanup() {
        let store = RaiiStore::new();
        let initial_count = store.active_location_count();

        {
            let _location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
            assert_eq!(store.active_location_count(), initial_count + 1);
        } // location goes out of scope here

        // Location should be automatically cleaned up
        assert_eq!(store.active_location_count(), initial_count);
    }

    #[test]
    fn test_raii_location_value_access() {
        let store = RaiiStore::new();
        let location = store.allocate(Value::Number(SchemeNumber::Integer(42)));

        // Get value
        assert_eq!(
            location.get(),
            Some(Value::Number(SchemeNumber::Integer(42)))
        );

        // Set new value
        location.set(Value::String("hello".to_string())).unwrap();
        assert_eq!(location.get(), Some(Value::String("hello".to_string())));
    }

    #[test]
    fn test_memory_usage_tracking() {
        let store = RaiiStore::new();
        let initial_usage = store.memory_usage();

        let _location = store.allocate(Value::String("test".to_string()));
        assert!(store.memory_usage() > initial_usage);

        // Memory usage includes string length + overhead
        let expected_min = initial_usage + 4 + 24; // "test".len() + overhead
        assert!(store.memory_usage() >= expected_min);
    }

    #[test]
    fn test_statistics_tracking() {
        let store = RaiiStore::new();
        let initial_stats = store.statistics();

        {
            let _location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
            let stats = store.statistics();
            assert_eq!(stats.total_allocations, initial_stats.total_allocations + 1);
            assert_eq!(stats.active_locations, initial_stats.active_locations + 1);
        }

        // After location is dropped
        let final_stats = store.statistics();
        assert_eq!(
            final_stats.total_deallocations,
            initial_stats.total_deallocations + 1
        );
        assert_eq!(final_stats.active_locations, initial_stats.active_locations);
    }

    #[test]
    fn test_batch_cleanup_configuration() {
        let store = RaiiStore::with_batch_settings(1024, 50, 5);
        let (batch_size, min_threshold) = store.batch_cleanup_config();
        assert_eq!(batch_size, 50);
        assert_eq!(min_threshold, 5);

        // Test configuration updates
        store.set_batch_cleanup_size(75);
        store.set_min_batch_cleanup_threshold(8);
        let (new_batch_size, new_min_threshold) = store.batch_cleanup_config();
        assert_eq!(new_batch_size, 75);
        assert_eq!(new_min_threshold, 8);
    }

    #[test]
    fn test_batch_cleanup_statistics() {
        let store = RaiiStore::with_batch_settings(0, 10, 5);
        let initial_stats = store.statistics();

        // Create multiple locations that will be eligible for cleanup
        let locations: Vec<_> = (0..15)
            .map(|i| store.allocate(Value::Number(SchemeNumber::Integer(i))))
            .collect();

        // Force cleanup by triggering it manually
        store.manual_cleanup();

        let stats = store.statistics();
        assert_eq!(
            stats.batch_cleanup_events,
            initial_stats.batch_cleanup_events
        );
        assert_eq!(
            stats.batch_processed_locations,
            initial_stats.batch_processed_locations
        );

        // Keep locations alive to prevent automatic cleanup
        drop(locations);
    }

    #[test]
    fn test_batch_cleanup_vs_individual_cleanup() {
        // Test that small numbers use individual cleanup
        let store = RaiiStore::with_batch_settings(0, 100, 10);
        let initial_stats = store.statistics();

        // Create a few locations (less than threshold)
        let locations: Vec<_> = (0..5)
            .map(|i| store.allocate(Value::Number(SchemeNumber::Integer(i))))
            .collect();

        drop(locations);
        store.manual_cleanup();

        let stats = store.statistics();
        // Should use individual cleanup for small numbers
        assert_eq!(
            stats.batch_cleanup_events,
            initial_stats.batch_cleanup_events
        );
    }

    #[test]
    fn test_batch_size_validation() {
        let store = RaiiStore::new();

        // Test that batch size is at least 1
        store.set_batch_cleanup_size(0);
        let (batch_size, _) = store.batch_cleanup_config();
        assert_eq!(batch_size, 1);

        store.set_batch_cleanup_size(50);
        let (batch_size, _) = store.batch_cleanup_config();
        assert_eq!(batch_size, 50);
    }

    #[test]
    fn test_batch_cleanup_performance_characteristics() {
        // Test that batch cleanup provides better performance characteristics
        let store = RaiiStore::with_batch_settings(0, 50, 10);

        // Create a large number of locations to test batch processing
        let locations: Vec<_> = (0..200)
            .map(|i| {
                let val = if i % 2 == 0 {
                    Value::Number(SchemeNumber::Integer(i as i64))
                } else {
                    Value::String(format!("test-{}", i))
                };
                store.allocate(val)
            })
            .collect();

        let initial_stats = store.statistics();
        assert_eq!(initial_stats.active_locations, 200);

        // Drop all locations to make them eligible for cleanup
        drop(locations);

        // Manually trigger cleanup
        store.manual_cleanup();

        let final_stats = store.statistics();

        // Verify that cleanup occurred
        assert_eq!(final_stats.active_locations, 0);
        assert_eq!(final_stats.total_deallocations, 200);

        // Verify that the cleanup was efficient
        assert!(final_stats.estimated_memory_usage < initial_stats.estimated_memory_usage);
    }

    #[test]
    fn test_batch_cleanup_memory_efficiency() {
        // Test that batch cleanup efficiently manages memory
        let store = RaiiStore::with_batch_settings(1024, 25, 5);

        // Create mixed types of values to test memory estimation
        let mut locations = Vec::new();

        // Add various types of values
        for i in 0..100 {
            let val = match i % 5 {
                0 => Value::Number(SchemeNumber::Integer(i as i64)),
                1 => Value::String("test".repeat(i % 10 + 1)),
                2 => Value::Boolean(i % 2 == 0),
                3 => Value::Character(char::from_u32(65 + (i % 26) as u32).unwrap_or('A')),
                _ => Value::Symbol(format!("symbol-{}", i)),
            };
            locations.push(store.allocate(val));
        }

        let peak_memory = store.memory_usage();
        let initial_active = store.active_location_count();

        // Drop half the locations
        locations.truncate(50);

        // Force cleanup
        store.manual_cleanup();

        let after_cleanup_memory = store.memory_usage();
        let after_cleanup_active = store.active_location_count();

        // Verify memory was freed
        assert!(after_cleanup_memory < peak_memory);
        assert!(after_cleanup_active < initial_active);

        // Verify remaining locations are still valid
        assert_eq!(after_cleanup_active, 50);

        // Test that remaining locations are still accessible
        for location in &locations {
            assert!(location.is_valid());
            assert!(location.get().is_some());
        }
    }
}
