//! Memory management types and implementations
//!
//! This module defines the memory management system including locations,
//! memory cells, store, and statistics for the R7RS evaluator.

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// Location identifier for R7RS memory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location(usize);

impl Location {
    /// Create a new location
    pub fn new(id: usize) -> Self {
        Location(id)
    }

    /// Get the raw location ID
    pub fn id(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "location:{}", self.0)
    }
}

/// Memory cell for tracking location usage
#[derive(Debug, Clone)]
struct MemoryCell {
    /// The stored value
    value: Value,
    /// Reference count for garbage collection
    ref_count: usize,
    /// Generation for generational GC
    #[allow(dead_code)]
    generation: u32,
    /// Mark for mark-and-sweep GC
    marked: bool,
}

impl MemoryCell {
    fn new(value: Value) -> Self {
        MemoryCell {
            value,
            ref_count: 1,
            generation: 0,
            marked: false,
        }
    }
}

/// Store (memory) for locations with R7RS-compliant memory management
#[derive(Debug, Clone)]
pub struct Store {
    /// Mapping from locations to memory cells
    locations: HashMap<usize, MemoryCell>,
    /// Next available location ID
    next_location: usize,
    /// Total memory usage in bytes (approximation)
    memory_usage: usize,
    /// Maximum memory limit (0 = unlimited)
    memory_limit: usize,
    /// Garbage collection threshold
    gc_threshold: usize,
    /// Current generation for generational GC
    current_generation: u32,
    /// Statistics for monitoring
    pub stats: StoreStatistics,
    /// Memory pool for reusing freed cells (Phase 3 optimization)
    cell_pool: Vec<MemoryCell>,
    /// Location pool for reusing location IDs (Phase 3 optimization)
    location_pool: Vec<usize>,
    /// Maximum pool size to prevent unbounded growth
    max_pool_size: usize,
}

/// Statistics for store monitoring
#[derive(Debug, Clone, Default)]
pub struct StoreStatistics {
    /// Total allocations made
    pub total_allocations: usize,
    /// Total deallocations made
    pub total_deallocations: usize,
    /// Number of GC cycles run
    pub gc_cycles: usize,
    /// Peak memory usage
    pub peak_memory_usage: usize,
    /// Memory pool hits (Phase 3 optimization)
    pub pool_hits: usize,
    /// Clone eliminations (Phase 3 optimization)
    pub clone_eliminations: usize,
    /// Memory pool efficiency (0.0 - 1.0)
    pub memory_pool_efficiency: f64,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    /// Create a new store with default settings
    pub fn new() -> Self {
        Store {
            locations: HashMap::new(),
            next_location: 0,
            memory_usage: 0,
            memory_limit: 0,           // Unlimited by default
            gc_threshold: 1024 * 1024, // 1MB default GC threshold
            current_generation: 0,
            stats: StoreStatistics::default(),
            cell_pool: Vec::new(),
            location_pool: Vec::new(),
            max_pool_size: 256, // Default max pool size
        }
    }

    /// Create a new store with custom memory limit
    pub fn with_memory_limit(memory_limit: usize) -> Self {
        Store {
            locations: HashMap::new(),
            next_location: 0,
            memory_usage: 0,
            memory_limit,
            gc_threshold: memory_limit / 4, // GC when 25% of limit is reached
            current_generation: 0,
            stats: StoreStatistics::default(),
            cell_pool: Vec::new(),
            location_pool: Vec::new(),
            max_pool_size: 256, // Default max pool size
        }
    }

    /// Allocate a new location
    pub fn allocate(&mut self, value: Value) -> Location {
        // Check memory limit before allocation
        if self.should_collect_garbage() {
            self.collect_garbage();
        }

        let loc_id = self.next_location;
        let cell = MemoryCell::new(value.clone());

        // Approximate memory usage calculation
        let value_size = self.estimate_value_size(&value);
        self.memory_usage += value_size;

        self.locations.insert(loc_id, cell);
        self.next_location += 1;
        self.stats.total_allocations += 1;

        // Update peak memory usage
        if self.memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.memory_usage;
        }

        Location::new(loc_id)
    }

    /// Allocate a new location using memory pool optimization (Phase 3)
    pub fn allocate_pooled(&mut self, value: Value) -> Location {
        // Check memory limit before allocation
        if self.should_collect_garbage() {
            self.collect_garbage();
        }

        // Try to reuse location ID from pool
        let loc_id = if let Some(reused_id) = self.location_pool.pop() {
            reused_id
        } else {
            let id = self.next_location;
            self.next_location += 1;
            id
        };

        // Try to reuse memory cell from pool
        let cell = if let Some(mut pooled_cell) = self.cell_pool.pop() {
            // Reuse pooled cell, avoiding allocation
            pooled_cell.value = value;
            pooled_cell.ref_count = 1;
            pooled_cell.marked = false;
            pooled_cell.generation += 1; // Increment generation for reuse
            self.stats.pool_hits += 1; // Track pool hit
            pooled_cell
        } else {
            // Create new cell if pool is empty
            MemoryCell::new(value)
        };

        // Approximate memory usage calculation
        let value_size = self.estimate_value_size(&cell.value);
        self.memory_usage += value_size;

        self.locations.insert(loc_id, cell);
        self.stats.total_allocations += 1;

        // Update peak memory usage
        if self.memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.memory_usage;
        }

        Location::new(loc_id)
    }

    /// Get value at location
    pub fn get(&self, location: Location) -> Option<&Value> {
        self.locations.get(&location.id()).map(|cell| &cell.value)
    }

    /// Set value at location
    pub fn set(&mut self, location: Location, value: Value) -> Result<()> {
        let location_id = location.id();

        // Get old value size first
        let old_size = if let Some(cell) = self.locations.get(&location_id) {
            self.estimate_value_size(&cell.value)
        } else {
            return Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )));
        };

        // Calculate new value size
        let new_size = self.estimate_value_size(&value);

        // Now update the cell
        if let Some(cell) = self.locations.get_mut(&location_id) {
            cell.value = value;

            // Update memory usage
            self.memory_usage = self.memory_usage.saturating_sub(old_size) + new_size;

            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )))
        }
    }

    /// Check if location exists
    pub fn contains(&self, location: Location) -> bool {
        self.locations.contains_key(&location.id())
    }

    /// Increment reference count for a location
    pub fn incref(&mut self, location: Location) -> Result<()> {
        if let Some(cell) = self.locations.get_mut(&location.id()) {
            cell.ref_count += 1;
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location for incref: {}",
                location
            )))
        }
    }

    /// Decrement reference count for a location
    pub fn decref(&mut self, location: Location) -> Result<()> {
        if let Some(cell) = self.locations.get_mut(&location.id()) {
            if cell.ref_count > 0 {
                cell.ref_count -= 1;
                if cell.ref_count == 0 {
                    // Location can be garbage collected
                    self.deallocate(location);
                }
            }
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location for decref: {}",
                location
            )))
        }
    }

    /// Deallocate a location
    pub fn deallocate(&mut self, location: Location) {
        if let Some(cell) = self.locations.remove(&location.id()) {
            let value_size = self.estimate_value_size(&cell.value);
            self.memory_usage = self.memory_usage.saturating_sub(value_size);
            self.stats.total_deallocations += 1;

            // Add to memory pools if space available (Phase 3 optimization)
            self.pool_deallocated_resources(location.id(), cell);
        }
    }

    /// Pool deallocated resources for reuse (Phase 3 optimization)
    fn pool_deallocated_resources(&mut self, location_id: usize, mut cell: MemoryCell) {
        // Add location ID to pool if space available
        if self.location_pool.len() < self.max_pool_size {
            self.location_pool.push(location_id);
        }

        // Clear cell value to prevent holding references and add to pool
        if self.cell_pool.len() < self.max_pool_size {
            // Clear the value to release memory and reset state
            cell.value = Value::Undefined;
            cell.ref_count = 0;
            cell.marked = false;
            // Keep generation for validation
            self.cell_pool.push(cell);
        }
    }

    /// Update memory pool efficiency statistics (Phase 3 optimization)
    pub fn update_pool_efficiency(&mut self) {
        if self.stats.total_allocations > 0 {
            self.stats.memory_pool_efficiency =
                self.stats.pool_hits as f64 / self.stats.total_allocations as f64;
        }
    }

    /// Get current pool utilization for monitoring
    pub fn get_pool_utilization(&self) -> (f64, f64) {
        let cell_pool_util = self.cell_pool.len() as f64 / self.max_pool_size as f64;
        let location_pool_util = self.location_pool.len() as f64 / self.max_pool_size as f64;
        (cell_pool_util, location_pool_util)
    }

    /// Force garbage collection
    pub fn collect_garbage(&mut self) {
        self.stats.gc_cycles += 1;

        // Mark phase: mark all reachable locations
        self.mark_phase();

        // Sweep phase: deallocate unmarked locations
        self.sweep_phase();

        // Increment generation
        self.current_generation += 1;
    }

    /// Check if garbage collection should be triggered
    fn should_collect_garbage(&self) -> bool {
        if self.memory_limit > 0 && self.memory_usage >= self.memory_limit {
            return true;
        }

        self.memory_usage >= self.gc_threshold
    }

    /// Mark phase of garbage collection
    fn mark_phase(&mut self) {
        // Reset all marks
        for cell in self.locations.values_mut() {
            cell.marked = false;
        }

        // Mark all locations with positive reference count
        for cell in self.locations.values_mut() {
            if cell.ref_count > 0 {
                cell.marked = true;
            }
        }
    }

    /// Sweep phase of garbage collection
    fn sweep_phase(&mut self) {
        let mut to_remove = Vec::new();

        for (loc_id, cell) in &self.locations {
            if !cell.marked {
                to_remove.push(*loc_id);
            }
        }

        for loc_id in to_remove {
            if let Some(cell) = self.locations.remove(&loc_id) {
                let value_size = self.estimate_value_size(&cell.value);
                self.memory_usage = self.memory_usage.saturating_sub(value_size);
                self.stats.total_deallocations += 1;
            }
        }
    }

    /// Estimate memory size of a value (approximation)
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Boolean(_) => 1,
            Value::Number(_) => 8,
            Value::Character(_) => 4,
            Value::String(s) => s.len() + 24, // String overhead
            Value::Symbol(s) => s.len() + 24,
            Value::Pair(_) => 32,                 // Approximate size of pair
            Value::Vector(v) => v.len() * 8 + 24, // Vector overhead
            Value::HashTable(_) => 64,            // Approximate hash table overhead
            Value::Procedure(_) => 48,            // Approximate procedure overhead
            Value::Promise(_) => 32,
            Value::Port(_) => 64,     // Port overhead
            Value::External(_) => 48, // External object overhead
            Value::Record(_) => 64,   // Record overhead
            Value::Values(v) => {
                v.iter()
                    .map(|val| self.estimate_value_size(val))
                    .sum::<usize>()
                    + 24
            }
            Value::Continuation(_) => 96, // Continuation overhead
            Value::Nil => 8,
            Value::Undefined => 8,
            Value::Box(_) => 32,          // Box overhead
            Value::Comparator(_) => 64,   // Comparator overhead
            Value::StringCursor(_) => 48, // StringCursor overhead
        }
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// Get number of allocated locations
    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Set memory limit
    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_limit = limit;
        self.gc_threshold = if limit > 0 { limit / 4 } else { 1024 * 1024 };
    }

    /// Get memory statistics
    pub fn get_statistics(&self) -> &StoreStatistics {
        &self.stats
    }
}
