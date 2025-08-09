//! Performance optimizations for numeric operations
//!
//! Provides SIMD-accelerated operations, lookup tables, and specialized
//! algorithms for high-performance numeric computing.

use super::{Complex, Rational};
use std::sync::OnceLock;

/// Fast lookup tables for common mathematical functions
pub struct LookupTables {
    /// Sine table for angles in [0, π/2] with 1024 entries
    sin_table: Vec<f64>,
    /// Cosine table for angles in [0, π/2] with 1024 entries
    cos_table: Vec<f64>,
    /// Natural logarithm table for values in [1, 2] with 1024 entries
    ln_table: Vec<f64>,
    /// Exponential table for values in [-1, 1] with 1024 entries
    exp_table: Vec<f64>,
}

impl LookupTables {
    /// Creates new lookup tables
    fn new() -> Self {
        const TABLE_SIZE: usize = 1024;
        let mut sin_table = Vec::with_capacity(TABLE_SIZE);
        let mut cos_table = Vec::with_capacity(TABLE_SIZE);
        let mut ln_table = Vec::with_capacity(TABLE_SIZE);
        let mut exp_table = Vec::with_capacity(TABLE_SIZE);

        // Generate sin/cos tables for [0, π/2]
        for i in 0..TABLE_SIZE {
            let angle = (i as f64) * std::f64::consts::FRAC_PI_2 / (TABLE_SIZE - 1) as f64;
            sin_table.push(angle.sin());
            cos_table.push(angle.cos());
        }

        // Generate ln table for [1, 2]
        for i in 0..TABLE_SIZE {
            let x = 1.0 + (i as f64) / (TABLE_SIZE - 1) as f64;
            ln_table.push(x.ln());
        }

        // Generate exp table for [-1, 1]
        for i in 0..TABLE_SIZE {
            let x = -1.0 + 2.0 * (i as f64) / (TABLE_SIZE - 1) as f64;
            exp_table.push(x.exp());
        }

        Self {
            sin_table,
            cos_table,
            ln_table,
            exp_table,
        }
    }

    /// Get global lookup tables instance
    fn get() -> &'static Self {
        static TABLES: OnceLock<LookupTables> = OnceLock::new();
        TABLES.get_or_init(Self::new)
    }

    /// Fast sine approximation using table lookup with linear interpolation
    pub fn fast_sin(x: f64) -> f64 {
        let tables = Self::get();
        
        // Reduce to [0, 2π]
        let x = x.rem_euclid(2.0 * std::f64::consts::PI);
        
        // Use symmetry to reduce to [0, π/2]
        let (x, sign, table) = if x <= std::f64::consts::FRAC_PI_2 {
            (x, 1.0, &tables.sin_table)
        } else if x <= std::f64::consts::PI {
            (std::f64::consts::PI - x, 1.0, &tables.sin_table)
        } else if x <= 3.0 * std::f64::consts::FRAC_PI_2 {
            (x - std::f64::consts::PI, -1.0, &tables.sin_table)
        } else {
            (2.0 * std::f64::consts::PI - x, -1.0, &tables.sin_table)
        };

        // Table lookup with linear interpolation
        let scaled = x / std::f64::consts::FRAC_PI_2 * (table.len() - 1) as f64;
        let index = scaled as usize;
        let frac = scaled - index as f64;

        if index >= table.len() - 1 {
            sign * table[table.len() - 1]
        } else {
            let y0 = table[index];
            let y1 = table[index + 1];
            sign * (y0 + frac * (y1 - y0))
        }
    }

    /// Fast cosine approximation using table lookup
    pub fn fast_cos(x: f64) -> f64 {
        Self::fast_sin(x + std::f64::consts::FRAC_PI_2)
    }

    /// Fast natural logarithm approximation
    pub fn fast_ln(x: f64) -> f64 {
        if x <= 0.0 {
            return f64::NAN;
        }

        let tables = Self::get();
        
        // Use log properties: ln(x) = ln(2^k * m) = k*ln(2) + ln(m)
        // where m is in [1, 2]
        let k = x.log2().floor();
        let m = x / 2.0_f64.powf(k);

        // Table lookup for ln(m)
        let scaled = (m - 1.0) * (tables.ln_table.len() - 1) as f64;
        let index = scaled as usize;
        let frac = scaled - index as f64;

        let ln_m = if index >= tables.ln_table.len() - 1 {
            tables.ln_table[tables.ln_table.len() - 1]
        } else {
            let y0 = tables.ln_table[index];
            let y1 = tables.ln_table[index + 1];
            y0 + frac * (y1 - y0)
        };

        k * std::f64::consts::LN_2 + ln_m
    }

    /// Fast exponential approximation
    pub fn fast_exp(x: f64) -> f64 {
        if x.abs() > 700.0 {
            // Avoid overflow/underflow
            return if x > 0.0 { f64::INFINITY } else { 0.0 };
        }

        let tables = Self::get();
        
        // Use exp properties: exp(x) = exp(floor(x)) * exp(x - floor(x))
        let integer_part = x.floor();
        let fractional_part = x - integer_part;

        // For fractional part in [0, 1], we need to scale to [-1, 1] for our table
        let scaled_frac = 2.0 * fractional_part - 1.0;
        let scaled = (scaled_frac + 1.0) / 2.0 * (tables.exp_table.len() - 1) as f64;
        let index = scaled as usize;
        let frac = scaled - index as f64;

        let exp_frac = if index >= tables.exp_table.len() - 1 {
            tables.exp_table[tables.exp_table.len() - 1]
        } else {
            let y0 = tables.exp_table[index];
            let y1 = tables.exp_table[index + 1];
            y0 + frac * (y1 - y0)
        };

        // Adjust for our scaling
        let corrected_exp_frac = exp_frac * std::f64::consts::E.powf(fractional_part - scaled_frac);
        
        integer_part.exp() * corrected_exp_frac
    }
}

/// Optimized complex number operations
impl Complex {
    /// Fast complex multiplication optimized for real performance
    pub fn fast_mul(&self, other: &Self) -> Self {
        // Use optimized formula to reduce operations
        let a = self.real;
        let b = self.imaginary;
        let c = other.real;
        let d = other.imaginary;
        
        // Standard: (a + bi)(c + di) = (ac - bd) + (ad + bc)i
        // Optimized: use 3 multiplications instead of 4
        let k1 = a * (c + d);
        let k2 = d * (a + b);
        let k3 = c * (b - a);
        
        Self::new(k1 - k2, k1 + k3)
    }

    /// Fast complex division using optimized algorithm
    pub fn fast_div(&self, other: &Self) -> Self {
        if other.is_zero() {
            return Self::new(f64::INFINITY, f64::NAN);
        }

        // Use Smith's algorithm to avoid overflow
        let (c, d) = (other.real.abs(), other.imaginary.abs());
        
        if c >= d {
            let r = other.imaginary / other.real;
            let denominator = other.real + r * other.imaginary;
            Self::new(
                (self.real + r * self.imaginary) / denominator,
                (self.imaginary - r * self.real) / denominator,
            )
        } else {
            let r = other.real / other.imaginary;
            let denominator = other.imaginary + r * other.real;
            Self::new(
                (r * self.real + self.imaginary) / denominator,
                (r * self.imaginary - self.real) / denominator,
            )
        }
    }

    /// Fast magnitude calculation using bit tricks for small numbers
    pub fn fast_magnitude(&self) -> f64 {
        let a = self.real.abs();
        let b = self.imaginary.abs();
        
        if a == 0.0 {
            return b;
        }
        if b == 0.0 {
            return a;
        }
        
        // Use optimized hypot algorithm
        let max = a.max(b);
        let min = a.min(b);
        let r = min / max;
        
        max * (1.0 + r * r).sqrt()
    }

    /// Fast power using binary exponentiation for integer exponents
    pub fn fast_powi(&self, n: i32) -> Self {
        if n == 0 {
            return Self::ONE;
        }
        
        if n < 0 {
            return Self::ONE.fast_div(&self.fast_powi(-n));
        }
        
        let mut result = Self::ONE;
        let mut base = *self;
        let mut exp = n as u32;
        
        while exp > 0 {
            if exp & 1 == 1 {
                result = result.fast_mul(&base);
            }
            base = base.fast_mul(&base);
            exp >>= 1;
        }
        
        result
    }
}

/// Optimized rational number operations
impl Rational {
    /// Fast GCD using binary GCD algorithm (Stein's algorithm)
    pub fn binary_gcd(mut a: i64, mut b: i64) -> i64 {
        if a == 0 || b == 0 {
            return a | b;
        }
        
        // Count common trailing zeros
        let shift = (a | b).trailing_zeros();
        a >>= shift;
        b >>= shift;
        
        // Remove remaining factors of 2 from a
        a >>= a.trailing_zeros();
        
        loop {
            // Remove factors of 2 from b
            b >>= b.trailing_zeros();
            
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            
            b -= a;
            
            if b == 0 {
                break;
            }
        }
        
        a << shift
    }

    /// Fast creation with binary GCD
    pub fn fast_new(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Rational number cannot have zero denominator");
        }

        if numerator == 0 {
            return Self {
                numerator: 0,
                denominator: 1,
            };
        }

        let gcd = Self::binary_gcd(numerator.abs(), denominator.abs());
        let mut num = numerator / gcd;
        let mut den = denominator / gcd;

        if den < 0 {
            num = -num;
            den = -den;
        }

        Self {
            numerator: num,
            denominator: den,
        }
    }
}

/// SIMD-accelerated vector operations (when available)
#[cfg(target_arch = "x86_64")]
pub mod simd {
    use std::arch::x86_64::*;

    /// SIMD-accelerated addition of f64 arrays
    /// 
    /// # Safety
    /// 
    /// This function requires AVX2 instruction set support. The caller must ensure:
    /// - The target CPU supports AVX2 instructions
    /// - All input slices have the same length
    /// - The result slice has sufficient capacity for all elements
    #[target_feature(enable = "avx2")]
    pub unsafe fn simd_add_f64(a: &[f64], b: &[f64], result: &mut [f64]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());
        
        let len = a.len();
        let simd_len = len - (len % 4);
        
        // Process 4 elements at a time using AVX2
        for i in (0..simd_len).step_by(4) {
            let va = _mm256_loadu_pd(a.as_ptr().add(i));
            let vb = _mm256_loadu_pd(b.as_ptr().add(i));
            let vr = _mm256_add_pd(va, vb);
            _mm256_storeu_pd(result.as_mut_ptr().add(i), vr);
        }
        
        // Handle remaining elements
        for i in simd_len..len {
            result[i] = a[i] + b[i];
        }
    }

    /// SIMD-accelerated multiplication of f64 arrays
    /// 
    /// # Safety
    /// 
    /// This function requires AVX2 instruction set support. The caller must ensure:
    /// - The target CPU supports AVX2 instructions
    /// - All input slices have the same length
    /// - The result slice has sufficient capacity for all elements
    #[target_feature(enable = "avx2")]
    pub unsafe fn simd_mul_f64(a: &[f64], b: &[f64], result: &mut [f64]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());
        
        let len = a.len();
        let simd_len = len - (len % 4);
        
        for i in (0..simd_len).step_by(4) {
            let va = _mm256_loadu_pd(a.as_ptr().add(i));
            let vb = _mm256_loadu_pd(b.as_ptr().add(i));
            let vr = _mm256_mul_pd(va, vb);
            _mm256_storeu_pd(result.as_mut_ptr().add(i), vr);
        }
        
        for i in simd_len..len {
            result[i] = a[i] * b[i];
        }
    }

    /// SIMD-accelerated dot product
    /// 
    /// # Safety
    /// 
    /// This function requires AVX2 instruction set support. The caller must ensure:
    /// - The target CPU supports AVX2 instructions
    /// - Both input slices have the same length
    #[target_feature(enable = "avx2")]
    pub unsafe fn simd_dot_product_f64(a: &[f64], b: &[f64]) -> f64 {
        assert_eq!(a.len(), b.len());
        
        let len = a.len();
        let simd_len = len - (len % 4);
        let mut sum_vec = _mm256_setzero_pd();
        
        // Accumulate in SIMD register
        for i in (0..simd_len).step_by(4) {
            let va = _mm256_loadu_pd(a.as_ptr().add(i));
            let vb = _mm256_loadu_pd(b.as_ptr().add(i));
            let prod = _mm256_mul_pd(va, vb);
            sum_vec = _mm256_add_pd(sum_vec, prod);
        }
        
        // Horizontal sum of SIMD register
        let mut sums = [0.0; 4];
        _mm256_storeu_pd(sums.as_mut_ptr(), sum_vec);
        let mut result = sums[0] + sums[1] + sums[2] + sums[3];
        
        // Handle remaining elements
        for i in simd_len..len {
            result += a[i] * b[i];
        }
        
        result
    }
}

/// Benchmarking utilities
pub mod benchmark {
    use super::*;
    use std::time::{Duration, Instant};

    /// Benchmark result
    pub struct BenchmarkResult {
        /// The name of the benchmarked operation.
        pub operation: String,
        /// Number of iterations performed.
        pub iterations: usize,
        /// Total time taken for all iterations.
        pub total_time: Duration,
        /// Average time per iteration.
        pub avg_time: Duration,
        /// Operations performed per second.
        pub operations_per_second: f64,
    }

    impl BenchmarkResult {
        /// Creates a new benchmark result from basic timing data.
        pub fn new(operation: String, iterations: usize, total_time: Duration) -> Self {
            let avg_time = total_time / iterations as u32;
            let operations_per_second = iterations as f64 / total_time.as_secs_f64();
            
            Self {
                operation,
                iterations,
                total_time,
                avg_time,
                operations_per_second,
            }
        }
    }

    /// Benchmark a function
    pub fn benchmark<F, R>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> R,
    {
        let start = Instant::now();
        
        for _ in 0..iterations {
            std::hint::black_box(f());
        }
        
        let elapsed = start.elapsed();
        BenchmarkResult::new(name.to_string(), iterations, elapsed)
    }

    /// Benchmark complex arithmetic operations
    pub fn benchmark_complex_ops(iterations: usize) -> Vec<BenchmarkResult> {
        let c1 = Complex::new(3.0, 4.0);
        let c2 = Complex::new(1.0, 2.0);
        
        vec![
            benchmark("Complex Add", iterations, || c1 + c2),
            benchmark("Complex Mul", iterations, || c1 * c2),
            benchmark("Complex Fast Mul", iterations, || c1.fast_mul(&c2)),
            benchmark("Complex Div", iterations, || c1 / c2),
            benchmark("Complex Fast Div", iterations, || c1.fast_div(&c2)),
            benchmark("Complex Magnitude", iterations, || c1.magnitude()),
            benchmark("Complex Fast Magnitude", iterations, || c1.fast_magnitude()),
        ]
    }

    /// Benchmark trigonometric functions
    pub fn benchmark_trig_functions(iterations: usize) -> Vec<BenchmarkResult> {
        let x: f64 = 1.5;
        
        vec![
            benchmark("Sin (std)", iterations, || x.sin()),
            benchmark("Sin (fast)", iterations, || LookupTables::fast_sin(x)),
            benchmark("Cos (std)", iterations, || x.cos()),
            benchmark("Cos (fast)", iterations, || LookupTables::fast_cos(x)),
            benchmark("Exp (std)", iterations, || x.exp()),
            benchmark("Exp (fast)", iterations, || LookupTables::fast_exp(x)),
            benchmark("Ln (std)", iterations, || x.ln()),
            benchmark("Ln (fast)", iterations, || LookupTables::fast_ln(x)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::EPSILON;

    #[test]
    fn test_lookup_tables() {
        let x = 1.0;
        let fast_sin = LookupTables::fast_sin(x);
        let std_sin = x.sin();
        assert!((fast_sin - std_sin).abs() < 1e-3);

        let fast_cos = LookupTables::fast_cos(x);
        let std_cos = x.cos();
        assert!((fast_cos - std_cos).abs() < 1e-3);

        let x = 2.0;
        let fast_ln = LookupTables::fast_ln(x);
        let std_ln = x.ln();
        assert!((fast_ln - std_ln).abs() < 1e-3);

        let x = 0.5;
        let fast_exp = LookupTables::fast_exp(x);
        let std_exp = x.exp();
        assert!((fast_exp - std_exp).abs() < 1e-3);
    }

    #[test]
    fn test_optimized_complex() {
        let c1 = Complex::new(3.0, 4.0);
        let c2 = Complex::new(1.0, 2.0);

        let std_mul = c1 * c2;
        let fast_mul = c1.fast_mul(&c2);
        assert!((std_mul.real - fast_mul.real).abs() < EPSILON);
        assert!((std_mul.imaginary - fast_mul.imaginary).abs() < EPSILON);

        let std_div = c1 / c2;
        let fast_div = c1.fast_div(&c2);
        assert!((std_div.real - fast_div.real).abs() < 1e-10);
        assert!((std_div.imaginary - fast_div.imaginary).abs() < 1e-10);

        let std_mag = c1.magnitude();
        let fast_mag = c1.fast_magnitude();
        assert!((std_mag - fast_mag).abs() < EPSILON);
    }

    #[test]
    fn test_binary_gcd() {
        assert_eq!(Rational::binary_gcd(48, 18), 6);
        assert_eq!(Rational::binary_gcd(17, 13), 1);
        assert_eq!(Rational::binary_gcd(0, 5), 5);
        assert_eq!(Rational::binary_gcd(5, 0), 5);
    }

    #[test]
    fn test_fast_rational() {
        let r1 = Rational::fast_new(6, 8);
        let r2 = Rational::new(6, 8);
        assert_eq!(r1, r2);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_simd_operations() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = [2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut result = [0.0; 8];

        unsafe {
            simd::simd_add_f64(&a, &b, &mut result);
        }

        for i in 0..8 {
            assert_eq!(result[i], a[i] + b[i]);
        }

        unsafe {
            simd::simd_mul_f64(&a, &b, &mut result);
        }

        for i in 0..8 {
            assert_eq!(result[i], a[i] * b[i]);
        }

        let dot_product = unsafe { simd::simd_dot_product_f64(&a, &b) };
        let expected: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((dot_product - expected).abs() < EPSILON);
    }
}