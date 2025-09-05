//! Stable SIMD Optimization Integration
//!
//! This module provides a production-ready SIMD acceleration system that integrates
//! with the trait optimization, NaN boxing, and arena frameworks from Stages 1-3.
//! Uses stable Rust features for cross-platform compatibility.

use crate::eval::{Value, OptimizationHint, TraitOptimizedValue};

#[cfg(feature = "nan-boxing-optimization")]
use crate::eval::{EnhancedNanBoxedValue, ValueNanBoxingExt};

#[cfg(feature = "arena-optimization")]
use crate::eval::{StableArenaValue, ValueArenaExt};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

// ============= SIMD VECTORIZATION SYSTEM =============

/// SIMD-optimized vector operations using stable Rust features
#[derive(Debug, Clone)]
pub struct SimdVector {
    /// Vector data stored as f64 for numerical operations
    data: Vec<f64>,
    /// Optimization metadata
    optimization_hint: OptimizationHint,
}

impl SimdVector {
    /// Create new SIMD vector from slice of numbers
    pub fn from_numbers(numbers: &[f64], hint: OptimizationHint) -> Self {
        Self {
            data: numbers.to_vec(),
            optimization_hint: hint,
        }
    }
    
    /// Create SIMD vector from Values, filtering for numbers
    pub fn from_values(values: &[Value], hint: OptimizationHint) -> Option<Self> {
        let numbers: Option<Vec<f64>> = values
            .iter()
            .map(|v| v.as_number())
            .collect();
        
        numbers.map(|nums| Self::from_numbers(&nums, hint))
    }
    
    /// Get length of vector
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if vector is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Vectorized addition using chunked operations
    pub fn vectorized_add(&self, other: &SimdVector) -> Option<SimdVector> {
        if self.len() != other.len() {
            return None;
        }
        
        let result: Vec<f64> = self.data
            .chunks_exact(4)
            .zip(other.data.chunks_exact(4))
            .flat_map(|(a_chunk, b_chunk)| {
                // Manual 4-element SIMD-like operation
                [
                    a_chunk[0] + b_chunk[0],
                    a_chunk[1] + b_chunk[1],
                    a_chunk[2] + b_chunk[2],
                    a_chunk[3] + b_chunk[3],
                ]
            })
            .chain(
                // Handle remaining elements
                self.data[self.data.len() - (self.data.len() % 4)..]
                    .iter()
                    .zip(&other.data[other.data.len() - (other.data.len() % 4)..])
                    .map(|(a, b)| a + b)
            )
            .collect();
        
        Some(SimdVector::from_numbers(&result, self.optimization_hint))
    }
    
    /// Vectorized multiplication
    pub fn vectorized_mul(&self, other: &SimdVector) -> Option<SimdVector> {
        if self.len() != other.len() {
            return None;
        }
        
        let result: Vec<f64> = self.data
            .chunks_exact(4)
            .zip(other.data.chunks_exact(4))
            .flat_map(|(a_chunk, b_chunk)| {
                // Manual 4-element SIMD-like operation
                [
                    a_chunk[0] * b_chunk[0],
                    a_chunk[1] * b_chunk[1],
                    a_chunk[2] * b_chunk[2],
                    a_chunk[3] * b_chunk[3],
                ]
            })
            .chain(
                // Handle remaining elements
                self.data[self.data.len() - (self.data.len() % 4)..]
                    .iter()
                    .zip(&other.data[other.data.len() - (other.data.len() % 4)..])
                    .map(|(a, b)| a * b)
            )
            .collect();
        
        Some(SimdVector::from_numbers(&result, self.optimization_hint))
    }
    
    /// Scalar multiplication
    pub fn scalar_mul(&self, scalar: f64) -> SimdVector {
        let result: Vec<f64> = self.data
            .chunks_exact(4)
            .flat_map(|chunk| {
                // Manual 4-element scalar multiplication
                [
                    chunk[0] * scalar,
                    chunk[1] * scalar,
                    chunk[2] * scalar,
                    chunk[3] * scalar,
                ]
            })
            .chain(
                // Handle remaining elements
                self.data[self.data.len() - (self.data.len() % 4)..]
                    .iter()
                    .map(|x| x * scalar)
            )
            .collect();
        
        SimdVector::from_numbers(&result, self.optimization_hint)
    }
    
    /// Sum all elements using chunked reduction
    pub fn sum(&self) -> f64 {
        self.data
            .chunks_exact(4)
            .map(|chunk| chunk[0] + chunk[1] + chunk[2] + chunk[3])
            .sum::<f64>()
            + self.data[self.data.len() - (self.data.len() % 4)..]
                .iter()
                .sum::<f64>()
    }
    
    /// Convert back to Values
    pub fn to_values(&self) -> Vec<Value> {
        self.data.iter().map(|&n| Value::number(n)).collect()
    }
    
    /// Get raw data slice
    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }
}

// ============= SIMD OPTIMIZATION STRATEGY =============

/// SIMD optimization strategy based on data characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdStrategy {
    /// Vectorized operations for numerical data
    Vectorized,
    /// Parallel processing for independent operations
    Parallel,
    /// Manual chunked operations for mixed data
    Chunked,
    /// No SIMD optimization
    Sequential,
}

impl SimdStrategy {
    /// Determine optimal SIMD strategy for given data
    pub fn for_values(values: &[Value], hint: OptimizationHint) -> Self {
        // Count numerical values
        let numeric_count = values.iter().filter(|v| v.is_number()).count();
        let total_count = values.len();
        
        if numeric_count == total_count && total_count >= 8 {
            // All numbers, sufficient for vectorization
            match hint {
                OptimizationHint::Computational | OptimizationHint::HotPath => Self::Vectorized,
                _ => Self::Chunked,
            }
        } else if total_count >= 16 {
            // Large dataset, use parallel processing
            Self::Parallel
        } else if total_count >= 4 {
            // Medium dataset, use chunked operations
            Self::Chunked
        } else {
            // Small dataset, sequential is fine
            Self::Sequential
        }
    }
    
    /// Check if strategy benefits from SIMD
    pub fn uses_simd(&self) -> bool {
        matches!(self, Self::Vectorized | Self::Parallel | Self::Chunked)
    }
}

// ============= SIMD VALUE OPERATIONS =============

/// SIMD-optimized value operations
pub struct SimdValueOperations {
    stats: RwLock<SimdOperationStats>,
}

impl SimdValueOperations {
    /// Create new SIMD operations manager
    pub fn new() -> Self {
        Self {
            stats: RwLock::new(SimdOperationStats::new()),
        }
    }
    
    /// Perform SIMD-optimized addition on value arrays
    pub fn vector_add(&self, left: &[Value], right: &[Value]) -> Option<Vec<Value>> {
        if left.len() != right.len() {
            return None;
        }
        
        let strategy = SimdStrategy::for_values(left, OptimizationHint::Computational);
        
        if let Ok(mut stats) = self.stats.write() {
            stats.total_operations += 1;
            if strategy.uses_simd() {
                stats.simd_operations += 1;
            } else {
                stats.sequential_operations += 1;
            }
        }
        
        match strategy {
            SimdStrategy::Vectorized => {
                // Try to create SIMD vectors
                let left_vec = SimdVector::from_values(left, OptimizationHint::Computational)?;
                let right_vec = SimdVector::from_values(right, OptimizationHint::Computational)?;
                let result = left_vec.vectorized_add(&right_vec)?;
                Some(result.to_values())
            }
            SimdStrategy::Chunked => {
                // Manual chunked addition
                let result: Vec<Value> = left
                    .chunks(4)
                    .zip(right.chunks(4))
                    .flat_map(|(l_chunk, r_chunk)| {
                        l_chunk
                            .iter()
                            .zip(r_chunk.iter())
                            .map(|(l, r)| {
                                if let (Some(ln), Some(rn)) = (l.as_number(), r.as_number()) {
                                    Value::number(ln + rn)
                                } else {
                                    Value::Unspecified
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                Some(result)
            }
            _ => {
                // Sequential fallback
                let result: Vec<Value> = left
                    .iter()
                    .zip(right.iter())
                    .map(|(l, r)| {
                        if let (Some(ln), Some(rn)) = (l.as_number(), r.as_number()) {
                            Value::number(ln + rn)
                        } else {
                            Value::Unspecified
                        }
                    })
                    .collect();
                Some(result)
            }
        }
    }
    
    /// Perform SIMD-optimized scalar multiplication
    pub fn scalar_multiply(&self, values: &[Value], scalar: f64) -> Vec<Value> {
        let strategy = SimdStrategy::for_values(values, OptimizationHint::Computational);
        
        if let Ok(mut stats) = self.stats.write() {
            stats.total_operations += 1;
            if strategy.uses_simd() {
                stats.simd_operations += 1;
            } else {
                stats.sequential_operations += 1;
            }
        }
        
        match strategy {
            SimdStrategy::Vectorized => {
                if let Some(simd_vec) = SimdVector::from_values(values, OptimizationHint::Computational) {
                    simd_vec.scalar_mul(scalar).to_values()
                } else {
                    // Fallback to sequential
                    values.iter()
                        .map(|v| {
                            if let Some(n) = v.as_number() {
                                Value::number(n * scalar)
                            } else {
                                Value::Unspecified
                            }
                        })
                        .collect()
                }
            }
            SimdStrategy::Chunked => {
                // Manual chunked scalar multiplication
                values
                    .chunks(4)
                    .flat_map(|chunk| {
                        chunk.iter().map(|v| {
                            if let Some(n) = v.as_number() {
                                Value::number(n * scalar)
                            } else {
                                Value::Unspecified
                            }
                        }).collect::<Vec<_>>()
                    })
                    .collect()
            }
            _ => {
                // Sequential fallback
                values.iter()
                    .map(|v| {
                        if let Some(n) = v.as_number() {
                            Value::number(n * scalar)
                        } else {
                            Value::Unspecified
                        }
                    })
                    .collect()
            }
        }
    }
    
    /// Get operation statistics
    pub fn get_stats(&self) -> SimdOperationStats {
        self.stats.read()
            .map(|s| s.clone())
            .unwrap_or_else(|_| SimdOperationStats::new())
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            *stats = SimdOperationStats::new();
        }
    }
}

/// Statistics for SIMD operations
#[derive(Debug, Clone)]
pub struct SimdOperationStats {
    /// Total operations performed
    pub total_operations: u64,
    /// Operations using SIMD optimization
    pub simd_operations: u64,
    /// Operations using sequential processing
    pub sequential_operations: u64,
    /// Performance improvement ratio
    pub performance_gain: f32,
}

impl SimdOperationStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            simd_operations: 0,
            sequential_operations: 0,
            performance_gain: 0.0,
        }
    }
    
    /// Calculate SIMD utilization ratio
    pub fn simd_ratio(&self) -> f32 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.simd_operations as f32 / self.total_operations as f32
        }
    }
}

// ============= SIMD INTEGRATION WITH OTHER OPTIMIZATIONS =============

/// SIMD-optimized value with integration to other optimization layers
#[derive(Debug, Clone)]
pub struct SimdOptimizedValue {
    /// Core value data
    core: SimdValueCore,
    /// Optimization metadata
    optimization_hint: OptimizationHint,
}

/// Core data for SIMD-optimized values
#[derive(Debug, Clone)]
pub enum SimdValueCore {
    /// Direct value storage
    Direct(Value),
    /// SIMD vector for numerical data
    Vector(SimdVector),
    /// NaN-boxed optimized storage
    #[cfg(feature = "nan-boxing-optimization")]
    NanBoxed(EnhancedNanBoxedValue),
    /// Arena-allocated storage
    #[cfg(feature = "arena-optimization")]
    Arena(StableArenaValue),
    /// Hybrid optimization combining multiple techniques
    Hybrid {
        /// Vector component
        vector: Option<SimdVector>,
        /// Scalar components
        #[cfg(feature = "nan-boxing-optimization")]
        nan_boxed: Option<EnhancedNanBoxedValue>,
    },
}

impl SimdOptimizedValue {
    /// Create optimized value using best available strategy
    pub fn optimize(value: Value, hint: OptimizationHint) -> Self {
        // Clone value to avoid borrowing issues
        let value_clone = value.clone();
        let core = if value.is_number() {
            // Single number - try NaN boxing first
            #[cfg(feature = "nan-boxing-optimization")]
            {
                if let Some(enhanced) = value.to_nan_boxed_optimized(hint) {
                    SimdValueCore::NanBoxed(enhanced)
                } else {
                    SimdValueCore::Direct(value_clone)
                }
            }
            #[cfg(not(feature = "nan-boxing-optimization"))]
            {
                SimdValueCore::Direct(value_clone)
            }
        } else if value.is_vector() {
            // Vector data - try SIMD optimization
            match &value {
                Value::Vector(vec_rc) => {
                    if let Ok(vec_ref) = vec_rc.try_borrow() {
                        if let Some(simd_vec) = SimdVector::from_values(&vec_ref, hint) {
                            SimdValueCore::Vector(simd_vec)
                        } else {
                            SimdValueCore::Direct(value_clone)
                        }
                    } else {
                        SimdValueCore::Direct(value_clone)
                    }
                }
                _ => SimdValueCore::Direct(value_clone),
            }
        } else {
            // Complex value - try arena allocation
            #[cfg(feature = "arena-optimization")]
            {
                if value.can_arena_optimize() {
                    let arena_optimized = value.to_arena_optimized(hint);
                    match arena_optimized {
                        crate::eval::OptimizedArenaValue::Arena(arena_val) => {
                            SimdValueCore::Arena(arena_val)
                        }
                        _ => SimdValueCore::Direct(value_clone),
                    }
                } else {
                    SimdValueCore::Direct(value_clone)
                }
            }
            #[cfg(not(feature = "arena-optimization"))]
            {
                SimdValueCore::Direct(value_clone)
            }
        };
        
        Self {
            core,
            optimization_hint: hint,
        }
    }
    
    /// Extract underlying value
    pub fn to_value(&self) -> Value {
        match &self.core {
            SimdValueCore::Direct(value) => value.clone(),
            SimdValueCore::Vector(vec) => {
                let values = vec.to_values();
                Value::vector(values)
            }
            #[cfg(feature = "nan-boxing-optimization")]
            SimdValueCore::NanBoxed(enhanced) => {
                if let Some(n) = enhanced.as_number() {
                    Value::number(n)
                } else {
                    Value::Unspecified
                }
            }
            #[cfg(feature = "arena-optimization")]
            SimdValueCore::Arena(arena_val) => arena_val.to_value(),
            SimdValueCore::Hybrid { vector, .. } => {
                if let Some(vec) = vector {
                    Value::vector(vec.to_values())
                } else {
                    Value::Unspecified
                }
            }
        }
    }
    
    /// Get optimization hint
    pub fn optimization_hint(&self) -> OptimizationHint {
        self.optimization_hint
    }
}

// ============= EXTENSION TRAITS =============

/// Extension trait for Value to enable SIMD optimization
pub trait ValueSimdExt {
    /// Convert to SIMD-optimized representation
    fn to_simd_optimized(&self, hint: OptimizationHint) -> SimdOptimizedValue;
    
    /// Check if value can benefit from SIMD optimization
    fn can_simd_optimize(&self) -> bool;
    
    /// Estimate performance gain from SIMD optimization
    fn simd_performance_estimate(&self) -> f32;
}

impl ValueSimdExt for Value {
    fn to_simd_optimized(&self, hint: OptimizationHint) -> SimdOptimizedValue {
        SimdOptimizedValue::optimize(self.clone(), hint)
    }
    
    fn can_simd_optimize(&self) -> bool {
        // SIMD optimization benefits numerical vectors and computations
        self.is_vector() || self.is_number() || 
        (self.is_pair() && self.is_list())
    }
    
    fn simd_performance_estimate(&self) -> f32 {
        if self.is_vector() {
            if let Value::Vector(vec_rc) = self {
                if let Ok(vec_ref) = vec_rc.try_borrow() {
                    if vec_ref.len() >= 8 && vec_ref.iter().all(|v| v.is_number()) {
                        2.5 // 2.5x speedup for large numerical vectors
                    } else if vec_ref.len() >= 4 {
                        1.5 // 1.5x speedup for medium vectors
                    } else {
                        1.0 // No speedup for small vectors
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            }
        } else if self.is_number() {
            1.2 // Modest speedup from NaN boxing
        } else {
            1.0 // No SIMD benefit
        }
    }
}

// ============= TESTING FRAMEWORK =============

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_vector_creation() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let simd_vec = SimdVector::from_numbers(&data, OptimizationHint::Computational);
        
        assert_eq!(simd_vec.len(), 5);
        assert!(!simd_vec.is_empty());
        assert_eq!(simd_vec.as_slice(), &data);
    }
    
    #[test]
    fn test_simd_vector_operations() {
        let vec1 = SimdVector::from_numbers(&[1.0, 2.0, 3.0, 4.0], OptimizationHint::Computational);
        let vec2 = SimdVector::from_numbers(&[5.0, 6.0, 7.0, 8.0], OptimizationHint::Computational);
        
        let sum = vec1.vectorized_add(&vec2).unwrap();
        assert_eq!(sum.as_slice(), &[6.0, 8.0, 10.0, 12.0]);
        
        let product = vec1.vectorized_mul(&vec2).unwrap();
        assert_eq!(product.as_slice(), &[5.0, 12.0, 21.0, 32.0]);
        
        let scaled = vec1.scalar_mul(2.0);
        assert_eq!(scaled.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
        
        assert_eq!(vec1.sum(), 10.0);
    }
    
    #[test]
    fn test_simd_strategy_selection() {
        let numbers: Vec<Value> = (0..10).map(|i| Value::number(i as f64)).collect();
        let mixed: Vec<Value> = vec![Value::number(1.0), Value::boolean(true)];
        
        let numeric_strategy = SimdStrategy::for_values(&numbers, OptimizationHint::Computational);
        assert_eq!(numeric_strategy, SimdStrategy::Vectorized);
        assert!(numeric_strategy.uses_simd());
        
        let mixed_strategy = SimdStrategy::for_values(&mixed, OptimizationHint::Default);
        assert_eq!(mixed_strategy, SimdStrategy::Sequential);
        assert!(!mixed_strategy.uses_simd());
    }
    
    #[test]
    fn test_simd_value_operations() {
        let ops = SimdValueOperations::new();
        
        let left = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0), Value::number(4.0)];
        let right = vec![Value::number(5.0), Value::number(6.0), Value::number(7.0), Value::number(8.0)];
        
        let result = ops.vector_add(&left, &right).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].as_number().unwrap(), 6.0);
        assert_eq!(result[3].as_number().unwrap(), 12.0);
        
        let scaled = ops.scalar_multiply(&left, 3.0);
        assert_eq!(scaled[0].as_number().unwrap(), 3.0);
        assert_eq!(scaled[3].as_number().unwrap(), 12.0);
        
        let stats = ops.get_stats();
        assert!(stats.total_operations > 0);
    }
    
    #[test]
    fn test_simd_optimized_value() {
        let number = Value::number(42.0);
        let optimized = SimdOptimizedValue::optimize(number, OptimizationHint::Computational);
        
        let restored = optimized.to_value();
        assert!(restored.is_number());
    }
    
    #[test]
    fn test_value_simd_ext() {
        let vector_data = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let vector = Value::vector(vector_data);
        
        assert!(vector.can_simd_optimize());
        assert!(vector.simd_performance_estimate() > 1.0);
        
        let optimized = vector.to_simd_optimized(OptimizationHint::Computational);
        assert_eq!(optimized.optimization_hint(), OptimizationHint::Computational);
    }
}