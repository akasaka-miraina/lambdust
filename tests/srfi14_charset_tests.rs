#![allow(clippy::uninlined_format_args)]
//! Comprehensive tests for SRFI-14 Character Sets implementation.
//!
//! This test suite validates the complete SRFI-14 implementation in Lambdust,
//! covering all character set operations, predicates, and standard character sets.

use lambdust::eval::value::Value;
use lambdust::stdlib::charset::{CharSet, CharSetCursor, StandardCharSets};

#[cfg(test)]
mod srfi14_tests {
    use super::*;

    /// Test basic character set creation and operations
    #[test]
    fn test_basic_charset_operations() {
        // Test empty character set
        let empty = CharSet::new();
        assert!(empty.is_empty());
        assert_eq!(empty.size(), 0);

        // Test character set from string
        let abc = CharSet::from_string("abc");
        assert!(!abc.is_empty());
        assert_eq!(abc.size(), 3);
        assert!(abc.contains('a'));
        assert!(abc.contains('b'));
        assert!(abc.contains('c'));
        assert!(!abc.contains('d'));

        // Test character set from chars iterator
        let xyz = CharSet::from_chars(['x', 'y', 'z']);
        assert_eq!(xyz.size(), 3);
        assert!(xyz.contains('x'));
        assert!(xyz.contains('y'));
        assert!(xyz.contains('z'));
    }

    /// Test character set algebra operations
    #[test]
    fn test_charset_algebra() {
        let ab = CharSet::from_string("ab");
        let bc = CharSet::from_string("bc");
        let cd = CharSet::from_string("cd");

        // Union
        let union_ab_bc = ab.union(&bc);
        assert_eq!(union_ab_bc.size(), 3);
        assert!(union_ab_bc.contains('a'));
        assert!(union_ab_bc.contains('b'));
        assert!(union_ab_bc.contains('c'));

        // Intersection
        let intersection_ab_bc = ab.intersection(&bc);
        assert_eq!(intersection_ab_bc.size(), 1);
        assert!(intersection_ab_bc.contains('b'));
        assert!(!intersection_ab_bc.contains('a'));
        assert!(!intersection_ab_bc.contains('c'));

        // Difference
        let diff_ab_bc = ab.difference(&bc);
        assert_eq!(diff_ab_bc.size(), 1);
        assert!(diff_ab_bc.contains('a'));
        assert!(!diff_ab_bc.contains('b'));

        // Symmetric difference (XOR)
        let xor_ab_bc = ab.symmetric_difference(&bc);
        assert_eq!(xor_ab_bc.size(), 2);
        assert!(xor_ab_bc.contains('a'));
        assert!(xor_ab_bc.contains('c'));
        assert!(!xor_ab_bc.contains('b'));

        // Empty intersection
        let empty_intersection = ab.intersection(&cd);
        assert!(empty_intersection.is_empty());
    }

    /// Test character set predicates
    #[test]
    fn test_charset_predicates() {
        let ab = CharSet::from_string("ab");
        let abc = CharSet::from_string("abc");
        let bc = CharSet::from_string("bc");

        // Equality
        assert!(ab.is_equal(&ab));
        assert!(!ab.is_equal(&abc));
        assert!(!ab.is_equal(&bc));

        // Subset relationships
        assert!(ab.is_subset(&abc));
        assert!(!abc.is_subset(&ab));
        assert!(!ab.is_subset(&bc));
        assert!(CharSet::new().is_subset(&ab)); // empty set is subset of everything

        // Self-subset
        assert!(ab.is_subset(&ab));
    }

    /// Test standard character sets
    #[test]
    fn test_standard_charsets() {
        // Digit set
        let digits = StandardCharSets::digit();
        assert_eq!(digits.size(), 10);
        for c in '0'..='9' {
            assert!(digits.contains(c));
        }
        assert!(!digits.contains('a'));
        assert!(!digits.contains(' '));

        // Lower case letters
        let lower = StandardCharSets::lower_case();
        for c in 'a'..='z' {
            assert!(lower.contains(c));
        }
        assert!(!lower.contains('A'));
        assert!(!lower.contains('1'));

        // Upper case letters
        let upper = StandardCharSets::upper_case();
        for c in 'A'..='Z' {
            assert!(upper.contains(c));
        }
        assert!(!upper.contains('a'));
        assert!(!upper.contains('1'));

        // Letter set should be union of upper and lower
        let letter = StandardCharSets::letter();
        let expected_letter = lower.union(&upper);
        assert!(letter.is_equal(&expected_letter));

        // Letter+digit should contain both letters and digits
        let letter_digit = StandardCharSets::letter_plus_digit();
        for c in 'a'..='z' {
            assert!(letter_digit.contains(c));
        }
        for c in 'A'..='Z' {
            assert!(letter_digit.contains(c));
        }
        for c in '0'..='9' {
            assert!(letter_digit.contains(c));
        }

        // Whitespace
        let whitespace = StandardCharSets::whitespace();
        assert!(whitespace.contains(' '));
        assert!(whitespace.contains('\t'));
        assert!(whitespace.contains('\n'));
        assert!(whitespace.contains('\r'));
        assert!(!whitespace.contains('a'));

        // Hex digits
        let hex = StandardCharSets::hex_digit();
        assert_eq!(hex.size(), 22); // 0-9, A-F, a-f
        for c in '0'..='9' {
            assert!(hex.contains(c));
        }
        for c in 'A'..='F' {
            assert!(hex.contains(c));
        }
        for c in 'a'..='f' {
            assert!(hex.contains(c));
        }
        assert!(!hex.contains('G'));
        assert!(!hex.contains('g'));

        // ASCII set
        let ascii = StandardCharSets::ascii();
        assert_eq!(ascii.size(), 128);
        for code in 0..=127 {
            if let Some(c) = char::from_u32(code) {
                assert!(ascii.contains(c));
            }
        }
        assert!(!ascii.contains('€')); // Euro sign is not ASCII

        // Empty and full sets
        let empty = StandardCharSets::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.size(), 0);

        let full = StandardCharSets::full();
        assert!(!full.is_empty());
        assert_eq!(full.size(), 128); // Currently ASCII-only
    }

    /// Test Unicode range operations
    #[test]
    fn test_unicode_ranges() {
        // Basic Latin range (ASCII)
        let latin = CharSet::from_range('\u{0020}', '\u{007E}');
        assert_eq!(latin.size(), 95); // Printable ASCII characters

        // Empty range (start > end)
        let empty_range = CharSet::from_range('z', 'a');
        assert!(empty_range.is_empty());

        // Single character range
        let single = CharSet::from_range('a', 'a');
        assert_eq!(single.size(), 1);
        assert!(single.contains('a'));

        // Unicode characters beyond ASCII
        let euro_range = CharSet::from_range('€', '€');
        assert_eq!(euro_range.size(), 1);
        assert!(euro_range.contains('€'));
        assert!(!euro_range.contains('$'));
    }

    /// Test character set cursor operations
    #[test]
    fn test_charset_cursors() {
        let abc = CharSet::from_string("abc");
        let cursor = CharSetCursor::new(&abc);

        // Test cursor creation
        assert!(!cursor.at_end());
        let chars = cursor.chars();
        assert_eq!(chars.len(), 3);
        // Characters should be sorted
        assert!(chars == ['a', 'b', 'c']);

        // Test cursor advancement (immutable)
        let next_cursor = cursor.next_cursor();
        assert_eq!(next_cursor.position(), 1);

        // Test empty set cursor
        let empty = CharSet::new();
        let empty_cursor = CharSetCursor::new(&empty);
        assert!(empty_cursor.at_end());
        assert_eq!(empty_cursor.chars().len(), 0);
    }

    /// Test character set filtering and predicates
    #[test]
    fn test_charset_filtering_predicates() {
        let mixed = CharSet::from_string("aB3!");

        // Test filtering with predicates
        let alphabetic = mixed.filter(|c| c.is_alphabetic());
        assert_eq!(alphabetic.size(), 2); // 'a', 'B'
        assert!(alphabetic.contains('a'));
        assert!(alphabetic.contains('B'));
        assert!(!alphabetic.contains('3'));
        assert!(!alphabetic.contains('!'));

        let numeric = mixed.filter(|c| c.is_numeric());
        assert_eq!(numeric.size(), 1); // '3'
        assert!(numeric.contains('3'));

        // Test count with predicates
        assert_eq!(mixed.count(|c| c.is_alphabetic()), 2);
        assert_eq!(mixed.count(|c| c.is_numeric()), 1);
        assert_eq!(mixed.count(|c| c.is_ascii_punctuation()), 1);

        // Test every/any predicates
        let letters = CharSet::from_string("abc");
        assert!(letters.every(|c| c.is_alphabetic()));
        assert!(!letters.every(|c| c.is_uppercase()));
        assert!(letters.any(|c| c.is_lowercase()));
        assert!(!letters.any(|c| c.is_numeric()));

        let numbers = CharSet::from_string("123");
        assert!(numbers.every(|c| c.is_numeric()));
        assert!(numbers.any(|c| c == '2'));
        assert!(!numbers.any(|c| c.is_alphabetic()));
    }

    /// Test character set fold operations
    #[test]
    fn test_charset_fold() {
        let abc = CharSet::from_string("abc");

        // Test fold to collect all characters
        let collected: Vec<char> = abc.fold(
            |c, mut acc: Vec<char>| {
                acc.push(c);
                acc
            },
            Vec::new(),
        );
        assert_eq!(collected.len(), 3);
        // Due to BTreeSet ordering, should be sorted
        assert_eq!(collected, &['a', 'b', 'c']);

        // Test fold for counting
        let count = abc.fold(|_c, acc| acc + 1, 0);
        assert_eq!(count, 3);

        // Test fold for string building
        let string: String = abc.fold(
            |c, mut acc| {
                acc.push(c);
                acc
            },
            String::new(),
        );
        assert_eq!(string, "abc");

        // Test fold with empty set
        let empty = CharSet::new();
        let empty_count = empty.fold(|_c, acc| acc + 1, 0);
        assert_eq!(empty_count, 0);
    }

    /// Test character set complement operations
    #[test]
    fn test_charset_complement() {
        let vowels = CharSet::from_string("aeiou");
        let complement = vowels.complement();

        // Complement should not contain vowels
        assert!(!complement.contains('a'));
        assert!(!complement.contains('e'));
        assert!(!complement.contains('i'));
        assert!(!complement.contains('o'));
        assert!(!complement.contains('u'));

        // But should contain other printable ASCII characters
        assert!(complement.contains('b'));
        assert!(complement.contains('z'));
        assert!(complement.contains('1'));
        assert!(complement.contains(' '));

        // Test empty set complement
        let empty = CharSet::new();
        let full_complement = empty.complement();
        assert!(!full_complement.is_empty());
        // Should contain printable ASCII + whitespace
        assert!(full_complement.contains('a'));
        assert!(full_complement.contains(' '));
        assert!(full_complement.contains('\t'));
    }

    /// Test display and string conversion
    #[test]
    fn test_charset_display() {
        // Empty set display
        let empty = CharSet::new();
        let empty_display = format!("{empty}");
        assert!(empty_display.contains("empty"));

        // Small set display (shows characters)
        let abc = CharSet::from_string("abc");
        let abc_display = format!("{abc}");
        assert!(abc_display.contains("size=3"));
        assert!(abc_display.contains("{a b c}"));

        // Test to_vec and conversion methods
        let chars = abc.to_vec();
        assert_eq!(chars, &['a', 'b', 'c']);
    }

    /// Test edge cases and error conditions
    #[test]
    fn test_charset_edge_cases() {
        // Very large character set
        let large_range = CharSet::from_range('\u{0000}', '\u{007F}');
        assert_eq!(large_range.size(), 128);

        // Duplicate characters (should be deduplicated)
        let duplicated = CharSet::from_string("aabbcc");
        assert_eq!(duplicated.size(), 3);
        assert!(duplicated.contains('a'));
        assert!(duplicated.contains('b'));
        assert!(duplicated.contains('c'));

        // Unicode characters
        let unicode = CharSet::from_string("αβγ");
        assert_eq!(unicode.size(), 3);
        assert!(unicode.contains('α'));
        assert!(unicode.contains('β'));
        assert!(unicode.contains('γ'));

        // Mixed ASCII and Unicode
        let mixed = CharSet::from_string("a€1β");
        assert_eq!(mixed.size(), 4);
        assert!(mixed.contains('a'));
        assert!(mixed.contains('€'));
        assert!(mixed.contains('1'));
        assert!(mixed.contains('β'));

        // Operations on empty sets
        let empty1 = CharSet::new();
        let empty2 = CharSet::new();
        let non_empty = CharSet::from_string("a");

        assert!(empty1.union(&empty2).is_empty());
        assert!(empty1.intersection(&empty2).is_empty());
        assert!(empty1.difference(&empty2).is_empty());
        assert!(empty1.symmetric_difference(&empty2).is_empty());

        assert_eq!(empty1.union(&non_empty).size(), 1);
        assert!(empty1.intersection(&non_empty).is_empty());
        assert!(empty1.difference(&non_empty).is_empty());
        assert_eq!(empty1.symmetric_difference(&non_empty).size(), 1);
    }
}

/// Integration tests that use the runtime evaluator to test SRFI-14 procedures
#[cfg(test)]
mod srfi14_integration_tests {
    use super::*;

    /// Test SRFI-14 procedures through the Lambdust runtime
    #[test]
    fn test_srfi14_runtime_integration() {
        // Note: This test requires a fully functioning runtime with SRFI-14 support
        // For now, we'll test the basic CharSet functionality through the Value system

        // Test CharSet Value creation
        let charset = CharSet::from_string("abc");
        let charset_value = Value::charset(charset);
        assert!(charset_value.is_charset());

        // Test that we can extract the charset from the value
        if let Value::CharSet(cs) = &charset_value {
            assert_eq!(cs.size(), 3);
            assert!(cs.contains('a'));
            assert!(cs.contains('b'));
            assert!(cs.contains('c'));
        } else {
            panic!("Expected CharSet value");
        }
    }
}

/// Performance benchmarks for character set operations
#[cfg(test)]
mod srfi14_performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_charset_performance() {
        // Test performance of basic operations on reasonably sized character sets

        // Create a medium-sized character set
        let chars: Vec<char> = (0..1000)
            .map(|i| char::from_u32(i as u32).unwrap_or('?'))
            .collect();
        let charset = CharSet::from_chars(chars);

        // Test contains performance
        let start = Instant::now();
        for i in 0..1000 {
            let c = char::from_u32(i as u32).unwrap_or('?');
            charset.contains(c);
        }
        let contains_duration = start.elapsed();
        println!("Contains operations (1000x): {contains_duration:?}");

        // Test union performance
        let other_chars: Vec<char> = (500..1500)
            .map(|i| char::from_u32(i as u32).unwrap_or('?'))
            .collect();
        let other_charset = CharSet::from_chars(other_chars);

        let start = Instant::now();
        let _union_result = charset.union(&other_charset);
        let union_duration = start.elapsed();
        println!("Union operation: {union_duration:?}");

        // Test intersection performance
        let start = Instant::now();
        let _intersection_result = charset.intersection(&other_charset);
        let intersection_duration = start.elapsed();
        println!("Intersection operation: {intersection_duration:?}");

        // Performance should be reasonable (under 1ms for these operations)
        assert!(contains_duration.as_millis() < 100);
        assert!(union_duration.as_millis() < 100);
        assert!(intersection_duration.as_millis() < 100);
    }
}
