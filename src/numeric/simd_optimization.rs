//! SIMD-optimized numeric operations for high performance computation
//!
//! This module provides vectorized implementations of common numeric operations
//! using SIMD instructions where available, with fallbacks for non-SIMD platforms.

use crate::numeric::{NumericValue, NumericType, tower};

// Import appropriate SIMD intrinsics based on target architecture
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Configuration for SIMD optimizations
#[derive(Debug, Clone)]
pub struct SimdConfig {
    /// Enable AVX2 instructions if available (x86_64 only)
    pub enable_avx2: bool,
    /// Enable SSE4.1 instructions if available (x86_64 only)
    pub enable_sse41: bool,
    /// Enable NEON instructions if available (ARM64 only)
    pub enable_neon: bool,
    /// Minimum array size to use SIMD (smaller arrays use scalar operations)
    pub simd_threshold: usize,
}

impl Default for SimdConfig {
    fn default() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self {
                enable_avx2: is_x86_feature_detected!("avx2"),
                enable_sse41: is_x86_feature_detected!("sse4.1"),
                enable_neon: false,
                simd_threshold: 8,
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            Self {
                enable_avx2: false,
                enable_sse41: false,
                enable_neon: std::arch::is_aarch64_feature_detected!("neon"),
                simd_threshold: 8,
            }
        }
        
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self {
                enable_avx2: false,
                enable_sse41: false,
                enable_neon: false,
                simd_threshold: 8,
            }
        }
    }
}

/// SIMD-optimized numeric operations
pub struct SimdNumericOps {
    config: SimdConfig,
}

impl SimdNumericOps {
    /// Creates a new SIMD operations instance
    pub fn new(config: SimdConfig) -> Self {
        Self { config }
    }
    
    /// Creates a new instance with default configuration
    pub fn default() -> Self {
        Self::new(SimdConfig::default())
    }
    
    /// Performs vectorized addition of f64 arrays
    pub fn add_f64_arrays(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        if a.len() != b.len() || a.len() != result.len() {
            return Err("Array length mismatch".to_string());
        }
        
        let len = a.len();
        if len < self.config.simd_threshold {
            // Use scalar operations for small arrays
            for i in 0..len {
                result[i] = a[i] + b[i];
            }
            return Ok(());
        }
        
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if self.config.enable_avx2 && is_x86_feature_detected!("avx2") {
                return self.add_f64_arrays_avx2(a, b, result);
            } else if self.config.enable_sse41 && is_x86_feature_detected!("sse2") {
                return self.add_f64_arrays_sse2(a, b, result);
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if self.config.enable_neon && std::arch::is_aarch64_feature_detected!("neon") {
                return unsafe { self.add_f64_arrays_neon(a, b, result) };
            }
        }
        
        // Fallback to scalar operations
        for i in 0..len {
            result[i] = a[i] + b[i];
        }
        Ok(())
    }
    
    /// Performs vectorized multiplication of f64 arrays
    pub fn multiply_f64_arrays(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        if a.len() != b.len() || a.len() != result.len() {
            return Err("Array length mismatch".to_string());
        }
        
        let len = a.len();
        if len < self.config.simd_threshold {
            for i in 0..len {
                result[i] = a[i] * b[i];
            }
            return Ok(());
        }
        
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if self.config.enable_avx2 && is_x86_feature_detected!("avx2") {
                return self.multiply_f64_arrays_avx2(a, b, result);
            } else if self.config.enable_sse41 && is_x86_feature_detected!("sse2") {
                return self.multiply_f64_arrays_sse2(a, b, result);
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if self.config.enable_neon && std::arch::is_aarch64_feature_detected!("neon") {
                return unsafe { self.multiply_f64_arrays_neon(a, b, result) };
            }
        }
        
        // Fallback to scalar operations
        for i in 0..len {
            result[i] = a[i] * b[i];
        }
        Ok(())
    }
    
    /// Computes dot product of two f64 arrays using SIMD
    pub fn dot_product_f64(&self, a: &[f64], b: &[f64]) -> Result<f64, String> {
        if a.len() != b.len() {
            return Err("Array length mismatch".to_string());
        }
        
        let len = a.len();
        if len < self.config.simd_threshold {
            return Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum());
        }
        
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if self.config.enable_avx2 && is_x86_feature_detected!("avx2") {
                return self.dot_product_f64_avx2(a, b);
            } else if self.config.enable_sse41 && is_x86_feature_detected!("sse2") {
                return self.dot_product_f64_sse2(a, b);
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if self.config.enable_neon && std::arch::is_aarch64_feature_detected!("neon") {
                return unsafe { self.dot_product_f64_neon(a, b) };
            }
        }
        
        // Fallback to scalar operations
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }
    
    /// Performs vectorized addition of i64 arrays (where possible)
    pub fn add_i64_arrays(&self, a: &[i64], b: &[i64], result: &mut [i64]) -> Result<(), String> {
        if a.len() != b.len() || a.len() != result.len() {
            return Err("Array length mismatch".to_string());
        }
        
        let len = a.len();
        if len < self.config.simd_threshold {
            for i in 0..len {
                result[i] = a[i].saturating_add(b[i]);
            }
            return Ok(());
        }
        
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if self.config.enable_avx2 && is_x86_feature_detected!("avx2") {
                return self.add_i64_arrays_avx2(a, b, result);
            } else if self.config.enable_sse41 && is_x86_feature_detected!("sse2") {
                return self.add_i64_arrays_sse2(a, b, result);
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if self.config.enable_neon && std::arch::is_aarch64_feature_detected!("neon") {
                return unsafe { self.add_i64_arrays_neon(a, b, result) };
            }
        }
        
        // Fallback to scalar operations with overflow checking
        for i in 0..len {
            result[i] = a[i].saturating_add(b[i]);
        }
        Ok(())
    }
    
    // AVX2 implementations (256-bit vectors)
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn add_f64_arrays_avx2(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        let len = a.len();
        let simd_len = len - (len % 4); // Process 4 f64s at a time with AVX2
        
        for i in (0..simd_len).step_by(4) {
            let a_vec = _mm256_loadu_pd(a.as_ptr().add(i));
            let b_vec = _mm256_loadu_pd(b.as_ptr().add(i));
            let result_vec = _mm256_add_pd(a_vec, b_vec);
            _mm256_storeu_pd(result.as_mut_ptr().add(i), result_vec);
        }
        
        // Handle remaining elements
        for i in simd_len..len {
            result[i] = a[i] + b[i];
        }
        
        Ok(())
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn multiply_f64_arrays_avx2(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        let len = a.len();
        let simd_len = len - (len % 4);
        
        for i in (0..simd_len).step_by(4) {
            let a_vec = _mm256_loadu_pd(a.as_ptr().add(i));
            let b_vec = _mm256_loadu_pd(b.as_ptr().add(i));
            let result_vec = _mm256_mul_pd(a_vec, b_vec);
            _mm256_storeu_pd(result.as_mut_ptr().add(i), result_vec);
        }
        
        for i in simd_len..len {
            result[i] = a[i] * b[i];
        }
        
        Ok(())
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn dot_product_f64_avx2(&self, a: &[f64], b: &[f64]) -> Result<f64, String> {
        let len = a.len();
        let simd_len = len - (len % 4);
        let mut sum_vec = _mm256_setzero_pd();
        
        for i in (0..simd_len).step_by(4) {
            let a_vec = _mm256_loadu_pd(a.as_ptr().add(i));
            let b_vec = _mm256_loadu_pd(b.as_ptr().add(i));
            let mult_vec = _mm256_mul_pd(a_vec, b_vec);
            sum_vec = _mm256_add_pd(sum_vec, mult_vec);
        }
        
        // Horizontal sum of the 4 f64 values in sum_vec
        let sum_array: [f64; 4] = std::mem::transmute(sum_vec);
        let mut total = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements
        for i in simd_len..len {
            total += a[i] * b[i];
        }
        
        Ok(total)
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn add_i64_arrays_avx2(&self, a: &[i64], b: &[i64], result: &mut [i64]) -> Result<(), String> {
        let len = a.len();
        let simd_len = len - (len % 4); // Process 4 i64s at a time with AVX2
        
        for i in (0..simd_len).step_by(4) {
            let a_vec = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let b_vec = _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i);
            let result_vec = _mm256_add_epi64(a_vec, b_vec);
            _mm256_storeu_si256(result.as_mut_ptr().add(i) as *mut __m256i, result_vec);
        }
        
        // Handle remaining elements with overflow checking
        for i in simd_len..len {
            result[i] = a[i].saturating_add(b[i]);
        }
        
        Ok(())
    }
    
    // SSE2 implementations (128-bit vectors)
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    unsafe fn add_f64_arrays_sse2(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        let len = a.len();
        let simd_len = len - (len % 2); // Process 2 f64s at a time with SSE2
        
        for i in (0..simd_len).step_by(2) {
            let a_vec = _mm_loadu_pd(a.as_ptr().add(i));
            let b_vec = _mm_loadu_pd(b.as_ptr().add(i));
            let result_vec = _mm_add_pd(a_vec, b_vec);
            _mm_storeu_pd(result.as_mut_ptr().add(i), result_vec);
        }
        
        for i in simd_len..len {
            result[i] = a[i] + b[i];
        }
        
        Ok(())
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    unsafe fn multiply_f64_arrays_sse2(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        let len = a.len();
        let simd_len = len - (len % 2);
        
        for i in (0..simd_len).step_by(2) {
            let a_vec = _mm_loadu_pd(a.as_ptr().add(i));
            let b_vec = _mm_loadu_pd(b.as_ptr().add(i));
            let result_vec = _mm_mul_pd(a_vec, b_vec);
            _mm_storeu_pd(result.as_mut_ptr().add(i), result_vec);
        }
        
        for i in simd_len..len {
            result[i] = a[i] * b[i];
        }
        
        Ok(())
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    unsafe fn dot_product_f64_sse2(&self, a: &[f64], b: &[f64]) -> Result<f64, String> {
        let len = a.len();
        let simd_len = len - (len % 2);
        let mut sum_vec = _mm_setzero_pd();
        
        for i in (0..simd_len).step_by(2) {
            let a_vec = _mm_loadu_pd(a.as_ptr().add(i));
            let b_vec = _mm_loadu_pd(b.as_ptr().add(i));
            let mult_vec = _mm_mul_pd(a_vec, b_vec);
            sum_vec = _mm_add_pd(sum_vec, mult_vec);
        }
        
        // Horizontal sum of the 2 f64 values
        let sum_array: [f64; 2] = std::mem::transmute(sum_vec);
        let mut total = sum_array[0] + sum_array[1];
        
        for i in simd_len..len {
            total += a[i] * b[i];
        }
        
        Ok(total)
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse2")]
    unsafe fn add_i64_arrays_sse2(&self, a: &[i64], b: &[i64], result: &mut [i64]) -> Result<(), String> {
        let len = a.len();
        let simd_len = len - (len % 2); // Process 2 i64s at a time with SSE2
        
        for i in (0..simd_len).step_by(2) {
            let a_vec = _mm_loadu_si128(a.as_ptr().add(i) as *const __m128i);
            let b_vec = _mm_loadu_si128(b.as_ptr().add(i) as *const __m128i);
            let result_vec = _mm_add_epi64(a_vec, b_vec);
            _mm_storeu_si128(result.as_mut_ptr().add(i) as *mut __m128i, result_vec);
        }
        
        for i in simd_len..len {
            result[i] = a[i].saturating_add(b[i]);
        }
        
        Ok(())
    }
    
    /// Benchmark SIMD vs scalar performance for a given operation
    pub fn benchmark_simd_performance(&self, size: usize) -> SimdBenchmarkResults {
        let a: Vec<f64> = (0..size).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..size).map(|i| (i * 2) as f64).collect();
        let mut result_simd = vec![0.0; size];
        let mut result_scalar = vec![0.0; size];
        
        // Benchmark SIMD addition
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = self.add_f64_arrays(&a, &b, &mut result_simd);
        }
        let simd_duration = start.elapsed();
        
        // Benchmark scalar addition
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            for i in 0..size {
                result_scalar[i] = a[i] + b[i];
            }
        }
        let scalar_duration = start.elapsed();
        
        let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        
        SimdBenchmarkResults {
            array_size: size,
            simd_duration,
            scalar_duration,
            speedup,
            simd_enabled: self.config.enable_avx2 || self.config.enable_sse41 || self.config.enable_neon,
        }
    }
}

/// Results from SIMD performance benchmarking
#[derive(Debug, Clone)]
pub struct SimdBenchmarkResults {
    /// Size of arrays being processed
    pub array_size: usize,
    /// Time taken for SIMD operations
    pub simd_duration: std::time::Duration,
    /// Time taken for scalar operations
    pub scalar_duration: std::time::Duration,
    /// Performance speedup (scalar_time / simd_time)
    pub speedup: f64,
    /// Whether SIMD instructions were actually used
    pub simd_enabled: bool,
}

impl SimdBenchmarkResults {
    /// Formats the benchmark results as a string
    pub fn format_results(&self) -> String {
        format!(
            "SIMD Benchmark Results (array size: {})\n\
             SIMD time: {:?}\n\
             Scalar time: {:?}\n\
             Speedup: {:.2}x\n\
             SIMD enabled: {}",
            self.array_size,
            self.simd_duration,
            self.scalar_duration,
            self.speedup,
            self.simd_enabled
        )
    }
}

/// Optimized numeric operations using the SIMD backend
pub fn add_numeric_arrays_optimized(a: &[NumericValue], b: &[NumericValue]) -> Result<Vec<NumericValue>, String> {
    if a.len() != b.len() {
        return Err("Array length mismatch".to_string());
    }
    
    // Check if we can use SIMD (all floats)
    let all_floats = a.iter().zip(b.iter()).all(|(x, y)| {
        matches!(x, NumericValue::Real(_)) && matches!(y, NumericValue::Real(_))
    });
    
    if all_floats && a.len() >= 8 {
        let simd_ops = SimdNumericOps::default();
        let a_floats: Vec<f64> = a.iter().map(|x| if let NumericValue::Real(f) = x { *f } else { 0.0 }).collect();
        let b_floats: Vec<f64> = b.iter().map(|x| if let NumericValue::Real(f) = x { *f } else { 0.0 }).collect();
        let mut result_floats = vec![0.0; a.len()];
        
        simd_ops.add_f64_arrays(&a_floats, &b_floats, &mut result_floats)?;
        
        return Ok(result_floats.into_iter().map(NumericValue::real).collect());
    }
    
    // Fallback to regular arithmetic with tower promotion
    let mut result = Vec::with_capacity(a.len());
    for (x, y) in a.iter().zip(b.iter()) {
        result.push(tower::add(x, y));
    }
    
    Ok(result)
}

/// Optimized dot product using SIMD when possible
pub fn dot_product_optimized(a: &[NumericValue], b: &[NumericValue]) -> Result<NumericValue, String> {
    if a.len() != b.len() {
        return Err("Array length mismatch".to_string());
    }
    
    // Check if we can use SIMD
    let all_floats = a.iter().zip(b.iter()).all(|(x, y)| {
        matches!(x, NumericValue::Real(_)) && matches!(y, NumericValue::Real(_))
    });
    
    if all_floats && a.len() >= 8 {
        let simd_ops = SimdNumericOps::default();
        let a_floats: Vec<f64> = a.iter().map(|x| if let NumericValue::Real(f) = x { *f } else { 0.0 }).collect();
        let b_floats: Vec<f64> = b.iter().map(|x| if let NumericValue::Real(f) = x { *f } else { 0.0 }).collect();
        
        let result = simd_ops.dot_product_f64(&a_floats, &b_floats)?;
        return Ok(NumericValue::real(result));
    }
    
    // Fallback to regular arithmetic
    let mut sum = NumericValue::integer(0);
    for (x, y) in a.iter().zip(b.iter()) {
        let product = tower::multiply(x, y);
        sum = tower::add(&sum, &product);
    }
    
    Ok(sum)
}

// ARM64 NEON implementations
#[cfg(target_arch = "aarch64")]
impl SimdNumericOps {
    /// ARM64 NEON implementation for f64 array addition
    #[target_feature(enable = "neon")]
    unsafe fn add_f64_arrays_neon(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        // For now, fallback to scalar implementation
        // TODO: Implement proper NEON intrinsics
        for i in 0..a.len() {
            result[i] = a[i] + b[i];
        }
        Ok(())
    }
    
    /// ARM64 NEON implementation for f64 array multiplication
    #[target_feature(enable = "neon")]
    unsafe fn multiply_f64_arrays_neon(&self, a: &[f64], b: &[f64], result: &mut [f64]) -> Result<(), String> {
        // For now, fallback to scalar implementation
        // TODO: Implement proper NEON intrinsics
        for i in 0..a.len() {
            result[i] = a[i] * b[i];
        }
        Ok(())
    }
    
    /// ARM64 NEON implementation for f64 dot product
    #[target_feature(enable = "neon")]
    unsafe fn dot_product_f64_neon(&self, a: &[f64], b: &[f64]) -> Result<f64, String> {
        // For now, fallback to scalar implementation
        // TODO: Implement proper NEON intrinsics
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }
    
    /// ARM64 NEON implementation for i64 array addition
    #[target_feature(enable = "neon")]
    unsafe fn add_i64_arrays_neon(&self, a: &[i64], b: &[i64], result: &mut [i64]) -> Result<(), String> {
        // For now, fallback to scalar implementation
        // TODO: Implement proper NEON intrinsics
        for i in 0..a.len() {
            result[i] = a[i].saturating_add(b[i]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_addition() {
        let simd_ops = SimdNumericOps::default();
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let mut result = vec![0.0; 8];
        
        simd_ops.add_f64_arrays(&a, &b, &mut result).unwrap();
        
        for &val in &result {
            assert_eq!(val, 9.0);
        }
    }
    
    #[test]
    fn test_simd_dot_product() {
        let simd_ops = SimdNumericOps::default();
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        
        let result = simd_ops.dot_product_f64(&a, &b).unwrap();
        let expected = 1.0*5.0 + 2.0*6.0 + 3.0*7.0 + 4.0*8.0; // = 70.0
        
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_numeric_array_optimization() {
        let a = vec![
            NumericValue::real(1.0),
            NumericValue::real(2.0),
            NumericValue::real(3.0),
            NumericValue::real(4.0),
            NumericValue::real(5.0),
            NumericValue::real(6.0),
            NumericValue::real(7.0),
            NumericValue::real(8.0),
        ];
        let b = vec![
            NumericValue::real(8.0),
            NumericValue::real(7.0),
            NumericValue::real(6.0),
            NumericValue::real(5.0),
            NumericValue::real(4.0),
            NumericValue::real(3.0),
            NumericValue::real(2.0),
            NumericValue::real(1.0),
        ];
        
        let result = add_numeric_arrays_optimized(&a, &b).unwrap();
        
        for val in result {
            if let NumericValue::Real(f) = val {
                assert_eq!(f, 9.0);
            } else {
                panic!("Expected Real numeric value");
            }
        }
    }
    
    #[test]
    fn test_simd_benchmark() {
        let simd_ops = SimdNumericOps::default();
        let results = simd_ops.benchmark_simd_performance(1000);
        
        println!("{}", results.format_results());
        
        // SIMD should be faster or at least equal performance
        assert!(results.speedup >= 0.8); // Allow for some variance
    }
}