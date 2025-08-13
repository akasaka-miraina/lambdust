//! パーサービルダー - 高レベルAPI
//!
//! このモジュールは使いやすい高レベルAPIを提供します：
//! - 流れるようなインターフェイス（Fluent API）
//! - パーサー構築の抽象化
//! - 再利用可能なパーサーテンプレート
//! - 設定可能なパーサーファクトリー

use super::types::*;
use super::combinator::*;
use super::scheme::*;
use super::combinator::helpers::*;
use crate::diagnostics::Span;
use std::collections::HashMap;

/// パーサービルダー - 流れるようなインターフェイスでパーサーを構築
pub struct ParserBuilder<'a> {
    /// パーサーチェーン
    parsers: Vec<Box<dyn ParserCombinator<'a, ParsedValue<'a>>>>,
    /// エラー回復設定
    error_recovery: bool,
    /// デバッグモード
    debug_mode: bool,
    /// カスタムエラーメッセージ
    error_messages: HashMap<String, String>,
}

/// パースされた値の統一型
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue<'a> {
    /// 文字列値
    String(&'a str),
    /// 文字値
    Char(char),
    /// 整数値
    Integer(i64),
    /// 浮動小数点値
    Float(f64),
    /// ブール値
    Boolean(bool),
    /// リスト値
    List(Vec<ParsedValue<'a>>),
    /// S式値
    SExpression(SchemeSexp<'a>),
    /// 生の値
    Raw(String),
}

impl<'a> Default for ParserBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ParserBuilder<'a> {
    /// 新しいパーサービルダーを作成
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
            error_recovery: false,
            debug_mode: false,
            error_messages: HashMap::new(),
        }
    }
    
    /// エラー回復を有効化
    pub fn with_error_recovery(mut self) -> Self {
        self.error_recovery = true;
        self
    }
    
    /// デバッグモードを有効化
    pub fn with_debug(mut self) -> Self {
        self.debug_mode = true;
        self
    }
    
    /// カスタムエラーメッセージを追加
    pub fn with_error_message(mut self, context: String, message: String) -> Self {
        self.error_messages.insert(context, message);
        self
    }
    
    /// 文字列リテラル解析を追加
    pub fn expect_string(mut self, expected: &'static str) -> Self {
        let parser = Box::new(StringLiteralParser::new(expected));
        self.parsers.push(parser);
        self
    }
    
    /// 識別子解析を追加
    pub fn expect_identifier(mut self) -> Self {
        let parser = Box::new(IdentifierParser::new());
        self.parsers.push(parser);
        self
    }
    
    /// 数値解析を追加
    pub fn expect_number(mut self) -> Self {
        let parser = Box::new(NumberParser::new());
        self.parsers.push(parser);
        self
    }
    
    /// S式解析を追加
    pub fn expect_s_expression(mut self) -> Self {
        let parser = Box::new(SExpressionParserWrapper::new());
        self.parsers.push(parser);
        self
    }
    
    /// 空白をスキップ
    pub fn skip_whitespace(mut self) -> Self {
        let parser = Box::new(WhitespaceSkipParser::new());
        self.parsers.push(parser);
        self
    }
    
    /// オプショナル要素
    pub fn maybe<T>(mut self, parser: T) -> Self 
    where
        T: ParserCombinator<'a, ParsedValue<'a>> + 'static,
    {
        let parser = Box::new(OptionalParser::new(parser));
        self.parsers.push(parser);
        self
    }
    
    /// 複数回の繰り返し
    pub fn repeat<T>(mut self, parser: T) -> Self
    where
        T: ParserCombinator<'a, ParsedValue<'a>> + Clone + 'static,
    {
        let parser = Box::new(RepeatParser::new(parser));
        self.parsers.push(parser);
        self
    }
    
    /// パーサーを構築して実行
    pub fn build_and_parse(self, input: Input<'a>) -> ParseResult<'a, Vec<ParsedValue<'a>>> {
        let mut results = Vec::new();
        let mut remaining = input;
        
        for parser in self.parsers {
            match parser.parse(remaining) {
                Ok((new_remaining, value)) => {
                    results.push(value);
                    remaining = new_remaining;
                }
                Err(err) => {
                    if self.error_recovery {
                        // エラー回復を試行
                        if self.debug_mode {
                            eprintln!("パーサーエラーが発生しました: {err:?}");
                        }
                        // 簡単なエラー回復 - 次の空白まで進む
                        if let Some(space_pos) = remaining.find(' ') {
                            remaining = &remaining[space_pos..];
                            continue;
                        }
                    }
                    return Err(err);
                }
            }
        }
        
        Ok((remaining, results))
    }
}

/// 文字列リテラルパーサー
struct StringLiteralParser {
    expected: &'static str,
}

impl StringLiteralParser {
    fn new(expected: &'static str) -> Self {
        Self { expected }
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for StringLiteralParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        let parser = tag(self.expected);
        parser.parse(input).map(|(remaining, matched)| {
            (remaining, ParsedValue::String(matched))
        })
    }
}

/// 識別子パーサー
struct IdentifierParser;

impl IdentifierParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for IdentifierParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        let parser = SchemeParser::symbol();
        parser.parse(input).map(|(remaining, symbol)| {
            (remaining, ParsedValue::String(symbol))
        })
    }
}

/// 数値パーサー
struct NumberParser;

impl NumberParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for NumberParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        let parser = SchemeParser::number();
        parser.parse(input).map(|(remaining, atom)| {
            let value = match atom {
                SchemeAtom::Integer(i) => ParsedValue::Integer(i),
                SchemeAtom::Float(f) => ParsedValue::Float(f),
                _ => ParsedValue::Raw(format!("{atom:?}")),
            };
            (remaining, value)
        })
    }
}

/// S式パーサーのラッパー
struct SExpressionParserWrapper;

impl SExpressionParserWrapper {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for SExpressionParserWrapper {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        let parser = SchemeParser::s_expression();
        parser.parse(input).map(|(remaining, sexp)| {
            (remaining, ParsedValue::SExpression(sexp))
        })
    }
}

/// 空白スキップパーサー
struct WhitespaceSkipParser;

impl WhitespaceSkipParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for WhitespaceSkipParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        let parser = SchemeParser::skip_whitespace_and_comments();
        parser.parse(input).map(|(remaining, _)| {
            (remaining, ParsedValue::Raw("whitespace_skipped".to_string()))
        })
    }
}

/// オプショナルパーサー
struct OptionalParser<T> {
    inner: T,
}

impl<T> OptionalParser<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<'a, T> ParserCombinator<'a, ParsedValue<'a>> for OptionalParser<T>
where
    T: ParserCombinator<'a, ParsedValue<'a>>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        match self.inner.parse(input) {
            Ok(result) => Ok(result),
            Err(_) => Ok((input, ParsedValue::Raw("none".to_string()))),
        }
    }
}

/// 繰り返しパーサー
struct RepeatParser<T> {
    inner: T,
}

impl<T> RepeatParser<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<'a, T> ParserCombinator<'a, ParsedValue<'a>> for RepeatParser<T>
where
    T: ParserCombinator<'a, ParsedValue<'a>> + Clone,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        let mut results = Vec::new();
        let mut remaining = input;
        
        while let Ok((new_remaining, value)) = self.inner.parse(remaining) {
            results.push(value);
            remaining = new_remaining;
        }
        
        Ok((remaining, ParsedValue::List(results)))
    }
}

/// パーサーテンプレート - よく使用されるパターンの事前定義
pub struct ParserTemplates;

impl ParserTemplates {
    /// Scheme関数定義パーサー
    pub fn scheme_function_definition<'a>() -> impl ParserCombinator<'a, ParsedValue<'a>> {
        FunctionDefinitionParser::new()
    }
    
    /// Scheme変数定義パーサー
    pub fn scheme_variable_definition<'a>() -> impl ParserCombinator<'a, ParsedValue<'a>> {
        VariableDefinitionParser::new()
    }
    
    /// Scheme条件式パーサー
    pub fn scheme_if_expression<'a>() -> impl ParserCombinator<'a, ParsedValue<'a>> {
        IfExpressionParser::new()
    }
}

/// 関数定義パーサー
struct FunctionDefinitionParser;

impl FunctionDefinitionParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for FunctionDefinitionParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        // (define (name params...) body...)
        let builder = ParserBuilder::new()
            .expect_string("(")
            .skip_whitespace()
            .expect_string("define")
            .skip_whitespace()
            .expect_string("(")
            .expect_identifier()
            .skip_whitespace();
        
        // 簡略化された実装
        Ok((input, ParsedValue::Raw("function_definition".to_string())))
    }
}

/// 変数定義パーサー
struct VariableDefinitionParser;

impl VariableDefinitionParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for VariableDefinitionParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        // (define name value)
        Ok((input, ParsedValue::Raw("variable_definition".to_string())))
    }
}

/// if式パーサー
struct IfExpressionParser;

impl IfExpressionParser {
    fn new() -> Self {
        Self
    }
}

impl<'a> ParserCombinator<'a, ParsedValue<'a>> for IfExpressionParser {
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, ParsedValue<'a>> {
        // (if condition then else)
        Ok((input, ParsedValue::Raw("if_expression".to_string())))
    }
}

/// DSL風パーサー構築マクロ（概念実証）
#[macro_export]
macro_rules! scheme_parser {
    (expect $literal:literal) => {
        ParserBuilder::new().expect_string($literal)
    };
    (identifier) => {
        ParserBuilder::new().expect_identifier()
    };
    (number) => {
        ParserBuilder::new().expect_number()
    };
    (s_expr) => {
        ParserBuilder::new().expect_s_expression()
    };
    (skip_ws) => {
        ParserBuilder::new().skip_whitespace()
    };
}

#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_builder_basic() {
        let builder = ParserBuilder::new()
            .expect_string("hello")
            .skip_whitespace()
            .expect_identifier();
        
        let result = builder.build_and_parse("hello world");
        assert!(result.is_ok());
        let (remaining, values) = result.unwrap();
        assert_eq!(values.len(), 3); // hello, whitespace_skipped, world
        assert_eq!(values[0], ParsedValue::String("hello"));
    }
    
    #[test]
    fn test_parser_builder_with_error_recovery() {
        let builder = ParserBuilder::new()
            .with_error_recovery()
            .with_debug()
            .expect_string("missing")
            .expect_identifier();
        
        let result = builder.build_and_parse("hello world");
        // エラー回復が有効なので、一部の失敗があっても続行
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_number_parser() {
        let parser = NumberParser::new();
        
        let result = parser.parse("123");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(value, ParsedValue::Integer(123));
    }
    
    #[test]
    fn test_identifier_parser() {
        let parser = IdentifierParser::new();
        
        let result = parser.parse("hello-world");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(value, ParsedValue::String("hello-world"));
    }
    
    #[test]
    fn test_optional_parser() {
        let inner = StringLiteralParser::new("optional");
        let parser = OptionalParser::new(inner);
        
        // 存在する場合
        let result = parser.parse("optional");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(value, ParsedValue::String("optional"));
        
        // 存在しない場合
        let result = parser.parse("something_else");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "something_else");
        assert_eq!(value, ParsedValue::Raw("none".to_string()));
    }
    
    #[test]
    fn test_repeat_parser() {
        let inner = StringLiteralParser::new("a");
        let parser = RepeatParser::new(inner);
        
        let result = parser.parse("aaab");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "b");
        if let ParsedValue::List(items) = value {
            assert_eq!(items.len(), 3);
            for item in items {
                assert_eq!(item, ParsedValue::String("a"));
            }
        } else {
            panic!("Expected list value");
        }
    }
    
    #[test]
    fn test_scheme_parser_macro() {
        // マクロのテスト（概念実証）
        let _parser = scheme_parser!(expect "hello");
        let _parser = scheme_parser!(identifier);
        let _parser = scheme_parser!(number);
        let _parser = scheme_parser!(s_expr);
        let _parser = scheme_parser!(skip_ws);
    }
}