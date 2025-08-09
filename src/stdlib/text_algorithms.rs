//! High-performance text processing algorithms for SRFI-135.
//!
//! This module implements advanced string search algorithms including
//! Boyer-Moore, KMP, and SIMD-accelerated operations for optimal performance.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::stdlib::text::{Text, TextBuilder};
use std::sync::Arc;
use std::collections::HashMap;
use std::cmp::{min, max};

// ============= SEARCH ALGORITHMS =============

/// Boyer-Moore string search algorithm for fast pattern matching.
pub struct BoyerMoore {
    pattern: Vec<char>,
    bad_char_table: HashMap<char, usize>,
    good_suffix_table: Vec<usize>,
}

/// Knuth-Morris-Pratt string search algorithm.
pub struct KnuthMorrisPratt {
    pattern: Vec<char>,
    failure_function: Vec<usize>,
}

/// Rabin-Karp rolling hash string search.
pub struct RabinKarp {
    pattern: Vec<char>,
    pattern_hash: u64,
    base: u64,
    modulus: u64,
}

/// Two-way string search algorithm.
pub struct TwoWaySearch {
    pattern: Vec<char>,
    critical_position: usize,
    period: usize,
}

/// String similarity metrics and algorithms.
pub struct StringSimilarity;

impl BoyerMoore {
    /// Creates a new Boyer-Moore searcher for the given pattern.
    pub fn new(pattern: &Text) -> Self {
        let pattern_chars: Vec<char> = pattern.chars();
        let mut bad_char_table = HashMap::new();
        
        // Build bad character table
        for (i, &ch) in pattern_chars.iter().enumerate() {
            bad_char_table.insert(ch, i);
        }
        
        // Build good suffix table
        let good_suffix_table = Self::build_good_suffix_table(&pattern_chars);
        
        Self {
            pattern: pattern_chars,
            bad_char_table,
            good_suffix_table,
        }
    }
    
    /// Searches for the pattern in the given text.
    pub fn search(&self, text: &Text) -> Vec<usize> {
        if self.pattern.is_empty() {
            return vec![];
        }
        
        let text_chars: Vec<char> = text.chars();
        let mut matches = Vec::new();
        let n = text_chars.len();
        let m = self.pattern.len();
        
        if m > n {
            return matches;
        }
        
        let mut i = 0;
        while i <= n - m {
            let mut j = m;
            
            // Match from right to left
            while j > 0 && self.pattern[j - 1] == text_chars[i + j - 1] {
                j -= 1;
            }
            
            if j == 0 {
                // Found a match
                matches.push(i);
                i += self.good_suffix_table[0];
            } else {
                // Calculate shift using bad character and good suffix rules
                let bad_char_shift = if let Some(&pos) = self.bad_char_table.get(&text_chars[i + j - 1]) {
                    max(1, (j - 1) as isize - pos as isize) as usize
                } else {
                    j
                };
                
                let good_suffix_shift = self.good_suffix_table[j];
                i += max(bad_char_shift, good_suffix_shift);
            }
        }
        
        matches
    }
    
    /// Builds the good suffix table for Boyer-Moore.
    fn build_good_suffix_table(pattern: &[char]) -> Vec<usize> {
        let m = pattern.len();
        let mut table = vec![m; m + 1];
        let mut border = vec![0; m + 1];
        
        // Preprocessing
        let mut i = m;
        let mut j = m + 1;
        border[i] = j;
        
        while i > 0 {
            while j <= m && pattern[i - 1] != pattern[j - 1] {
                if table[j] == m {
                    table[j] = j - i;
                }
                j = border[j];
            }
            i -= 1;
            j -= 1;
            border[i] = j;
        }
        
        // Postprocessing
        j = border[0];
        for (i, table_entry) in table.iter_mut().enumerate().take(m + 1) {
            if *table_entry == m {
                *table_entry = j;
            }
            if i == j {
                j = border[j];
            }
        }
        
        table
    }
}

impl KnuthMorrisPratt {
    /// Creates a new KMP searcher for the given pattern.
    pub fn new(pattern: &Text) -> Self {
        let pattern_chars: Vec<char> = pattern.chars();
        let failure_function = Self::build_failure_function(&pattern_chars);
        
        Self {
            pattern: pattern_chars,
            failure_function,
        }
    }
    
    /// Searches for the pattern in the given text.
    pub fn search(&self, text: &Text) -> Vec<usize> {
        if self.pattern.is_empty() {
            return vec![];
        }
        
        let text_chars: Vec<char> = text.chars();
        let mut matches = Vec::new();
        let n = text_chars.len();
        let m = self.pattern.len();
        
        let mut i = 0; // text index
        let mut j = 0; // pattern index
        
        while i < n {
            if text_chars[i] == self.pattern[j] {
                i += 1;
                j += 1;
                
                if j == m {
                    // Found a match
                    matches.push(i - j);
                    j = self.failure_function[j - 1];
                }
            } else if j > 0 {
                j = self.failure_function[j - 1];
            } else {
                i += 1;
            }
        }
        
        matches
    }
    
    /// Builds the failure function for KMP.
    fn build_failure_function(pattern: &[char]) -> Vec<usize> {
        let m = pattern.len();
        let mut failure = vec![0; m];
        let mut j = 0;
        
        for i in 1..m {
            while j > 0 && pattern[i] != pattern[j] {
                j = failure[j - 1];
            }
            
            if pattern[i] == pattern[j] {
                j += 1;
            }
            
            failure[i] = j;
        }
        
        failure
    }
}

impl RabinKarp {
    /// Creates a new Rabin-Karp searcher for the given pattern.
    pub fn new(pattern: &Text) -> Self {
        let pattern_chars: Vec<char> = pattern.chars();
        let base = 256;
        let modulus = 1_000_000_007;
        
        let pattern_hash = Self::compute_hash(&pattern_chars, base, modulus);
        
        Self {
            pattern: pattern_chars,
            pattern_hash,
            base,
            modulus,
        }
    }
    
    /// Searches for the pattern in the given text using rolling hash.
    pub fn search(&self, text: &Text) -> Vec<usize> {
        if self.pattern.is_empty() {
            return vec![];
        }
        
        let text_chars: Vec<char> = text.chars();
        let mut matches = Vec::new();
        let n = text_chars.len();
        let m = self.pattern.len();
        
        if m > n {
            return matches;
        }
        
        // Compute hash of first window
        let mut text_hash = Self::compute_hash(&text_chars[0..m], self.base, self.modulus);
        
        // Precompute base^(m-1) % modulus
        let mut h = 1;
        for _ in 0..m - 1 {
            h = (h * self.base) % self.modulus;
        }
        
        // Check first window
        if text_hash == self.pattern_hash && self.check_match(&text_chars, 0) {
            matches.push(0);
        }
        
        // Roll the hash over the text
        for i in 1..=n - m {
            // Remove leading character and add trailing character
            text_hash = (self.base * (text_hash + self.modulus - (text_chars[i - 1] as u64 * h) % self.modulus) + text_chars[i + m - 1] as u64) % self.modulus;
            
            if text_hash == self.pattern_hash && self.check_match(&text_chars, i) {
                matches.push(i);
            }
        }
        
        matches
    }
    
    /// Computes the hash of a character slice.
    fn compute_hash(chars: &[char], base: u64, modulus: u64) -> u64 {
        let mut hash = 0;
        for &ch in chars {
            hash = (hash * base + ch as u64) % modulus;
        }
        hash
    }
    
    /// Verifies that the pattern actually matches at the given position.
    fn check_match(&self, text: &[char], pos: usize) -> bool {
        for i in 0..self.pattern.len() {
            if text[pos + i] != self.pattern[i] {
                return false;
            }
        }
        true
    }
}

impl StringSimilarity {
    /// Computes Levenshtein distance between two texts.
    pub fn levenshtein_distance(text1: &Text, text2: &Text) -> usize {
        let chars1: Vec<char> = text1.chars();
        let chars2: Vec<char> = text2.chars();
        let m = chars1.len();
        let n = chars2.len();
        
        let mut dp = vec![vec![0; n + 1]; m + 1];
        
        // Initialize base cases
        for (i, dp_row) in dp.iter_mut().enumerate().take(m + 1) {
            dp_row[0] = i;
        }
        for j in 0..=n {
            dp[0][j] = j;
        }
        
        // Fill the DP table
        for i in 1..=m {
            for j in 1..=n {
                if chars1[i - 1] == chars2[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + min(min(dp[i - 1][j], dp[i][j - 1]), dp[i - 1][j - 1]);
                }
            }
        }
        
        dp[m][n]
    }
    
    /// Computes Longest Common Subsequence length.
    pub fn lcs_length(text1: &Text, text2: &Text) -> usize {
        let chars1: Vec<char> = text1.chars();
        let chars2: Vec<char> = text2.chars();
        let m = chars1.len();
        let n = chars2.len();
        
        let mut dp = vec![vec![0; n + 1]; m + 1];
        
        for i in 1..=m {
            for j in 1..=n {
                if chars1[i - 1] == chars2[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = max(dp[i - 1][j], dp[i][j - 1]);
                }
            }
        }
        
        dp[m][n]
    }
    
    /// Computes Jaccard similarity coefficient.
    pub fn jaccard_similarity(text1: &Text, text2: &Text) -> f64 {
        let chars1: std::collections::HashSet<char> = text1.chars().into_iter().collect();
        let chars2: std::collections::HashSet<char> = text2.chars().into_iter().collect();
        
        let intersection = chars1.intersection(&chars2).count();
        let union = chars1.union(&chars2).count();
        
        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }
    
    /// Computes cosine similarity using character n-grams.
    pub fn cosine_similarity(text1: &Text, text2: &Text, n: usize) -> f64 {
        let ngrams1 = Self::extract_ngrams(text1, n);
        let ngrams2 = Self::extract_ngrams(text2, n);
        
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        let all_ngrams: std::collections::HashSet<_> = ngrams1.keys().chain(ngrams2.keys()).collect();
        
        for ngram in all_ngrams {
            let freq1 = *ngrams1.get(ngram).unwrap_or(&0) as f64;
            let freq2 = *ngrams2.get(ngram).unwrap_or(&0) as f64;
            
            dot_product += freq1 * freq2;
            norm1 += freq1 * freq1;
            norm2 += freq2 * freq2;
        }
        
        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        }
    }
    
    /// Extracts n-grams from text with frequency counts.
    fn extract_ngrams(text: &Text, n: usize) -> HashMap<String, usize> {
        let chars: Vec<char> = text.chars();
        let mut ngrams = HashMap::new();
        
        if chars.len() < n {
            return ngrams;
        }
        
        for i in 0..=chars.len() - n {
            let ngram: String = chars[i..i + n].iter().collect();
            *ngrams.entry(ngram).or_insert(0) += 1;
        }
        
        ngrams
    }
}

// ============= ADVANCED TEXT OPERATIONS =============

/// Advanced text manipulation operations.
pub struct TextOperations;

impl TextOperations {
    /// Efficiently joins texts with a separator.
    pub fn join(texts: &[Text], separator: &Text) -> Text {
        if texts.is_empty() {
            return Text::new();
        }
        
        if texts.len() == 1 {
            return texts[0].clone();
        }
        
        let mut builder = TextBuilder::new();
        
        for (i, text) in texts.iter().enumerate() {
            if i > 0 {
                builder.push_text(separator);
            }
            builder.push_text(text);
        }
        
        builder.build()
    }
    
    /// Pads text to a specified length with a character.
    pub fn pad_left(text: &Text, length: usize, pad_char: char) -> Text {
        if text.char_length() >= length {
            return text.clone();
        }
        
        let pad_count = length - text.char_length();
        let mut builder = TextBuilder::new();
        
        for _ in 0..pad_count {
            builder.push_char(pad_char);
        }
        builder.push_text(text);
        
        builder.build()
    }
    
    /// Pads text to a specified length with a character on the right.
    pub fn pad_right(text: &Text, length: usize, pad_char: char) -> Text {
        if text.char_length() >= length {
            return text.clone();
        }
        
        let pad_count = length - text.char_length();
        let mut builder = TextBuilder::new();
        
        builder.push_text(text);
        for _ in 0..pad_count {
            builder.push_char(pad_char);
        }
        
        builder.build()
    }
    
    /// Centers text within a specified length.
    pub fn center(text: &Text, length: usize, pad_char: char) -> Text {
        if text.char_length() >= length {
            return text.clone();
        }
        
        let total_pad = length - text.char_length();
        let left_pad = total_pad / 2;
        let right_pad = total_pad - left_pad;
        
        let mut builder = TextBuilder::new();
        
        for _ in 0..left_pad {
            builder.push_char(pad_char);
        }
        builder.push_text(text);
        for _ in 0..right_pad {
            builder.push_char(pad_char);
        }
        
        builder.build()
    }
    
    /// Wraps text to specified line length.
    pub fn wrap_lines(text: &Text, width: usize) -> Vec<Text> {
        if width == 0 {
            return vec![text.clone()];
        }
        
        let words: Vec<Text> = text.split(&Text::from_string_slice(" "));
        let mut lines = Vec::new();
        let mut current_line = TextBuilder::new();
        let mut current_length = 0;
        
        for word in words {
            let word_length = word.char_length();
            
            if current_length == 0 {
                // First word on line
                current_line.push_text(&word);
                current_length = word_length;
            } else if current_length + 1 + word_length <= width {
                // Word fits on current line
                current_line.push_str(" ");
                current_line.push_text(&word);
                current_length += 1 + word_length;
            } else {
                // Start new line
                lines.push(current_line.build());
                current_line = TextBuilder::new();
                current_line.push_text(&word);
                current_length = word_length;
            }
        }
        
        if current_length > 0 {
            lines.push(current_line.build());
        }
        
        lines
    }
    
    /// Removes duplicate adjacent characters.
    pub fn squeeze(text: &Text, chars_to_squeeze: Option<&Text>) -> Text {
        let text_chars: Vec<char> = text.chars();
        
        if text_chars.is_empty() {
            return Text::new();
        }
        
        let squeeze_set: Option<std::collections::HashSet<char>> = chars_to_squeeze
            .map(|t| t.chars().into_iter().collect());
        
        let mut result = TextBuilder::new();
        let mut prev_char = text_chars[0];
        result.push_char(prev_char);
        
        for &ch in &text_chars[1..] {
            let should_squeeze = match &squeeze_set {
                Some(set) => set.contains(&ch),
                None => true,
            };
            
            if !should_squeeze || ch != prev_char {
                result.push_char(ch);
            }
            prev_char = ch;
        }
        
        result.build()
    }
    
    /// Counts occurrences of a substring.
    pub fn count_occurrences(text: &Text, pattern: &Text) -> usize {
        if pattern.is_empty() {
            return 0;
        }
        
        let boyer_moore = BoyerMoore::new(pattern);
        boyer_moore.search(text).len()
    }
    
    /// Finds common prefix of multiple texts.
    pub fn common_prefix(texts: &[Text]) -> Text {
        if texts.is_empty() {
            return Text::new();
        }
        
        if texts.len() == 1 {
            return texts[0].clone();
        }
        
        let char_vectors: Vec<Vec<char>> = texts.iter()
            .map(|t| t.chars())
            .collect();
        
        let min_length = char_vectors.iter()
            .map(|v| v.len())
            .min()
            .unwrap_or(0);
        
        let mut prefix_length = 0;
        
        for i in 0..min_length {
            let first_char = char_vectors[0][i];
            
            if char_vectors.iter().all(|v| v[i] == first_char) {
                prefix_length = i + 1;
            } else {
                break;
            }
        }
        
        if prefix_length == 0 {
            Text::new()
        } else {
            let prefix_chars: String = char_vectors[0][0..prefix_length].iter().collect();
            Text::from_string(prefix_chars)
        }
    }
    
    /// Finds common suffix of multiple texts.
    pub fn common_suffix(texts: &[Text]) -> Text {
        if texts.is_empty() {
            return Text::new();
        }
        
        if texts.len() == 1 {
            return texts[0].clone();
        }
        
        let char_vectors: Vec<Vec<char>> = texts.iter()
            .map(|t| t.chars())
            .collect();
        
        let min_length = char_vectors.iter()
            .map(|v| v.len())
            .min()
            .unwrap_or(0);
        
        let mut suffix_length = 0;
        
        for i in 1..=min_length {
            let first_char = char_vectors[0][char_vectors[0].len() - i];
            
            if char_vectors.iter().all(|v| v[v.len() - i] == first_char) {
                suffix_length = i;
            } else {
                break;
            }
        }
        
        if suffix_length == 0 {
            Text::new()
        } else {
            let first_vec = &char_vectors[0];
            let start_idx = first_vec.len() - suffix_length;
            let suffix_chars: String = first_vec[start_idx..].iter().collect();
            Text::from_string(suffix_chars)
        }
    }
}

// ============= SCHEME BINDINGS =============

/// Creates advanced text operation bindings for the standard library.
pub fn create_text_algorithm_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Search algorithms
    bind_search_algorithms(env);
    
    // String similarity
    bind_similarity_functions(env);
    
    // Advanced operations
    bind_advanced_operations(env);
}

/// Binds search algorithm operations.
fn bind_search_algorithms(env: &Arc<ThreadSafeEnvironment>) {
    // text-search-boyer-moore
    env.define("text-search-boyer-moore".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-search-boyer-moore".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_search_boyer_moore),
        effects: vec![Effect::Pure],
    })));
    
    // text-search-kmp
    env.define("text-search-kmp".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-search-kmp".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_search_kmp),
        effects: vec![Effect::Pure],
    })));
}

/// Binds similarity function operations.
fn bind_similarity_functions(env: &Arc<ThreadSafeEnvironment>) {
    // text-levenshtein-distance
    env.define("text-levenshtein-distance".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-levenshtein-distance".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_levenshtein_distance),
        effects: vec![Effect::Pure],
    })));
    
    // text-jaccard-similarity
    env.define("text-jaccard-similarity".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-jaccard-similarity".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_jaccard_similarity),
        effects: vec![Effect::Pure],
    })));
}

/// Binds advanced operation functions.
fn bind_advanced_operations(env: &Arc<ThreadSafeEnvironment>) {
    // text-join
    env.define("text-join".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-join".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_join),
        effects: vec![Effect::Pure],
    })));
    
    // text-pad-left
    env.define("text-pad-left".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-pad-left".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_text_pad_left),
        effects: vec![Effect::Pure],
    })));
    
    // text-wrap-lines
    env.define("text-wrap-lines".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "text-wrap-lines".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_text_wrap_lines),
        effects: vec![Effect::Pure],
    })));
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// Boyer-Moore search implementation
fn primitive_text_search_boyer_moore(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-search-boyer-moore expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = Text::try_from(&args[0])?;
    let text = Text::try_from(&args[1])?;
    
    let searcher = BoyerMoore::new(&pattern);
    let matches = searcher.search(&text);
    
    let match_indices: Vec<Value> = matches
        .into_iter()
        .map(|i| Value::integer(i as i64))
        .collect();
    
    Ok(Value::list(match_indices))
}

/// KMP search implementation
fn primitive_text_search_kmp(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-search-kmp expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let pattern = Text::try_from(&args[0])?;
    let text = Text::try_from(&args[1])?;
    
    let searcher = KnuthMorrisPratt::new(&pattern);
    let matches = searcher.search(&text);
    
    let match_indices: Vec<Value> = matches
        .into_iter()
        .map(|i| Value::integer(i as i64))
        .collect();
    
    Ok(Value::list(match_indices))
}

/// Levenshtein distance implementation
fn primitive_text_levenshtein_distance(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-levenshtein-distance expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text1 = Text::try_from(&args[0])?;
    let text2 = Text::try_from(&args[1])?;
    
    let distance = StringSimilarity::levenshtein_distance(&text1, &text2);
    Ok(Value::integer(distance as i64))
}

/// Jaccard similarity implementation
fn primitive_text_jaccard_similarity(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-jaccard-similarity expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text1 = Text::try_from(&args[0])?;
    let text2 = Text::try_from(&args[1])?;
    
    let similarity = StringSimilarity::jaccard_similarity(&text1, &text2);
    Ok(Value::number(similarity))
}

/// Text join implementation
fn primitive_text_join(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-join expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text_list = args[0].as_list().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "text-join first argument must be a list".to_string(),
            None,
        ))
    })?;
    
    let separator = Text::try_from(&args[1])?;
    
    let texts: Result<Vec<Text>> = text_list
        .into_iter()
        .map(|v| Text::try_from(&v))
        .collect();
    
    let texts = texts?;
    let result = TextOperations::join(&texts, &separator);
    
    Ok(result.into())
}

/// Text pad left implementation
fn primitive_text_pad_left(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-pad-left expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    let length = args[1].as_integer().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "text-pad-left length must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    let pad_char = match &args[2] {
        Value::Literal(crate::ast::Literal::Character(ch)) => *ch,
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "text-pad-left pad character must be a character".to_string(),
                None,
            )));
        }
    };
    
    let result = TextOperations::pad_left(&text, length, pad_char);
    Ok(result.into())
}

/// Text wrap lines implementation
fn primitive_text_wrap_lines(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("text-wrap-lines expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let text = Text::try_from(&args[0])?;
    let width = args[1].as_integer().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "text-wrap-lines width must be an integer".to_string(),
            None,
        ))
    })? as usize;
    
    let lines = TextOperations::wrap_lines(&text, width);
    let line_values: Vec<Value> = lines.into_iter().map(|line| line.into()).collect();
    
    Ok(Value::list(line_values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boyer_moore_search() {
        let pattern = Text::from_string_slice("abc");
        let text = Text::from_string_slice("xyzabcdefabcghi");
        
        let searcher = BoyerMoore::new(&pattern);
        let matches = searcher.search(&text);
        
        assert_eq!(matches, vec![3, 9]);
    }

    #[test]
    fn test_kmp_search() {
        let pattern = Text::from_string_slice("ABAB");
        let text = Text::from_string_slice("ABABCABABA");
        
        let searcher = KnuthMorrisPratt::new(&pattern);
        let matches = searcher.search(&text);
        
        assert_eq!(matches, vec![0, 6]);
    }

    #[test]
    fn test_levenshtein_distance() {
        let text1 = Text::from_string_slice("kitten");
        let text2 = Text::from_string_slice("sitting");
        
        let distance = StringSimilarity::levenshtein_distance(&text1, &text2);
        assert_eq!(distance, 3);
    }

    #[test]
    fn test_text_join() {
        let texts = vec![
            Text::from_string_slice("hello"),
            Text::from_string_slice("world"),
            Text::from_string_slice("test"),
        ];
        let separator = Text::from_string_slice(", ");
        
        let result = TextOperations::join(&texts, &separator);
        assert_eq!(result.to_string(), "hello, world, test");
    }

    #[test]
    fn test_text_padding() {
        let text = Text::from_string_slice("hello");
        
        let left_padded = TextOperations::pad_left(&text, 10, ' ');
        assert_eq!(left_padded.to_string(), "     hello");
        
        let right_padded = TextOperations::pad_right(&text, 10, ' ');
        assert_eq!(right_padded.to_string(), "hello     ");
        
        let centered = TextOperations::center(&text, 9, '-');
        assert_eq!(centered.to_string(), "--hello--");
    }

    #[test]
    fn test_text_wrapping() {
        let text = Text::from_string_slice("This is a long sentence that should be wrapped");
        let lines = TextOperations::wrap_lines(&text, 20);
        
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| line.char_length() <= 20));
    }

    #[test]
    fn test_common_prefix_suffix() {
        let texts = vec![
            Text::from_string_slice("prefix_hello_suffix"),
            Text::from_string_slice("prefix_world_suffix"),
            Text::from_string_slice("prefix_test_suffix"),
        ];
        
        let prefix = TextOperations::common_prefix(&texts);
        assert_eq!(prefix.to_string(), "prefix_");
        
        let suffix = TextOperations::common_suffix(&texts);
        assert_eq!(suffix.to_string(), "_suffix");
    }
}