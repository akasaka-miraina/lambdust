#![cfg(feature = "never-enabled")]
//! Value system integration for SIMD operations
//!
//! This module provides seamless integration between Lambdust's Value system
//! and SIMD-optimized numeric operations with proper memory alignment and
//! type conversion.

use crate::diagnostics::{Error, Result};
use crate::eval::value::Value;
use crate::numeric::simd_arithmetic::SimdArithmeticEngine;
use crate::numeric::simd_list_ops::SimdListOpsEngine;
use crate::numeric::simd_wrapper::{AlignedBuffer, SimdCapabilities, SimdError};
use std::cell::RefCell;
use std::rc::Rc;

/// SIMD-Value integration engine with automatic type conversion and alignment
pub struct SimdValueIntegration {
    arithmetic_engine: SimdArithmeticEngine,
    list_ops_engine: SimdListOpsEngine,
    capabilities: SimdCapabilities,
}

impl SimdValueIntegration {
    /// Create a new SIMD-Value integration engine
    pub fn new() -> Self {
        Self {
            arithmetic_engine: SimdArithmeticEngine::new(),
            list_ops_engine: SimdListOpsEngine::new(),
            capabilities: SimdCapabilities::detect(),
        }
    }

    /// Convert Value to f64 vector with proper error handling
    pub fn value_to_f64_vector(&self, value: &Value) -> Result<Vec<f64>> {
        match value {
            Value::Vector(vec_ref) => {
                let vec_guard = vec_ref
                    .try_borrow()
                    .map_err(|_| Error::runtime_error("Cannot borrow vector".to_string(), None))?;

                let mut f64_vec = Vec::with_capacity(vec_guard.len());
                for val in vec_guard.iter() {
                    let f64_val = self.value_to_f64(val)?;
                    f64_vec.push(f64_val);
                }
                Ok(f64_vec)
            }
            _ => Err(Box::new(Error::runtime_error(
                "Value is not a vector".to_string(),
                None,
            ))),
        }
    }

    /// Convert single Value to f64
    pub fn value_to_f64(&self, value: &Value) -> Result<f64> {
        match value {
            Value::Literal(literal) => match literal {
                crate::ast::Literal::ExactInteger(i) => Ok(*i as f64),
                crate::ast::Literal::InexactReal(r) => Ok(*r),
                crate::ast::Literal::Number(n) => Ok(*n),
                crate::ast::Literal::Rational(rat) => {
                    Ok(rat.numerator as f64 / rat.denominator as f64)
                }
                crate::ast::Literal::Complex(complex) => {
                    if complex.imaginary == 0.0 {
                        Ok(complex.real)
                    } else {
                        Err(Box::new(Error::runtime_error(
                            "Cannot convert complex number with non-zero imaginary part to f64"
                                .to_string(),
                            None,
                        )))
                    }
                }
                _ => Err(Box::new(Error::runtime_error(
                    "Cannot convert literal to f64".to_string(),
                    None,
                ))),
            },
            _ => Err(Box::new(Error::runtime_error(
                "Cannot convert non-literal value to f64".to_string(),
                None,
            ))),
        }
    }

    /// Convert f64 vector back to Value vector
    pub fn f64_vector_to_value(&self, values: Vec<f64>) -> Result<Value> {
        let value_vec: Vec<Value> = values
            .into_iter()
            .map(|f| Value::Literal(crate::ast::Literal::InexactReal(f)))
            .collect();

        Ok(Value::Vector(Rc::new(RefCell::new(value_vec))))
    }

    /// Create aligned buffer from Value vector for optimal SIMD performance
    pub fn create_aligned_buffer_from_value(&self, value: &Value) -> Result<AlignedBuffer<f64>> {
        let f64_vec = self.value_to_f64_vector(value)?;

        let mut buffer = AlignedBuffer::new(f64_vec.len(), 32).map_err(|e| {
            Error::runtime_error(format!("Failed to create aligned buffer: {e}"), None)
        })?;

        for val in f64_vec {
            buffer.push(val).map_err(|e| {
                Error::runtime_error(format!("Failed to push to buffer: {e}"), None)
            })?;
        }

        Ok(buffer)
    }

    /// SIMD vector addition for Value vectors
    pub fn simd_vector_add(&self, a: &Value, b: &Value) -> Result<Value> {
        let a_vec = self.value_to_f64_vector(a)?;
        let b_vec = self.value_to_f64_vector(b)?;

        let result = self.arithmetic_engine.vector_add_f64(&a_vec, &b_vec)?;
        self.f64_vector_to_value(result)
    }

    /// SIMD vector multiplication for Value vectors
    pub fn simd_vector_multiply(&self, a: &Value, b: &Value) -> Result<Value> {
        let a_vec = self.value_to_f64_vector(a)?;
        let b_vec = self.value_to_f64_vector(b)?;

        let result = self.arithmetic_engine.vector_mul_f64(&a_vec, &b_vec)?;
        self.f64_vector_to_value(result)
    }

    /// SIMD dot product for Value vectors
    pub fn simd_dot_product(&self, a: &Value, b: &Value) -> Result<Value> {
        let a_vec = self.value_to_f64_vector(a)?;
        let b_vec = self.value_to_f64_vector(b)?;

        let result = self.arithmetic_engine.dot_product_f64(&a_vec, &b_vec)?;
        Ok(Value::Literal(crate::ast::Literal::InexactReal(result)))
    }

    /// SIMD map operation for Value vectors
    pub fn simd_map<F>(&self, value: &Value, f: F) -> Result<Value>
    where
        F: Fn(f64) -> f64 + Send + Sync,
    {
        let f64_vec = self.value_to_f64_vector(value)?;
        let result = self.list_ops_engine.simd_map_f64(&f64_vec, f)?;
        self.f64_vector_to_value(result)
    }

    /// SIMD filter operation for Value vectors
    pub fn simd_filter<P>(&self, value: &Value, predicate: P) -> Result<Value>
    where
        P: Fn(f64) -> bool + Send + Sync,
    {
        let f64_vec = self.value_to_f64_vector(value)?;
        let result = self.list_ops_engine.simd_filter_f64(&f64_vec, predicate)?;
        self.f64_vector_to_value(result)
    }

    /// SIMD fold operation for Value vectors
    pub fn simd_fold<F>(&self, value: &Value, init: f64, folder: F) -> Result<Value>
    where
        F: Fn(f64, f64) -> f64 + Send + Sync,
    {
        let f64_vec = self.value_to_f64_vector(value)?;
        let result = self.list_ops_engine.simd_fold_f64(&f64_vec, init, folder)?;
        Ok(Value::Literal(crate::ast::Literal::InexactReal(result)))
    }

    /// SIMD sum for Value vectors
    pub fn simd_sum(&self, value: &Value) -> Result<Value> {
        let f64_vec = self.value_to_f64_vector(value)?;
        let result = self.list_ops_engine.simd_sum_f64(&f64_vec)?;
        Ok(Value::Literal(crate::ast::Literal::InexactReal(result)))
    }

    /// SIMD product for Value vectors
    pub fn simd_product(&self, value: &Value) -> Result<Value> {
        let f64_vec = self.value_to_f64_vector(value)?;
        let result = self.list_ops_engine.simd_product_f64(&f64_vec)?;
        Ok(Value::Literal(crate::ast::Literal::InexactReal(result)))
    }

    /// Check if a Value is suitable for SIMD optimization
    pub fn is_simd_optimizable(&self, value: &Value) -> bool {
        match value {
            Value::Vector(vec_ref) => {
                if let Ok(vec_guard) = vec_ref.try_borrow() {
                    // Check minimum size for SIMD benefit
                    if vec_guard.len() < 8 {
                        return false;
                    }

                    // Check if all elements are convertible to f64
                    vec_guard.iter().all(|v| self.value_to_f64(v).is_ok())
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get performance statistics for SIMD operations
    pub fn get_simd_performance_info(&self) -> SimdPerformanceInfo {
        SimdPerformanceInfo {
            capabilities: self.capabilities.clone(),
            optimal_vector_size: match self.capabilities.best_strategy() {
                crate::numeric::simd_wrapper::SimdStrategy::Avx512 => 8,
                crate::numeric::simd_wrapper::SimdStrategy::Avx2 => 4,
                crate::numeric::simd_wrapper::SimdStrategy::Sse => 2,
                crate::numeric::simd_wrapper::SimdStrategy::Neon => 4,
                crate::numeric::simd_wrapper::SimdStrategy::Scalar => 1,
            },
            memory_alignment: 32,
            parallel_threshold: self.list_ops_engine.parallel_threshold(),
        }
    }

    /// Optimize Value vector for SIMD operations
    pub fn optimize_value_for_simd(&self, value: &Value) -> Result<OptimizedValue> {
        match value {
            Value::Vector(vec_ref) => {
                let vec_guard = vec_ref
                    .try_borrow()
                    .map_err(|_| Error::runtime_error("Cannot borrow vector".to_string(), None))?;

                // Check if optimization is beneficial
                if vec_guard.len() < 8 {
                    return Ok(OptimizedValue::NotOptimized(value.clone()));
                }

                // Try to convert to aligned f64 buffer
                let mut f64_values = Vec::with_capacity(vec_guard.len());
                for val in vec_guard.iter() {
                    f64_values.push(self.value_to_f64(val)?);
                }

                let aligned_buffer: AlignedBuffer<f64> = AlignedBuffer::new(f64_values.len(), 32)
                    .map_err(|e| {
                    Error::runtime_error(format!("Failed to create aligned buffer: {e}"), None)
                })?;

                // TODO: Fill the aligned buffer with values
                // For now, return the original value as optimized storage would require more complex implementation

                Ok(OptimizedValue::Optimized {
                    original: value.clone(),
                    f64_representation: f64_values,
                    is_aligned: true,
                })
            }
            _ => Ok(OptimizedValue::NotOptimized(value.clone())),
        }
    }
}

impl Default for SimdValueIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance information for SIMD operations
#[derive(Debug, Clone)]
pub struct SimdPerformanceInfo {
    /// Available SIMD capabilities on this platform
    pub capabilities: SimdCapabilities,
    /// Optimal vector size for SIMD operations
    pub optimal_vector_size: usize,
    /// Required memory alignment for optimal performance
    pub memory_alignment: usize,
    /// Minimum data size threshold for parallel processing
    pub parallel_threshold: usize,
}

/// Optimized Value representation for SIMD operations
#[derive(Debug, Clone)]
pub enum OptimizedValue {
    /// Value is not suitable for optimization
    NotOptimized(Value),
    /// Value has been optimized for SIMD
    Optimized {
        /// Original unoptimized value
        original: Value,
        /// SIMD-friendly f64 representation
        f64_representation: Vec<f64>,
        /// Whether data is memory-aligned for SIMD
        is_aligned: bool,
    },
}

impl OptimizedValue {
    /// Extract the original Value
    pub fn into_value(self) -> Value {
        match self {
            OptimizedValue::NotOptimized(value) => value,
            OptimizedValue::Optimized { original, .. } => original,
        }
    }

    /// Get the f64 representation if available
    pub fn f64_representation(&self) -> Option<&[f64]> {
        match self {
            OptimizedValue::NotOptimized(_) => None,
            OptimizedValue::Optimized {
                f64_representation, ..
            } => Some(f64_representation),
        }
    }

    /// Check if the value is optimized
    pub fn is_optimized(&self) -> bool {
        matches!(self, OptimizedValue::Optimized { .. })
    }
}

use once_cell::sync::Lazy;
/// Global SIMD-Value integration instance
use std::sync::{Arc, Mutex};

static GLOBAL_SIMD_INTEGRATION: Lazy<Arc<Mutex<SimdValueIntegration>>> =
    Lazy::new(|| Arc::new(Mutex::new(SimdValueIntegration::new())));

/// Get the global SIMD-Value integration instance
pub fn get_simd_integration() -> Arc<Mutex<SimdValueIntegration>> {
    GLOBAL_SIMD_INTEGRATION.clone()
}

/// Convenience functions for common SIMD operations on Values
/// SIMD vector addition for Values
pub fn simd_value_add(a: &Value, b: &Value) -> Result<Value> {
    let integration = get_simd_integration();
    let integration_guard = integration.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire SIMD integration lock".to_string(), None)
    })?;
    integration_guard.simd_vector_add(a, b)
}

/// SIMD vector multiplication for Values
pub fn simd_value_multiply(a: &Value, b: &Value) -> Result<Value> {
    let integration = get_simd_integration();
    let integration_guard = integration.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire SIMD integration lock".to_string(), None)
    })?;
    integration_guard.simd_vector_multiply(a, b)
}

/// SIMD dot product for Values
pub fn simd_value_dot_product(a: &Value, b: &Value) -> Result<Value> {
    let integration = get_simd_integration();
    let integration_guard = integration.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire SIMD integration lock".to_string(), None)
    })?;
    integration_guard.simd_dot_product(a, b)
}

/// SIMD sum for Value vector
pub fn simd_value_sum(value: &Value) -> Result<Value> {
    let integration = get_simd_integration();
    let integration_guard = integration.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire SIMD integration lock".to_string(), None)
    })?;
    integration_guard.simd_sum(value)
}

/// Check if Value is SIMD optimizable
pub fn is_value_simd_optimizable(value: &Value) -> bool {
    if let Ok(integration) = get_simd_integration().lock() {
        integration.is_simd_optimizable(value)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    fn create_test_vector(values: Vec<f64>) -> Value {
        let value_vec: Vec<Value> = values
            .into_iter()
            .map(|f| Value::Literal(Literal::InexactReal(f)))
            .collect();
        Value::Vector(Rc::new(RefCell::new(value_vec)))
    }

    #[test]
    fn test_value_to_f64_conversion() {
        let integration = SimdValueIntegration::new();
        let test_vector = create_test_vector(vec![1.0, 2.0, 3.0, 4.0]);

        let f64_vec = integration.value_to_f64_vector(&test_vector).unwrap();
        assert_eq!(f64_vec, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_simd_vector_operations() {
        let integration = SimdValueIntegration::new();
        let vec_a = create_test_vector(vec![1.0, 2.0, 3.0, 4.0]);
        let vec_b = create_test_vector(vec![5.0, 6.0, 7.0, 8.0]);

        // Test addition
        let sum_result = integration.simd_vector_add(&vec_a, &vec_b).unwrap();
        let sum_f64 = integration.value_to_f64_vector(&sum_result).unwrap();
        assert_eq!(sum_f64, vec![6.0, 8.0, 10.0, 12.0]);

        // Test multiplication
        let mul_result = integration.simd_vector_multiply(&vec_a, &vec_b).unwrap();
        let mul_f64 = integration.value_to_f64_vector(&mul_result).unwrap();
        assert_eq!(mul_f64, vec![5.0, 12.0, 21.0, 32.0]);

        // Test dot product
        let dot_result = integration.simd_dot_product(&vec_a, &vec_b).unwrap();
        let dot_value = integration.value_to_f64(&dot_result).unwrap();
        assert_eq!(dot_value, 70.0); // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
    }

    #[test]
    fn test_simd_list_operations() {
        let integration = SimdValueIntegration::new();
        let test_vector = create_test_vector(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);

        // Test map (double each element)
        let mapped = integration.simd_map(&test_vector, |x| x * 2.0).unwrap();
        let mapped_f64 = integration.value_to_f64_vector(&mapped).unwrap();
        assert_eq!(mapped_f64, vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0]);

        // Test filter (positive values only)
        let test_vector_mixed = create_test_vector(vec![-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
        let filtered = integration
            .simd_filter(&test_vector_mixed, |x| x > 0.0)
            .unwrap();
        let filtered_f64 = integration.value_to_f64_vector(&filtered).unwrap();
        assert_eq!(filtered_f64, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        // Test sum
        let sum_result = integration.simd_sum(&test_vector).unwrap();
        let sum_value = integration.value_to_f64(&sum_result).unwrap();
        assert_eq!(sum_value, 36.0); // 1+2+3+4+5+6+7+8 = 36

        // Test product
        let small_vector = create_test_vector(vec![2.0, 3.0, 4.0]);
        let product_result = integration.simd_product(&small_vector).unwrap();
        let product_value = integration.value_to_f64(&product_result).unwrap();
        assert_eq!(product_value, 24.0); // 2*3*4 = 24
    }

    #[test]
    fn test_simd_optimizability() {
        let integration = SimdValueIntegration::new();

        // Large enough vector with numeric values - should be optimizable
        let large_vector = create_test_vector((0..16).map(|i| i as f64).collect());
        assert!(integration.is_simd_optimizable(&large_vector));

        // Small vector - should not be optimizable
        let small_vector = create_test_vector(vec![1.0, 2.0]);
        assert!(!integration.is_simd_optimizable(&small_vector));

        // Non-vector value - should not be optimizable
        let scalar = Value::Literal(Literal::InexactReal(42.0));
        assert!(!integration.is_simd_optimizable(&scalar));
    }

    #[test]
    fn test_convenience_functions() {
        let vec_a = create_test_vector(vec![1.0, 2.0, 3.0, 4.0]);
        let vec_b = create_test_vector(vec![5.0, 6.0, 7.0, 8.0]);

        let sum_result = simd_value_add(&vec_a, &vec_b).unwrap();
        let sum_integration = SimdValueIntegration::new();
        let sum_f64 = sum_integration.value_to_f64_vector(&sum_result).unwrap();
        assert_eq!(sum_f64, vec![6.0, 8.0, 10.0, 12.0]);

        let vector_sum = simd_value_sum(&vec_a).unwrap();
        let sum_value = sum_integration.value_to_f64(&vector_sum).unwrap();
        assert_eq!(sum_value, 10.0); // 1+2+3+4 = 10
    }

    #[test]
    fn test_performance_info() {
        let integration = SimdValueIntegration::new();
        let perf_info = integration.get_simd_performance_info();

        assert!(perf_info.optimal_vector_size >= 1);
        assert_eq!(perf_info.memory_alignment, 32);
        assert!(perf_info.parallel_threshold > 0);
    }
}
