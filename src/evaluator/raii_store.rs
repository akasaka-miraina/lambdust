//! RAII-based Store implementation leveraging Rust's ownership model
//!
//! This module provides a more Rust-idiomatic memory management system
//! that reduces the need for explicit garbage collection by utilizing
//! RAII (Resource Acquisition Is Initialization) and smart pointers.

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
        let mut to_remove = Vec::new();

        for (id, cell) in &self.cells {
            // Remove very old or long-idle locations
            if cell.age() > self.cleanup_age_threshold
                || cell.idle_time() > self.cleanup_idle_threshold
            {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
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
            Value::Box(_) => 24, // Rc<RefCell<Value>> overhead
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
}
