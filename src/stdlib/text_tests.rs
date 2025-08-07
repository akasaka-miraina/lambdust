//! Comprehensive tests for SRFI-135 Text implementation.
//!
//! This module contains integration tests, compliance tests,
//! and benchmarks for the complete text processing system.

#[cfg(test)]
mod tests {
    use crate::stdlib::text::{Text, TextBuilder, TextMatch, TextRegex, NormalizationForm};
    use crate::stdlib::text_regex::*;
    use crate::stdlib::text_algorithms::*;
    use std::sync::Arc;
    use crate::stdlib::text_srfi135::*;
    use crate::stdlib::text_performance::*;
    use std::time::Instant;
    
    // Note: These dependencies would need to be added to Cargo.toml:
    // quickcheck = "1.0" (for property-based testing)

    // ============= BASIC TEXT TESTS =============

    #[test]
    fn test_text_creation_and_properties() {
        let empty = Text::new();
        assert!(empty.is_empty());
        assert_eq!(empty.char_length(), 0);
        assert_eq!(empty.byte_length(), 0);
        assert_eq!(empty.grapheme_length(), 0);

        let hello = Text::from_str("Hello, 世界! 🌍");
        assert!(!hello.is_empty());
        assert_eq!(hello.char_length(), 11);
        assert!(hello.byte_length() > hello.char_length()); // UTF-8 encoding
        assert_eq!(hello.grapheme_length(), 11); // In this case, same as char length
    }

    #[test]
    fn test_text_indexing_and_substring() {
        let text = Text::from_str("Hello, World!");
        
        assert_eq!(text.char_at(0), Some('H'));
        assert_eq!(text.char_at(7), Some('W'));
        assert_eq!(text.char_at(100), None);
        
        let hello = text.substring(0, 5).unwrap();
        assert_eq!(hello.to_string(), "Hello");
        
        let world = text.substring(7, 12).unwrap();
        assert_eq!(world.to_string(), "World");
        
        assert!(text.substring(10, 5).is_none()); // Invalid range
    }

    #[test]
    fn test_text_concatenation() {
        let hello = Text::from_str("Hello");
        let world = Text::from_str("World");
        let space = Text::from_str(" ");
        
        let greeting = hello.concat(&space).concat(&world);
        assert_eq!(greeting.to_string(), "Hello World");
        assert_eq!(greeting.char_length(), 11);
        
        // Test empty concatenation
        let empty = Text::new();
        let result = hello.concat(&empty);
        assert_eq!(result.to_string(), "Hello");
    }

    #[test]
    fn test_text_search_operations() {
        let text = Text::from_str("The quick brown fox jumps over the lazy dog");
        let pattern = Text::from_str("fox");
        
        assert!(text.contains(&pattern));
        assert_eq!(text.find(&pattern), Some(16));
        assert_eq!(text.rfind(&pattern), Some(16));
        
        let the = Text::from_str("the");
        assert_eq!(text.find(&the), Some(31)); // Case sensitive
        
        assert!(text.starts_with(&Text::from_str("The")));
        assert!(text.ends_with(&Text::from_str("dog")));
    }

    #[test]
    fn test_text_case_operations() {
        let mixed = Text::from_str("Hello, World!");
        
        let upper = mixed.to_uppercase();
        assert_eq!(upper.to_string(), "HELLO, WORLD!");
        
        let lower = mixed.to_lowercase();
        assert_eq!(lower.to_string(), "hello, world!");
        
        let title = Text::from_str("hello world").to_titlecase();
        assert_eq!(title.to_string(), "Hello World");
        
        let folded = mixed.fold_case();
        assert_eq!(folded.to_string(), "hello, world!");
    }

    #[test]
    fn test_text_splitting_and_joining() {
        let csv = Text::from_str("apple,banana,cherry,date");
        let comma = Text::from_str(",");
        
        let parts = csv.split(&comma);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].to_string(), "apple");
        assert_eq!(parts[3].to_string(), "date");
        
        let joined = TextOperations::join(&parts, &Text::from_str("; "));
        assert_eq!(joined.to_string(), "apple; banana; cherry; date");
    }

    #[test]
    fn test_text_replacement() {
        let text = Text::from_str("Hello, World! Hello, Universe!");
        let old = Text::from_str("Hello");
        let new = Text::from_str("Hi");
        
        let replaced = text.replace(&old, &new);
        assert_eq!(replaced.to_string(), "Hi, World! Hi, Universe!");
    }

    #[test]
    fn test_text_trimming() {
        let padded = Text::from_str("  \t  hello world  \n  ");
        
        let trimmed = padded.trim();
        assert_eq!(trimmed.to_string(), "hello world");
        
        let left_trimmed = padded.trim_start();
        assert_eq!(left_trimmed.to_string(), "hello world  \n  ");
        
        let right_trimmed = padded.trim_end();
        assert_eq!(right_trimmed.to_string(), "  \t  hello world");
    }

    // ============= UNICODE TESTS =============

    #[test]
    fn test_unicode_normalization() {
        // Test with composed vs decomposed characters
        let composed = Text::from_str("é"); // Single character
        let decomposed = Text::from_str("e\u{0301}"); // e + combining acute accent
        
        // They should not be equal as strings
        assert_ne!(composed.to_string(), decomposed.to_string());
        
        // But should be equal after normalization
        let nfc_composed = composed.normalize(NormalizationForm::NFC);
        let nfc_decomposed = decomposed.normalize(NormalizationForm::NFC);
        assert_eq!(nfc_composed.to_string(), nfc_decomposed.to_string());
        
        // Test NFD normalization
        let nfd_composed = composed.normalize(NormalizationForm::NFD);
        let nfd_decomposed = decomposed.normalize(NormalizationForm::NFD);
        assert_eq!(nfd_composed.to_string(), nfd_decomposed.to_string());
    }

    #[test]
    fn test_unicode_properties() {
        let text = Text::from_str("Hello, 世界! 🌍");
        
        // Test character properties
        assert_eq!(text.char_at(0), Some('H'));
        assert_eq!(text.char_at(7), Some('世'));
        assert_eq!(text.char_at(10), Some('🌍'));
        
        // Test normalization detection
        assert!(text.is_normalized(NormalizationForm::NFC));
    }

    #[test]
    fn test_grapheme_clusters() {
        // Test with combining characters
        let text = Text::from_str("a\u{0301}b\u{0308}c"); // a with acute, b with diaeresis, c
        
        // Should have 3 characters but 3 grapheme clusters
        assert_eq!(text.char_length(), 5); // a, acute, b, diaeresis, c
        assert_eq!(text.grapheme_length(), 3); // ä, b̈, c
    }

    // ============= REGEX TESTS =============

    #[test]
    fn test_regex_compilation_and_matching() {
        let regex = TextRegex::new(r"\d+").unwrap();
        let text = Text::from_str("Price: $123.45");
        
        assert!(regex.is_match(&text));
        
        let match_result = regex.find(&text).unwrap();
        assert_eq!(match_result.matched_text.to_string(), "123");
        assert_eq!(match_result.start, 8);
        assert_eq!(match_result.end, 11);
    }

    #[test]
    fn test_regex_replacement() {
        let regex = TextRegex::new(r"\b\w+@\w+\.\w+\b").unwrap();
        let text = Text::from_str("Contact us at john@example.com or mary@test.org");
        let replacement = Text::from_str("[EMAIL]");
        
        let result = regex.replace_all(&text, &replacement);
        assert_eq!(result.to_string(), "Contact us at [EMAIL] or [EMAIL]");
    }

    #[test]
    fn test_regex_groups() {
        let regex = TextRegex::new(r"(\w+)\s+(\d+)").unwrap();
        let text = Text::from_str("apple 123 banana 456");
        
        let matches = regex.find_all(&text);
        assert_eq!(matches.len(), 2);
        
        let first_match = &matches[0];
        assert_eq!(first_match.matched_text.to_string(), "apple 123");
        assert_eq!(first_match.groups.len(), 2);
        assert_eq!(first_match.groups[0].as_ref().unwrap().to_string(), "apple");
        assert_eq!(first_match.groups[1].as_ref().unwrap().to_string(), "123");
    }

    #[test]
    fn test_regex_splitting() {
        let regex = TextRegex::new(r"\s*,\s*").unwrap();
        let text = Text::from_str("apple, banana , cherry,  date");
        
        let parts = regex.split(&text);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].to_string(), "apple");
        assert_eq!(parts[1].to_string(), "banana");
        assert_eq!(parts[2].to_string(), "cherry");
        assert_eq!(parts[3].to_string(), "date");
    }

    // ============= ALGORITHM TESTS =============

    #[test]
    fn test_boyer_moore_search() {
        let pattern = Text::from_str("pattern");
        let text = Text::from_str("This is a test pattern for pattern matching algorithm");
        
        let searcher = BoyerMoore::new(&pattern);
        let matches = searcher.search(&text);
        
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], 15); // First occurrence
        assert_eq!(matches[1], 27); // Second occurrence
    }

    #[test]
    fn test_kmp_search() {
        let pattern = Text::from_str("ABAB");
        let text = Text::from_str("ABABCABABABAB");
        
        let searcher = KnuthMorrisPratt::new(&pattern);
        let matches = searcher.search(&text);
        
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0], 0);
        assert_eq!(matches[1], 5);
        assert_eq!(matches[2], 9);
    }

    #[test]
    fn test_string_similarity() {
        let text1 = Text::from_str("kitten");
        let text2 = Text::from_str("sitting");
        
        let distance = StringSimilarity::levenshtein_distance(&text1, &text2);
        assert_eq!(distance, 3);
        
        let lcs_len = StringSimilarity::lcs_length(&text1, &text2);
        assert_eq!(lcs_len, 4); // "ittn"
        
        let jaccard = StringSimilarity::jaccard_similarity(&text1, &text2);
        assert!(jaccard > 0.0 && jaccard < 1.0);
    }

    #[test]
    fn test_text_operations() {
        let text = Text::from_str("hello");
        
        // Test padding
        let left_padded = TextOperations::pad_left(&text, 10, '-');
        assert_eq!(left_padded.to_string(), "-----hello");
        
        let right_padded = TextOperations::pad_right(&text, 10, '-');
        assert_eq!(right_padded.to_string(), "hello-----");
        
        let centered = TextOperations::center(&text, 9, '-');
        assert_eq!(centered.to_string(), "--hello--");
        
        // Test wrapping
        let long_text = Text::from_str("This is a very long sentence that should be wrapped");
        let wrapped = TextOperations::wrap_lines(&long_text, 15);
        assert!(wrapped.len() > 1);
        assert!(wrapped.iter().all(|line| line.char_length() <= 15));
    }

    #[test]
    fn test_common_prefix_suffix() {
        let texts = vec![
            Text::from_str("prefix_hello_suffix"),
            Text::from_str("prefix_world_suffix"),
            Text::from_str("prefix_test_suffix"),
        ];
        
        let prefix = TextOperations::common_prefix(&texts);
        assert_eq!(prefix.to_string(), "prefix_");
        
        let suffix = TextOperations::common_suffix(&texts);
        assert_eq!(suffix.to_string(), "_suffix");
    }

    // ============= PERFORMANCE TESTS =============

    #[test]
    fn test_string_interning() {
        let pool = StringInterningPool::new();
        
        let s1 = pool.intern("test_string".to_string());
        let s2 = pool.intern("test_string".to_string());
        
        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&s1, &s2));
        
        let stats = pool.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_memory_pooling() {
        let pool = TextMemoryPool::new();
        
        // Allocate and deallocate buffers
        let buf1 = pool.allocate(100);
        assert!(buf1.capacity() >= 100);
        
        pool.deallocate(buf1);
        
        let buf2 = pool.allocate(100);
        assert!(buf2.capacity() >= 100);
        
        let stats = pool.stats();
        assert!(stats.returns_to_pool > 0 || stats.pool_allocations > 0);
    }

    #[test]
    fn test_optimized_text_builder() {
        let mut builder = OptimizedTextBuilder::new();
        
        for i in 0..1000 {
            builder.push_str(&format!("item{} ", i));
        }
        
        let text = builder.build();
        assert!(text.char_length() > 1000);
        assert!(text.to_string().contains("item999"));
    }

    #[test]
    fn test_simd_operations() {
        let text = Text::from_str("hello world hello universe hello galaxy");
        
        let count = SimdTextOps::count_char(&text, 'l');
        assert_eq!(count, 6);
        
        let needle = Text::from_str("universe");
        let pos = SimdTextOps::find_substring(&text, &needle);
        assert_eq!(pos, Some(18));
        
        let upper = SimdTextOps::to_ascii_uppercase(&text);
        assert_eq!(upper.to_string(), "HELLO WORLD HELLO UNIVERSE HELLO GALAXY");
    }

    // ============= INTEGRATION TESTS =============

    #[test]
    fn test_text_builder_integration() {
        let mut builder = TextBuilder::new();
        
        builder.push_str("Hello");
        builder.push_char(',');
        builder.push_char(' ');
        builder.push_text(&Text::from_str("World"));
        builder.push_char('!');
        
        let result = builder.build();
        assert_eq!(result.to_string(), "Hello, World!");
        assert_eq!(result.char_length(), 13);
    }

    #[test]
    fn test_complex_unicode_operations() {
        // Test with various Unicode scripts
        let mixed = Text::from_str("Hello 世界 مرحبا Привет 🌍");
        
        assert!(mixed.char_length() > 10);
        assert!(mixed.byte_length() > mixed.char_length());
        
        // Test normalization
        let normalized = mixed.normalize(NormalizationForm::NFC);
        assert!(normalized.is_normalized(NormalizationForm::NFC));
        
        // Test case operations with Unicode
        let upper = mixed.to_uppercase();
        let lower = mixed.to_lowercase();
        
        assert_ne!(upper.to_string(), lower.to_string());
    }

    #[test]
    fn test_regex_with_unicode() {
        let regex = TextRegex::new(r"\p{L}+").unwrap(); // Match any letter
        let text = Text::from_str("Hello 世界 مرحبا Привет");
        
        let matches = regex.find_all(&text);
        assert_eq!(matches.len(), 4);
        
        assert_eq!(matches[0].matched_text.to_string(), "Hello");
        assert_eq!(matches[1].matched_text.to_string(), "世界");
        assert_eq!(matches[2].matched_text.to_string(), "مرحبا");
        assert_eq!(matches[3].matched_text.to_string(), "Привет");
    }

    #[test]
    fn test_large_text_operations() {
        // Test with larger texts to verify performance
        let large_text = Text::from_string("word ".repeat(10000));
        
        assert_eq!(large_text.char_length(), 50000); // "word " * 10000
        
        let word = Text::from_str("word");
        let count = TextOperations::count_occurrences(&large_text, &word);
        assert_eq!(count, 10000);
        
        // Test substring operations
        let beginning = large_text.substring(0, 100).unwrap();
        assert_eq!(beginning.char_length(), 100);
        
        let ending = large_text.substring(49900, 50000).unwrap();
        assert_eq!(ending.char_length(), 100);
    }

    // ============= COMPLIANCE TESTS =============

    #[test]
    fn test_srfi_135_basic_compliance() {
        // Test basic SRFI-135 operations
        let text = Text::from_str("Hello, World!");
        
        // Text properties
        assert!(!text.is_empty());
        assert_eq!(text.char_length(), 13);
        
        // Character access
        assert_eq!(text.char_at(0), Some('H'));
        assert_eq!(text.char_at(7), Some('W'));
        
        // Subtext operations
        let hello = text.substring(0, 5).unwrap();
        assert_eq!(hello.to_string(), "Hello");
        
        let world = text.substring(7, 12).unwrap();
        assert_eq!(world.to_string(), "World");
        
        // Concatenation
        let greeting = hello.concat(&Text::from_str(" ")).concat(&world);
        assert_eq!(greeting.to_string(), "Hello World");
    }

    #[test]
    fn test_error_handling() {
        let text = Text::from_str("test");
        
        // Out of bounds access
        assert_eq!(text.char_at(100), None);
        assert!(text.substring(10, 20).is_none());
        assert!(text.substring(5, 2).is_none());
        
        // Empty text operations
        let empty = Text::new();
        assert_eq!(empty.char_at(0), None);
        assert!(empty.substring(0, 1).is_none());
    }

    // ============= BENCHMARK TESTS =============

    #[test]
    fn benchmark_text_creation() {
        let start = Instant::now();
        
        for i in 0..1000 {
            let _text = Text::from_string(format!("benchmark text {}", i));
        }
        
        let duration = start.elapsed();
        println!("Text creation benchmark: {:?}", duration);
        
        // Should complete reasonably quickly
        assert!(duration.as_millis() < 1000);
    }

    #[test]
    fn benchmark_text_concatenation() {
        let base = Text::from_str("base");
        let mut result = Text::new();
        
        let start = Instant::now();
        
        for _ in 0..1000 {
            result = result.concat(&base);
        }
        
        let duration = start.elapsed();
        println!("Text concatenation benchmark: {:?}", duration);
        
        assert_eq!(result.char_length(), 4000); // "base" * 1000
        assert!(duration.as_millis() < 5000);
    }

    #[test]
    fn benchmark_regex_search() {
        let regex = TextRegex::new(r"\b\w+\b").unwrap();
        let text = Text::from_string("word ".repeat(1000));
        
        let start = Instant::now();
        
        for _ in 0..100 {
            let _matches = regex.find_all(&text);
        }
        
        let duration = start.elapsed();
        println!("Regex search benchmark: {:?}", duration);
        
        assert!(duration.as_millis() < 5000);
    }

    #[test]
    fn benchmark_string_search_algorithms() {
        let pattern = Text::from_str("needle");
        let haystack = Text::from_string(format!("{}needle{}", "hay ".repeat(1000), " stack".repeat(1000)));
        
        // Boyer-Moore benchmark
        let start = Instant::now();
        let bm_searcher = BoyerMoore::new(&pattern);
        for _ in 0..100 {
            let _matches = bm_searcher.search(&haystack);
        }
        let bm_duration = start.elapsed();
        
        // KMP benchmark
        let start = Instant::now();
        let kmp_searcher = KnuthMorrisPratt::new(&pattern);
        for _ in 0..100 {
            let _matches = kmp_searcher.search(&haystack);
        }
        let kmp_duration = start.elapsed();
        
        println!("Boyer-Moore benchmark: {:?}", bm_duration);
        println!("KMP benchmark: {:?}", kmp_duration);
        
        assert!(bm_duration.as_millis() < 5000);
        assert!(kmp_duration.as_millis() < 5000);
    }

    #[test]
    fn benchmark_unicode_normalization() {
        let text = Text::from_string("café ".repeat(1000));
        
        let start = Instant::now();
        
        for _ in 0..100 {
            let _normalized = text.normalize(NormalizationForm::NFC);
        }
        
        let duration = start.elapsed();
        println!("Unicode normalization benchmark: {:?}", duration);
        
        assert!(duration.as_millis() < 5000);
    }

    // ============= STRESS TESTS =============

    #[test]
    fn stress_test_large_text() {
        // Create a very large text (1MB)
        let large_text = Text::from_string("x".repeat(1024 * 1024));
        
        assert_eq!(large_text.char_length(), 1024 * 1024);
        assert_eq!(large_text.byte_length(), 1024 * 1024);
        
        // Test operations on large text
        let beginning = large_text.substring(0, 1000).unwrap();
        assert_eq!(beginning.char_length(), 1000);
        
        let ending = large_text.substring(1024 * 1024 - 1000, 1024 * 1024).unwrap();
        assert_eq!(ending.char_length(), 1000);
    }

    #[test]
    fn stress_test_many_small_texts() {
        let mut texts = Vec::new();
        
        // Create many small texts
        for i in 0..10000 {
            texts.push(Text::from_string(format!("text{}", i)));
        }
        
        assert_eq!(texts.len(), 10000);
        
        // Test operations on all texts
        let total_length: usize = texts.iter().map(|t| t.char_length()).sum();
        assert!(total_length > 50000); // At least "text" + number for each
    }

    #[test]
    fn stress_test_deep_concatenation() {
        let mut result = Text::from_str("start");
        
        // Deep concatenation chain
        for i in 0..1000 {
            let next = Text::from_string(format!("-{}", i));
            result = result.concat(&next);
        }
        
        assert!(result.char_length() > 5000);
        assert!(result.to_string().starts_with("start-0-1-2"));
        assert!(result.to_string().ends_with("-999"));
    }
}

// ============= PROPERTY-BASED TESTS =============

// Property tests disabled - quickcheck not available
// TODO: Migrate to proptest when property-testing feature is enabled
/*
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    #[quickcheck]
    fn prop_text_length_consistency(s: String) -> bool {
        let text = Text::from_string(s.clone());
        text.char_length() == s.chars().count() &&
        text.byte_length() == s.len()
    }

    #[quickcheck]
    fn prop_substring_length(s: String, start: usize, len: usize) -> TestResult {
        if s.is_empty() || start >= s.chars().count() || len == 0 {
            return TestResult::discard();
        }

        let text = Text::from_string(s);
        let end = std::cmp::min(start + len, text.char_length());
        
        if let Some(sub) = text.substring(start, end) {
            TestResult::from_bool(sub.char_length() == end - start)
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn prop_concatenation_length(s1: String, s2: String) -> bool {
        let t1 = Text::from_string(s1.clone());
        let t2 = Text::from_string(s2.clone());
        let concat = t1.concat(&t2);
        
        concat.char_length() == t1.char_length() + t2.char_length() &&
        concat.to_string() == s1 + &s2
    }

    #[quickcheck]
    fn prop_case_conversion_roundtrip(s: String) -> bool {
        let text = Text::from_string(s);
        let upper = text.to_uppercase();
        let lower = text.to_lowercase();
        
        // Upper then lower should give consistent results
        let upper_then_lower = upper.to_lowercase();
        let lower_then_upper = lower.to_uppercase();
        
        // At minimum, the operations should be self-consistent
        upper_then_lower.to_string() == lower.to_string() &&
        lower_then_upper.to_string() == upper.to_string()
    }

    #[quickcheck]
    fn prop_normalization_idempotent(s: String) -> bool {
        let text = Text::from_string(s);
        let nfc1 = text.normalize(NormalizationForm::NFC);
        let nfc2 = nfc1.normalize(NormalizationForm::NFC);
        
        nfc1.to_string() == nfc2.to_string()
    }

    #[quickcheck]
    fn prop_split_join_roundtrip(s: String, delimiter: char) -> TestResult {
        if s.contains(delimiter) && !s.is_empty() {
            let text = Text::from_string(s.clone());
            let delim = Text::from_char(delimiter);
            
            let parts = text.split(&delim);
            let rejoined = TextOperations::join(&parts, &delim);
            
            TestResult::from_bool(rejoined.to_string() == s)
        } else {
            TestResult::discard()
        }
    }
}
*/

// ============= BENCHMARK MODULE =============

#[cfg(test)]
mod benchmarks {
    use super::*;
    use crate::stdlib::text::{Text, TextBuilder, TextMatch, TextRegex, NormalizationForm};
    use crate::stdlib::text_algorithms::{BoyerMoore, KnuthMorrisPratt};
    use std::time::{Duration, Instant};

    fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> Duration
    where
        F: FnMut(),
    {
        let start = Instant::now();
        
        for _ in 0..iterations {
            f();
        }
        
        let duration = start.elapsed();
        println!("{}: {:?} ({} iterations)", name, duration, iterations);
        duration
    }

    #[test]
    fn comprehensive_benchmarks() {
        println!("\n=== Text Processing Benchmarks ===");

        // Text creation benchmark
        benchmark("Text creation", 10000, || {
            let _text = Text::from_str("benchmark string");
        });

        // Concatenation benchmark
        let text1 = Text::from_str("hello");
        let text2 = Text::from_str(" world");
        benchmark("Text concatenation", 10000, || {
            let _result = text1.concat(&text2);
        });

        // Substring benchmark
        let long_text = Text::from_string("a".repeat(1000));
        benchmark("Substring extraction", 10000, || {
            let _sub = long_text.substring(100, 200);
        });

        // Search benchmark
        let haystack = Text::from_string(format!("{}needle{}", "hay ".repeat(100), " stack".repeat(100)));
        let needle = Text::from_str("needle");
        benchmark("Text search", 1000, || {
            let _pos = haystack.find(&needle);
        });

        // Case conversion benchmark
        let mixed_case = Text::from_str("Hello World This Is A Test String");
        benchmark("Case conversion", 10000, || {
            let _upper = mixed_case.to_uppercase();
        });

        // Unicode normalization benchmark
        let unicode_text = Text::from_str("café naïve résumé");
        benchmark("Unicode normalization", 1000, || {
            let _normalized = unicode_text.normalize(NormalizationForm::NFC);
        });

        // Regex benchmark
        let regex = TextRegex::new(r"\b\w+\b").unwrap();
        let word_text = Text::from_string("word ".repeat(100));
        benchmark("Regex matching", 1000, || {
            let _matches = regex.find_all(&word_text);
        });

        // String algorithms benchmark
        let pattern = Text::from_str("pattern");
        let text_with_pattern = Text::from_string(format!("{}pattern{}", "text ".repeat(100), " more".repeat(100)));
        
        let boyer_moore = BoyerMoore::new(&pattern);
        benchmark("Boyer-Moore search", 1000, || {
            let _matches = boyer_moore.search(&text_with_pattern);
        });

        let kmp = KnuthMorrisPratt::new(&pattern);
        benchmark("KMP search", 1000, || {
            let _matches = kmp.search(&text_with_pattern);
        });

        println!("=== Benchmarks Complete ===\n");
    }
}