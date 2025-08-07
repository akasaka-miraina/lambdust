//! Advanced R7RS-large feature tests for Lambdust
//! 
//! This module tests R7RS-large features including:
//! - Numeric tower with arbitrary precision
//! - Advanced container operations (hash tables, priority queues)
//! - Unicode text processing and internationalization
//! - SRFI implementations and compliance

use lambdust::{
    numeric::*,
    containers::*,
    stdlib::text::*,
    eval::Value,
    ast::Literal,
    diagnostics::Result,
};
use std::collections::HashMap;

#[test]
fn test_arbitrary_precision_integers() {
    let big_int = BigInteger::from_string("123456789012345678901234567890").unwrap();
    
    // Test basic arithmetic with big integers
    let another_big = BigInteger::from_string("987654321098765432109876543210").unwrap();
    let sum = big_int.add(&another_big);
    
    assert_eq!(
        sum.to_string(),
        "1111111110111111110011111111100"
    );
    
    // Test multiplication
    let small_int = BigInteger::from_i64(1000);
    let product = big_int.multiply(&small_int);
    
    assert_eq!(
        product.to_string(),
        "123456789012345678901234567890000"
    );
    
    // Test division
    let (quotient, remainder) = big_int.divide(&small_int);
    assert_eq!(quotient.to_string(), "123456789012345678901234567");
    assert_eq!(remainder.to_string(), "890");
    
    // Test comparison
    assert!(big_int < another_big);
    assert!(another_big > big_int);
    assert_eq!(big_int, big_int.clone());
}

#[test]
fn test_rational_numbers() {
    // Test exact rational arithmetic
    let rat1 = Rational::new(BigInteger::from_i64(22), BigInteger::from_i64(7)); // 22/7
    let rat2 = Rational::new(BigInteger::from_i64(1), BigInteger::from_i64(3));  // 1/3
    
    // Test addition: 22/7 + 1/3 = 66/21 + 7/21 = 73/21
    let sum = rat1.add(&rat2);
    assert_eq!(sum.numerator().to_i64().unwrap(), 73);
    assert_eq!(sum.denominator().to_i64().unwrap(), 21);
    
    // Test subtraction: 22/7 - 1/3 = 66/21 - 7/21 = 59/21
    let diff = rat1.subtract(&rat2);
    assert_eq!(diff.numerator().to_i64().unwrap(), 59);
    assert_eq!(diff.denominator().to_i64().unwrap(), 21);
    
    // Test multiplication: (22/7) * (1/3) = 22/21
    let product = rat1.multiply(&rat2);
    assert_eq!(product.numerator().to_i64().unwrap(), 22);
    assert_eq!(product.denominator().to_i64().unwrap(), 21);
    
    // Test division: (22/7) / (1/3) = (22/7) * (3/1) = 66/7
    let quotient = rat1.divide(&rat2);
    assert_eq!(quotient.numerator().to_i64().unwrap(), 66);
    assert_eq!(quotient.denominator().to_i64().unwrap(), 7);
    
    // Test reduction to lowest terms
    let unreduced = Rational::new(BigInteger::from_i64(100), BigInteger::from_i64(25));
    assert_eq!(unreduced.numerator().to_i64().unwrap(), 4);
    assert_eq!(unreduced.denominator().to_i64().unwrap(), 1);
}

#[test]
fn test_complex_numbers() {
    // Test complex number arithmetic
    let c1 = ComplexNumber::new(3.0, 4.0); // 3 + 4i
    let c2 = ComplexNumber::new(1.0, 2.0); // 1 + 2i
    
    // Test addition: (3 + 4i) + (1 + 2i) = 4 + 6i
    let sum = c1.add(&c2);
    assert_eq!(sum.real(), 4.0);
    assert_eq!(sum.imaginary(), 6.0);
    
    // Test multiplication: (3 + 4i) * (1 + 2i) = 3 + 6i + 4i + 8i² = 3 + 10i - 8 = -5 + 10i
    let product = c1.multiply(&c2);
    assert_eq!(product.real(), -5.0);
    assert_eq!(product.imaginary(), 10.0);
    
    // Test magnitude: |3 + 4i| = √(3² + 4²) = √25 = 5
    assert_eq!(c1.magnitude(), 5.0);
    
    // Test conjugate: conj(3 + 4i) = 3 - 4i
    let conj = c1.conjugate();
    assert_eq!(conj.real(), 3.0);
    assert_eq!(conj.imaginary(), -4.0);
    
    // Test polar form conversion
    let polar = c1.to_polar();
    assert_eq!(polar.magnitude, 5.0);
    assert!((polar.angle - 0.9272952180016122).abs() < 1e-10);
}

#[test]
fn test_numeric_tower_conversions() {
    let tower = NumericTower::new();
    
    // Test automatic promotion through numeric tower
    let int_val = NumericValue::Integer(BigInteger::from_i64(42));
    let rat_val = NumericValue::Rational(Rational::new(BigInteger::from_i64(22), BigInteger::from_i64(7)));
    let float_val = NumericValue::Real(3.14159);
    let complex_val = NumericValue::Complex(ComplexNumber::new(1.0, 1.0));
    
    // Test promotion in arithmetic operations
    let int_plus_rat = tower.add(&int_val, &rat_val).unwrap();
    match int_plus_rat {
        NumericValue::Rational(r) => {
            // 42 + 22/7 = 294/7 + 22/7 = 316/7
            assert_eq!(r.numerator().to_i64().unwrap(), 316);
            assert_eq!(r.denominator().to_i64().unwrap(), 7);
        }
        _ => panic!("Expected rational result"),
    }
    
    let rat_plus_float = tower.add(&rat_val, &float_val).unwrap();
    match rat_plus_float {
        NumericValue::Real(f) => {
            let expected = 22.0 / 7.0 + 3.14159;
            assert!((f - expected).abs() < 1e-10);
        }
        _ => panic!("Expected real result"),
    }
    
    let float_plus_complex = tower.add(&float_val, &complex_val).unwrap();
    match float_plus_complex {
        NumericValue::Complex(c) => {
            assert!((c.real() - 4.14159).abs() < 1e-10);
            assert_eq!(c.imaginary(), 1.0);
        }
        _ => panic!("Expected complex result"),
    }
}

#[test]
fn test_advanced_hash_tables() {
    // Test SRFI-125 compliant hash tables
    let mut hash_table = AdvancedHashTable::new();
    
    // Test custom hash and equality functions
    let string_hash = |key: &Value| -> u64 {
        match key {
            Value::Literal(Literal::String(s)) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                hasher.finish()
            }
            _ => 0,
        }
    };
    
    let string_equal = |a: &Value, b: &Value| -> bool {
        match (a, b) {
            (Value::Literal(Literal::String(s1)), Value::Literal(Literal::String(s2))) => s1 == s2,
            _ => false,
        }
    };
    
    hash_table.set_hash_function(Box::new(string_hash));
    hash_table.set_equality_function(Box::new(string_equal));
    
    // Test hash table operations
    let key1 = Value::Literal(Literal::string("name".to_string()));
    let val1 = Value::Literal(Literal::string("Alice".to_string()));
    
    hash_table.insert(key1.clone(), val1.clone());
    assert_eq!(hash_table.size(), 1);
    
    let retrieved = hash_table.get(&key1).unwrap();
    assert_eq!(retrieved, val1);
    
    // Test hash table iteration
    let mut count = 0;
    hash_table.for_each(|_key, _value| {
        count += 1;
    });
    assert_eq!(count, 1);
    
    // Test hash table with weak references
    let weak_table = WeakHashTable::new();
    let strong_key = std::sync::Arc::new(Value::Literal(Literal::number(42.0)));
    let weak_key = std::sync::Arc::downgrade(&strong_key);
    
    weak_table.insert_weak(weak_key.clone(), val1.clone());
    assert!(weak_table.contains_weak_key(&weak_key));
    
    // When strong_key is dropped, the weak reference should become invalid
    drop(strong_key);
    // In a real implementation, this would require garbage collection to clean up
}

#[test]
fn test_priority_queues() {
    // Test priority queue with custom comparator
    let mut pq = PriorityQueue::new();
    
    // Set up a custom comparator for numbers (min-heap)
    pq.set_comparator(Box::new(|a: &Value, b: &Value| -> std::cmp::Ordering {
        match (a, b) {
            (Value::Literal(Literal::Number(n1)), Value::Literal(Literal::Number(n2))) => {
                n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal)
            }
            _ => std::cmp::Ordering::Equal,
        }
    }));
    
    // Insert elements
    pq.insert(Value::Literal(Literal::number(5.0)), 5);
    pq.insert(Value::Literal(Literal::number(1.0)), 1);
    pq.insert(Value::Literal(Literal::number(3.0)), 3);
    pq.insert(Value::Literal(Literal::number(7.0)), 7);
    
    assert_eq!(pq.size(), 4);
    
    // Extract elements in priority order (min-heap)
    let min = pq.extract_min().unwrap();
    if let Value::Literal(Literal::Number(n)) = min.0 {
        assert_eq!(n, 1.0);
        assert_eq!(min.1, 1);
    }
    
    let next = pq.peek_min().unwrap();
    if let Value::Literal(Literal::Number(n)) = &next.0 {
        assert_eq!(*n, 3.0);
    }
    
    // Test priority update
    pq.update_priority(&Value::Literal(Literal::number(7.0)), 0);
    let new_min = pq.peek_min().unwrap();
    if let Value::Literal(Literal::Number(n)) = &new_min.0 {
        assert_eq!(*n, 7.0); // Should now be minimum due to priority 0
    }
}

#[test]
fn test_persistent_data_structures() {
    // Test persistent ideque (SRFI-134)
    let mut ideque = PersistentIdeque::empty();
    
    // Add elements to both ends
    ideque = ideque.add_front(Value::Literal(Literal::number(1.0)));
    ideque = ideque.add_rear(Value::Literal(Literal::number(2.0)));
    ideque = ideque.add_front(Value::Literal(Literal::number(0.0)));
    ideque = ideque.add_rear(Value::Literal(Literal::number(3.0)));
    
    assert_eq!(ideque.length(), 4);
    
    // Check front and rear elements
    let front = ideque.front().unwrap();
    if let Value::Literal(Literal::Number(n)) = front {
        assert_eq!(*n, 0.0);
    }
    
    let rear = ideque.rear().unwrap();
    if let Value::Literal(Literal::Number(n)) = rear {
        assert_eq!(*n, 3.0);
    }
    
    // Remove elements and verify persistence
    let ideque2 = ideque.remove_front();
    assert_eq!(ideque.length(), 4);  // Original unchanged
    assert_eq!(ideque2.length(), 3); // New version modified
    
    // Test random access list (SRFI-101)
    let mut ral = RandomAccessList::empty();
    for i in 0..10 {
        ral = ral.cons(Value::Literal(Literal::number(i as f64)));
    }
    
    // Test O(log n) random access
    let element_5 = ral.ref_at(5).unwrap();
    if let Value::Literal(Literal::Number(n)) = element_5 {
        assert_eq!(*n, 4.0); // 0-indexed, so 5th element is 4
    }
    
    // Test O(log n) update
    let updated_ral = ral.set_at(5, Value::Literal(Literal::number(99.0)));
    let new_element_5 = updated_ral.ref_at(5).unwrap();
    if let Value::Literal(Literal::Number(n)) = new_element_5 {
        assert_eq!(*n, 99.0);
    }
    
    // Original should be unchanged
    let orig_element_5 = ral.ref_at(5).unwrap();
    if let Value::Literal(Literal::Number(n)) = orig_element_5 {
        assert_eq!(*n, 4.0);
    }
}

#[test]
fn test_unicode_text_processing() {
    let text_processor = UnicodeTextProcessor::new();
    
    // Test Unicode normalization
    let text = "café"; // Contains é as a single character
    let composed = "cafe\u{0301}"; // Contains e + combining accent
    
    let normalized_text = text_processor.normalize_nfc(text);
    let normalized_composed = text_processor.normalize_nfc(composed);
    
    assert_eq!(normalized_text, normalized_composed);
    
    // Test Unicode case folding
    let mixed_case = "Hello WørLD";
    let folded = text_processor.case_fold(mixed_case);
    assert_eq!(folded, "hello wørld");
    
    // Test grapheme cluster iteration
    let emoji_text = "👨‍👩‍👧‍👦🇺🇸";  // Family emoji + flag emoji
    let clusters = text_processor.grapheme_clusters(emoji_text);
    assert_eq!(clusters.len(), 2); // Should recognize as 2 graphemes
    
    // Test Unicode character properties
    assert!(text_processor.is_alphabetic('A'));
    assert!(text_processor.is_numeric('5'));
    assert!(text_processor.is_whitespace(' '));
    assert!(text_processor.is_punctuation('!'));
    
    // Test text segmentation
    let sentence = "Hello world. How are you? Fine, thanks!";
    let sentences = text_processor.segment_sentences(sentence);
    assert_eq!(sentences.len(), 3);
    
    let words = text_processor.segment_words(sentence);
    assert!(words.len() > 5); // Should break into multiple words
}

#[test]
fn test_internationalization() {
    let i18n = InternationalizationManager::new();
    
    // Test locale-specific text collation
    i18n.set_locale("en_US");
    let mut words = vec!["zebra", "apple", "Banana"];
    i18n.sort_by_locale(&mut words);
    // English collation should be case-insensitive by default
    assert_eq!(words, vec!["apple", "Banana", "zebra"]);
    
    // Test different locale
    i18n.set_locale("sv_SE"); // Swedish
    let mut swedish_words = vec!["ö", "z", "a", "å"];
    i18n.sort_by_locale(&mut swedish_words);
    // In Swedish, å and ö come after z
    assert_eq!(swedish_words, vec!["a", "z", "å", "ö"]);
    
    // Test number formatting
    let number = 1234567.89;
    let us_format = i18n.format_number(number, "en_US");
    assert_eq!(us_format, "1,234,567.89");
    
    let de_format = i18n.format_number(number, "de_DE");
    assert_eq!(de_format, "1.234.567,89");
    
    // Test date formatting
    let date = chrono::NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
    let us_date = i18n.format_date(date, "en_US");
    let iso_date = i18n.format_date(date, "ISO");
    
    assert!(us_date.contains("12"));
    assert!(us_date.contains("25"));
    assert_eq!(iso_date, "2024-12-25");
}

#[test]
fn test_srfi_implementations() {
    // Test SRFI-1 (List Library)
    let srfi1 = SRFI1::new();
    
    let list = vec![
        Value::Literal(Literal::number(1.0)),
        Value::Literal(Literal::number(2.0)),
        Value::Literal(Literal::number(3.0)),
        Value::Literal(Literal::number(4.0)),
        Value::Literal(Literal::number(5.0)),
    ];
    
    // Test filter
    let is_even = |v: &Value| -> bool {
        if let Value::Literal(Literal::Number(n)) = v {
            (*n as i64) % 2 == 0
        } else {
            false
        }
    };
    
    let evens = srfi1.filter(&list, is_even);
    assert_eq!(evens.len(), 2);
    
    // Test fold
    let sum = srfi1.fold_left(
        &list,
        Value::Literal(Literal::number(0.0)),
        |acc, item| {
            if let (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(i))) = (acc, item) {
                Value::Literal(Literal::number(a + i))
            } else {
                acc.clone()
            }
        },
    );
    
    if let Value::Literal(Literal::Number(n)) = sum {
        assert_eq!(n, 15.0); // 1+2+3+4+5
    }
    
    // Test SRFI-13 (String Library)
    let srfi13 = SRFI13::new();
    
    let string1 = "hello world";
    let string2 = "HELLO WORLD";
    
    assert!(srfi13.string_ci_equal(string1, string2));
    assert_eq!(srfi13.string_upcase(string1), "HELLO WORLD");
    assert_eq!(srfi13.string_downcase(string2), "hello world");
    
    let prefix = srfi13.string_take(string1, 5);
    assert_eq!(prefix, "hello");
    
    let suffix = srfi13.string_drop(string1, 6);
    assert_eq!(suffix, "world");
    
    // Test SRFI-14 (Character Sets)
    let srfi14 = SRFI14::new();
    
    let digit_set = srfi14.char_set_digit();
    assert!(digit_set.contains('5'));
    assert!(!digit_set.contains('a'));
    
    let alpha_set = srfi14.char_set_letter();
    assert!(alpha_set.contains('A'));
    assert!(alpha_set.contains('z'));
    assert!(!alpha_set.contains('5'));
    
    let alphanum_set = srfi14.char_set_union(&alpha_set, &digit_set);
    assert!(alphanum_set.contains('A'));
    assert!(alphanum_set.contains('5'));
    assert!(!alphanum_set.contains('!'));
}

#[test]
fn test_advanced_text_algorithms() {
    let text_algorithms = TextAlgorithms::new();
    
    // Test string searching algorithms
    let text = "The quick brown fox jumps over the lazy dog";
    let pattern = "brown fox";
    
    let kmp_result = text_algorithms.kmp_search(text, pattern);
    assert_eq!(kmp_result, Some(10));
    
    let boyer_moore_result = text_algorithms.boyer_moore_search(text, pattern);
    assert_eq!(boyer_moore_result, Some(10));
    
    // Test edit distance (Levenshtein)
    let str1 = "kitten";
    let str2 = "sitting";
    let distance = text_algorithms.levenshtein_distance(str1, str2);
    assert_eq!(distance, 3);
    
    // Test longest common subsequence
    let seq1 = "ABCDGH";
    let seq2 = "AEDFHR";
    let lcs = text_algorithms.longest_common_subsequence(seq1, seq2);
    assert_eq!(lcs, "ADH");
    
    // Test regular expression matching
    let regex_engine = text_algorithms.regex_engine();
    let pattern = regex_engine.compile(r"\d{3}-\d{3}-\d{4}").unwrap();
    
    let phone1 = "555-123-4567";
    let phone2 = "not-a-phone";
    
    assert!(pattern.matches(phone1));
    assert!(!pattern.matches(phone2));
    
    // Test text compression
    let original = "abracadabra";
    let compressed = text_algorithms.huffman_encode(original);
    let decompressed = text_algorithms.huffman_decode(&compressed);
    
    assert_eq!(decompressed, original);
    assert!(compressed.len() < original.len()); // Should be smaller
}

// Integration test combining multiple R7RS-large features
#[test]
fn test_r7rs_large_integration() {
    // Create a comprehensive test using multiple advanced features
    let mut system = AdvancedR7RSSystem::new();
    
    // Set up numeric tower with all types
    let numbers = vec![
        NumericValue::Integer(BigInteger::from_string("12345678901234567890").unwrap()),
        NumericValue::Rational(Rational::new(BigInteger::from_i64(22), BigInteger::from_i64(7))),
        NumericValue::Real(std::f64::consts::PI),
        NumericValue::Complex(ComplexNumber::new(3.0, 4.0)),
    ];
    
    // Perform arithmetic operations across the tower
    let mut sum = NumericValue::Integer(BigInteger::from_i64(0));
    for number in &numbers {
        sum = system.numeric_tower().add(&sum, number).unwrap();
    }
    
    // The result should be promoted to Complex
    assert!(matches!(sum, NumericValue::Complex(_)));
    
    // Set up advanced containers
    let mut hash_table = AdvancedHashTable::new();
    let mut priority_queue = PriorityQueue::new();
    let persistent_list = PersistentIdeque::empty();
    
    // Store the numeric results in containers
    for (i, number) in numbers.iter().enumerate() {
        let key = Value::Literal(Literal::string(format!("number_{}", i)));
        let value = system.numeric_to_value(number.clone());
        
        hash_table.insert(key.clone(), value.clone());
        priority_queue.insert(value.clone(), i);
        // persistent_list operations would go here
    }
    
    // Test Unicode text processing with the results
    let text_processor = UnicodeTextProcessor::new();
    let description = "Numeric computation results: π, complex numbers, rationals!";
    
    let normalized = text_processor.normalize_nfc(description);
    let graphemes = text_processor.grapheme_clusters(&normalized);
    
    // Verify the integration worked
    assert_eq!(hash_table.size(), 4);
    assert_eq!(priority_queue.size(), 4);
    assert!(!graphemes.is_empty());
    
    // Test SRFI integration with the processed data
    let srfi1 = SRFI1::new();
    let container_values: Vec<_> = hash_table.values().collect();
    
    let filtered_values = srfi1.filter(&container_values, |v| {
        // Filter for numeric values
        matches!(v, Value::Literal(Literal::Number(_)))
    });
    
    // Should have found the numeric values
    assert!(!filtered_values.is_empty());
}