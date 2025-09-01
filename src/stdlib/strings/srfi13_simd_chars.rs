//! SRFI-13 SIMD Character Processing
//!
//! This module implements vectorized character operations following the
//! cs-architect's design for 8-16x speedup in character processing.
//!
//! Target Performance:
//! - 8-16x speedup for ASCII character operations
//! - Vectorized case conversion with lookup tables
//! - Bloom filter character classification
//! - Multi-character search optimization

use crate::diagnostics::{Error, Result};
use crate::eval::value::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// ASCII lookup tables for optimized character operations
#[derive(Debug, Clone)]
pub struct ASCIILookupTables {
    /// Uppercase conversion table
    to_upper: [u8; 256],
    /// Lowercase conversion table  
    to_lower: [u8; 256],
    /// Character class classification
    char_classes: [CharacterClassFlags; 256],
}

/// Bit flags for character classification
#[derive(Debug, Clone, Copy)]
pub struct CharacterClassFlags(u8);

impl CharacterClassFlags {
    const ALPHABETIC: u8 = 1 << 0;
    const NUMERIC: u8 = 1 << 1;
    const WHITESPACE: u8 = 1 << 2;
    const PUNCTUATION: u8 = 1 << 3;
    const UPPER_CASE: u8 = 1 << 4;
    const LOWER_CASE: u8 = 1 << 5;
    const CONTROL: u8 = 1 << 6;
    const GRAPHIC: u8 = 1 << 7;
    
    pub fn new() -> Self {
        Self(0)
    }
    
    pub fn with_alphabetic(mut self) -> Self {
        self.0 |= Self::ALPHABETIC;
        self
    }
    
    pub fn with_numeric(mut self) -> Self {
        self.0 |= Self::NUMERIC;
        self
    }
    
    pub fn with_whitespace(mut self) -> Self {
        self.0 |= Self::WHITESPACE;
        self
    }
    
    pub fn with_punctuation(mut self) -> Self {
        self.0 |= Self::PUNCTUATION;
        self
    }
    
    pub fn with_upper_case(mut self) -> Self {
        self.0 |= Self::UPPER_CASE;
        self
    }
    
    pub fn with_lower_case(mut self) -> Self {
        self.0 |= Self::LOWER_CASE;
        self
    }
    
    pub fn with_control(mut self) -> Self {
        self.0 |= Self::CONTROL;
        self
    }
    
    pub fn with_graphic(mut self) -> Self {
        self.0 |= Self::GRAPHIC;
        self
    }
    
    pub fn is_alphabetic(&self) -> bool {
        self.0 & Self::ALPHABETIC != 0
    }
    
    pub fn is_numeric(&self) -> bool {
        self.0 & Self::NUMERIC != 0
    }
    
    pub fn is_whitespace(&self) -> bool {
        self.0 & Self::WHITESPACE != 0
    }
    
    pub fn is_punctuation(&self) -> bool {
        self.0 & Self::PUNCTUATION != 0
    }
    
    pub fn is_upper_case(&self) -> bool {
        self.0 & Self::UPPER_CASE != 0
    }
    
    pub fn is_lower_case(&self) -> bool {
        self.0 & Self::LOWER_CASE != 0
    }
    
    pub fn is_control(&self) -> bool {
        self.0 & Self::CONTROL != 0
    }
    
    pub fn is_graphic(&self) -> bool {
        self.0 & Self::GRAPHIC != 0
    }
}

impl ASCIILookupTables {
    pub fn new() -> Self {
        let mut to_upper = [0u8; 256];
        let mut to_lower = [0u8; 256];
        let mut char_classes = [CharacterClassFlags::new(); 256];
        
        // Initialize lookup tables
        for i in 0u8..=255 {
            let ch = i as char;
            
            // Case conversion
            to_upper[i as usize] = if ch.is_ascii_lowercase() {
                ch.to_ascii_uppercase() as u8
            } else {
                i
            };
            
            to_lower[i as usize] = if ch.is_ascii_uppercase() {
                ch.to_ascii_lowercase() as u8
            } else {
                i
            };
            
            // Character classification
            let mut flags = CharacterClassFlags::new();
            
            if ch.is_alphabetic() {
                flags = flags.with_alphabetic();
            }
            if ch.is_numeric() {
                flags = flags.with_numeric();
            }
            if ch.is_whitespace() {
                flags = flags.with_whitespace();
            }
            if ch.is_ascii_punctuation() {
                flags = flags.with_punctuation();
            }
            if ch.is_uppercase() {
                flags = flags.with_upper_case();
            }
            if ch.is_lowercase() {
                flags = flags.with_lower_case();
            }
            if ch.is_control() {
                flags = flags.with_control();
            }
            if !ch.is_whitespace() && !ch.is_control() {
                flags = flags.with_graphic();
            }
            
            char_classes[i as usize] = flags;
        }
        
        Self {
            to_upper,
            to_lower,
            char_classes,
        }
    }
    
    #[inline]
    pub fn to_uppercase_byte(&self, byte: u8) -> u8 {
        self.to_upper[byte as usize]
    }
    
    #[inline]
    pub fn to_lowercase_byte(&self, byte: u8) -> u8 {
        self.to_lower[byte as usize]
    }
    
    #[inline]
    pub fn get_char_class(&self, byte: u8) -> CharacterClassFlags {
        self.char_classes[byte as usize]
    }
}

impl Default for ASCIILookupTables {
    fn default() -> Self {
        Self::new()
    }
}

/// Bloom filter for efficient character set testing
#[derive(Debug, Clone)]
pub struct CharacterBloomFilter {
    /// Bit vector for bloom filter
    bits: [u64; 4], // 256 bits total
    /// Hash function count
    hash_count: usize,
}

impl CharacterBloomFilter {
    pub fn new() -> Self {
        Self {
            bits: [0; 4],
            hash_count: 3,
        }
    }
    
    pub fn from_chars<I>(chars: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        let mut filter = Self::new();
        for ch in chars {
            filter.insert(ch);
        }
        filter
    }
    
    fn hash1(&self, ch: char) -> usize {
        (ch as u32 as usize) % 256
    }
    
    fn hash2(&self, ch: char) -> usize {
        ((ch as u32).wrapping_mul(31) as usize) % 256
    }
    
    fn hash3(&self, ch: char) -> usize {
        ((ch as u32).wrapping_mul(17).wrapping_add(7) as usize) % 256
    }
    
    pub fn insert(&mut self, ch: char) {
        let h1 = self.hash1(ch);
        let h2 = self.hash2(ch);
        let h3 = self.hash3(ch);
        
        self.set_bit(h1);
        self.set_bit(h2);
        self.set_bit(h3);
    }
    
    pub fn may_contain(&self, ch: char) -> bool {
        let h1 = self.hash1(ch);
        let h2 = self.hash2(ch);
        let h3 = self.hash3(ch);
        
        self.get_bit(h1) && self.get_bit(h2) && self.get_bit(h3)
    }
    
    fn set_bit(&mut self, pos: usize) {
        let word_idx = pos / 64;
        let bit_idx = pos % 64;
        self.bits[word_idx] |= 1u64 << bit_idx;
    }
    
    fn get_bit(&self, pos: usize) -> bool {
        let word_idx = pos / 64;
        let bit_idx = pos % 64;
        (self.bits[word_idx] & (1u64 << bit_idx)) != 0
    }
}

impl Default for CharacterBloomFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache for character bloom filters
#[derive(Debug)]
pub struct CharacterBloomFilterCache {
    cache: RwLock<HashMap<String, CharacterBloomFilter>>,
    capacity: usize,
}

impl CharacterBloomFilterCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            capacity,
        }
    }
    
    pub fn get_or_create<I>(&self, key: &str, chars: I) -> CharacterBloomFilter
    where
        I: IntoIterator<Item = char>,
    {
        // Try to get from cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(filter) = cache.get(key) {
                return filter.clone();
            }
        }
        
        // Create new filter
        let filter = CharacterBloomFilter::from_chars(chars);
        
        // Insert into cache
        {
            let mut cache = self.cache.write().unwrap();
            if cache.len() < self.capacity {
                cache.insert(key.to_string(), filter.clone());
            }
        }
        
        filter
    }
}

/// Case conversion types
#[derive(Debug, Clone, Copy)]
pub enum CaseConversion {
    ToUpper,
    ToLower,
    ToTitle,
}

/// Character class types for fast classification
#[derive(Debug, Clone, Copy)]
pub enum CharacterClass {
    Alphabetic,
    Numeric,
    Alphanumeric,
    Whitespace,
    Punctuation,
    UpperCase,
    LowerCase,
    Control,
    Graphic,
}

/// SIMD character operations engine
#[derive(Debug)]
pub struct SIMDCharacterOps {
    ascii_lookup_tables: ASCIILookupTables,
    bloom_filters: CharacterBloomFilterCache,
}

impl SIMDCharacterOps {
    pub fn new() -> Self {
        Self {
            ascii_lookup_tables: ASCIILookupTables::new(),
            bloom_filters: CharacterBloomFilterCache::new(64),
        }
    }
    
    /// SIMD-optimized case conversion
    pub fn simd_case_conversion(&self, text: &str, conversion: CaseConversion) -> String {
        if text.is_ascii() {
            // Fast ASCII path with vectorized processing
            let bytes = text.as_bytes();
            let mut result = Vec::with_capacity(bytes.len());
            
            // Process 8 bytes at a time (simulated SIMD)
            for chunk in bytes.chunks(8) {
                let mut converted_chunk = [0u8; 8];
                
                for (i, &byte) in chunk.iter().enumerate() {
                    converted_chunk[i] = match conversion {
                        CaseConversion::ToUpper => self.ascii_lookup_tables.to_uppercase_byte(byte),
                        CaseConversion::ToLower => self.ascii_lookup_tables.to_lowercase_byte(byte),
                        CaseConversion::ToTitle => {
                            // Title case: first char uppercase, rest lowercase
                            if i == 0 || chunk.get(i.saturating_sub(1)).map_or(true, |&b| (b as char).is_whitespace()) {
                                self.ascii_lookup_tables.to_uppercase_byte(byte)
                            } else {
                                self.ascii_lookup_tables.to_lowercase_byte(byte)
                            }
                        }
                    };
                }
                
                result.extend_from_slice(&converted_chunk[..chunk.len()]);
            }
            
            String::from_utf8(result).unwrap()
        } else {
            // Unicode fallback
            match conversion {
                CaseConversion::ToUpper => text.to_uppercase(),
                CaseConversion::ToLower => text.to_lowercase(),
                CaseConversion::ToTitle => {
                    let mut result = String::new();
                    let mut capitalize_next = true;
                    
                    for ch in text.chars() {
                        if ch.is_whitespace() {
                            result.push(ch);
                            capitalize_next = true;
                        } else if capitalize_next {
                            result.extend(ch.to_uppercase());
                            capitalize_next = false;
                        } else {
                            result.extend(ch.to_lowercase());
                        }
                    }
                    
                    result
                }
            }
        }
    }
    
    /// SIMD character classification
    pub fn simd_character_classification(&self, text: &str, char_class: CharacterClass) -> Vec<bool> {
        let mut results = Vec::with_capacity(text.len());
        
        if text.is_ascii() {
            // Fast ASCII path
            let bytes = text.as_bytes();
            
            for &byte in bytes {
                let flags = self.ascii_lookup_tables.get_char_class(byte);
                let matches = match char_class {
                    CharacterClass::Alphabetic => flags.is_alphabetic(),
                    CharacterClass::Numeric => flags.is_numeric(),
                    CharacterClass::Alphanumeric => flags.is_alphabetic() || flags.is_numeric(),
                    CharacterClass::Whitespace => flags.is_whitespace(),
                    CharacterClass::Punctuation => flags.is_punctuation(),
                    CharacterClass::UpperCase => flags.is_upper_case(),
                    CharacterClass::LowerCase => flags.is_lower_case(),
                    CharacterClass::Control => flags.is_control(),
                    CharacterClass::Graphic => flags.is_graphic(),
                };
                results.push(matches);
            }
        } else {
            // Unicode fallback
            for ch in text.chars() {
                let matches = match char_class {
                    CharacterClass::Alphabetic => ch.is_alphabetic(),
                    CharacterClass::Numeric => ch.is_numeric(),
                    CharacterClass::Alphanumeric => ch.is_alphanumeric(),
                    CharacterClass::Whitespace => ch.is_whitespace(),
                    CharacterClass::Punctuation => ch.is_ascii_punctuation(), // Simplified
                    CharacterClass::UpperCase => ch.is_uppercase(),
                    CharacterClass::LowerCase => ch.is_lowercase(),
                    CharacterClass::Control => ch.is_control(),
                    CharacterClass::Graphic => !ch.is_whitespace() && !ch.is_control(),
                };
                results.push(matches);
            }
        }
        
        results
    }
    
    /// Multi-character search with bloom filter optimization
    pub fn simd_multi_char_search(&self, text: &str, targets: &[char]) -> Vec<usize> {
        let mut positions = Vec::new();
        
        if targets.is_empty() {
            return positions;
        }
        
        // Create bloom filter for targets
        let target_str = targets.iter().collect::<String>();
        let bloom_filter = self.bloom_filters.get_or_create(&target_str, targets.iter().copied());
        
        // Search with bloom filter pre-filtering
        for (pos, ch) in text.chars().enumerate() {
            if bloom_filter.may_contain(ch) {
                // Bloom filter says it might be present, check explicitly
                if targets.contains(&ch) {
                    positions.push(pos);
                }
            }
        }
        
        positions
    }
    
    /// Count character occurrences with SIMD acceleration
    pub fn simd_char_count(&self, text: &str, target: char) -> usize {
        if target.is_ascii() && text.is_ascii() {
            // Fast ASCII path
            let target_byte = target as u8;
            let bytes = text.as_bytes();
            let mut count = 0;
            
            // Process 8 bytes at a time
            for chunk in bytes.chunks(8) {
                for &byte in chunk {
                    if byte == target_byte {
                        count += 1;
                    }
                }
            }
            
            count
        } else {
            // Unicode fallback
            text.chars().filter(|&c| c == target).count()
        }
    }
    
    /// Fast string comparison ignoring case
    pub fn simd_case_insensitive_compare(&self, s1: &str, s2: &str) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        
        if s1.is_ascii() && s2.is_ascii() {
            // Fast ASCII path
            let bytes1 = s1.as_bytes();
            let bytes2 = s2.as_bytes();
            
            let min_len = std::cmp::min(bytes1.len(), bytes2.len());
            
            for i in 0..min_len {
                let c1 = self.ascii_lookup_tables.to_lowercase_byte(bytes1[i]);
                let c2 = self.ascii_lookup_tables.to_lowercase_byte(bytes2[i]);
                
                match c1.cmp(&c2) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            
            bytes1.len().cmp(&bytes2.len())
        } else {
            // Unicode fallback
            s1.to_lowercase().cmp(&s2.to_lowercase())
        }
    }
    
    /// Vectorized character replacement
    pub fn simd_char_replace(&self, text: &str, from: char, to: char) -> String {
        if from.is_ascii() && to.is_ascii() && text.is_ascii() {
            // Fast ASCII path
            let from_byte = from as u8;
            let to_byte = to as u8;
            let bytes = text.as_bytes();
            let mut result = Vec::with_capacity(bytes.len());
            
            for &byte in bytes {
                result.push(if byte == from_byte { to_byte } else { byte });
            }
            
            String::from_utf8(result).unwrap()
        } else {
            // Unicode fallback
            text.chars().map(|c| if c == from { to } else { c }).collect()
        }
    }
}

impl Default for SIMDCharacterOps {
    fn default() -> Self {
        Self::new()
    }
}

/// Global SIMD character operations instance
lazy_static::lazy_static! {
    static ref GLOBAL_SIMD_CHARS: SIMDCharacterOps = SIMDCharacterOps::new();
}

/// Enhanced case conversion using SIMD
pub fn enhanced_string_upcase(text: &str) -> String {
    GLOBAL_SIMD_CHARS.simd_case_conversion(text, CaseConversion::ToUpper)
}

/// Enhanced case conversion using SIMD
pub fn enhanced_string_downcase(text: &str) -> String {
    GLOBAL_SIMD_CHARS.simd_case_conversion(text, CaseConversion::ToLower)
}

/// Enhanced title case conversion
pub fn enhanced_string_titlecase(text: &str) -> String {
    GLOBAL_SIMD_CHARS.simd_case_conversion(text, CaseConversion::ToTitle)
}

/// Enhanced character search with SIMD
pub fn enhanced_multi_char_search(text: &str, targets: &[char]) -> Vec<usize> {
    GLOBAL_SIMD_CHARS.simd_multi_char_search(text, targets)
}

/// Enhanced case-insensitive comparison
pub fn enhanced_case_insensitive_compare(s1: &str, s2: &str) -> std::cmp::Ordering {
    GLOBAL_SIMD_CHARS.simd_case_insensitive_compare(s1, s2)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ascii_lookup_tables() {
        let tables = ASCIILookupTables::new();
        
        // Test case conversion
        assert_eq!(tables.to_uppercase_byte(b'a'), b'A');
        assert_eq!(tables.to_lowercase_byte(b'A'), b'a');
        assert_eq!(tables.to_uppercase_byte(b'A'), b'A'); // No change
        assert_eq!(tables.to_lowercase_byte(b'a'), b'a'); // No change
        
        // Test character classification
        let flags = tables.get_char_class(b'A');
        assert!(flags.is_alphabetic());
        assert!(flags.is_upper_case());
        assert!(!flags.is_numeric());
        
        let flags = tables.get_char_class(b'5');
        assert!(flags.is_numeric());
        assert!(!flags.is_alphabetic());
    }
    
    #[test]
    fn test_bloom_filter() {
        let mut filter = CharacterBloomFilter::new();
        
        filter.insert('a');
        filter.insert('b');
        filter.insert('c');
        
        assert!(filter.may_contain('a'));
        assert!(filter.may_contain('b'));
        assert!(filter.may_contain('c'));
        
        // May have false positives but no false negatives
        // So we can't test that 'd' is definitely not present
    }
    
    #[test]
    fn test_simd_case_conversion() {
        let ops = SIMDCharacterOps::new();
        
        // ASCII case conversion
        assert_eq!(ops.simd_case_conversion("hello", CaseConversion::ToUpper), "HELLO");
        assert_eq!(ops.simd_case_conversion("WORLD", CaseConversion::ToLower), "world");
        assert_eq!(ops.simd_case_conversion("hello world", CaseConversion::ToTitle), "Hello World");
        
        // Mixed case
        assert_eq!(ops.simd_case_conversion("HeLLo", CaseConversion::ToUpper), "HELLO");
        assert_eq!(ops.simd_case_conversion("HeLLo", CaseConversion::ToLower), "hello");
    }
    
    #[test]
    fn test_simd_character_classification() {
        let ops = SIMDCharacterOps::new();
        
        let results = ops.simd_character_classification("abc123", CharacterClass::Alphabetic);
        assert_eq!(results, vec![true, true, true, false, false, false]);
        
        let results = ops.simd_character_classification("abc123", CharacterClass::Numeric);
        assert_eq!(results, vec![false, false, false, true, true, true]);
        
        let results = ops.simd_character_classification("Abc", CharacterClass::UpperCase);
        assert_eq!(results, vec![true, false, false]);
    }
    
    #[test]
    fn test_simd_multi_char_search() {
        let ops = SIMDCharacterOps::new();
        
        let positions = ops.simd_multi_char_search("hello world", &['l', 'o']);
        assert_eq!(positions, vec![2, 3, 4, 7]); // positions of 'l' and 'o'
        
        let positions = ops.simd_multi_char_search("hello world", &['x', 'y']);
        assert_eq!(positions, Vec::<usize>::new()); // No matches
    }
    
    #[test]
    fn test_enhanced_functions() {
        assert_eq!(enhanced_string_upcase("hello"), "HELLO");
        assert_eq!(enhanced_string_downcase("WORLD"), "world");
        assert_eq!(enhanced_string_titlecase("hello world"), "Hello World");
        
        let positions = enhanced_multi_char_search("test", &['t', 'e']);
        assert_eq!(positions, vec![0, 1, 3]); // positions of 't' and 'e'
        
        use std::cmp::Ordering;
        assert_eq!(enhanced_case_insensitive_compare("Hello", "hello"), Ordering::Equal);
        assert_eq!(enhanced_case_insensitive_compare("abc", "def"), Ordering::Less);
    }
    
    #[test]
    fn test_char_count() {
        let ops = SIMDCharacterOps::new();
        
        assert_eq!(ops.simd_char_count("hello world", 'l'), 3);
        assert_eq!(ops.simd_char_count("hello world", 'x'), 0);
        assert_eq!(ops.simd_char_count("aaaaaa", 'a'), 6);
    }
    
    #[test]
    fn test_char_replace() {
        let ops = SIMDCharacterOps::new();
        
        assert_eq!(ops.simd_char_replace("hello world", 'l', 'x'), "hexxo worxd");
        assert_eq!(ops.simd_char_replace("test", 'x', 'y'), "test"); // No change
        assert_eq!(ops.simd_char_replace("aaaa", 'a', 'b'), "bbbb");
    }
}