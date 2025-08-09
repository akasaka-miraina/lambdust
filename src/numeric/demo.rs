//! Demonstration and example usage of the R7RS-large numeric library
//!
//! This module provides examples and demonstrations of the advanced
//! numeric capabilities implemented in Lambdust.

use super::*;
use crate::diagnostics::Result;

/// Comprehensive demonstration of numeric library capabilities
pub fn run_numeric_demo() -> Result<()> {
    println!("=== Lambdust R7RS-Large Numeric Library Demo ===\n");

    // Basic arithmetic with type promotion
    demo_basic_arithmetic()?;
    
    // Complex number operations
    demo_complex_numbers()?;
    
    // Rational number system
    demo_rational_numbers()?;
    
    // Big integer operations
    demo_big_integers()?;
    
    // Advanced mathematical functions
    demo_mathematical_functions()?;
    
    // Constants and utilities
    demo_constants()?;
    
    // Performance optimizations
    demo_performance_optimizations()?;

    println!("=== Demo Complete ===");
    Ok(())
}

fn demo_basic_arithmetic() -> Result<()> {
    println!("--- Basic Arithmetic with Type Promotion ---");
    
    // Integer arithmetic with overflow detection
    let large_int = NumericValue::integer(i64::MAX);
    let one = NumericValue::integer(1);
    let overflow_result = tower::add(&large_int, &one);
    println!("i64::MAX + 1 = {overflow_result} (promoted to BigInt)");
    
    // Mixed type arithmetic
    let int_val = NumericValue::integer(10);
    let rat_val = NumericValue::rational(1, 3);
    let sum = tower::add(&int_val, &rat_val);
    println!("10 + 1/3 = {sum} (promoted to rational)");
    
    // Division preserving exactness
    let result = tower::divide(&NumericValue::integer(10), &NumericValue::integer(3))
        .map_err(|e| crate::diagnostics::Error::runtime_error(e, None))?;
    println!("10 / 3 = {result} (exact rational)");
    
    println!();
    Ok(())
}

fn demo_complex_numbers() -> Result<()> {
    println!("--- Complex Number Operations ---");
    
    let c1 = Complex::new(3.0, 4.0);
    let c2 = Complex::new(1.0, 2.0);
    
    println!("c1 = {c1}");
    println!("c2 = {c2}");
    println!("c1 + c2 = {}", c1 + c2);
    println!("c1 * c2 = {}", c1 * c2);
    println!("c1 / c2 = {}", c1 / c2);
    println!("|c1| = {}", c1.magnitude());
    println!("arg(c1) = {} radians", c1.argument());
    
    // Complex exponential and logarithm
    println!("exp(c1) = {}", c1.exp());
    println!("ln(c1) = {}", c1.ln());
    println!("sqrt(c1) = {}", c1.sqrt());
    
    // Complex trigonometric functions
    println!("sin(c1) = {}", c1.sin());
    println!("cos(c1) = {}", c1.cos());
    
    // Power operations
    println!("c1^2 = {}", c1.powf(2.0));
    println!("c1^c2 = {}", c1.pow(c2));
    
    println!();
    Ok(())
}

fn demo_rational_numbers() -> Result<()> {
    println!("--- Rational Number System ---");
    
    let r1 = Rational::new(22, 7); // π approximation
    let r2 = Rational::new(355, 113); // Better π approximation
    
    println!("π ≈ {} = {:.10}", r1, r1.to_f64());
    println!("π ≈ {} = {:.10}", r2, r2.to_f64());
    println!("π = {:.10} (actual)", std::f64::consts::PI);
    
    // Rational arithmetic
    println!("{} + {} = {}", r1, r2, r1 + r2);
    println!("{} * {} = {}", r1, r2, r1 * r2);
    
    // Continued fraction representation
    let cf = r1.to_continued_fraction(10);
    println!("Continued fraction of {r1}: {cf:?}");
    let reconstructed = Rational::from_continued_fraction(&cf);
    println!("Reconstructed: {reconstructed}");
    
    // Powers and roots
    println!("{}^2 = {}", r1, r1.powi(2));
    println!("1/{} = {}", r1, r1.reciprocal());
    
    println!();
    Ok(())
}

fn demo_big_integers() -> Result<()> {
    println!("--- Big Integer Operations ---");
    
    // Large number arithmetic
    let big1 = BigInt::from_str_radix("123456789012345678901234567890", 10)
        .map_err(|e| crate::diagnostics::Error::runtime_error(&e, None))?;
    let big2 = BigInt::from_str_radix("987654321098765432109876543210", 10)
        .map_err(|e| crate::diagnostics::Error::runtime_error(&e, None))?;
    
    println!("big1 = {big1}");
    println!("big2 = {big2}");
    println!("big1 + big2 = {}", &big1 + &big2);
    println!("big1 * big2 = {}", &big1 * &big2);
    
    // Bit operations
    let big3 = BigInt::from_i64(12345);
    println!("{} << 10 = {}", big3, big3.clone() << 10);
    println!("{} >> 2 = {}", big3, big3.clone() >> 2);
    
    // GCD and LCM
    let a = BigInt::from_i64(48);
    let b = BigInt::from_i64(18);
    println!("gcd({}, {}) = {}", a, b, a.gcd(&b));
    println!("lcm({}, {}) = {}", a, b, a.lcm(&b));
    
    // Prime testing (basic)
    let candidate = BigInt::from_i64(97);
    println!("{} is prime: {}", candidate, candidate.is_prime(10));
    
    println!();
    Ok(())
}

fn demo_mathematical_functions() -> Result<()> {
    println!("--- Advanced Mathematical Functions ---");
    
    // Special functions
    println!("Γ(0.5) = {} (should be √π = {})", 
             functions::gamma(0.5), std::f64::consts::PI.sqrt());
    println!("Γ(4) = {} (should be 6)", functions::gamma(4.0));
    
    // Error function
    println!("erf(1) = {}", functions::erf(1.0));
    println!("erfc(1) = {}", functions::erfc(1.0));
    
    // Bessel functions
    println!("J₀(0) = {} (should be 1)", functions::bessel_j0(0.0));
    println!("J₁(0) = {} (should be 0)", functions::bessel_j1(0.0));
    
    // Statistical distributions
    println!("Φ(0) = {} (standard normal CDF at 0, should be 0.5)", 
             functions::normal_cdf(0.0));
    println!("φ(0) = {} (standard normal PDF at 0)", 
             functions::normal_pdf(0.0));
    
    // Numerical integration
    let integral = functions::integrate_simpson(|x| x * x, 0.0, 1.0, 1e-10);
    println!("∫₀¹ x² dx = {} (should be 1/3 = {})", integral, 1.0/3.0);
    
    // Numerical differentiation
    let derivative = functions::differentiate(|x| x * x, 2.0, 1e-6);
    println!("d/dx(x²)|ₓ₌₂ = {derivative} (should be 4)");
    
    // FFT demonstration
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let fft_result = functions::fft(&data);
    println!("FFT of [1,0,0,0]: {fft_result:?}");
    
    println!();
    Ok(())
}

fn demo_constants() -> Result<()> {
    println!("--- Mathematical and Physical Constants ---");
    
    // Mathematical constants
    println!("π = {:.15}", constants::MathConstants::PI);
    println!("e = {:.15}", constants::MathConstants::E);
    println!("φ (golden ratio) = {:.15}", constants::MathConstants::GOLDEN_RATIO);
    println!("γ (Euler-Mascheroni) = {:.15}", constants::MathConstants::EULER_GAMMA);
    
    // Physical constants
    println!("c (speed of light) = {} m/s", constants::PhysicalConstants::SPEED_OF_LIGHT);
    println!("h (Planck constant) = {} J⋅s", constants::PhysicalConstants::PLANCK);
    println!("k (Boltzmann constant) = {} J/K", constants::PhysicalConstants::BOLTZMANN);
    println!("G (gravitational constant) = {} m³/(kg⋅s²)", 
             constants::PhysicalConstants::GRAVITATIONAL);
    
    // Unit conversions
    println!("90° = {} radians", constants::UnitConversions::degrees_to_radians(90.0));
    println!("100°C = {} K", constants::UnitConversions::celsius_to_kelvin(100.0));
    
    // Constant lookup
    if let Some(pi_val) = constants::get_constant("pi") {
        println!("Retrieved π from constant lookup: {pi_val}");
    }
    
    println!();
    Ok(())
}

fn demo_performance_optimizations() -> Result<()> {
    println!("--- Performance Optimizations ---");
    
    // Lookup table demonstrations
    let x = 1.0f64;
    let std_sin = x.sin();
    let fast_sin = optimization::LookupTables::fast_sin(x);
    println!("sin({}) - std: {:.10}, fast: {:.10}, error: {:.2e}", 
             x, std_sin, fast_sin, (std_sin - fast_sin).abs());
    
    // Complex number optimizations
    let c1 = Complex::new(3.0, 4.0);
    let c2 = Complex::new(1.0, 2.0);
    let std_mul = c1 * c2;
    let fast_mul = c1.fast_mul(&c2);
    println!("Complex mul - std: {}, fast: {}, equal: {}", 
             std_mul, fast_mul, std_mul == fast_mul);
    
    // Benchmarking
    println!("\nRunning benchmarks...");
    let complex_benchmarks = optimization::benchmark::benchmark_complex_ops(100000);
    for bench in complex_benchmarks {
        println!("{}: {:.2} ops/sec", bench.operation, bench.operations_per_second);
    }
    
    let trig_benchmarks = optimization::benchmark::benchmark_trig_functions(100000);
    for bench in trig_benchmarks {
        println!("{}: {:.2} ops/sec", bench.operation, bench.operations_per_second);
    }
    
    println!();
    Ok(())
}

/// Interactive numeric REPL demo
pub fn numeric_repl_demo() -> Result<()> {
    println!("=== Numeric REPL Demo ===");
    println!("(This would be an interactive session in a real implementation)");
    
    // Simulate REPL interactions
    let repl_examples = vec![
        "(+ 1 2 3 4 5)",
        "(* 1/2 2/3 3/4)",
        "(sqrt -1)",
        "(exp (* 0+1i pi))",
        "(gamma 0.5)",
        "(constant \"pi\")",
        "(exact (/ 22 7))",
        "(inexact 1/3)",
    ];
    
    for expr in repl_examples {
        println!("> {expr}");
        // In a real implementation, this would parse and evaluate the expression
        match expr {
            "(+ 1 2 3 4 5)" => println!("15"),
            "(* 1/2 2/3 3/4)" => println!("1/4"),
            "(sqrt -1)" => println!("0+1i"),
            "(exp (* 0+1i pi))" => println!("-1+1.2246467991473532e-16i"),
            "(gamma 0.5)" => println!("1.7724538509055159"),
            "(constant \"pi\")" => println!("3.141592653589793"),
            "(exact (/ 22 7))" => println!("22/7"),
            "(inexact 1/3)" => println!("0.3333333333333333"),
            _ => println!("#<result>"),
        }
        println!();
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs() {
        // Test that the demo runs without panicking
        run_numeric_demo().unwrap();
        numeric_repl_demo().unwrap();
    }
    
    #[test]
    fn test_examples_work() {
        // Test some specific examples from the demo
        let large_int = NumericValue::integer(i64::MAX);
        let one = NumericValue::integer(1);
        let result = tower::add(&large_int, &one);
        assert!(matches!(result, NumericValue::BigInteger(_)));
        
        let c1 = Complex::new(3.0, 4.0);
        let c2 = Complex::new(1.0, 2.0);
        let product = c1 * c2;
        assert!((product.real - (-5.0)).abs() < 1e-10);
        assert!((product.imaginary - 10.0).abs() < 1e-10);
        
        let gamma_half = functions::gamma(0.5);
        let sqrt_pi = std::f64::consts::PI.sqrt();
        assert!((gamma_half - sqrt_pi).abs() < 1e-10);
    }
}