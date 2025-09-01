//! Advanced NaN Boxing Optimization with Trait Integration
//!
//! This module extends the existing NaN boxing implementation with
//! the trait-based optimization framework, providing unified
//! memory-efficient value representation.

use crate::eval::{NanBoxedValue, Value, OptimizedNanBoxedValue, TraitOptimizedValue, OptimizationHint};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// ============= ENHANCED NAN BOXING =============

/// Enhanced NaN-boxed value with optimization tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnhancedNanBoxedValue {
    inner: u64,
    _optimization_hint: OptimizationHint,
}

impl EnhancedNanBoxedValue {
    /// Create new enhanced NaN-boxed value from raw value and hint
    pub fn new(inner: u64, hint: OptimizationHint) -> Self {
        Self {
            inner,
            _optimization_hint: hint,
        }
    }
    
    /// Create from existing NaN-boxed value with optimization hint
    pub fn from_nan_boxed(value: NanBoxedValue, hint: OptimizationHint) -> Self {
        Self {
            inner: value.into_raw(),
            _optimization_hint: hint,
        }
    }
    
    /// Create optimized value directly from native types
    pub fn from_f64_optimized(value: f64) -> Self {
        if value.is_finite() {
            // Use direct representation for finite numbers
            Self {
                inner: value.to_bits(),
                _optimization_hint: OptimizationHint::Computational,
            }
        } else {
            // Use NaN boxing for special values
            Self {
                inner: NanBoxedValue::from_number(value).into_raw(),
                _optimization_hint: OptimizationHint::Default,
            }
        }
    }
    
    /// Create from integer with range optimization
    pub fn from_i64_optimized(value: i64) -> Self {
        if let Some(small_int) = NanBoxedValue::from_small_int(value) {
            // Use small integer encoding
            Self {
                inner: small_int.into_raw(),
                _optimization_hint: OptimizationHint::Computational,
            }
        } else {
            // Fall back to boxed integer
            Self {
                inner: NanBoxedValue::from_boxed_int(value).into_raw(),
                _optimization_hint: OptimizationHint::LongTerm,
            }
        }
    }
    
    /// Extract the inner NaN-boxed value
    pub fn to_nan_boxed(&self) -> NanBoxedValue {
        NanBoxedValue::from_raw(self.inner)
    }
    
    /// Get optimization hint
    pub fn optimization_hint(&self) -> OptimizationHint {
        self._optimization_hint
    }
    
    /// Check if value can be optimized further
    pub fn can_optimize(&self) -> bool {
        match self._optimization_hint {
            OptimizationHint::Default => true,
            _ => false,
        }
    }
}

impl TraitOptimizedValue for EnhancedNanBoxedValue {
    type Inner = NanBoxedValue;
    type MemoryStats = NanBoxingStats;
    
    fn from_inner(inner: Self::Inner, hint: OptimizationHint) -> Self {
        Self {
            inner: inner.into_raw(),
            _optimization_hint: hint,
        }
    }
    
    fn into_inner(self) -> Self::Inner {
        NanBoxedValue::from_raw(self.inner)
    }
    
    fn memory_stats(&self) -> Self::MemoryStats {
        NanBoxingStats {
            total_size: std::mem::size_of::<Self>(),
            effective_compression: self.compression_ratio(),
            type_distribution: self.type_stats(),
        }
    }
    
    fn optimize(&mut self) -> bool {
        if !self.can_optimize() {
            return false;
        }
        
        let nan_boxed = self.to_nan_boxed();
        
        // Apply type-specific optimizations
        if nan_boxed.is_number() {
            if let Some(f) = nan_boxed.as_number() {
                *self = Self::from_f64_optimized(f);
                return true;
            }
        }
        
        if nan_boxed.is_small_int() {
            if let Some(i) = nan_boxed.as_small_int() {
                *self = Self::from_i64_optimized(i);
                return true;
            }
        }
        
        false
    }
}

impl EnhancedNanBoxedValue {
    /// Calculate compression ratio compared to Value enum
    fn compression_ratio(&self) -> f32 {
        let value_size = std::mem::size_of::<Value>();
        let nan_boxed_size = std::mem::size_of::<Self>();
        value_size as f32 / nan_boxed_size as f32
    }
    
    /// Get type distribution statistics
    fn type_stats(&self) -> TypeStats {
        let nan_boxed = self.to_nan_boxed();
        
        TypeStats {
            is_number: nan_boxed.is_number(),
            is_boolean: nan_boxed.is_boolean(),
            is_small_int: nan_boxed.is_small_int(),
            is_character: nan_boxed.is_character(),
            is_nil: nan_boxed.is_nil(),
            is_boxed: !nan_boxed.is_immediate(),
        }
    }
}

// ============= MEMORY OPTIMIZATION STATISTICS =============

/// Statistics about NaN boxing memory usage
#[derive(Debug, Clone)]
pub struct NanBoxingStats {
    pub total_size: usize,
    pub effective_compression: f32,
    pub type_distribution: TypeStats,
}

/// Type distribution in NaN-boxed values
#[derive(Debug, Clone)]
pub struct TypeStats {
    pub is_number: bool,
    pub is_boolean: bool, 
    pub is_small_int: bool,
    pub is_character: bool,
    pub is_nil: bool,
    pub is_boxed: bool,
}

/// Global NaN boxing performance tracker
pub struct NanBoxingTracker {
    total_values: AtomicU64,
    optimized_values: AtomicU64,
    memory_saved: AtomicU64,
    type_counts: HashMap<&'static str, AtomicU64>,
}

impl NanBoxingTracker {
    /// Create new tracking instance
    pub fn new() -> Self {
        let mut type_counts = HashMap::new();
        type_counts.insert("number", AtomicU64::new(0));
        type_counts.insert("boolean", AtomicU64::new(0));
        type_counts.insert("small_int", AtomicU64::new(0));
        type_counts.insert("character", AtomicU64::new(0));
        type_counts.insert("nil", AtomicU64::new(0));
        type_counts.insert("boxed", AtomicU64::new(0));
        
        Self {
            total_values: AtomicU64::new(0),
            optimized_values: AtomicU64::new(0),
            memory_saved: AtomicU64::new(0),
            type_counts,
        }
    }
    
    /// Track a new optimized value
    pub fn track_optimization(&self, value: &EnhancedNanBoxedValue) {
        self.total_values.fetch_add(1, Ordering::Relaxed);
        
        let stats = value.type_stats();
        
        if stats.is_number {
            self.type_counts.get("number").unwrap().fetch_add(1, Ordering::Relaxed);
        } else if stats.is_boolean {
            self.type_counts.get("boolean").unwrap().fetch_add(1, Ordering::Relaxed);
        } else if stats.is_small_int {
            self.type_counts.get("small_int").unwrap().fetch_add(1, Ordering::Relaxed);
        } else if stats.is_character {
            self.type_counts.get("character").unwrap().fetch_add(1, Ordering::Relaxed);
        } else if stats.is_nil {
            self.type_counts.get("nil").unwrap().fetch_add(1, Ordering::Relaxed);
        } else if stats.is_boxed {
            self.type_counts.get("boxed").unwrap().fetch_add(1, Ordering::Relaxed);
        }
        
        // Calculate memory savings
        let original_size = std::mem::size_of::<Value>();
        let optimized_size = std::mem::size_of::<EnhancedNanBoxedValue>();
        let savings = original_size - optimized_size;
        
        self.memory_saved.fetch_add(savings as u64, Ordering::Relaxed);
        self.optimized_values.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get comprehensive statistics
    pub fn get_stats(&self) -> GlobalNanBoxingStats {
        GlobalNanBoxingStats {
            total_values: self.total_values.load(Ordering::Relaxed),
            optimized_values: self.optimized_values.load(Ordering::Relaxed),
            total_memory_saved: self.memory_saved.load(Ordering::Relaxed),
            compression_rate: self.calculate_compression_rate(),
            type_distribution: self.get_type_distribution(),
        }
    }
    
    fn calculate_compression_rate(&self) -> f32 {
        let total = self.total_values.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        
        let optimized = self.optimized_values.load(Ordering::Relaxed);
        optimized as f32 / total as f32
    }
    
    fn get_type_distribution(&self) -> HashMap<String, u64> {
        self.type_counts
            .iter()
            .map(|(k, v)| (k.to_string(), v.load(Ordering::Relaxed)))
            .collect()
    }
}

/// Global statistics for NaN boxing optimization
#[derive(Debug, Clone)]
pub struct GlobalNanBoxingStats {
    pub total_values: u64,
    pub optimized_values: u64,
    pub total_memory_saved: u64,
    pub compression_rate: f32,
    pub type_distribution: HashMap<String, u64>,
}

// ============= CONVERSION UTILITIES =============

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
            optimization_threshold: 1000, // Optimize after 1000 uses
        }
    }
    
    /// Convert Value to optimized representation
    pub fn optimize_value(&self, value: Value, hint: OptimizationHint) -> EnhancedNanBoxedValue {
        let nan_boxed = match value {
            Value::number(n) => NanBoxedValue::from_number(n),
            Value::Boolean(b) => NanBoxedValue::from_boolean(b),
            Value::Character(c) => NanBoxedValue::from_char(c),
            Value::Nil => NanBoxedValue::nil_value(),
            Value::Unspecified => NanBoxedValue::unspecified_value(),
            _ => {
                // For complex types, use boxed representation
                // This is a placeholder - actual implementation would handle
                // proper conversion for all Value variants
                NanBoxedValue::unspecified_value()
            }
        };
        
        let optimized = EnhancedNanBoxedValue::from_nan_boxed(nan_boxed, hint);
        self.tracker.track_optimization(&optimized);
        optimized
    }
    
    /// Convert optimized value back to Value
    pub fn deoptimize_value(&self, value: EnhancedNanBoxedValue) -> Value {
        let nan_boxed = value.to_nan_boxed();
        
        if nan_boxed.is_number() {
            if let Some(n) = nan_boxed.as_number() {
                return Value::number(n);
            }
        }
        
        if nan_boxed.is_boolean() {
            // Check for specific boolean patterns
            if nan_boxed == NanBoxedValue::true_value() {
                return Value::boolean(true);
            } else if nan_boxed == NanBoxedValue::false_value() {
                return Value::boolean(false);
            }
        }
        
        if nan_boxed.is_character() {
            // Characters need proper extraction from NaN boxing
            // This is a placeholder - actual implementation would extract the character
            return Value::character('?');
        }
        
        if nan_boxed.is_nil() {
            return Value::nil();
        }
        
        if nan_boxed.is_unspecified() {
            return Value::unspecified();
        }
        
        // For other types, attempt to reconstruct from boxed representation
        Value::unspecified() // Placeholder
    }
    
    /// Get optimization statistics
    pub fn get_stats(&self) -> GlobalNanBoxingStats {
        self.tracker.get_stats()
    }
}

// ============= INTEGRATION WITH EXISTING SYSTEM =============

/// Extension trait for Value to enable NaN boxing optimization
pub trait ValueNanBoxingExt {
    /// Convert to optimized NaN-boxed representation
    fn to_nan_boxed_optimized(&self, hint: OptimizationHint) -> EnhancedNanBoxedValue;
    /// Check if value can be NaN-boxed efficiently
    fn can_nan_box(&self) -> bool;
    /// Get estimated memory savings from NaN boxing
    fn nan_boxing_savings(&self) -> usize;
}

impl ValueNanBoxingExt for Value {
    fn to_nan_boxed_optimized(&self, hint: OptimizationHint) -> EnhancedNanBoxedValue {
        let optimizer = ValueOptimizer::new();
        optimizer.optimize_value(self.clone(), hint)
    }
    
    fn can_nan_box(&self) -> bool {
        match self {
            Value::number(_) => true,
            Value::Boolean(_) => true,
            Value::Character(_) => true,
            Value::Nil => true,
            Value::Unspecified => true,
            // String, symbols, and other types that fit in 48 bits
            _ => false,
        }
    }
    
    fn nan_boxing_savings(&self) -> usize {
        if self.can_nan_box() {
            let original_size = std::mem::size_of::<Value>();
            let optimized_size = std::mem::size_of::<EnhancedNanBoxedValue>();
            original_size - optimized_size
        } else {
            0
        }
    }
}

// ============= TESTING AND VALIDATION =============

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_nan_boxing() {
        let value = EnhancedNanBoxedValue::from_f64_optimized(42.0);
        let nan_boxed = value.to_nan_boxed();
        
        assert!(nan_boxed.is_number());
        assert_eq!(nan_boxed.as_number(), Some(42.0));
    }
    
    #[test]
    fn test_optimization() {
        let mut value = EnhancedNanBoxedValue::from_nan_boxed(
            NanBoxedValue::from_number(3.14),
            OptimizationHint::Default
        );
        
        let optimized = value.optimize();
        assert!(optimized);
    }
    
    #[test]
    fn test_memory_stats() {
        let value = EnhancedNanBoxedValue::from_f64_optimized(123.456);
        let stats = value.memory_stats();
        
        assert!(stats.effective_compression > 1.0);
        assert_eq!(stats.total_size, std::mem::size_of::<EnhancedNanBoxedValue>());
    }
    
    #[test]
    fn test_value_conversion() {
        let original = Value::number(42.0);
        let optimized = original.to_nan_boxed_optimized(OptimizationHint::Computational);
        
        let optimizer = ValueOptimizer::new();
        let restored = optimizer.deoptimize_value(optimized);
        
        // Should be functionally equivalent
        // Check if restored value is equivalent (using match pattern)
        if let Value::number(_) = restored {
            // Test passed
        } else {
            panic!("Expected number value");
        }
    }
    
    #[test]
    fn test_global_tracking() {
        let tracker = NanBoxingTracker::new();
        let value = EnhancedNanBoxedValue::from_f64_optimized(3.14);
        
        tracker.track_optimization(&value);
        let stats = tracker.get_stats();
        
        assert_eq!(stats.total_values, 1);
        assert_eq!(stats.optimized_values, 1);
        assert!(stats.total_memory_saved > 0);
    }
}