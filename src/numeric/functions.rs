//! Advanced mathematical functions for R7RS-large compliance
//!
//! Implements special functions, statistical distributions, and numerical algorithms
//! required for comprehensive mathematical computing.

use super::Complex;
use std::f64::consts::{PI, TAU};


/// Gamma function implementation using Lanczos approximation
pub fn gamma(x: f64) -> f64 {
    // Lanczos coefficients for g=7, n=9
    const G: f64 = 7.0;
    const COEFFICIENTS: [f64; 9] = [
        0.9999999999998099,
        676.5203681218851,
        -1259.1392167224028,
        771.3234287776531,
        -176.6150291621406,
        12.507343278686905,
        -0.13857109526572012,
        9.984369578019572e-6,
        1.5056327351493116e-7,
    ];

    if x < 0.5 {
        // Use reflection formula: Γ(z)Γ(1-z) = π/sin(πz)
        return PI / ((PI * x).sin() * gamma(1.0 - x));
    }

    let z = x - 1.0;
    let mut sum = COEFFICIENTS[0];
    for (i, &coeff) in COEFFICIENTS.iter().enumerate().skip(1) {
        sum += coeff / (z + i as f64);
    }

    let t = z + G + 0.5;
    (TAU).sqrt() * t.powf(z + 0.5) * (-t).exp() * sum
}

/// Log gamma function (more numerically stable than log(gamma(x)))
pub fn log_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NAN;
    }

    // Use Stirling's approximation for large x
    if x > 12.0 {
        let inv_x = 1.0 / x;
        let inv_x2 = inv_x * inv_x;
        return (x - 0.5) * x.ln() - x + 0.5 * (TAU).ln() 
            + inv_x / 12.0 - inv_x2 * inv_x / 360.0;
    }

    // For smaller x, use the gamma function
    gamma(x).ln()
}

/// Beta function: B(x,y) = Γ(x)Γ(y)/Γ(x+y)
pub fn beta(x: f64, y: f64) -> f64 {
    (log_gamma(x) + log_gamma(y) - log_gamma(x + y)).exp()
}

/// Incomplete gamma function (lower)
pub fn gamma_incomplete_lower(a: f64, x: f64) -> f64 {
    if x < 0.0 || a <= 0.0 {
        return f64::NAN;
    }
    
    if x == 0.0 {
        return 0.0;
    }

    // Use series expansion for small x relative to a
    if x < a + 1.0 {
        let mut sum: f64 = 1.0;
        let mut term: f64 = 1.0;
        let mut n: f64 = 1.0;
        
        while term.abs() > 1e-15_f64 * sum.abs() && n < 1000.0_f64 {
            term *= x / (a + n - 1.0);
            sum += term;
            n += 1.0;
        }
        
        (a * x.ln() - x - log_gamma(a)).exp() * sum
    } else {
        // Use continued fraction for large x
        gamma(a) - gamma_incomplete_upper_cf(a, x)
    }
}

/// Incomplete gamma function (upper) using continued fraction
fn gamma_incomplete_upper_cf(a: f64, x: f64) -> f64 {
    let mut b = x + 1.0 - a;
    let mut c = 1e30;
    let mut d = 1.0 / b;
    let mut h = d;
    
    for i in 1..=1000 {
        let an = -i as f64 * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = b + an / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    
    (a * x.ln() - x - log_gamma(a)).exp() * h
}

/// Error function using Chebyshev approximation
pub fn erf(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    
    let abs_x = x.abs();
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    
    if abs_x > 5.0 {
        return sign;
    }
    
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    
    let t = 1.0 / (1.0 + p * abs_x);
    let y = 1.0 - ((a1 * t + a2) * t * t + (a3 * t + a4) * t + a5) * t * (-abs_x * abs_x).exp();
    
    sign * y
}

/// Complementary error function
pub fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

/// Bessel function of the first kind, order 0
pub fn bessel_j0(x: f64) -> f64 {
    let abs_x = x.abs();
    
    if abs_x < 8.0 {
        // Polynomial approximation for small x
        let y = x * x;
        let ans1 = 57568490574.0 + y * (-13362590354.0 + y * (651619640.7
            + y * (-11214424.18 + y * (77392.33017 + y * (-184.9052456)))));
        let ans2 = 57568490411.0 + y * (1029532985.0 + y * (9494680.718
            + y * (59272.64853 + y * (267.8532712 + y))));
        ans1 / ans2
    } else {
        // Asymptotic expansion for large x
        let z = 8.0 / abs_x;
        let y = z * z;
        let xx = abs_x - 0.785398164;
        let p0 = 1.0;
        let p1 = -0.1098628627e-2;
        let p2 = 0.2734510407e-4;
        let p3 = -0.2073370639e-5;
        let p4 = 0.2093887211e-6;
        let q0 = -0.1562499995e-1;
        let q1 = 0.1430488765e-3;
        let q2 = -0.6911147651e-5;
        let q3 = 0.7621095161e-6;
        let q4 = -0.934945152e-7;
        
        let p = p0 + y * (p1 + y * (p2 + y * (p3 + y * p4)));
        let q = z * (q0 + y * (q1 + y * (q2 + y * (q3 + y * q4))));
        
        (TAU / abs_x).sqrt() * (p * xx.cos() - q * xx.sin())
    }
}

/// Bessel function of the first kind, order 1
pub fn bessel_j1(x: f64) -> f64 {
    let abs_x = x.abs();
    
    if abs_x < 8.0 {
        let y = x * x;
        let ans1 = x * (72362614232.0 + y * (-7895059235.0 + y * (242396853.1
            + y * (-2972611.439 + y * (15704.48260 + y * (-30.16036606))))));
        let ans2 = 144725228442.0 + y * (2300535178.0 + y * (18583304.74
            + y * (99447.43394 + y * (376.9991397 + y))));
        ans1 / ans2
    } else {
        let z = 8.0 / abs_x;
        let y = z * z;
        let xx = abs_x - 2.356194491;
        let p0 = 1.0;
        let p1 = 0.183105e-2;
        let p2 = -0.3516396496e-4;
        let p3 = 0.2457520174e-5;
        let p4 = -0.240337019e-6;
        let q0 = 0.04687499995;
        let q1 = -0.2002690873e-3;
        let q2 = 0.8449199096e-5;
        let q3 = -0.88228987e-6;
        let q4 = 0.105787412e-6;
        
        let p = p0 + y * (p1 + y * (p2 + y * (p3 + y * p4)));
        let q = z * (q0 + y * (q1 + y * (q2 + y * (q3 + y * q4))));
        
        let result = (TAU / abs_x).sqrt() * (p * xx.cos() - q * xx.sin());
        if x < 0.0 { -result } else { result }
    }
}

/// Standard normal cumulative distribution function
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal probability density function
pub fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (TAU).sqrt()
}

/// Chi-squared cumulative distribution function
pub fn chi_squared_cdf(x: f64, k: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    gamma_incomplete_lower(k / 2.0, x / 2.0) / gamma(k / 2.0)
}

/// Student's t cumulative distribution function
pub fn student_t_cdf(t: f64, nu: f64) -> f64 {
    let x = nu / (t * t + nu);
    0.5 + 0.5 * t.signum() * (1.0 - incomplete_beta(x, nu / 2.0, 0.5))
}

/// Regularized incomplete beta function I_x(a,b)
pub fn incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    
    // Use continued fraction representation
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (log_gamma(a + b) - log_gamma(a) - log_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp()
    };
    
    if x < (a + 1.0) / (a + b + 2.0) {
        bt * beta_continued_fraction(x, a, b) / a
    } else {
        1.0 - bt * beta_continued_fraction(1.0 - x, b, a) / b
    }
}

/// Continued fraction for incomplete beta function
fn beta_continued_fraction(x: f64, a: f64, b: f64) -> f64 {
    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < 1e-30 {
        d = 1e-30;
    }
    d = 1.0 / d;
    let mut h = d;
    
    for m in 1..=1000 {
        let m_f = m as f64;
        let two_m = 2.0 * m_f;
        
        // Even iteration
        let aa = m_f * (b - m_f) * x / ((a + two_m - 1.0) * (a + two_m));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        h *= d * c;
        
        // Odd iteration
        let aa = -(a + m_f) * (a + b + m_f) * x / ((a + two_m) * (a + two_m + 1.0));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    
    h
}

/// Numerical integration using adaptive Simpson's rule
pub fn integrate_simpson<F>(f: F, a: f64, b: f64, tolerance: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    /// Parameters for Simpson's rule recursive computation.
    struct SimpsonParams<'a, F> {
        f: &'a F,
        a: f64,
        b: f64,
        tolerance: f64,
        s: f64,
        fa: f64,
        fb: f64,
        fc: f64,
        bottom: i32,
    }

    fn simpson_recursive<F>(params: SimpsonParams<'_, F>) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let c = (params.a + params.b) / 2.0;
        let h = params.b - params.a;
        let d = (params.a + c) / 2.0;
        let e = (c + params.b) / 2.0;
        let fd = (params.f)(d);
        let fe = (params.f)(e);
        let s_left = (h / 12.0) * (params.fa + 4.0 * fd + params.fc);
        let s_right = (h / 12.0) * (params.fc + 4.0 * fe + params.fb);
        let s2 = s_left + s_right;
        
        if params.bottom <= 0 || (s2 - params.s).abs() <= 15.0 * params.tolerance {
            s2 + (s2 - params.s) / 15.0
        } else {
            simpson_recursive(SimpsonParams {
                f: params.f,
                a: params.a,
                b: c,
                tolerance: params.tolerance / 2.0,
                s: s_left,
                fa: params.fa,
                fb: params.fc,
                fc: fd,
                bottom: params.bottom - 1,
            }) +
            simpson_recursive(SimpsonParams {
                f: params.f,
                a: c,
                b: params.b,
                tolerance: params.tolerance / 2.0,
                s: s_right,
                fa: params.fc,
                fb: params.fb,
                fc: fe,
                bottom: params.bottom - 1,
            })
        }
    }
    
    let c = (a + b) / 2.0;
    let h = b - a;
    let fa = f(a);
    let fb = f(b);
    let fc = f(c);
    let s = (h / 6.0) * (fa + 4.0 * fc + fb);
    
    simpson_recursive(SimpsonParams {
        f: &f,
        a,
        b,
        tolerance,
        s,
        fa,
        fb,
        fc,
        bottom: 50,
    })
}

/// Numerical differentiation using central difference
pub fn differentiate<F>(f: F, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    (f(x + h) - f(x - h)) / (2.0 * h)
}

/// Fast Fourier Transform (Cooley-Tukey algorithm)
pub fn fft(data: &[Complex]) -> Vec<Complex> {
    let n = data.len();
    if n <= 1 {
        return data.to_vec();
    }
    
    // Ensure n is a power of 2
    let mut padded_n = 1;
    while padded_n < n {
        padded_n <<= 1;
    }
    
    let mut padded_data: Vec<Complex> = data.to_vec();
    padded_data.resize(padded_n, Complex::ZERO);
    
    fft_recursive(&padded_data)
}

fn fft_recursive(data: &[Complex]) -> Vec<Complex> {
    let n = data.len();
    if n <= 1 {
        return data.to_vec();
    }
    
    // Divide
    let even: Vec<Complex> = data.iter().step_by(2).cloned().collect();
    let odd: Vec<Complex> = data.iter().skip(1).step_by(2).cloned().collect();
    
    // Conquer
    let fft_even = fft_recursive(&even);
    let fft_odd = fft_recursive(&odd);
    
    // Combine
    let mut result = vec![Complex::ZERO; n];
    for k in 0..n / 2 {
        let t = Complex::from_polar(1.0, -2.0 * PI * k as f64 / n as f64) * fft_odd[k];
        result[k] = fft_even[k] + t;
        result[k + n / 2] = fft_even[k] - t;
    }
    
    result
}

/// Inverse Fast Fourier Transform
pub fn ifft(data: &[Complex]) -> Vec<Complex> {
    let n = data.len();
    
    // Conjugate input
    let conjugated: Vec<Complex> = data.iter().map(|c| c.conjugate()).collect();
    
    // Compute FFT of conjugated data
    let fft_result = fft(&conjugated);
    
    // Conjugate output and scale
    fft_result.iter()
        .map(|c| c.conjugate() * (1.0 / n as f64))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::EPSILON;

    #[test]
    fn test_gamma_function() {
        // Test known values
        assert!((gamma(1.0) - 1.0).abs() < EPSILON);
        assert!((gamma(2.0) - 1.0).abs() < EPSILON);
        assert!((gamma(3.0) - 2.0).abs() < EPSILON);
        assert!((gamma(4.0) - 6.0).abs() < EPSILON);
        
        // Test half-integer: Γ(1/2) = √π
        assert!((gamma(0.5) - PI.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_beta_function() {
        // Test B(1,1) = 1
        assert!((beta(1.0, 1.0) - 1.0).abs() < EPSILON);
        
        // Test B(2,3) = 1/12
        assert!((beta(2.0, 3.0) - 1.0/12.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_function() {
        // Test erf(0) = 0
        assert!(erf(0.0).abs() < EPSILON);
        
        // Test erf is odd: erf(-x) = -erf(x)
        let x = 1.5;
        assert!((erf(-x) + erf(x)).abs() < 1e-10);
        
        // Test erf(∞) = 1
        assert!((erf(5.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_bessel_functions() {
        // Test J0(0) = 1
        assert!((bessel_j0(0.0) - 1.0).abs() < EPSILON);
        
        // Test J1(0) = 0
        assert!(bessel_j1(0.0).abs() < EPSILON);
    }

    #[test]
    fn test_normal_distribution() {
        // Test standard normal CDF at 0
        assert!((normal_cdf(0.0) - 0.5).abs() < EPSILON);
        
        // Test PDF normalization (approximate)
        let integral = integrate_simpson(normal_pdf, -5.0, 5.0, 1e-6);
        assert!((integral - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_integration() {
        // Test integration of x^2 from 0 to 1 (should be 1/3)
        let result = integrate_simpson(|x| x * x, 0.0, 1.0, 1e-10);
        assert!((result - 1.0/3.0).abs() < 1e-8);
    }

    #[test]
    fn test_differentiation() {
        // Test derivative of x^2 at x=2 (should be 4)
        let result = differentiate(|x| x * x, 2.0, 1e-6);
        assert!((result - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_fft() {
        // Test FFT of simple impulse
        let data = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ];
        
        let fft_result = fft(&data);
        let ifft_result = ifft(&fft_result);
        
        // Check that IFFT recovers original data
        for (original, recovered) in data.iter().zip(ifft_result.iter()) {
            assert!((original.real - recovered.real).abs() < 1e-10);
            assert!((original.imaginary - recovered.imaginary).abs() < 1e-10);
        }
    }
}