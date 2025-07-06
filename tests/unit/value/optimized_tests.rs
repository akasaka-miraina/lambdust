//! Unit tests for optimized value system (Phase 4-Step3)
//!
//! Tests the memory-optimized value representation with SmallInt,
//! ShortString, and tag-based optimization for reduced heap usage.

use lambdust::lexer::SchemeNumber;
use lambdust::value::{OptimizedValue, OptimizationStats, ShortStringData, Value, ValueOptimizer};

#[test]
fn test_short_string_data_creation() {
    // Test creating short string within limit
    let short = ShortStringData::new("hello").unwrap();
    assert_eq!(short.as_str(), "hello");
    assert_eq!(short.len(), 5);
    assert!(!short.is_empty());

    // Test creating string at limit (15 bytes)
    let at_limit = ShortStringData::new("123456789012345").unwrap();
    assert_eq!(at_limit.len(), 15);
    assert_eq!(at_limit.as_str(), "123456789012345");

    // Test string too long
    let too_long = ShortStringData::new("1234567890123456"); // 16 bytes
    assert!(too_long.is_none());

    // Test empty string
    let empty = ShortStringData::new("").unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.as_str(), "");
}

#[test]
fn test_short_string_data_unicode() {
    // Test UTF-8 strings
    let unicode = ShortStringData::new("こんにちは").unwrap();
    assert_eq!(unicode.as_str(), "こんにちは");
    assert_eq!(unicode.len(), 15); // 5 characters × 3 bytes each

    // Test emoji (should not fit in 15 bytes)
    let emoji = ShortStringData::new("😀😀😀😀😀"); // 5 × 4 = 20 bytes
    assert!(emoji.is_none());

    // Test mixed ASCII and Unicode
    let mixed = ShortStringData::new("Hi!你好").unwrap();
    assert_eq!(mixed.as_str(), "Hi!你好");
}

#[test]
fn test_short_string_data_conversion() {
    let short = ShortStringData::new("test").unwrap();
    
    // Test to_string
    assert_eq!(short.to_string(), "test".to_string());
    
    // Test Display trait
    let displayed = format!("{}", short);
    assert_eq!(displayed, "test");
    
    // Test Debug trait
    let debug = format!("{:?}", short);
    assert_eq!(debug, "ShortString(\"test\")");
}

#[test]
fn test_optimized_value_new_int() {
    // Test small integer optimization
    let small_positive = OptimizedValue::new_int(100);
    assert!(matches!(small_positive, OptimizedValue::SmallInt(100)));
    
    let small_negative = OptimizedValue::new_int(-50);
    assert!(matches!(small_negative, OptimizedValue::SmallInt(-50)));
    
    // Test boundary values
    let at_min = OptimizedValue::new_int(-128);
    assert!(matches!(at_min, OptimizedValue::SmallInt(-128)));
    
    let at_max = OptimizedValue::new_int(127);
    assert!(matches!(at_max, OptimizedValue::SmallInt(127)));
    
    // Test large integers (should use Number)
    let large_positive = OptimizedValue::new_int(1000);
    assert!(matches!(large_positive, OptimizedValue::Number(SchemeNumber::Integer(1000))));
    
    let large_negative = OptimizedValue::new_int(-1000);
    assert!(matches!(large_negative, OptimizedValue::Number(SchemeNumber::Integer(-1000))));
}

#[test]
fn test_optimized_value_new_string() {
    // Test short string optimization
    let short = OptimizedValue::new_string("hello".to_string());
    assert!(matches!(short, OptimizedValue::ShortString(_)));
    
    if let OptimizedValue::ShortString(data) = short {
        assert_eq!(data.as_str(), "hello");
    }
    
    // Test long string (should use String)
    let long = OptimizedValue::new_string("this is a very long string that exceeds the 15 byte limit".to_string());
    assert!(matches!(long, OptimizedValue::String(_)));
    
    if let OptimizedValue::String(s) = long {
        assert_eq!(s, "this is a very long string that exceeds the 15 byte limit");
    }
    
    // Test string reference version
    let short_ref = OptimizedValue::new_string_ref("test");
    assert!(matches!(short_ref, OptimizedValue::ShortString(_)));
}

#[test]
fn test_optimized_value_new_symbol() {
    // Test short symbol optimization
    let short = OptimizedValue::new_symbol("var".to_string());
    assert!(matches!(short, OptimizedValue::ShortSymbol(_)));
    
    // Test long symbol
    let long = OptimizedValue::new_symbol("very-long-symbol-name-that-exceeds-limit".to_string());
    assert!(matches!(long, OptimizedValue::Symbol(_)));
    
    // Test symbol reference version
    let short_ref = OptimizedValue::new_symbol_ref("x");
    assert!(matches!(short_ref, OptimizedValue::ShortSymbol(_)));
}

#[test]
fn test_optimized_value_conversions() {
    // Test to_standard_value
    let small_int = OptimizedValue::SmallInt(42);
    let standard = small_int.to_standard_value();
    assert_eq!(standard, Value::Number(SchemeNumber::Integer(42)));
    
    let short_string = OptimizedValue::ShortString(ShortStringData::new("test").unwrap());
    let standard_string = short_string.to_standard_value();
    assert_eq!(standard_string, Value::String("test".to_string()));
    
    // Test from_standard_value
    let standard_int = Value::Number(SchemeNumber::Integer(50));
    let optimized = OptimizedValue::from_standard_value(standard_int);
    assert!(matches!(optimized, OptimizedValue::SmallInt(50)));
    
    let standard_long_int = Value::Number(SchemeNumber::Integer(1000));
    let optimized_long = OptimizedValue::from_standard_value(standard_long_int);
    assert!(matches!(optimized_long, OptimizedValue::Number(SchemeNumber::Integer(1000))));
}

#[test]
fn test_optimized_value_memory_size() {
    let small_int = OptimizedValue::SmallInt(42);
    assert_eq!(small_int.memory_size(), 1);
    
    let boolean = OptimizedValue::Boolean(true);
    assert_eq!(boolean.memory_size(), 1);
    
    let short_string = OptimizedValue::ShortString(ShortStringData::new("test").unwrap());
    assert_eq!(short_string.memory_size(), 16); // 15 bytes + 1 length
    
    let character = OptimizedValue::Character('c');
    assert_eq!(character.memory_size(), 4);
    
    let undefined = OptimizedValue::Undefined;
    assert_eq!(undefined.memory_size(), 0);
}

#[test]
fn test_optimized_value_predicates() {
    let small_int = OptimizedValue::SmallInt(42);
    assert!(small_int.is_number());
    assert!(!small_int.is_string());
    assert!(!small_int.is_symbol());
    assert!(small_int.is_inline());
    
    let large_int = OptimizedValue::Number(SchemeNumber::Integer(1000));
    assert!(large_int.is_number());
    assert!(!large_int.is_inline());
    
    let short_string = OptimizedValue::ShortString(ShortStringData::new("test").unwrap());
    assert!(short_string.is_string());
    assert!(!short_string.is_number());
    assert!(short_string.is_inline());
    
    let long_string = OptimizedValue::String("long string".to_string());
    assert!(short_string.is_string());
    assert!(!long_string.is_inline());
}

#[test]
fn test_optimized_value_accessors() {
    // Test integer accessors
    let small_int = OptimizedValue::SmallInt(42);
    assert_eq!(small_int.as_int(), Some(42));
    
    let large_int = OptimizedValue::Number(SchemeNumber::Integer(1000));
    assert_eq!(large_int.as_int(), Some(1000));
    
    let non_int = OptimizedValue::Boolean(true);
    assert_eq!(non_int.as_int(), None);
    
    // Test string accessors
    let short_string = OptimizedValue::ShortString(ShortStringData::new("hello").unwrap());
    assert_eq!(short_string.as_string(), Some("hello".to_string()));
    assert_eq!(short_string.as_string_ref(), Some("hello"));
    
    let long_string = OptimizedValue::String("world".to_string());
    assert_eq!(long_string.as_string(), Some("world".to_string()));
    assert_eq!(long_string.as_string_ref(), Some("world"));
    
    // Test symbol accessors
    let short_symbol = OptimizedValue::ShortSymbol(ShortStringData::new("var").unwrap());
    assert_eq!(short_symbol.as_symbol(), Some("var".to_string()));
    assert_eq!(short_symbol.as_symbol_ref(), Some("var"));
}

#[test]
fn test_optimized_value_equality() {
    // Test same variant equality
    let int1 = OptimizedValue::SmallInt(42);
    let int2 = OptimizedValue::SmallInt(42);
    let int3 = OptimizedValue::SmallInt(43);
    assert_eq!(int1, int2);
    assert_ne!(int1, int3);
    
    // Test cross-variant integer equality
    let small_int = OptimizedValue::SmallInt(42);
    let large_int = OptimizedValue::Number(SchemeNumber::Integer(42));
    assert_eq!(small_int, large_int);
    
    // Test string equality across variants
    let short_string = OptimizedValue::ShortString(ShortStringData::new("test").unwrap());
    let long_string = OptimizedValue::String("test".to_string());
    assert_eq!(short_string, long_string);
    
    // Test symbol equality across variants
    let short_symbol = OptimizedValue::ShortSymbol(ShortStringData::new("var").unwrap());
    let long_symbol = OptimizedValue::Symbol("var".to_string());
    assert_eq!(short_symbol, long_symbol);
}

#[test]
fn test_optimized_value_truthiness() {
    assert!(OptimizedValue::Boolean(true).is_truthy());
    assert!(!OptimizedValue::Boolean(false).is_truthy());
    
    // All non-false values should be truthy
    assert!(OptimizedValue::SmallInt(0).is_truthy());
    assert!(OptimizedValue::SmallInt(42).is_truthy());
    assert!(OptimizedValue::String("".to_string()).is_truthy());
    assert!(OptimizedValue::Undefined.is_truthy());
}

#[test]
fn test_optimized_value_type_name() {
    assert_eq!(OptimizedValue::Undefined.type_name(), "undefined");
    assert_eq!(OptimizedValue::Boolean(true).type_name(), "boolean");
    assert_eq!(OptimizedValue::SmallInt(42).type_name(), "number");
    assert_eq!(OptimizedValue::Number(SchemeNumber::Integer(1000)).type_name(), "number");
    assert_eq!(OptimizedValue::ShortString(ShortStringData::new("test").unwrap()).type_name(), "string");
    assert_eq!(OptimizedValue::String("test".to_string()).type_name(), "string");
    assert_eq!(OptimizedValue::Character('c').type_name(), "character");
    assert_eq!(OptimizedValue::ShortSymbol(ShortStringData::new("var").unwrap()).type_name(), "symbol");
    assert_eq!(OptimizedValue::Symbol("var".to_string()).type_name(), "symbol");
}

#[test]
fn test_optimization_stats() {
    let mut stats = OptimizationStats::new();
    
    // Test initial state
    assert_eq!(stats.optimization_ratio(), 0.0);
    assert_eq!(stats.memory_savings_ratio(), 0.0);
    
    // Test recording optimizations
    let original = Value::Number(SchemeNumber::Integer(42));
    let optimized = OptimizedValue::SmallInt(42);
    stats.record(&original, &optimized);
    
    assert_eq!(stats.small_ints, 1);
    assert_eq!(stats.total_values, 1);
    assert!(stats.optimization_ratio() > 0.0);
    
    // Test string optimization
    let original_string = Value::String("hello".to_string());
    let optimized_string = OptimizedValue::ShortString(ShortStringData::new("hello").unwrap());
    stats.record(&original_string, &optimized_string);
    
    assert_eq!(stats.short_strings, 1);
    assert_eq!(stats.total_values, 2);
    assert_eq!(stats.optimization_ratio(), 1.0); // Both values were optimized
}

#[test]
fn test_value_optimizer() {
    let mut optimizer = ValueOptimizer::new();
    
    // Test single value optimization
    let value = Value::Number(SchemeNumber::Integer(42));
    let optimized = optimizer.optimize(value);
    assert!(matches!(optimized, OptimizedValue::SmallInt(42)));
    
    // Test batch optimization
    let values = vec![
        Value::Number(SchemeNumber::Integer(10)),
        Value::String("test".to_string()),
        Value::Symbol("var".to_string()),
        Value::Number(SchemeNumber::Integer(1000)), // Large int
    ];
    
    let optimized_values = optimizer.optimize_batch(values);
    
    assert_eq!(optimized_values.len(), 4);
    assert!(matches!(optimized_values[0], OptimizedValue::SmallInt(10)));
    assert!(matches!(optimized_values[1], OptimizedValue::ShortString(_)));
    assert!(matches!(optimized_values[2], OptimizedValue::ShortSymbol(_)));
    assert!(matches!(optimized_values[3], OptimizedValue::Number(SchemeNumber::Integer(1000))));
    
    // Check stats
    let stats = optimizer.stats();
    assert_eq!(stats.total_values, 5); // 1 from single optimize + 4 from batch
    assert_eq!(stats.small_ints, 2); // 1 from single optimize + 1 from batch
    assert_eq!(stats.short_strings, 1);
    assert_eq!(stats.short_symbols, 1);
}

#[test]
fn test_value_optimizer_reset() {
    let mut optimizer = ValueOptimizer::new();
    
    // Optimize some values
    let value = Value::Number(SchemeNumber::Integer(42));
    let _ = optimizer.optimize(value);
    
    assert_eq!(optimizer.stats().total_values, 1);
    
    // Reset stats
    optimizer.reset_stats();
    assert_eq!(optimizer.stats().total_values, 0);
    assert_eq!(optimizer.stats().small_ints, 0);
}

#[test]
fn test_optimized_value_debug_display() {
    let small_int = OptimizedValue::SmallInt(42);
    let debug = format!("{:?}", small_int);
    assert_eq!(debug, "SmallInt(42)");
    
    let short_string = OptimizedValue::ShortString(ShortStringData::new("test").unwrap());
    let debug_string = format!("{:?}", short_string);
    assert_eq!(debug_string, "ShortString(\"test\")");
    
    let boolean = OptimizedValue::Boolean(true);
    let debug_bool = format!("{:?}", boolean);
    assert_eq!(debug_bool, "Boolean(true)");
}

#[test]
fn test_optimization_edge_cases() {
    // Test boundary conditions for SmallInt
    let min_small = OptimizedValue::new_int(i8::MIN as i64);
    assert!(matches!(min_small, OptimizedValue::SmallInt(_)));
    
    let max_small = OptimizedValue::new_int(i8::MAX as i64);
    assert!(matches!(max_small, OptimizedValue::SmallInt(_)));
    
    let just_over_max = OptimizedValue::new_int(i8::MAX as i64 + 1);
    assert!(matches!(just_over_max, OptimizedValue::Number(_)));
    
    let just_under_min = OptimizedValue::new_int(i8::MIN as i64 - 1);
    assert!(matches!(just_under_min, OptimizedValue::Number(_)));
    
    // Test boundary conditions for ShortString
    let max_short_string = "123456789012345"; // Exactly 15 bytes
    let short = OptimizedValue::new_string_ref(max_short_string);
    assert!(matches!(short, OptimizedValue::ShortString(_)));
    
    let too_long_string = "1234567890123456"; // 16 bytes
    let long = OptimizedValue::new_string_ref(too_long_string);
    assert!(matches!(long, OptimizedValue::String(_)));
}