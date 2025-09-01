//! Branch Prediction Optimization for Numeric Tower Operations
//!
//! This module implements intelligent branch prediction optimizations to reduce
//! branch mispredictions in numeric tower operations by using lookup tables,
//! type pattern caching, and operation frequency analysis.

use crate::numeric::{NumericType, NumericValue};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Branch prediction optimization engine
pub struct BranchPredictor {
    /// Type combination frequency table for binary operations
    type_combinations: HashMap<(NumericType, NumericType), TypeCombinationStats>,
    /// Operation frequency counters
    operation_stats: HashMap<String, AtomicU64>,
    /// Lookup table for common numeric conversions
    conversion_cache: ConversionLookupTable,
    /// Pattern recognition for operation sequences
    pattern_cache: PatternCache,
}

/// Statistics for type combination patterns
#[derive(Debug, Clone)]
struct TypeCombinationStats {
    /// Number of times this combination occurred
    frequency: u64,
    /// Average execution time in nanoseconds
    avg_time_ns: f64,
    /// Preferred promotion target for this combination
    preferred_target: NumericType,
    /// Success rate of this promotion path
    promotion_success_rate: f64,
}

/// Fast lookup table for common numeric conversions
struct ConversionLookupTable {
    /// Small integer to float conversions (most common)
    int_to_float: [f64; 1024], // -512 to 511
    /// Common rational approximations of floats
    float_to_rational: HashMap<u64, (i32, i32)>, // bits -> (num, denom)
    /// Powers of 2 for quick checks
    powers_of_two: [i64; 64],
    /// Common mathematical constants
    math_constants: HashMap<String, f64>,
}

/// Pattern recognition cache for operation sequences
struct PatternCache {
    /// Recently seen operation sequences
    recent_sequences: Vec<OperationPattern>,
    /// Predicted next operations based on patterns
    predictions: HashMap<u64, Vec<(String, f32)>>, // hash -> [(op_name, probability)]
}

/// Operation pattern for sequence prediction
#[derive(Debug, Clone)]
struct OperationPattern {
    /// Sequence of operation types
    operations: Vec<String>,
    /// Types involved in each operation
    types: Vec<(NumericType, NumericType)>,
    /// Timestamp for aging
    timestamp: std::time::Instant,
    /// Hash for quick lookup
    hash: u64,
}

impl BranchPredictor {
    /// Creates a new branch predictor with pre-populated lookup tables
    pub fn new() -> Self {
        let mut predictor = BranchPredictor {
            type_combinations: HashMap::new(),
            operation_stats: HashMap::new(),
            conversion_cache: ConversionLookupTable::new(),
            pattern_cache: PatternCache::new(),
        };

        predictor.initialize_common_patterns();
        predictor
    }

    /// Predicts the optimal promotion path for two numeric values
    pub fn predict_promotion(
        &mut self,
        left: &NumericValue,
        right: &NumericValue,
    ) -> (NumericType, f32) {
        let left_type = left.numeric_type();
        let right_type = right.numeric_type();
        let combo_key = (left_type, right_type);

        // Check if we have statistics for this combination
        if let Some(stats) = self.type_combinations.get(&combo_key) {
            if stats.frequency > 10 && stats.promotion_success_rate > 0.8 {
                // High confidence prediction
                return (stats.preferred_target, stats.promotion_success_rate as f32);
            }
        }

        // Fallback to heuristic-based prediction
        let predicted_type = self.heuristic_promotion(left_type, right_type);
        (predicted_type, 0.5) // Medium confidence for heuristic
    }

    /// Records the result of an operation for learning
    pub fn record_operation(
        &mut self,
        op_name: &str,
        left_type: NumericType,
        right_type: NumericType,
        result_type: NumericType,
        execution_time_ns: u64,
        success: bool,
    ) {
        // Update operation frequency
        let counter = self
            .operation_stats
            .entry(op_name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);

        // Update type combination statistics
        let combo_key = (left_type, right_type);
        let stats =
            self.type_combinations
                .entry(combo_key)
                .or_insert_with(|| TypeCombinationStats {
                    frequency: 0,
                    avg_time_ns: 0.0,
                    preferred_target: result_type,
                    promotion_success_rate: 0.0,
                });

        // Update statistics with exponential moving average
        stats.frequency += 1;
        let alpha = 0.1; // Learning rate
        stats.avg_time_ns = (1.0 - alpha) * stats.avg_time_ns + alpha * execution_time_ns as f64;

        if success {
            stats.promotion_success_rate = (1.0 - alpha) * stats.promotion_success_rate + alpha;
            if stats.frequency > 5 {
                stats.preferred_target = result_type;
            }
        } else {
            stats.promotion_success_rate *= 1.0 - alpha;
        }

        // Update pattern cache
        self.pattern_cache
            .record_operation(op_name, left_type, right_type);
    }

    /// Optimized type check with branch prediction hints
    #[inline(always)]
    pub fn likely_type_check(&self, value: &NumericValue, expected: NumericType) -> bool {
        let actual = value.numeric_type();

        // Use CPU branch prediction hints based on our statistics
        if let Some(stats) = self.type_combinations.get(&(actual, expected)) {
            if stats.frequency > 100 && stats.promotion_success_rate > 0.9 {
                // Very likely match - hint to processor
                return likely(actual == expected);
            }
        }

        actual == expected
    }

    /// Fast conversion with lookup table optimization
    pub fn fast_convert_small_int(&self, value: i64) -> Option<f64> {
        if (-512..=511).contains(&value) {
            // Use lookup table for common small integers
            let index = (value + 512) as usize;
            Some(self.conversion_cache.int_to_float[index])
        } else {
            None // Fall back to regular conversion
        }
    }

    /// Predicts whether a float can be exactly represented as a rational
    pub fn predict_rational_conversion(&self, value: f64) -> Option<(i32, i32)> {
        let bits = value.to_bits();
        self.conversion_cache.float_to_rational.get(&bits).copied()
    }

    /// Checks if a value is a power of two using lookup table
    pub fn is_power_of_two_fast(&self, value: i64) -> bool {
        if value <= 0 {
            return false;
        }

        // Check against precomputed powers of two
        for &power in &self.conversion_cache.powers_of_two {
            if power == value {
                return true;
            }
            if power > value {
                break;
            }
        }

        false
    }

    /// Gets a mathematical constant if recognized
    pub fn get_math_constant(&self, name: &str) -> Option<f64> {
        self.conversion_cache.math_constants.get(name).copied()
    }

    /// Predicts the next likely operation based on current pattern
    pub fn predict_next_operation(&self, recent_ops: &[String]) -> Option<(String, f32)> {
        if recent_ops.is_empty() {
            return None;
        }

        let pattern_hash = self.compute_pattern_hash(recent_ops);

        if let Some(predictions) = self.pattern_cache.predictions.get(&pattern_hash) {
            // Return the most likely next operation
            predictions
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(op, prob)| (op.clone(), *prob))
        } else {
            None
        }
    }

    /// Generates branch prediction hints for conditional compilation
    pub fn generate_optimization_hints(&self) -> Vec<String> {
        let mut hints = Vec::new();

        // Find most common type combinations
        let mut sorted_combinations: Vec<_> = self.type_combinations.iter().collect();
        sorted_combinations.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.frequency));

        for ((left, right), stats) in sorted_combinations.iter().take(10) {
            if stats.frequency > 100 {
                hints.push(format!(
                    "// LIKELY: {:?} + {:?} -> {:?} (freq: {}, success: {:.2}%)",
                    left,
                    right,
                    stats.preferred_target,
                    stats.frequency,
                    stats.promotion_success_rate * 100.0
                ));
            }
        }

        hints
    }

    /// Internal: Initialize common patterns and conversions
    fn initialize_common_patterns(&mut self) {
        // Pre-populate common integer conversions
        for i in -512i64..=511 {
            self.conversion_cache.int_to_float[(i + 512) as usize] = i as f64;
        }

        // Initialize powers of two
        for i in 0..64 {
            self.conversion_cache.powers_of_two[i] = 1i64 << i;
        }

        // Common mathematical constants
        self.conversion_cache
            .math_constants
            .insert("pi".to_string(), std::f64::consts::PI);
        self.conversion_cache
            .math_constants
            .insert("e".to_string(), std::f64::consts::E);
        self.conversion_cache
            .math_constants
            .insert("sqrt2".to_string(), std::f64::consts::SQRT_2);
        self.conversion_cache
            .math_constants
            .insert("ln2".to_string(), std::f64::consts::LN_2);

        // Common rational approximations
        let common_rationals: [(f64, (i32, i32)); 7] = [
            (0.5, (1, 2)),
            (0.25, (1, 4)),
            (0.75, (3, 4)),
            (0.333333, (1, 3)),
            (0.666667, (2, 3)),
            (0.2, (1, 5)),
            (0.125, (1, 8)),
        ];

        for (float_val, (num, denom)) in common_rationals.iter() {
            self.conversion_cache
                .float_to_rational
                .insert((*float_val).to_bits(), (*num, *denom));
        }
    }

    /// Internal: Heuristic-based type promotion when no statistics available
    fn heuristic_promotion(&self, left: NumericType, right: NumericType) -> NumericType {
        use NumericType::*;

        match (left, right) {
            // Same types
            (Integer, Integer) => Integer,
            (BigInteger, BigInteger) => BigInteger,
            (Rational, Rational) => Rational,
            (Real, Real) => Real,
            (Complex, Complex) => Complex,
            (Vector, Vector) => Vector,

            // Common promotions (in order of frequency)
            (Integer, Real) | (Real, Integer) => Real,
            (Integer, Rational) | (Rational, Integer) => Rational,
            (Rational, Real) | (Real, Rational) => Real,
            (Integer, Complex) | (Complex, Integer) => Complex,
            (Real, Complex) | (Complex, Real) => Complex,
            (Rational, Complex) | (Complex, Rational) => Complex,

            // BigInteger promotions
            (Integer, BigInteger) | (BigInteger, Integer) => BigInteger,
            (BigInteger, Real) | (Real, BigInteger) => Real,
            (BigInteger, Rational) | (Rational, BigInteger) => Rational,
            (BigInteger, Complex) | (Complex, BigInteger) => Complex,

            // Vector operations
            (Vector, _) | (_, Vector) => Vector,
        }
    }

    /// Resets all performance statistics
    pub fn reset_performance_stats(&mut self) {
        self.type_combinations.clear();
        self.operation_stats.clear();
        self.pattern_cache = PatternCache::new();
    }

    /// Internal: Compute hash for operation pattern
    fn compute_pattern_hash(&self, ops: &[String]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for op in ops {
            op.hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl ConversionLookupTable {
    fn new() -> Self {
        Self {
            int_to_float: [0.0; 1024],
            float_to_rational: HashMap::new(),
            powers_of_two: [0; 64],
            math_constants: HashMap::new(),
        }
    }
}

impl PatternCache {
    fn new() -> Self {
        Self {
            recent_sequences: Vec::new(),
            predictions: HashMap::new(),
        }
    }

    fn record_operation(&mut self, op_name: &str, left_type: NumericType, right_type: NumericType) {
        // Implementation for pattern recording
        // This would build up patterns over time and generate predictions

        // For now, simple implementation
        if self.recent_sequences.len() > 100 {
            // Remove old patterns
            self.recent_sequences
                .retain(|p| p.timestamp.elapsed().as_secs() < 300); // 5 minutes
        }
    }
}

/// Branch prediction hint macro for likely conditions
#[inline(always)]
fn likely(condition: bool) -> bool {
    // On supporting compilers, this would translate to __builtin_expect(condition, 1)
    // For now, just return the condition
    condition
}

/// Branch prediction hint macro for unlikely conditions
#[inline(always)]
pub fn unlikely(condition: bool) -> bool {
    // On supporting compilers, this would translate to __builtin_expect(condition, 0)
    // For now, just return the condition
    condition
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_predictor_creation() {
        let predictor = BranchPredictor::new();

        // Test that lookup tables are initialized
        assert_eq!(predictor.fast_convert_small_int(42), Some(42.0));
        assert_eq!(predictor.fast_convert_small_int(1000), None);

        // Test math constants
        assert_eq!(
            predictor.get_math_constant("pi"),
            Some(std::f64::consts::PI)
        );
        assert_eq!(predictor.get_math_constant("unknown"), None);
    }

    #[test]
    fn test_type_promotion_prediction() {
        let mut predictor = BranchPredictor::new();

        let int_val = NumericValue::integer(42);
        let real_val = NumericValue::real(3.14);

        let (predicted_type, confidence) = predictor.predict_promotion(&int_val, &real_val);
        assert_eq!(predicted_type, NumericType::Real);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_power_of_two_detection() {
        let predictor = BranchPredictor::new();

        assert!(predictor.is_power_of_two_fast(1));
        assert!(predictor.is_power_of_two_fast(2));
        assert!(predictor.is_power_of_two_fast(4));
        assert!(predictor.is_power_of_two_fast(1024));
        assert!(!predictor.is_power_of_two_fast(3));
        assert!(!predictor.is_power_of_two_fast(100));
        assert!(!predictor.is_power_of_two_fast(0));
        assert!(!predictor.is_power_of_two_fast(-1));
    }

    #[test]
    fn test_learning_from_operations() {
        let mut predictor = BranchPredictor::new();

        // Simulate multiple integer + real operations
        for _ in 0..20 {
            predictor.record_operation(
                "add",
                NumericType::Integer,
                NumericType::Real,
                NumericType::Real,
                1000, // 1 microsecond
                true,
            );
        }

        // Check that the predictor learned the pattern
        let int_val = NumericValue::integer(5);
        let real_val = NumericValue::real(2.5);
        let (predicted_type, confidence) = predictor.predict_promotion(&int_val, &real_val);

        assert_eq!(predicted_type, NumericType::Real);
        assert!(confidence > 0.8); // High confidence after learning
    }

    #[test]
    fn test_rational_conversion_prediction() {
        let predictor = BranchPredictor::new();

        // Test common rational conversions
        assert_eq!(predictor.predict_rational_conversion(0.5), Some((1, 2)));
        assert_eq!(predictor.predict_rational_conversion(0.25), Some((1, 4)));
        assert_eq!(predictor.predict_rational_conversion(0.75), Some((3, 4)));
        assert_eq!(predictor.predict_rational_conversion(0.123456), None);
    }
}
