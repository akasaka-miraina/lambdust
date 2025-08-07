//! Mathematical and physical constants for scientific computing
//!
//! Provides high-precision constants commonly used in mathematical and
//! scientific applications, following R7RS-large requirements.

use super::NumericValue;
use std::f64::consts::{PI, E, TAU, SQRT_2, LN_2, LN_10, FRAC_1_PI, FRAC_2_PI, FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8};

/// Collection of mathematical constants
pub struct MathConstants;

impl MathConstants {
    // ============= FUNDAMENTAL CONSTANTS =============
    
    /// π (pi) - ratio of circle circumference to diameter
    pub const PI: f64 = PI;
    
    /// τ (tau) - 2π, full circle in radians
    pub const TAU: f64 = TAU;
    
    /// e - Euler's number, base of natural logarithm
    pub const E: f64 = E;
    
    /// φ (phi) - Golden ratio: (1 + √5)/2
    pub const GOLDEN_RATIO: f64 = 1.6180339887498949;
    
    /// γ (gamma) - Euler-Mascheroni constant
    pub const EULER_GAMMA: f64 = 0.5772156649015329;
    
    // ============= SQUARE ROOTS =============
    
    /// √2 - square root of 2
    pub const SQRT_2: f64 = SQRT_2;
    
    /// √3 - square root of 3
    pub const SQRT_3: f64 = 1.7320508075688772;
    
    /// √5 - square root of 5
    pub const SQRT_5: f64 = 2.23606797749979;
    
    /// √π - square root of π
    pub const SQRT_PI: f64 = 1.7724538509055159;
    
    /// √(2π) - square root of 2π
    pub const SQRT_2PI: f64 = 2.5066282746310005;
    
    /// 1/√2 - reciprocal of square root of 2
    pub const FRAC_1_SQRT_2: f64 = 0.7071067811865476;
    
    /// 1/√(2π) - reciprocal of square root of 2π
    pub const FRAC_1_SQRT_2PI: f64 = 0.3989422804014327;
    
    // ============= LOGARITHMS =============
    
    /// ln(2) - natural logarithm of 2
    pub const LN_2: f64 = LN_2;
    
    /// ln(3) - natural logarithm of 3
    pub const LN_3: f64 = 1.0986122886681097;
    
    /// ln(10) - natural logarithm of 10
    pub const LN_10: f64 = LN_10;
    
    /// ln(π) - natural logarithm of π
    pub const LN_PI: f64 = 1.1447298858494002;
    
    /// log₂(e) - logarithm base 2 of e
    pub const LOG2_E: f64 = 1.4426950408889634;
    
    /// log₁₀(e) - logarithm base 10 of e  
    pub const LOG10_E: f64 = 0.4342944819032518;
    
    /// log₂(10) - logarithm base 2 of 10
    pub const LOG2_10: f64 = 3.3219280948873626;
    
    /// log₁₀(2) - logarithm base 10 of 2
    pub const LOG10_2: f64 = 0.3010299956639812;
    
    // ============= PI FRACTIONS =============
    
    /// 1/π - reciprocal of π
    pub const FRAC_1_PI: f64 = FRAC_1_PI;
    
    /// 2/π - 2 divided by π
    pub const FRAC_2_PI: f64 = FRAC_2_PI;
    
    /// π/2 - π divided by 2
    pub const FRAC_PI_2: f64 = FRAC_PI_2;
    
    /// π/3 - π divided by 3
    pub const FRAC_PI_3: f64 = FRAC_PI_3;
    
    /// π/4 - π divided by 4
    pub const FRAC_PI_4: f64 = FRAC_PI_4;
    
    /// π/6 - π divided by 6
    pub const FRAC_PI_6: f64 = FRAC_PI_6;
    
    /// π/8 - π divided by 8
    pub const FRAC_PI_8: f64 = FRAC_PI_8;
    
    /// 3π/2 - 3π divided by 2
    pub const FRAC_3PI_2: f64 = 4.71238898038469;
    
    /// 2π/3 - 2π divided by 3
    pub const FRAC_2PI_3: f64 = 2.0943951023931955;
    
    /// 3π/4 - 3π divided by 4
    pub const FRAC_3PI_4: f64 = 2.356194490192345;
    
    /// 5π/6 - 5π divided by 6
    pub const FRAC_5PI_6: f64 = 2.6179938779914944;
    
    // ============= MATHEMATICAL SEQUENCES =============
    
    /// Catalan's constant - G = Σ((-1)ⁿ/(2n+1)²) from n=0 to ∞
    pub const CATALAN: f64 = 0.915965594177219;
    
    /// Apéry's constant - ζ(3) = Σ(1/n³) from n=1 to ∞
    pub const APERY: f64 = 1.2020569031595943;
    
    /// Khinchin's constant - geometric mean of continued fraction coefficients
    pub const KHINCHIN: f64 = 2.6854520010653064;
    
    /// Glaisher-Kinkelin constant - related to hyperfactorial function
    pub const GLAISHER_KINKELIN: f64 = 1.2824271291006226;
    
    // ============= CONVERSION CONSTANTS =============
    
    /// Degrees to radians conversion factor: π/180
    pub const DEG_TO_RAD: f64 = 0.017453292519943295;
    
    /// Radians to degrees conversion factor: 180/π
    pub const RAD_TO_DEG: f64 = 57.29577951308232;
}

/// Physical constants (CODATA 2018 values)
pub struct PhysicalConstants;

impl PhysicalConstants {
    // ============= FUNDAMENTAL PHYSICAL CONSTANTS =============
    
    /// Speed of light in vacuum (m/s)
    pub const SPEED_OF_LIGHT: f64 = 299792458.0;
    
    /// Planck constant (J⋅s)
    pub const PLANCK: f64 = 6.62607015e-34;
    
    /// Reduced Planck constant ℏ = h/(2π) (J⋅s)
    pub const PLANCK_REDUCED: f64 = 1.0545718176461565e-34;
    
    /// Elementary charge (C)
    pub const ELEMENTARY_CHARGE: f64 = 1.602176634e-19;
    
    /// Electron rest mass (kg)
    pub const ELECTRON_MASS: f64 = 9.1093837015e-31;
    
    /// Proton rest mass (kg)
    pub const PROTON_MASS: f64 = 1.67262192369e-27;
    
    /// Neutron rest mass (kg)
    pub const NEUTRON_MASS: f64 = 1.67492749804e-27;
    
    /// Atomic mass unit (kg)
    pub const ATOMIC_MASS_UNIT: f64 = 1.66053906660e-27;
    
    /// Avogadro constant (mol⁻¹)
    pub const AVOGADRO: f64 = 6.02214076e23;
    
    /// Boltzmann constant (J/K)
    pub const BOLTZMANN: f64 = 1.380649e-23;
    
    /// Gas constant (J/(mol⋅K))
    pub const GAS_CONSTANT: f64 = 8.314462618;
    
    /// Stefan-Boltzmann constant (W/(m²⋅K⁴))
    pub const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;
    
    /// Wien displacement law constant (m⋅K)
    pub const WIEN_DISPLACEMENT: f64 = 2.897771955e-3;
    
    /// Gravitational constant (m³/(kg⋅s²))
    pub const GRAVITATIONAL: f64 = 6.67430e-11;
    
    /// Standard gravity (m/s²)
    pub const STANDARD_GRAVITY: f64 = 9.80665;
    
    /// Standard atmosphere (Pa)
    pub const STANDARD_ATMOSPHERE: f64 = 101325.0;
    
    /// Vacuum permeability μ₀ (H/m)
    pub const VACUUM_PERMEABILITY: f64 = 1.25663706212e-6;
    
    /// Vacuum permittivity ε₀ (F/m)
    pub const VACUUM_PERMITTIVITY: f64 = 8.8541878128e-12;
    
    /// Characteristic impedance of vacuum (Ω)
    pub const IMPEDANCE_OF_VACUUM: f64 = 376.730313668;
    
    // ============= ELECTROMAGNETIC CONSTANTS =============
    
    /// Fine structure constant α
    pub const FINE_STRUCTURE: f64 = 7.2973525693e-3;
    
    /// Bohr radius (m)
    pub const BOHR_RADIUS: f64 = 5.29177210903e-11;
    
    /// Classical electron radius (m)
    pub const ELECTRON_RADIUS: f64 = 2.8179403262e-15;
    
    /// Thomson scattering cross section (m²)
    pub const THOMSON_CROSS_SECTION: f64 = 6.6524587321e-29;
    
    /// Bohr magneton (J/T)
    pub const BOHR_MAGNETON: f64 = 9.2740100783e-24;
    
    /// Nuclear magneton (J/T)
    pub const NUCLEAR_MAGNETON: f64 = 5.0507837461e-27;
    
    // ============= THERMODYNAMIC CONSTANTS =============
    
    /// Absolute zero temperature (°C)
    pub const ABSOLUTE_ZERO_CELSIUS: f64 = -273.15;
    
    /// Ice point temperature (K)
    pub const ICE_POINT: f64 = 273.15;
    
    /// Standard temperature (K)
    pub const STANDARD_TEMPERATURE: f64 = 273.15;
    
    /// Standard pressure (Pa)
    pub const STANDARD_PRESSURE: f64 = 100000.0;
}

/// Unit conversion utilities
pub struct UnitConversions;

impl UnitConversions {
    /// Convert degrees to radians
    pub fn degrees_to_radians(degrees: f64) -> f64 {
        degrees * MathConstants::DEG_TO_RAD
    }
    
    /// Convert radians to degrees
    pub fn radians_to_degrees(radians: f64) -> f64 {
        radians * MathConstants::RAD_TO_DEG
    }
    
    /// Convert Celsius to Kelvin
    pub fn celsius_to_kelvin(celsius: f64) -> f64 {
        celsius + PhysicalConstants::ICE_POINT
    }
    
    /// Convert Kelvin to Celsius
    pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
        kelvin - PhysicalConstants::ICE_POINT
    }
    
    /// Convert Fahrenheit to Celsius
    pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
        (fahrenheit - 32.0) * 5.0 / 9.0
    }
    
    /// Convert Celsius to Fahrenheit
    pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
        celsius * 9.0 / 5.0 + 32.0
    }
    
    /// Convert energy from Joules to electron volts
    pub fn joules_to_ev(joules: f64) -> f64 {
        joules / PhysicalConstants::ELEMENTARY_CHARGE
    }
    
    /// Convert energy from electron volts to Joules
    pub fn ev_to_joules(ev: f64) -> f64 {
        ev * PhysicalConstants::ELEMENTARY_CHARGE
    }
}

/// Provides constants as NumericValue for integration with the numeric system
pub fn get_constant(name: &str) -> Option<NumericValue> {
    match name {
        "pi" | "π" => Some(NumericValue::real(MathConstants::PI)),
        "tau" | "τ" => Some(NumericValue::real(MathConstants::TAU)),
        "e" => Some(NumericValue::real(MathConstants::E)),
        "golden-ratio" | "φ" => Some(NumericValue::real(MathConstants::GOLDEN_RATIO)),
        "euler-gamma" | "γ" => Some(NumericValue::real(MathConstants::EULER_GAMMA)),
        "sqrt-2" => Some(NumericValue::real(MathConstants::SQRT_2)),
        "sqrt-3" => Some(NumericValue::real(MathConstants::SQRT_3)),
        "sqrt-5" => Some(NumericValue::real(MathConstants::SQRT_5)),
        "sqrt-pi" => Some(NumericValue::real(MathConstants::SQRT_PI)),
        "ln-2" => Some(NumericValue::real(MathConstants::LN_2)),
        "ln-10" => Some(NumericValue::real(MathConstants::LN_10)),
        "catalan" => Some(NumericValue::real(MathConstants::CATALAN)),
        "apery" => Some(NumericValue::real(MathConstants::APERY)),
        "speed-of-light" | "c" => Some(NumericValue::real(PhysicalConstants::SPEED_OF_LIGHT)),
        "planck" | "h" => Some(NumericValue::real(PhysicalConstants::PLANCK)),
        "planck-reduced" | "ℏ" => Some(NumericValue::real(PhysicalConstants::PLANCK_REDUCED)),
        "elementary-charge" | "q" => Some(NumericValue::real(PhysicalConstants::ELEMENTARY_CHARGE)),
        "electron-mass" | "me" => Some(NumericValue::real(PhysicalConstants::ELECTRON_MASS)),
        "proton-mass" | "mp" => Some(NumericValue::real(PhysicalConstants::PROTON_MASS)),
        "avogadro" | "NA" => Some(NumericValue::real(PhysicalConstants::AVOGADRO)),
        "boltzmann" | "k" => Some(NumericValue::real(PhysicalConstants::BOLTZMANN)),
        "gas-constant" | "R" => Some(NumericValue::real(PhysicalConstants::GAS_CONSTANT)),
        "gravitational" | "G" => Some(NumericValue::real(PhysicalConstants::GRAVITATIONAL)),
        "fine-structure" | "α" => Some(NumericValue::real(PhysicalConstants::FINE_STRUCTURE)),
        _ => None,
    }
}

/// List all available constants
pub fn list_constants() -> Vec<&'static str> {
    vec![
        // Mathematical constants
        "pi", "τ", "e", "golden-ratio", "euler-gamma",
        "sqrt-2", "sqrt-3", "sqrt-5", "sqrt-pi",
        "ln-2", "ln-10", "catalan", "apery",
        
        // Physical constants
        "speed-of-light", "planck", "planck-reduced", 
        "elementary-charge", "electron-mass", "proton-mass",
        "avogadro", "boltzmann", "gas-constant", 
        "gravitational", "fine-structure",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::EPSILON;

    #[test]
    fn test_mathematical_constants() {
        // Test that constants are within expected ranges
        assert!(MathConstants::PI > 3.14 && MathConstants::PI < 3.15);
        assert!(MathConstants::E > 2.71 && MathConstants::E < 2.72);
        assert!(MathConstants::GOLDEN_RATIO > 1.61 && MathConstants::GOLDEN_RATIO < 1.62);
        
        // Test relationships
        assert!((MathConstants::TAU - 2.0 * MathConstants::PI).abs() < EPSILON);
        assert!((MathConstants::FRAC_1_SQRT_2 - 1.0 / MathConstants::SQRT_2).abs() < 1e-15);
    }

    #[test]
    fn test_physical_constants() {
        // Test that physical constants are reasonable
        assert!(PhysicalConstants::SPEED_OF_LIGHT > 2.9e8);
        assert!(PhysicalConstants::PLANCK > 6e-34 && PhysicalConstants::PLANCK < 7e-34);
        assert!(PhysicalConstants::AVOGADRO > 6e23 && PhysicalConstants::AVOGADRO < 7e23);
    }

    #[test]
    fn test_unit_conversions() {
        // Test angle conversions
        let right_angle_rad = UnitConversions::degrees_to_radians(90.0);
        assert!((right_angle_rad - MathConstants::FRAC_PI_2).abs() < 1e-10);
        
        let right_angle_deg = UnitConversions::radians_to_degrees(MathConstants::FRAC_PI_2);
        assert!((right_angle_deg - 90.0).abs() < 1e-10);
        
        // Test temperature conversions
        let boiling_k = UnitConversions::celsius_to_kelvin(100.0);
        assert!((boiling_k - 373.15).abs() < EPSILON);
        
        let freezing_c = UnitConversions::kelvin_to_celsius(273.15);
        assert!((freezing_c - 0.0).abs() < EPSILON);
        
        let body_temp_f = UnitConversions::celsius_to_fahrenheit(37.0);
        assert!((body_temp_f - 98.6).abs() < 0.1);
    }

    #[test]
    fn test_constant_lookup() {
        let pi_val = get_constant("pi").unwrap();
        assert!((pi_val.to_f64().unwrap() - MathConstants::PI).abs() < EPSILON);
        
        let c_val = get_constant("speed-of-light").unwrap();
        assert!((c_val.to_f64().unwrap() - PhysicalConstants::SPEED_OF_LIGHT).abs() < EPSILON);
        
        assert!(get_constant("nonexistent").is_none());
    }

    #[test]
    fn test_constant_precision() {
        // Verify high precision of mathematical constants
        let pi_str = format!("{:.50}", MathConstants::PI);
        assert!(pi_str.starts_with("3.14159265358979323846264338327950288419716939937510"));
        
        let e_str = format!("{:.50}", MathConstants::E);
        assert!(e_str.starts_with("2.71828182845904523536028747135266249775724709369995"));
    }
}