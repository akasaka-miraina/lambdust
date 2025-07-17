//! Lazy vector implementation for memory-efficient large vector handling
//!
//! Provides on-demand memory allocation for large vectors to prevent
//! out-of-memory crashes and improve CI stability.

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// Memory threshold for immediate allocation (32KB for testing, 10MB for production)
const IMMEDIATE_ALLOCATION_THRESHOLD: usize = 32 * 1024;

/// Size of each materialized segment (1024 elements)
const SEGMENT_SIZE: usize = 1024;

/// Lazy vector storage strategy
#[derive(Debug, Clone, PartialEq)]
pub enum VectorStorage {
    /// Fully materialized vector (for small vectors)
    Materialized(Vec<Value>),
    /// Lazy vector with on-demand materialization
    Lazy {
        /// Total size of the vector
        size: usize,
        /// Fill value for uninitialized elements
        fill_value: Value,
        /// Materialized segments (`segment_index` -> values)
        materialized_segments: HashMap<usize, Vec<Value>>,
        /// Size of each segment
        segment_size: usize,
    },
}

impl VectorStorage {
    /// Create a new vector with the specified size and fill value
    pub fn new(size: usize, fill_value: Value) -> Result<Self> {
        // Estimate memory usage (assuming 8 bytes per Value on average)
        let estimated_bytes = size * std::mem::size_of::<Value>();

        if estimated_bytes <= IMMEDIATE_ALLOCATION_THRESHOLD {
            // Small vector - allocate immediately
            Ok(VectorStorage::Materialized(vec![fill_value; size]))
        } else {
            // Large vector - use lazy allocation
            Ok(VectorStorage::Lazy {
                size,
                fill_value,
                materialized_segments: HashMap::new(),
                segment_size: SEGMENT_SIZE,
            })
        }
    }

    /// Get the length of the vector
    #[must_use] pub fn len(&self) -> usize {
        match self {
            VectorStorage::Materialized(vec) => vec.len(),
            VectorStorage::Lazy { size, .. } => *size,
        }
    }

    /// Check if the vector is empty
    #[must_use] pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an element at the specified index
    pub fn get(&mut self, index: usize) -> Result<Value> {
        if index >= self.len() {
            return Err(LambdustError::runtime_error(format!(
                "vector-ref: index {} out of bounds for vector of length {}",
                index,
                self.len()
            )));
        }

        match self {
            VectorStorage::Materialized(vec) => Ok(vec[index].clone()),
            VectorStorage::Lazy {
                fill_value,
                materialized_segments,
                segment_size,
                ..
            } => {
                let segment_index = index / *segment_size;
                let element_index = index % *segment_size;

                if let Some(segment) = materialized_segments.get(&segment_index) {
                    Ok(segment[element_index].clone())
                } else {
                    // Return fill value without materializing
                    Ok(fill_value.clone())
                }
            }
        }
    }

    /// Set an element at the specified index (materializes segment if needed)
    pub fn set(&mut self, index: usize, value: Value) -> Result<()> {
        if index >= self.len() {
            return Err(LambdustError::runtime_error(format!(
                "vector-set!: index {} out of bounds for vector of length {}",
                index,
                self.len()
            )));
        }

        match self {
            VectorStorage::Materialized(vec) => {
                vec[index] = value;
                Ok(())
            }
            VectorStorage::Lazy {
                fill_value,
                materialized_segments,
                segment_size,
                ..
            } => {
                let segment_index = index / *segment_size;
                let element_index = index % *segment_size;

                // Materialize segment if it doesn't exist
                materialized_segments
                    .entry(segment_index)
                    .or_insert_with(|| vec![fill_value.clone(); *segment_size]);

                // Set the value in the materialized segment
                if let Some(segment) = materialized_segments.get_mut(&segment_index) {
                    segment[element_index] = value;
                    Ok(())
                } else {
                    // This should never happen due to the check above
                    Err(LambdustError::runtime_error(
                        "Internal error: failed to materialize vector segment",
                    ))
                }
            }
        }
    }

    /// Convert to a fully materialized vector (may fail with large vectors)
    pub fn to_materialized(&self) -> Result<Vec<Value>> {
        match self {
            VectorStorage::Materialized(vec) => Ok(vec.clone()),
            VectorStorage::Lazy {
                size,
                fill_value,
                materialized_segments,
                segment_size,
            } => {
                // Check if we can safely materialize
                let estimated_bytes = size * std::mem::size_of::<Value>();
                if estimated_bytes > IMMEDIATE_ALLOCATION_THRESHOLD * 4 {
                    return Err(LambdustError::runtime_error(format!(
                        "Cannot materialize vector of size {size} (estimated {estimated_bytes} bytes): too large"
                    )));
                }

                let mut result = Vec::with_capacity(*size);

                for i in 0..*size {
                    let segment_index = i / segment_size;
                    let element_index = i % segment_size;

                    if let Some(segment) = materialized_segments.get(&segment_index) {
                        result.push(segment[element_index].clone());
                    } else {
                        result.push(fill_value.clone());
                    }
                }

                Ok(result)
            }
        }
    }

    /// Get memory usage statistics
    #[must_use] pub fn memory_stats(&self) -> MemoryStats {
        match self {
            VectorStorage::Materialized(vec) => MemoryStats {
                logical_size: vec.len(),
                materialized_elements: vec.len(),
                materialized_segments: 1,
                estimated_bytes: vec.len() * std::mem::size_of::<Value>(),
            },
            VectorStorage::Lazy {
                size,
                materialized_segments,
                segment_size,
                ..
            } => {
                let materialized_elements = materialized_segments.len() * segment_size;
                MemoryStats {
                    logical_size: *size,
                    materialized_elements,
                    materialized_segments: materialized_segments.len(),
                    estimated_bytes: materialized_elements * std::mem::size_of::<Value>(),
                }
            }
        }
    }

    /// Force materialization of a specific range (for testing/debugging)
    pub fn materialize_range(&mut self, start: usize, end: usize) -> Result<()> {
        if start >= self.len() || end > self.len() || start > end {
            return Err(LambdustError::runtime_error(
                "Invalid range for materialization",
            ));
        }

        match self {
            VectorStorage::Materialized(_) => Ok(()), // Already materialized
            VectorStorage::Lazy {
                fill_value,
                materialized_segments,
                segment_size,
                ..
            } => {
                let start_segment = start / *segment_size;
                let end_segment = (end - 1) / *segment_size;

                for segment_index in start_segment..=end_segment {
                    materialized_segments
                        .entry(segment_index)
                        .or_insert_with(|| vec![fill_value.clone(); *segment_size]);
                }

                Ok(())
            }
        }
    }
}

/// Memory usage statistics for lazy vectors
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Logical size of the vector
    pub logical_size: usize,
    /// Number of actually materialized elements
    pub materialized_elements: usize,
    /// Number of materialized segments
    pub materialized_segments: usize,
    /// Estimated memory usage in bytes
    pub estimated_bytes: usize,
}

impl MemoryStats {
    /// Calculate materialization ratio (0.0 to 1.0)
    #[must_use] pub fn materialization_ratio(&self) -> f64 {
        if self.logical_size == 0 {
            1.0
        } else {
            self.materialized_elements as f64 / self.logical_size as f64
        }
    }

    /// Check if the vector is efficiently utilizing memory
    #[must_use] pub fn is_efficient(&self) -> bool {
        self.materialization_ratio() < 0.1 || self.logical_size <= SEGMENT_SIZE
    }
}

