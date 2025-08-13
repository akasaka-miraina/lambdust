//! パフォーマンス最適化モジュール
//!
//! このモジュールは高性能なパーサー最適化を提供します：
//! - SIMD最適化（文字分類、パターンマッチング）
//! - 並列化（独立した部分の同時処理）
//! - キャッシュ最適化（メモ化、局所性改善）
//! - ゼロコピー最適化（不必要なアロケーション削除）

use super::types::*;
use super::combinator::*;
use super::scheme::*;
use crate::diagnostics::Span;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// SIMD最適化されたパーサープリミティブ
pub struct SIMDPrimitives;

#[cfg(target_arch = "x86_64")]
impl SIMDPrimitives {
    /// SIMD最適化された文字分類
    /// ASCII文字の分類を並列実行
    pub fn simd_classify_chars(input: &str) -> Vec<CharClass> {
        use std::arch::x86_64::*;
        
        let mut result = Vec::with_capacity(input.len());
        let bytes = input.as_bytes();
        
        // 16バイトずつ処理
        let chunks = bytes.chunks_exact(16);
        let remainder = chunks.remainder();
        
        unsafe {
            for chunk in chunks {
                // 16バイトを128bitレジスタにロード
                let data = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
                
                // 数字の判定 ('0'-'9')
                let is_digit = _mm_and_si128(
                    _mm_cmpgt_epi8(data, _mm_set1_epi8('0' as i8 - 1)),
                    _mm_cmplt_epi8(data, _mm_set1_epi8('9' as i8 + 1))
                );
                
                // 英字の判定 ('a'-'z', 'A'-'Z')
                let is_lower = _mm_and_si128(
                    _mm_cmpgt_epi8(data, _mm_set1_epi8('a' as i8 - 1)),
                    _mm_cmplt_epi8(data, _mm_set1_epi8('z' as i8 + 1))
                );
                let is_upper = _mm_and_si128(
                    _mm_cmpgt_epi8(data, _mm_set1_epi8('A' as i8 - 1)),
                    _mm_cmplt_epi8(data, _mm_set1_epi8('Z' as i8 + 1))
                );
                let is_alpha = _mm_or_si128(is_lower, is_upper);
                
                // 空白文字の判定
                let is_space = _mm_or_si128(
                    _mm_cmpeq_epi8(data, _mm_set1_epi8(' ' as i8)),
                    _mm_cmpeq_epi8(data, _mm_set1_epi8('\t' as i8))
                );
                
                // 結果を個別に処理
                for i in 0..16 {
                    let digit_mask = _mm_extract_epi8(is_digit, i) != 0;
                    let alpha_mask = _mm_extract_epi8(is_alpha, i) != 0;
                    let space_mask = _mm_extract_epi8(is_space, i) != 0;
                    
                    let class = if digit_mask {
                        CharClass::Digit
                    } else if alpha_mask {
                        CharClass::Alpha
                    } else if space_mask {
                        CharClass::Whitespace
                    } else {
                        CharClass::Other
                    };
                    
                    result.push(class);
                }
            }
        }
        
        // 残りをスカラー処理
        for &byte in remainder {
            let ch = byte as char;
            let class = if ch.is_ascii_digit() {
                CharClass::Digit
            } else if ch.is_alphabetic() {
                CharClass::Alpha
            } else if ch.is_whitespace() {
                CharClass::Whitespace
            } else {
                CharClass::Other
            };
            result.push(class);
        }
        
        result
    }
    
    /// SIMD最適化された文字列検索
    pub fn simd_find_pattern(haystack: &str, needle: &str) -> Option<usize> {
        if needle.len() > 16 || haystack.len() < 16 {
            // フォールバックしてスカラー処理
            return haystack.find(needle);
        }
        
        unsafe {
            use std::arch::x86_64::*;
            
            let needle_bytes = needle.as_bytes();
            let haystack_bytes = haystack.as_bytes();
            
            if needle_bytes.is_empty() {
                return Some(0);
            }
            
            let first_char = _mm_set1_epi8(needle_bytes[0] as i8);
            
            for i in 0..=(haystack_bytes.len().saturating_sub(16)) {
                let chunk = _mm_loadu_si128(haystack_bytes.as_ptr().add(i) as *const __m128i);
                let matches = _mm_cmpeq_epi8(chunk, first_char);
                let mask = _mm_movemask_epi8(matches);
                
                if mask != 0 {
                    // 最初の文字のマッチがある場合、完全一致を確認
                    for bit in 0..16 {
                        if (mask & (1 << bit)) != 0 {
                            let pos = i + bit;
                            if pos + needle.len() <= haystack.len() {
                                if &haystack_bytes[pos..pos + needle.len()] == needle_bytes {
                                    return Some(pos);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
}

/// 文字分類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharClass {
    Digit,
    Alpha,
    Whitespace,
    Other,
}

/// 並列パーサー - 独立した部分を並列処理
pub struct ParallelParser;

impl ParallelParser {
    /// 複数のS式を並列解析
    pub fn parallel_parse_expressions<'a>(
        inputs: Vec<&'a str>
    ) -> Vec<ParseResult<'a, SchemeSexp<'a>>> {
        use rayon::prelude::*;
        
        inputs.into_par_iter()
            .map(|input| {
                SchemeParser::s_expression().parse(input)
            })
            .collect()
    }
    
    /// 文字列リストの並列トークン化
    pub fn parallel_tokenize(inputs: Vec<String>) -> Vec<Vec<Token>> {
        use rayon::prelude::*;
        
        inputs.into_par_iter()
            .map(|input| {
                // 簡略化されたトークン化
                Self::simple_tokenize(&input)
            })
            .collect()
    }
    
    fn simple_tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();
        
        while let Some((pos, ch)) = chars.next() {
            match ch {
                '(' => tokens.push(Token::LeftParen(pos)),
                ')' => tokens.push(Token::RightParen(pos)),
                '\'' => tokens.push(Token::Quote(pos)),
                '`' => tokens.push(Token::Quasiquote(pos)),
                ',' => {
                    if chars.peek() == Some(&(pos + 1, '@')) {
                        chars.next();
                        tokens.push(Token::UnquoteSplicing(pos));
                    } else {
                        tokens.push(Token::Unquote(pos));
                    }
                }
                c if c.is_whitespace() => {
                    // 空白をスキップ
                    continue;
                }
                _ => {
                    // 識別子や数値として処理
                    tokens.push(Token::Identifier(pos, ch.to_string()));
                }
            }
        }
        
        tokens
    }
}

/// 簡略化されたトークン型
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen(usize),
    RightParen(usize),
    Quote(usize),
    Quasiquote(usize),
    Unquote(usize),
    UnquoteSplicing(usize),
    Identifier(usize, String),
}

/// メモ化パーサー - 同じ入力位置での結果をキャッシュ
pub struct MemoizedParser<'a> {
    cache: Arc<RwLock<HashMap<(usize, String), ParseResult<'a, SchemeSexp<'a>>>>>,
}

impl<'a> MemoizedParser<'a> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn parse_with_memo(&self, input: Input<'a>, position: usize) -> ParseResult<'a, SchemeSexp<'a>> {
        let key = (position, input.to_string());
        
        // キャッシュから確認
        {
            let cache = self.cache.read().unwrap();
            if let Some(result) = cache.get(&key) {
                return result.clone();
            }
        }
        
        // 実際にパースを実行
        let result = SchemeParser::s_expression().parse(input);
        
        // 結果をキャッシュ
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key, result.clone());
        }
        
        result
    }
    
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
    
    pub fn cache_size(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }
}

/// ゼロコピー最適化パーサー
pub struct ZeroCopyParser;

impl ZeroCopyParser {
    /// 文字列スライスを使った効率的な識別子解析
    pub fn efficient_symbol<'a>(input: Input<'a>) -> ParseResult<'a, &'a str> {
        let start = input.as_ptr();
        let mut end = start;
        let mut bytes_iter = input.bytes();
        
        // 最初の文字をチェック
        if let Some(first_byte) = bytes_iter.next() {
            let first_char = first_byte as char;
            if !Self::is_scheme_identifier_start(first_char) {
                return Err(Box::new(ParseError::new(
                    "Invalid identifier start".to_string(),
                    Span::new(0, 1)
                )));
            }
            unsafe { end = end.add(1); }
        } else {
            return Err(Box::new(ParseError::new(
                "Empty input".to_string(),
                Span::new(0, 0)
            )));
        }
        
        // 続く文字を処理
        for byte in bytes_iter {
            let ch = byte as char;
            if Self::is_scheme_identifier_continue(ch) {
                unsafe { end = end.add(1); }
            } else {
                break;
            }
        }
        
        let symbol_len = unsafe { end.offset_from(start) } as usize;
        let symbol = &input[..symbol_len];
        let remaining = &input[symbol_len..];
        
        Ok((remaining, symbol))
    }
    
    fn is_scheme_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || "!$%&*+-./:<=>?@^_~".contains(ch)
    }
    
    fn is_scheme_identifier_continue(ch: char) -> bool {
        ch.is_alphanumeric() || "!$%&*+-./:<=>?@^_~".contains(ch)
    }
    
    /// バイト境界を意識した効率的な文字列分割
    pub fn split_at_char_boundary(s: &str, mid: usize) -> Option<(&str, &str)> {
        if mid == 0 {
            return Some(("", s));
        }
        if mid >= s.len() {
            return Some((s, ""));
        }
        
        // UTF-8バイト境界をチェック
        if s.is_char_boundary(mid) {
            Some(s.split_at(mid))
        } else {
            None
        }
    }
}

/// プロファイリング支援
pub struct ParserProfiler {
    parse_times: HashMap<String, Vec<f64>>,
    memory_usage: HashMap<String, usize>,
}

impl ParserProfiler {
    pub fn new() -> Self {
        Self {
            parse_times: HashMap::new(),
            memory_usage: HashMap::new(),
        }
    }
    
    pub fn time_parse<T, F>(&mut self, name: &str, parser: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = std::time::Instant::now();
        let result = parser();
        let elapsed = start.elapsed().as_secs_f64();
        
        self.parse_times.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(elapsed);
        
        result
    }
    
    pub fn record_memory_usage(&mut self, name: &str, bytes: usize) {
        self.memory_usage.insert(name.to_string(), bytes);
    }
    
    pub fn get_average_time(&self, name: &str) -> Option<f64> {
        self.parse_times.get(name).map(|times| {
            times.iter().sum::<f64>() / times.len() as f64
        })
    }
    
    pub fn get_memory_usage(&self, name: &str) -> Option<usize> {
        self.memory_usage.get(name).copied()
    }
    
    pub fn print_report(&self) {
        println!("Parser Performance Report");
        println!("========================");
        
        println!("\nParse Times (average):");
        for (name, times) in &self.parse_times {
            let avg = times.iter().sum::<f64>() / times.len() as f64;
            let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            println!("  {}: avg={:.6}s, min={:.6}s, max={:.6}s, samples={}", 
                     name, avg, min, max, times.len());
        }
        
        println!("\nMemory Usage:");
        for (name, bytes) in &self.memory_usage {
            println!("  {}: {} bytes", name, bytes);
        }
    }
}

/// 最適化設定
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub enable_simd: bool,
    pub enable_parallel: bool,
    pub enable_memoization: bool,
    pub enable_zero_copy: bool,
    pub memoization_cache_size: usize,
    pub parallel_threshold: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_simd: cfg!(target_arch = "x86_64"),
            enable_parallel: true,
            enable_memoization: true,
            enable_zero_copy: true,
            memoization_cache_size: 1000,
            parallel_threshold: 4,
        }
    }
}

/// 最適化済みパーサーファクトリー
pub struct OptimizedParserFactory {
    config: OptimizationConfig,
    profiler: ParserProfiler,
}

impl OptimizedParserFactory {
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            profiler: ParserProfiler::new(),
        }
    }
    
    pub fn create_s_expression_parser<'a>(&self) -> Box<dyn ParserCombinator<'a, SchemeSexp<'a>>> {
        if self.config.enable_memoization {
            Box::new(MemoizedSExpressionParser::new())
        } else {
            Box::new(StandardSExpressionParser::new())
        }
    }
    
    pub fn get_profiler(&self) -> &ParserProfiler {
        &self.profiler
    }
}

/// 標準S式パーサー
struct StandardSExpressionParser;

impl StandardSExpressionParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, SchemeSexp<'a>> for StandardSExpressionParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, SchemeSexp<'a>> {
        SchemeParser::s_expression().parse(input)
    }
}

/// メモ化S式パーサー
struct MemoizedSExpressionParser<'a> {
    memoizer: MemoizedParser<'a>,
}

impl<'a> MemoizedSExpressionParser<'a> {
    fn new() -> Self {
        Self {
            memoizer: MemoizedParser::new(),
        }
    }
}

impl<'a> ParserCombinator<'a, SchemeSexp<'a>> for MemoizedSExpressionParser<'a> {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, SchemeSexp<'a>> {
        // 入力位置を計算（簡略化）
        let position = input.as_ptr() as usize;
        self.memoizer.parse_with_memo(input, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_simd_char_classification() {
        let input = "hello123 world";
        let classes = SIMDPrimitives::simd_classify_chars(input);
        
        assert_eq!(classes.len(), input.len());
        assert_eq!(classes[0], CharClass::Alpha); // 'h'
        assert_eq!(classes[5], CharClass::Digit); // '1'
        assert_eq!(classes[8], CharClass::Whitespace); // ' '
    }
    
    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_simd_pattern_search() {
        let haystack = "hello world hello universe";
        let needle = "hello";
        
        let result = SIMDPrimitives::simd_find_pattern(haystack, needle);
        assert_eq!(result, Some(0));
        
        let result = SIMDPrimitives::simd_find_pattern(&haystack[6..], needle);
        assert_eq!(result, Some(6)); // "hello" at position 6 in the substring
    }
    
    #[test]
    fn test_parallel_parsing() {
        let inputs = vec!["(+ 1 2)", "(* 3 4)", "'hello"];
        let results = ParallelParser::parallel_parse_expressions(inputs);
        
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_memoized_parser() {
        let parser = MemoizedParser::new();
        
        let input = "(+ 1 2)";
        let result1 = parser.parse_with_memo(input, 0);
        let result2 = parser.parse_with_memo(input, 0);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(parser.cache_size(), 1);
    }
    
    #[test]
    fn test_zero_copy_symbol_parsing() {
        let input = "hello-world";
        let result = ZeroCopyParser::efficient_symbol(input);
        
        assert!(result.is_ok());
        let (remaining, symbol) = result.unwrap();
        assert_eq!(symbol, "hello-world");
        assert_eq!(remaining, "");
    }
    
    #[test]
    fn test_parser_profiler() {
        let mut profiler = ParserProfiler::new();
        
        let result = profiler.time_parse("test_parse", || {
            std::thread::sleep(std::time::Duration::from_millis(1));
            42
        });
        
        assert_eq!(result, 42);
        let avg_time = profiler.get_average_time("test_parse");
        assert!(avg_time.is_some());
        assert!(avg_time.unwrap() > 0.0);
    }
    
    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig::default();
        assert!(config.enable_memoization);
        assert!(config.enable_zero_copy);
        assert_eq!(config.memoization_cache_size, 1000);
    }
    
    #[test]
    fn test_optimized_parser_factory() {
        let config = OptimizationConfig::default();
        let factory = OptimizedParserFactory::new(config);
        
        let parser = factory.create_s_expression_parser();
        let result = parser.parse("(+ 1 2)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_char_boundary_split() {
        let s = "hello世界";
        
        let result = ZeroCopyParser::split_at_char_boundary(s, 5);
        assert!(result.is_some());
        let (left, right) = result.unwrap();
        assert_eq!(left, "hello");
        assert_eq!(right, "世界");
        
        // 無効なバイト境界
        let result = ZeroCopyParser::split_at_char_boundary(s, 6);
        assert!(result.is_none());
    }
}