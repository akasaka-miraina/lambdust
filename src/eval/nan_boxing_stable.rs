//! Stable NaN Boxing Optimization Integration
//!
//! This module provides a production-ready NaN boxing system that integrates
//! with the trait optimization framework from Stage 1.

use crate::eval::{Value, NanBoxedValue, OptimizationHint, TraitOptimizedValue};
use crate::ast::literal::Literal;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// ============= ENHANCED NAN BOXING SYSTEM =============

/// Enhanced NaN-boxed value with optimization tracking and stability guarantees
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnhancedNanBoxedValue {
    inner: u64,
    hint: OptimizationHint,
}

impl EnhancedNanBoxedValue {
    /// Create from existing NaN-boxed value with optimization hint
    pub fn from_nan_boxed(value: NanBoxedValue, hint: OptimizationHint) -> Self {
        Self {
            inner: value.into_raw(),
            hint,
        }
    }
    
    /// Create optimized value directly from f64
    pub fn from_f64_optimized(value: f64) -> Self {
        if value.is_finite() {
            // Use direct representation for finite numbers
            Self {
                inner: value.to_bits(),
                hint: OptimizationHint::Computational,
            }
        } else {
            // Use NaN boxing for special values
            Self {
                inner: NanBoxedValue::from_number(value).into_raw(),
                hint: OptimizationHint::Default,
            }
        }
    }
    
    /// Create from integer with range optimization
    pub fn from_i64_optimized(value: i64) -> Self {
        if let Some(small_int) = NanBoxedValue::from_small_int(value) {
            // Use small integer encoding
            Self {
                inner: small_int.into_raw(),
                hint: OptimizationHint::Computational,
            }
        } else {
            // Fall back to f64 representation
            Self {
                inner: (value as f64).to_bits(),
                hint: OptimizationHint::LongTerm,
            }
        }
    }
    
    /// Extract the inner NaN-boxed value
    pub fn to_nan_boxed(&self) -> NanBoxedValue {
        NanBoxedValue::from_raw(self.inner)
    }
    
    /// Get optimization hint
    pub fn optimization_hint(&self) -> OptimizationHint {
        self.hint
    }
    
    /// Check if value can be optimized further
    pub fn can_optimize(&self) -> bool {
        matches!(self.hint, OptimizationHint::Default)
    }
    
    /// Check if this is a number value
    pub fn is_number(&self) -> bool {
        // Check if it represents a finite f64 or uses number encoding
        let nan_boxed = self.to_nan_boxed();
        nan_boxed.is_number() || {
            let f = f64::from_bits(self.inner);
            f.is_finite()
        }
    }
    
    /// Get number value if it is one
    pub fn as_number(&self) -> Option<f64> {
        // First try NaN boxed extraction
        let nan_boxed = self.to_nan_boxed();
        if let Some(n) = nan_boxed.as_number() {
            Some(n)
        } else {
            // Try direct f64 interpretation
            let f = f64::from_bits(self.inner);
            if f.is_finite() {
                Some(f)
            } else {
                None
            }
        }
    }
    
    /// Try to optimize representation in-place
    pub fn optimize(&mut self) -> bool {
        if !self.can_optimize() {
            return false;
        }
        
        // If it's a number, try to use computational hint
        if self.is_number() {
            self.hint = OptimizationHint::Computational;
            true
        } else {
            false
        }
    }
}

impl TraitOptimizedValue for EnhancedNanBoxedValue {
    type Inner = NanBoxedValue;
    
    fn from_inner(inner: Self::Inner, hint: OptimizationHint) -> Self {
        Self::from_nan_boxed(inner, hint)
    }
    
    fn into_inner(self) -> Self::Inner {
        self.to_nan_boxed()
    }
    
    fn should_optimize(&self) -> bool {
        matches!(self.hint, OptimizationHint::HotPath | OptimizationHint::Computational)
    }
}

// ============= NAN BOXING STATISTICS =============

/// Memory usage statistics for NaN boxing
#[derive(Debug, Clone)]
pub struct NanBoxingStats {
    /// Total size in bytes
    pub total_size: usize,
    /// Compression ratio vs original Value enum
    pub compression_ratio: f32,
    /// Type distribution
    pub type_distribution: TypeDistribution,
}

/// Distribution of types in NaN-boxed values
#[derive(Debug, Clone)]
pub struct TypeDistribution {
    /// Number of numeric values
    pub numbers: usize,
    /// Number of boolean values
    pub booleans: usize,
    /// Number of small integers
    pub small_integers: usize,
    /// Number of characters
    pub characters: usize,
    /// Number of nil values
    pub nil_values: usize,
    /// Number of other boxed values
    pub boxed_values: usize,
}

impl TypeDistribution {
    /// Create empty distribution
    pub fn new() -> Self {
        Self {
            numbers: 0,
            booleans: 0,
            small_integers: 0,
            characters: 0,
            nil_values: 0,
            boxed_values: 0,
        }
    }
    
    /// Total count of all values
    pub fn total(&self) -> usize {
        self.numbers + self.booleans + self.small_integers + 
        self.characters + self.nil_values + self.boxed_values
    }
    
    /// Calculate percentage of each type
    pub fn percentages(&self) -> TypePercentages {
        let total = self.total() as f32;
        if total == 0.0 {
            return TypePercentages::default();
        }
        
        TypePercentages {
            numbers: (self.numbers as f32 / total) * 100.0,
            booleans: (self.booleans as f32 / total) * 100.0,
            small_integers: (self.small_integers as f32 / total) * 100.0,
            characters: (self.characters as f32 / total) * 100.0,
            nil_values: (self.nil_values as f32 / total) * 100.0,
            boxed_values: (self.boxed_values as f32 / total) * 100.0,
        }
    }
}

/// Percentage breakdown of types
#[derive(Debug, Clone, Default)]
pub struct TypePercentages {
    /// Percentage of numbers
    pub numbers: f32,
    /// Percentage of booleans
    pub booleans: f32,
    /// Percentage of small integers
    pub small_integers: f32,
    /// Percentage of characters
    pub characters: f32,
    /// Percentage of nil values
    pub nil_values: f32,
    /// Percentage of boxed values
    pub boxed_values: f32,
}

// ============= PERFORMANCE TRACKING =============

/// Global NaN boxing performance tracker
pub struct NanBoxingTracker {
    total_values: AtomicU64,
    optimized_values: AtomicU64,
    memory_saved: AtomicU64,
    type_distribution: std::sync::RwLock<TypeDistribution>,
}

impl NanBoxingTracker {
    /// Create new tracking instance
    pub fn new() -> Self {
        Self {
            total_values: AtomicU64::new(0),
            optimized_values: AtomicU64::new(0),
            memory_saved: AtomicU64::new(0),
            type_distribution: std::sync::RwLock::new(TypeDistribution::new()),
        }
    }
    
    /// Track a new optimized value
    pub fn track_optimization(&self, value: &EnhancedNanBoxedValue) {
        self.total_values.fetch_add(1, Ordering::Relaxed);
        
        // Update type distribution
        if let Ok(mut dist) = self.type_distribution.write() {
            if value.is_number() {
                dist.numbers += 1;
            } else {
                // Determine type from NaN boxed representation
                let nan_boxed = value.to_nan_boxed();
                if nan_boxed.is_boolean() {
                    dist.booleans += 1;
                } else if nan_boxed.is_small_int() {
                    dist.small_integers += 1;
                } else if nan_boxed.is_character() {
                    dist.characters += 1;
                } else if nan_boxed.is_nil() {
                    dist.nil_values += 1;
                } else {
                    dist.boxed_values += 1;
                }
            }
        }
        
        // Calculate memory savings
        let original_size = std::mem::size_of::<Value>();
        let optimized_size = std::mem::size_of::<EnhancedNanBoxedValue>();
        let savings = original_size.saturating_sub(optimized_size);
        
        self.memory_saved.fetch_add(savings as u64, Ordering::Relaxed);
        self.optimized_values.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get comprehensive statistics
    pub fn get_stats(&self) -> GlobalNanBoxingStats {
        let total = self.total_values.load(Ordering::Relaxed);
        let optimized = self.optimized_values.load(Ordering::Relaxed);
        let compression_rate = if total == 0 { 0.0 } else { optimized as f32 / total as f32 };
        
        let type_dist = self.type_distribution.read()
            .map(|d| d.clone())
            .unwrap_or_else(|_| TypeDistribution::new());
        
        GlobalNanBoxingStats {
            total_values: total,
            optimized_values: optimized,
            total_memory_saved: self.memory_saved.load(Ordering::Relaxed),
            compression_rate,
            type_distribution: type_dist,
        }
    }
    
    /// Reset all statistics
    pub fn reset(&self) {
        self.total_values.store(0, Ordering::Relaxed);
        self.optimized_values.store(0, Ordering::Relaxed);
        self.memory_saved.store(0, Ordering::Relaxed);
        
        if let Ok(mut dist) = self.type_distribution.write() {
            *dist = TypeDistribution::new();
        }
    }
}

/// Global statistics for NaN boxing optimization
#[derive(Debug, Clone)]
pub struct GlobalNanBoxingStats {
    /// Total values processed
    pub total_values: u64,
    /// Number of optimized values
    pub optimized_values: u64,
    /// Total memory saved in bytes
    pub total_memory_saved: u64,
    /// Optimization success rate
    pub compression_rate: f32,
    /// Type distribution
    pub type_distribution: TypeDistribution,
}

// ============= VALUE CONVERSION UTILITIES =============

/// Utility for converting between Value and optimized representations
pub struct ValueOptimizer {
    tracker: NanBoxingTracker,
    optimization_threshold: usize,
}

impl ValueOptimizer {
    /// Create new value optimizer
    pub fn new() -> Self {
        Self {
            tracker: NanBoxingTracker::new(),
            optimization_threshold: 1000,
        }
    }
    
    /// Convert Value to optimized representation
    pub fn optimize_value(&self, value: &Value, hint: OptimizationHint) -> Option<EnhancedNanBoxedValue> {
        let enhanced = if value.is_number() {
            if let Some(n) = value.as_number() {
                Some(EnhancedNanBoxedValue::from_f64_optimized(n))
            } else {
                None
            }
        } else if value.is_boolean() {
            if let Some(b) = value.as_boolean() {
                let nan_boxed = NanBoxedValue::from_bool(b);
                Some(EnhancedNanBoxedValue::from_nan_boxed(nan_boxed, hint))
            } else {
                None
            }
        } else if let Value::Literal(Literal::Character(c)) = value {
            let nan_boxed = NanBoxedValue::from_char(*c);
            Some(EnhancedNanBoxedValue::from_nan_boxed(nan_boxed, hint))
        } else if value.is_nil() {
            let nan_boxed = NanBoxedValue::nil_value();
            Some(EnhancedNanBoxedValue::from_nan_boxed(nan_boxed, hint))
        } else {
            None // Cannot optimize complex types in stable version
        };
        
        if let Some(ref optimized) = enhanced {
            self.tracker.track_optimization(optimized);
        }
        
        enhanced
    }
    
    /// Convert optimized value back to Value
    pub fn deoptimize_value(&self, value: EnhancedNanBoxedValue) -> Value {
        if let Some(n) = value.as_number() {
            return Value::number(n);
        }
        
        let nan_boxed = value.to_nan_boxed();
        
        if nan_boxed.is_boolean() {
            if nan_boxed == NanBoxedValue::true_value() {
                Value::boolean(true)
            } else if nan_boxed == NanBoxedValue::false_value() {
                Value::boolean(false)
            } else {
                Value::Unspecified
            }
        } else if nan_boxed.is_nil() {
            Value::Nil
        } else if let Some(c) = nan_boxed.as_char() {
            Value::Literal(Literal::Character(c))
        } else {
            Value::Unspecified
        }
    }
    
    /// Get optimization statistics
    pub fn get_stats(&self) -> GlobalNanBoxingStats {
        self.tracker.get_stats()
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) {
        self.tracker.reset();
    }
}

// ============= EXTENSION TRAITS =============

/// Extension trait for Value to enable NaN boxing optimization
pub trait ValueNanBoxingExt {
    /// Convert to optimized NaN-boxed representation
    fn to_nan_boxed_optimized(&self, hint: OptimizationHint) -> Option<EnhancedNanBoxedValue>;
    
    /// Check if value can be NaN-boxed efficiently
    fn can_nan_box(&self) -> bool;
    
    /// Get estimated memory savings from NaN boxing
    fn nan_boxing_savings(&self) -> usize;
}

impl ValueNanBoxingExt for Value {
    fn to_nan_boxed_optimized(&self, hint: OptimizationHint) -> Option<EnhancedNanBoxedValue> {
        let optimizer = ValueOptimizer::new();
        optimizer.optimize_value(self, hint)
    }
    
    fn can_nan_box(&self) -> bool {
        // Only simple types can be reliably NaN-boxed in stable version
        // Only simple types can be reliably NaN-boxed in stable version
        self.is_number() || self.is_boolean() || matches!(self, Value::Literal(Literal::Character(_))) || self.is_nil()
    }
    
    fn nan_boxing_savings(&self) -> usize {
        if self.can_nan_box() {
            let original_size = std::mem::size_of::<Value>();
            let optimized_size = std::mem::size_of::<EnhancedNanBoxedValue>();
            original_size.saturating_sub(optimized_size)
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
    fn test_enhanced_nan_boxing_creation() {
        let value = EnhancedNanBoxedValue::from_f64_optimized(42.0);
        assert!(value.is_number());
        assert_eq!(value.as_number(), Some(42.0));
        assert_eq!(value.optimization_hint(), OptimizationHint::Computational);
    }
    
    #[test]
    fn test_integer_optimization() {
        let value = EnhancedNanBoxedValue::from_i64_optimized(123);
        assert!(value.is_number());
        // Should work with either small int or f64 representation
        let num = value.as_number().unwrap();
        assert!((num - 123.0).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_optimization_in_place() {
        let mut value = EnhancedNanBoxedValue::from_nan_boxed(
            NanBoxedValue::from_number(3.14),
            OptimizationHint::Default
        );
        
        let optimized = value.optimize();
        assert!(optimized);
        assert_eq!(value.optimization_hint(), OptimizationHint::Computational);
    }
    
    #[test]
    fn test_trait_optimized_value_impl() {
        let nan_boxed = NanBoxedValue::from_number(42.0);
        let enhanced = EnhancedNanBoxedValue::from_inner(nan_boxed, OptimizationHint::HotPath);
        
        assert!(enhanced.should_optimize());
        
        let extracted = enhanced.into_inner();
        assert_eq!(extracted.as_number(), Some(42.0));
    }
    
    #[test]
    fn test_type_distribution() {
        let mut dist = TypeDistribution::new();
        dist.numbers = 50;
        dist.booleans = 30;
        dist.nil_values = 20;
        
        assert_eq!(dist.total(), 100);
        
        let percentages = dist.percentages();
        assert_eq!(percentages.numbers, 50.0);
        assert_eq!(percentages.booleans, 30.0);
        assert_eq!(percentages.nil_values, 20.0);
    }
    
    #[test]
    fn test_nan_boxing_tracker() {
        let tracker = NanBoxingTracker::new();
        let value = EnhancedNanBoxedValue::from_f64_optimized(3.14);
        
        tracker.track_optimization(&value);
        let stats = tracker.get_stats();
        
        assert_eq!(stats.total_values, 1);
        assert_eq!(stats.optimized_values, 1);
        assert!(stats.total_memory_saved > 0);
        assert!(stats.compression_rate > 0.0);
    }
    
    #[test]
    fn test_value_optimizer() {
        let optimizer = ValueOptimizer::new();
        
        // Test number optimization
        let value = Value::number(42.0);
        let optimized = optimizer.optimize_value(&value, OptimizationHint::Computational);
        assert!(optimized.is_some());
        
        let enhanced = optimized.unwrap();
        assert_eq!(enhanced.as_number(), Some(42.0));
        
        // Test roundtrip
        let restored = optimizer.deoptimize_value(enhanced);
        assert!(restored.is_number());
    }
    
    #[test]
    fn test_value_nan_boxing_ext() {
        let value = Value::number(3.14);
        assert!(value.can_nan_box());
        assert!(value.nan_boxing_savings() > 0);
        
        let optimized = value.to_nan_boxed_optimized(OptimizationHint::Default);
        assert!(optimized.is_some());
        assert_eq!(optimized.unwrap().as_number(), Some(3.14));
    }
    
    #[test]
    fn test_boolean_optimization() {
        let value = Value::boolean(true);
        let optimizer = ValueOptimizer::new();
        
        let optimized = optimizer.optimize_value(&value, OptimizationHint::Default);
        assert!(optimized.is_some());
        
        let restored = optimizer.deoptimize_value(optimized.unwrap());
        assert!(restored.is_boolean());
        assert_eq!(restored.as_boolean(), Some(true));
    }
}