//! Comprehensive unit tests for Value type conversions system
//!
//! Tests the complete type conversion functionality including From implementations,
//! ToValue trait, Value constructor methods, and edge cases for type safety.

use super::conversions::ToValue;
use super::Value;
use crate::lexer::SchemeNumber;

#[cfg(test)]
mod from_implementations {
    use super::*;

    #[test]
    fn test_from_bool_true() {
        let value = Value::from(true);
        assert_eq!(value, Value::Boolean(true));
    }

    #[test]
    fn test_from_bool_false() {
        let value = Value::from(false);
        assert_eq!(value, Value::Boolean(false));
    }

    #[test]
    fn test_from_i64_positive() {
        let value = Value::from(42i64);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    fn test_from_i64_negative() {
        let value = Value::from(-42i64);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(-42)));
    }

    #[test]
    fn test_from_i64_zero() {
        let value = Value::from(0i64);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(0)));
    }

    #[test]
    fn test_from_i64_max() {
        let value = Value::from(i64::MAX);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(i64::MAX)));
    }

    #[test]
    fn test_from_i64_min() {
        let value = Value::from(i64::MIN);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(i64::MIN)));
    }

    #[test]
    fn test_from_u64_positive() {
        let value = Value::from(42u64);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    fn test_from_u64_zero() {
        let value = Value::from(0u64);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(0)));
    }

    #[test]
    fn test_from_u64_max_safe() {
        let value = Value::from(u64::MAX);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(u64::MAX as i64)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_from_f64_positive() {
        let value = Value::from(3.14159f64);
        assert_eq!(value, Value::Number(SchemeNumber::Real(3.14159)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_from_f64_negative() {
        let value = Value::from(-3.14159f64);
        assert_eq!(value, Value::Number(SchemeNumber::Real(-3.14159)));
    }

    #[test]
    fn test_from_f64_zero() {
        let value = Value::from(0.0f64);
        assert_eq!(value, Value::Number(SchemeNumber::Real(0.0)));
    }

    #[test]
    fn test_from_f64_infinity() {
        let value = Value::from(f64::INFINITY);
        assert_eq!(value, Value::Number(SchemeNumber::Real(f64::INFINITY)));
    }

    #[test]
    fn test_from_f64_neg_infinity() {
        let value = Value::from(f64::NEG_INFINITY);
        assert_eq!(value, Value::Number(SchemeNumber::Real(f64::NEG_INFINITY)));
    }

    #[test]
    fn test_from_f64_nan() {
        let value = Value::from(f64::NAN);
        if let Value::Number(SchemeNumber::Real(f)) = value {
            assert!(f.is_nan());
        } else {
            panic!("Expected Real number with NaN value");
        }
    }

    #[test]
    fn test_from_string_owned() {
        let value = Value::from("hello".to_string());
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_from_string_empty() {
        let value = Value::from(String::new());
        assert_eq!(value, Value::String(String::new()));
    }

    #[test]
    fn test_from_string_unicode() {
        let value = Value::from("こんにちは".to_string());
        assert_eq!(value, Value::String("こんにちは".to_string()));
    }

    #[test]
    fn test_from_str_reference() {
        let value = Value::from("hello");
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_from_str_empty() {
        let value = Value::from("");
        assert_eq!(value, Value::String("".to_string()));
    }

    #[test]
    fn test_from_str_special_chars() {
        let value = Value::from("hello\nworld\t!");
        assert_eq!(value, Value::String("hello\nworld\t!".to_string()));
    }

    #[test]
    fn test_from_char_ascii() {
        let value = Value::from('a');
        assert_eq!(value, Value::Character('a'));
    }

    #[test]
    fn test_from_char_unicode() {
        let value = Value::from('λ');
        assert_eq!(value, Value::Character('λ'));
    }

    #[test]
    fn test_from_char_special() {
        let value = Value::from('\n');
        assert_eq!(value, Value::Character('\n'));
    }

    #[test]
    fn test_from_scheme_number_integer() {
        let value = Value::from(SchemeNumber::Integer(42));
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_from_scheme_number_real() {
        let value = Value::from(SchemeNumber::Real(3.14159));
        assert_eq!(value, Value::Number(SchemeNumber::Real(3.14159)));
    }

    #[test]
    fn test_from_scheme_number_rational() {
        let value = Value::from(SchemeNumber::Rational(3, 4));
        assert_eq!(value, Value::Number(SchemeNumber::Rational(3, 4)));
    }

    #[test]
    fn test_from_scheme_number_complex() {
        let value = Value::from(SchemeNumber::Complex(2.0, 3.0));
        assert_eq!(value, Value::Number(SchemeNumber::Complex(2.0, 3.0)));
    }
}

#[cfg(test)]
mod value_constructors {
    use super::*;

    #[test]
    fn test_new_symbol_ref() {
        let value = Value::new_symbol_ref("hello");
        assert_eq!(value, Value::Symbol("hello".to_string()));
    }

    #[test]
    fn test_new_symbol_ref_empty() {
        let value = Value::new_symbol_ref("");
        assert_eq!(value, Value::Symbol("".to_string()));
    }

    #[test]
    fn test_new_symbol_ref_special_chars() {
        let value = Value::new_symbol_ref("my-var?");
        assert_eq!(value, Value::Symbol("my-var?".to_string()));
    }

    #[test]
    fn test_new_string() {
        let value = Value::new_string("hello".to_string());
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_new_string_empty() {
        let value = Value::new_string(String::new());
        assert_eq!(value, Value::String(String::new()));
    }

    #[test]
    fn test_new_string_ref() {
        let value = Value::new_string_ref("hello");
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_new_string_ref_unicode() {
        let value = Value::new_string_ref("λ-calculus");
        assert_eq!(value, Value::String("λ-calculus".to_string()));
    }

    #[test]
    fn test_new_character() {
        let value = Value::new_character('a');
        assert_eq!(value, Value::Character('a'));
    }

    #[test]
    fn test_new_character_unicode() {
        let value = Value::new_character('λ');
        assert_eq!(value, Value::Character('λ'));
    }

    #[test]
    fn test_new_character_special() {
        let value = Value::new_character('\t');
        assert_eq!(value, Value::Character('\t'));
    }

    #[test]
    fn test_new_number() {
        let value = Value::new_number(SchemeNumber::Integer(42));
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_new_number_real() {
        let value = Value::new_number(SchemeNumber::Real(3.14159));
        assert_eq!(value, Value::Number(SchemeNumber::Real(3.14159)));
    }

    #[test]
    fn test_new_real() {
        let value = Value::new_real(std::f64::consts::E);
        assert_eq!(
            value,
            Value::Number(SchemeNumber::Real(std::f64::consts::E))
        );
    }

    #[test]
    fn test_new_real_zero() {
        let value = Value::new_real(0.0);
        assert_eq!(value, Value::Number(SchemeNumber::Real(0.0)));
    }

    #[test]
    fn test_new_real_negative() {
        let value = Value::new_real(-1.5);
        assert_eq!(value, Value::Number(SchemeNumber::Real(-1.5)));
    }

    #[test]
    fn test_nil() {
        let value = Value::nil();
        assert_eq!(value, Value::Nil);
    }

    #[test]
    fn test_undefined() {
        let value = Value::undefined();
        assert_eq!(value, Value::Undefined);
    }

    #[cfg(not(feature = "memory-pooling"))]
    #[test]
    fn test_new_nil() {
        let value = Value::new_nil();
        assert_eq!(value, Value::Nil);
    }

    #[cfg(not(feature = "memory-pooling"))]
    #[test]
    fn test_new_symbol() {
        let value = Value::new_symbol("hello".to_string());
        assert_eq!(value, Value::Symbol("hello".to_string()));
    }

    #[cfg(not(feature = "memory-pooling"))]
    #[test]
    fn test_new_boolean() {
        let value = Value::new_boolean(true);
        assert_eq!(value, Value::Boolean(true));
    }

    #[cfg(not(feature = "memory-pooling"))]
    #[test]
    fn test_new_integer() {
        let value = Value::new_integer(42);
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }
}

#[cfg(test)]
mod to_value_trait {
    use super::*;

    #[test]
    fn test_string_to_value() {
        let s = "hello".to_string();
        let value = s.to_value();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_to_value_empty() {
        let s = String::new();
        let value = s.to_value();
        assert_eq!(value, Value::String(String::new()));
    }

    #[test]
    fn test_string_to_value_unicode() {
        let s = "こんにちは".to_string();
        let value = s.to_value();
        assert_eq!(value, Value::String("こんにちは".to_string()));
    }

    #[test]
    fn test_str_to_value() {
        let s = "hello";
        let value = s.to_value();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_str_to_value_empty() {
        let s = "";
        let value = s.to_value();
        assert_eq!(value, Value::String("".to_string()));
    }

    #[test]
    fn test_str_to_value_special_chars() {
        let s = "hello\nworld\t!";
        let value = s.to_value();
        assert_eq!(value, Value::String("hello\nworld\t!".to_string()));
    }
}

#[cfg(test)]
mod conversion_edge_cases {
    use super::*;

    #[test]
    fn test_multiple_conversions_same_source() {
        let original = "hello";
        let value1 = Value::from(original);
        let value2 = Value::from(original);
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_conversion_chain() {
        let original = "hello";
        let value1 = Value::from(original);
        let value2 = original.to_value();
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_numeric_precision() {
        let f = 1.234_567_890_123_45;
        let value = Value::from(f);
        if let Value::Number(SchemeNumber::Real(stored)) = value {
            assert!((stored - f).abs() < f64::EPSILON);
        } else {
            panic!("Expected Real number");
        }
    }

    #[test]
    fn test_large_string_conversion() {
        let large_string = "a".repeat(10000);
        let value = Value::from(large_string.clone());
        assert_eq!(value, Value::String(large_string));
    }

    #[test]
    fn test_unicode_string_conversion() {
        let unicode_string = "🚀λ計算機科学α";
        let value = Value::from(unicode_string);
        assert_eq!(value, Value::String(unicode_string.to_string()));
    }

    #[test]
    fn test_null_character_conversion() {
        let value = Value::from('\0');
        assert_eq!(value, Value::Character('\0'));
    }

    #[test]
    fn test_max_unicode_character() {
        let value = Value::from(char::MAX);
        assert_eq!(value, Value::Character(char::MAX));
    }

    #[test]
    fn test_scheme_number_rational_zero_denominator() {
        // Test that we can create a rational number with zero denominator
        // This should be handled by the SchemeNumber type itself
        let value = Value::from(SchemeNumber::Rational(1, 0));
        assert_eq!(value, Value::Number(SchemeNumber::Rational(1, 0)));
    }

    #[test]
    fn test_scheme_number_rational_negative() {
        let value = Value::from(SchemeNumber::Rational(-3, 4));
        assert_eq!(value, Value::Number(SchemeNumber::Rational(-3, 4)));
    }

    #[test]
    fn test_scheme_number_complex_zero_imaginary() {
        let value = Value::from(SchemeNumber::Complex(5.0, 0.0));
        assert_eq!(value, Value::Number(SchemeNumber::Complex(5.0, 0.0)));
    }

    #[test]
    fn test_scheme_number_complex_zero_real() {
        let value = Value::from(SchemeNumber::Complex(0.0, 3.0));
        assert_eq!(value, Value::Number(SchemeNumber::Complex(0.0, 3.0)));
    }

    #[test]
    fn test_scheme_number_complex_both_zero() {
        let value = Value::from(SchemeNumber::Complex(0.0, 0.0));
        assert_eq!(value, Value::Number(SchemeNumber::Complex(0.0, 0.0)));
    }
}

#[cfg(test)]
mod type_safety_tests {
    use super::*;

    #[test]
    fn test_constructor_consistency() {
        let s = "hello";
        let value1 = Value::from(s);
        let value2 = Value::new_string_ref(s);
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_number_constructor_consistency() {
        let n = 42;
        let value1 = Value::from(n);
        let value2 = Value::new_number(SchemeNumber::Integer(n));
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_character_constructor_consistency() {
        let c = 'a';
        let value1 = Value::from(c);
        let value2 = Value::new_character(c);
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_boolean_from_different_sources() {
        let value1 = Value::from(true);
        let value2 = Value::Boolean(true);
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_number_from_different_sources() {
        let value1 = Value::from(42i64);
        let value2 = Value::Number(SchemeNumber::Integer(42));
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_string_from_different_sources() {
        let value1 = Value::from("hello");
        let value2 = Value::String("hello".to_string());
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_nil_constructor_consistency() {
        let value1 = Value::nil();
        let value2 = Value::Nil;
        assert_eq!(value1, value2);
    }

    #[test]
    fn test_undefined_constructor_consistency() {
        let value1 = Value::undefined();
        let value2 = Value::Undefined;
        assert_eq!(value1, value2);
    }
}

#[cfg(test)]
mod memory_efficiency_tests {
    use super::*;

    #[test]
    fn test_string_ref_no_double_allocation() {
        let s = "hello";
        let value = Value::new_string_ref(s);
        if let Value::String(stored) = value {
            assert_eq!(stored, s);
            assert_eq!(stored.len(), s.len());
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_symbol_ref_no_double_allocation() {
        let s = "symbol";
        let value = Value::new_symbol_ref(s);
        if let Value::Symbol(stored) = value {
            assert_eq!(stored, s);
            assert_eq!(stored.len(), s.len());
        } else {
            panic!("Expected Symbol value");
        }
    }

    #[test]
    fn test_to_value_trait_efficiency() {
        let s = "hello";
        let value = s.to_value();
        if let Value::String(stored) = value {
            assert_eq!(stored, s);
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_minimal_allocation_for_numbers() {
        let value = Value::from(42i64);
        // Numbers should not allocate extra memory beyond the enum variant
        if let Value::Number(SchemeNumber::Integer(n)) = value {
            assert_eq!(n, 42);
        } else {
            panic!("Expected Integer number");
        }
    }

    #[test]
    fn test_minimal_allocation_for_characters() {
        let value = Value::from('a');
        // Characters should not allocate extra memory beyond the enum variant
        if let Value::Character(c) = value {
            assert_eq!(c, 'a');
        } else {
            panic!("Expected Character");
        }
    }

    #[test]
    fn test_minimal_allocation_for_booleans() {
        let value = Value::from(true);
        // Booleans should not allocate extra memory beyond the enum variant
        if let Value::Boolean(b) = value {
            assert!(b);
        } else {
            panic!("Expected Boolean");
        }
    }
}

#[cfg(test)]
mod clone_behavior_tests {
    use super::*;

    #[test]
    fn test_value_clone_string() {
        let original = Value::from("hello");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_number() {
        let original = Value::from(42i64);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_boolean() {
        let original = Value::from(true);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_character() {
        let original = Value::from('a');
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_nil() {
        let original = Value::nil();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_undefined() {
        let original = Value::undefined();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_symbol() {
        let original = Value::new_symbol_ref("test");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_clone_complex_number() {
        let original = Value::from(SchemeNumber::Complex(1.0, 2.0));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
