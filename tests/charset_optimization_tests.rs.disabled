//! Comprehensive tests for the optimized SRFI-14 character set implementation
//!
//! This test suite validates:
//! - Correctness of all optimized representations
//! - API compatibility with existing SRFI-14 implementation
//! - Performance characteristics of different optimization tiers
//! - Representation selection algorithms
//! - Thread safety and Arc wrapper integration

use lambdust::eval::value::Value;
use lambdust::stdlib::charset::{CHARSET_CONFIG, CharSet, CharSetConfig, StandardCharSets};
use std::collections::BTreeSet;
use std::sync::Arc;

/// Test that optimized and legacy implementations produce identical results
#[test]
fn test_optimization_correctness() {
    // Test various character sets with both optimized and legacy modes
    let test_cases = [
        "abcdefghijklmnopqrstuvwxyz", // ASCII letters
        "0123456789",                 // ASCII digits
        "αβγδεζηθικλμνξοπρστυφχψω",   // Greek letters
        "🎯🚀⚡🔥💎",                 // Unicode emojis
        "Hello, 世界! 🌍",            // Mixed content
        "",                           // Empty set
        "a",                          // Single character
        "ab",                         // Two characters
    ];

    for test_str in test_cases.iter() {
        // Create optimized version
        CHARSET_CONFIG.with(|config| {
            *config.borrow_mut() = CharSetConfig {
                enable_optimizations: true,
                force_representation: None,
            };
        });
        let optimized = CharSet::from_string(test_str);

        // Create legacy version
        CHARSET_CONFIG.with(|config| {
            *config.borrow_mut() = CharSetConfig {
                enable_optimizations: false,
                force_representation: None,
            };
        });
        let legacy = CharSet::from_string(test_str);

        // Test basic properties
        assert_eq!(
            optimized.size(),
            legacy.size(),
            "Size mismatch for: {}",
            test_str
        );
        assert_eq!(
            optimized.is_empty(),
            legacy.is_empty(),
            "Empty check mismatch for: {}",
            test_str
        );

        // Test membership for all characters
        for c in test_str.chars() {
            assert_eq!(
                optimized.contains(c),
                legacy.contains(c),
                "Membership mismatch for '{}' in: {}",
                c,
                test_str
            );
        }

        // Test that both contain exactly the same characters
        let opt_chars: BTreeSet<char> = optimized.to_vec().into_iter().collect();
        let leg_chars: BTreeSet<char> = legacy.to_vec().into_iter().collect();
        assert_eq!(
            opt_chars, leg_chars,
            "Character set mismatch for: {}",
            test_str
        );
    }

    // Reset to default
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig::default();
    });
}

/// Test ASCII-only optimization path
#[test]
fn test_ascii_optimization() {
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    // Create ASCII-only character sets
    let digits = CharSet::from_string("0123456789");
    let lower = CharSet::from_string("abcdefghijklmnopqrstuvwxyz");
    let upper = CharSet::from_string("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let symbols = CharSet::from_string("!@#$%^&*()");

    // Test that these use ASCII optimization (via display string)
    let digits_display = format!("{}", digits);
    assert!(
        digits_display.contains("type=ascii"),
        "Digits should use ASCII optimization: {}",
        digits_display
    );

    // Test performance-critical membership operations
    for i in 0..1000 {
        let c = char::from(b'0' + (i % 10) as u8);
        assert!(digits.contains(c));
    }

    for i in 0..1000 {
        let c = char::from(b'a' + (i % 26) as u8);
        assert!(lower.contains(c));
        assert!(!upper.contains(c)); // Should be very fast negative test
    }

    // Test set operations with ASCII sets
    let alphanumeric = lower.union(&digits);
    assert_eq!(alphanumeric.size(), 36);

    let letters_only = alphanumeric.difference(&digits);
    assert_eq!(letters_only.size(), 26);
    assert!(letters_only.is_equal(&lower));
}

/// Test small Unicode optimization
#[test]
fn test_small_unicode_optimization() {
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    // Create small mixed ASCII/Unicode sets
    let mixed_small = CharSet::from_string("aα5€");
    let greek_small = CharSet::from_string("αβγδε");

    // Test correctness
    assert_eq!(mixed_small.size(), 4);
    assert!(mixed_small.contains('a'));
    assert!(mixed_small.contains('α'));
    assert!(mixed_small.contains('5'));
    assert!(mixed_small.contains('€'));
    assert!(!mixed_small.contains('b'));

    // Test operations
    let combined = mixed_small.union(&greek_small);
    assert!(combined.contains('α'));
    assert!(combined.contains('β'));
    assert!(combined.contains('a'));
}

/// Test Unicode ranges optimization
#[test]
fn test_unicode_ranges_optimization() {
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    // Create character sets that should benefit from range compression
    let consecutive_ascii = CharSet::from_range('a', 'z');
    let consecutive_numbers = CharSet::from_range('0', '9');

    // Test basic properties
    assert_eq!(consecutive_ascii.size(), 26);
    assert_eq!(consecutive_numbers.size(), 10);

    // Test membership
    assert!(consecutive_ascii.contains('m'));
    assert!(!consecutive_ascii.contains('A'));
    assert!(consecutive_numbers.contains('5'));
    assert!(!consecutive_numbers.contains('a'));
}

/// Test large set optimization with bloom filter
#[test]
fn test_large_set_optimization() {
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    // Create a large character set
    let large_chars: String = (0..1000)
        .map(|i| char::from_u32(0x41 + (i % 26)).unwrap_or('A'))
        .collect();
    let large_set = CharSet::from_string(&large_chars);

    // Should use large set optimization for repeated characters
    assert!(!large_set.is_empty());

    // Test membership operations
    assert!(large_set.contains('A'));
    assert!(!large_set.contains('α')); // Should be fast negative via bloom filter
}

/// Test standard character sets optimization
#[test]
fn test_standard_charsets_optimization() {
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    // Test that standard character sets use optimizations
    let digits = StandardCharSets::digit();
    let letters = StandardCharSets::letter();
    let whitespace = StandardCharSets::whitespace();

    // Verify they use ASCII optimization where applicable
    let digits_display = format!("{}", digits);
    assert!(digits_display.contains("type=ascii") || digits_display.contains("optimized"));

    // Test correctness
    assert_eq!(digits.size(), 10);
    assert!(digits.contains('5'));
    assert!(!digits.contains('a'));

    assert_eq!(letters.size(), 52); // 26 lower + 26 upper
    assert!(letters.contains('a'));
    assert!(letters.contains('Z'));
    assert!(!letters.contains('5'));
}

/// Test set algebra operations correctness
#[test]
fn test_set_operations_correctness() {
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    let set_a = CharSet::from_string("abcdef");
    let set_b = CharSet::from_string("defghi");
    let set_c = CharSet::from_string("123");

    // Union
    let union_ab = set_a.union(&set_b);
    assert_eq!(union_ab.size(), 9); // abcdefghi
    assert!(union_ab.contains('a'));
    assert!(union_ab.contains('i'));

    // Intersection
    let intersect_ab = set_a.intersection(&set_b);
    assert_eq!(intersect_ab.size(), 3); // def
    assert!(intersect_ab.contains('d'));
    assert!(intersect_ab.contains('e'));
    assert!(intersect_ab.contains('f'));
    assert!(!intersect_ab.contains('a'));

    // Difference
    let diff_ab = set_a.difference(&set_b);
    assert_eq!(diff_ab.size(), 3); // abc
    assert!(diff_ab.contains('a'));
    assert!(diff_ab.contains('b'));
    assert!(diff_ab.contains('c'));
    assert!(!diff_ab.contains('d'));

    // Symmetric difference
    let symdiff_ab = set_a.symmetric_difference(&set_b);
    assert_eq!(symdiff_ab.size(), 6); // abc + ghi
    assert!(symdiff_ab.contains('a'));
    assert!(symdiff_ab.contains('g'));
    assert!(!symdiff_ab.contains('d'));

    // Empty intersection
    let empty_intersect = set_a.intersection(&set_c);
    assert!(empty_intersect.is_empty());
}

/// Test mixed representation operations
#[test]
fn test_mixed_representation_operations() {
    // Create sets with different optimizations enabled/disabled
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });
    let optimized_set = CharSet::from_string("abc123");

    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: false,
            force_representation: None,
        };
    });
    let legacy_set = CharSet::from_string("def456");

    // Test operations between mixed representations
    let union = optimized_set.union(&legacy_set);
    assert_eq!(union.size(), 9);

    let intersect = optimized_set.intersection(&legacy_set);
    assert!(intersect.is_empty());

    // Reset
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig::default();
    });
}

/// Test predicates and comparisons
#[test]
fn test_predicates_and_comparisons() {
    let set_a = CharSet::from_string("abc");
    let set_b = CharSet::from_string("abcdef");
    let set_c = CharSet::from_string("abc"); // Same as set_a
    let set_d = CharSet::from_string("xyz");

    // Equality
    assert!(set_a.is_equal(&set_c));
    assert!(!set_a.is_equal(&set_b));
    assert_eq!(set_a, set_c); // Test PartialEq implementation

    // Subset relationships
    assert!(set_a.is_subset(&set_b));
    assert!(!set_b.is_subset(&set_a));
    assert!(set_a.is_subset(&set_a)); // Self-subset
    assert!(!set_a.is_subset(&set_d)); // No overlap

    // Empty set is subset of everything
    let empty = CharSet::new();
    assert!(empty.is_subset(&set_a));
    assert!(empty.is_subset(&empty));
}

/// Test API compatibility with SRFI-14
#[test]
fn test_srfi14_api_compatibility() {
    // Test that all SRFI-14 operations work correctly
    let charset = CharSet::from_string("Hello");

    // Basic operations
    assert_eq!(charset.size(), 4); // H, e, l, o (l is deduplicated)
    assert!(charset.contains('H'));
    assert!(charset.contains('e'));
    assert!(charset.contains('l'));
    assert!(charset.contains('o'));
    assert!(!charset.contains('x'));

    // Iteration
    let chars: Vec<char> = charset.iter().collect();
    assert_eq!(chars.len(), 4);

    // Conversion
    let char_vec = charset.to_vec();
    assert_eq!(char_vec.len(), 4);

    // Predicates and other operations
    assert!(charset.every(|c| c.is_ascii()));
    assert!(charset.any(|c| c.is_uppercase()));
    assert_eq!(charset.count(|c| c.is_lowercase()), 3);
}

/// Test thread safety and Arc wrapper integration  
#[test]
fn test_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let charset = Arc::new(CharSet::from_string("abcdefghijklmnopqrstuvwxyz"));
    let mut handles = &[];

    // Spawn multiple threads that read from the character set
    for i in 0..10 {
        let charset_clone = Arc::clone(&charset);
        let handle = thread::spawn(move || {
            let test_char = char::from(b'a' + (i % 26) as u8);
            assert!(charset_clone.contains(test_char));
            charset_clone.size() // Read operation
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        let size = handle.join().unwrap();
        assert_eq!(size, 26);
    }
}

/// Test configuration and representation forcing
#[test]
fn test_configuration_system() {
    // Test enabling/disabling optimizations
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: false,
            force_representation: None,
        };
    });

    let legacy_set = CharSet::from_string("abc");
    let display_str = format!("{}", legacy_set);
    assert!(
        display_str.contains("type=legacy"),
        "Should use legacy representation: {}",
        display_str
    );

    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig {
            enable_optimizations: true,
            force_representation: None,
        };
    });

    let optimized_set = CharSet::from_string("abc");
    let opt_display = format!("{}", optimized_set);
    assert!(
        !opt_display.contains("type=legacy"),
        "Should use optimized representation: {}",
        opt_display
    );

    // Reset
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig::default();
    });
}

/// Performance regression test - ensures optimizations don't break functionality
#[test]
fn test_performance_regression() {
    // Test that optimized operations are at least as correct as legacy
    let test_strings = [
        "a",
        "abc",
        "0123456789",
        "abcdefghijklmnopqrstuvwxyz",
        "αβγδεζηθικλμνξοπρστυφχψω",
        "The quick brown fox jumps over the lazy dog! 123 αβγ 🚀🔥",
    ];

    for test_str in test_strings.iter() {
        // Create both versions
        CHARSET_CONFIG.with(|config| {
            *config.borrow_mut() = CharSetConfig {
                enable_optimizations: false,
                force_representation: None,
            };
        });
        let legacy = CharSet::from_string(test_str);

        CHARSET_CONFIG.with(|config| {
            *config.borrow_mut() = CharSetConfig {
                enable_optimizations: true,
                force_representation: None,
            };
        });
        let optimized = CharSet::from_string(test_str);

        // Test that they're functionally identical
        assert_eq!(optimized.size(), legacy.size());
        assert_eq!(optimized.is_empty(), legacy.is_empty());

        for c in test_str.chars() {
            assert_eq!(optimized.contains(c), legacy.contains(c));
        }

        // Test operations
        let other = CharSet::from_string("xyz123");
        assert_eq!(optimized.union(&other).size(), legacy.union(&other).size());
    }

    // Reset
    CHARSET_CONFIG.with(|config| {
        *config.borrow_mut() = CharSetConfig::default();
    });
}

/// Test that Value::CharSet integration still works
#[test]
fn test_value_integration() {
    let charset = CharSet::from_string("hello");
    let value = Value::CharSet(Arc::new(charset));

    // Test that we can extract the charset back
    match value {
        Value::CharSet(cs) => {
            assert!(cs.contains('h'));
            assert_eq!(cs.size(), 4); // h,e,l,o
        }
        _ => panic!("Expected CharSet value"),
    }
}
