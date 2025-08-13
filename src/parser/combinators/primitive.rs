//! 基本パーサープリミティブ実装
//!
//! このモジュールはパーサーコンビネータシステムの基盤を提供します：
//! - ゼロコピー文字列操作
//! - 基本的なパーサープリミティブ
//! - エラーハンドリングとspan管理
//! - UTF-8対応と文字境界処理

use crate::diagnostics::Span;
use super::types::*;
use std::str::Chars;
use std::iter::Peekable;

/// パーサー状態管理 - 現在位置とコンテキスト情報
#[derive(Debug, Clone)]
pub struct ParseState<'a> {
    /// 元の完全な入力
    pub full_input: Input<'a>,
    /// 現在解析中の入力
    pub current_input: Input<'a>,
    /// 現在のバイト位置
    pub position: usize,
    /// 行番号（1から開始）
    pub line: usize,
    /// 列番号（1から開始）  
    pub column: usize,
    /// パーサーコンテキストスタック
    pub context_stack: Vec<String>,
}

impl<'a> ParseState<'a> {
    /// 新しいパーサー状態を作成
    pub fn new(input: Input<'a>) -> Self {
        Self {
            full_input: input,
            current_input: input,
            position: 0,
            line: 1,
            column: 1,
            context_stack: Vec::new(),
        }
    }
    
    /// 現在のspan情報を取得
    pub fn current_span(&self, length: usize) -> Span {
        Span::new(self.position, self.position + length)
    }
    
    /// 指定文字数だけ状態を進める
    pub fn advance(&mut self, chars: usize) -> Result<(), Box<ParseError>> {
        let mut char_iter = self.current_input.chars();
        let mut byte_offset = 0;
        
        for _ in 0..chars {
            match char_iter.next() {
                Some(ch) => {
                    byte_offset += ch.len_utf8();
                    if ch == '\n' {
                        self.line += 1;
                        self.column = 1;
                    } else {
                        self.column += 1;
                    }
                }
                None => {
                    return Err(Box::new(ParseError::new(
                        "Unexpected end of input".to_string(),
                        self.current_span(0)
                    )));
                }
            }
        }
        
        self.position += byte_offset;
        self.current_input = &self.current_input[byte_offset..];
        Ok(())
    }
    
    /// 現在が入力の終端かチェック
    pub fn is_at_end(&self) -> bool {
        self.current_input.is_empty()
    }
    
    /// コンテキストをプッシュ
    pub fn push_context(&mut self, context: String) {
        self.context_stack.push(context);
    }
    
    /// コンテキストをポップ
    pub fn pop_context(&mut self) -> Option<String> {
        self.context_stack.pop()
    }
}

/// 基本パーサープリミティブ
pub struct Primitives;

impl Primitives {
    /// 文字列リテラルにマッチ
    pub fn tag<'a>(expected: &'static str) -> impl Fn(Input<'a>) -> ParseResult<'a, &'a str> {
        move |input: Input<'a>| {
            if let Some(remaining) = input.strip_prefix(expected) {
                Ok((remaining, &input[..expected.len()]))
            } else {
                let actual_len = input.chars().take(expected.len()).count();
                let actual: String = input.chars().take(expected.len()).collect();
                Err(Box::new(ParseError::new(
                    format!("Expected '{expected}'"),
                    Span::new(0, actual_len)
                ).with_expected(expected.to_string())
                .with_actual(actual)))
            }
        }
    }
    
    /// 単一文字にマッチ
    pub fn char<'a>(expected: char) -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        move |input: Input<'a>| {
            let mut chars = input.chars();
            match chars.next() {
                Some(ch) if ch == expected => {
                    let remaining = chars.as_str();
                    Ok((remaining, ch))
                }
                Some(ch) => Err(Box::new(ParseError::new(
                    format!("Expected character '{expected}'"),
                    Span::new(0, ch.len_utf8())
                ).with_expected(expected.to_string())
                .with_actual(ch.to_string()))),
                None => Err(Box::new(ParseError::new(
                    "Unexpected end of input".to_string(),
                    Span::new(0, 0)
                ).with_expected(expected.to_string()))),
            }
        }
    }
    
    /// 任意の文字にマッチ
    pub fn any_char<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        |input: Input<'a>| {
            let mut chars = input.chars();
            match chars.next() {
                Some(ch) => {
                    let remaining = chars.as_str();
                    Ok((remaining, ch))
                }
                None => Err(Box::new(ParseError::new(
                    "Unexpected end of input".to_string(),
                    Span::new(0, 0)
                ))),
            }
        }
    }
    
    /// 述語関数を満たす文字にマッチ
    pub fn satisfy<'a, P>(predicate: P) -> impl Fn(Input<'a>) -> ParseResult<'a, char>
    where
        P: Fn(char) -> bool,
    {
        move |input: Input<'a>| {
            let mut chars = input.chars();
            match chars.next() {
                Some(ch) if predicate(ch) => {
                    let remaining = chars.as_str();
                    Ok((remaining, ch))
                }
                Some(ch) => Err(Box::new(ParseError::new(
                    "Character does not satisfy predicate".to_string(),
                    Span::new(0, ch.len_utf8())
                ).with_actual(ch.to_string()))),
                None => Err(Box::new(ParseError::new(
                    "Unexpected end of input".to_string(),
                    Span::new(0, 0)
                ))),
            }
        }
    }
    
    /// 文字範囲にマッチ
    pub fn char_range<'a>(start: char, end: char) -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        move |input: Input<'a>| {
            Self::satisfy(|ch| ch >= start && ch <= end)(input)
        }
    }
    
    /// 数字文字にマッチ
    pub fn digit<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ch.is_ascii_digit())
    }
    
    /// 英字にマッチ
    pub fn alpha<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ch.is_alphabetic())
    }
    
    /// 英数字にマッチ
    pub fn alphanumeric<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ch.is_alphanumeric())
    }
    
    /// 空白文字にマッチ
    pub fn whitespace<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ch.is_whitespace())
    }
    
    /// 改行文字にマッチ
    pub fn newline<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::char('\n')
    }
    
    /// タブ文字にマッチ
    pub fn tab<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::char('\t')
    }
    
    /// 空白またはタブにマッチ
    pub fn space<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ch == ' ' || ch == '\t')
    }
    
    /// 16進数字にマッチ
    pub fn hex_digit<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ch.is_ascii_hexdigit())
    }
    
    /// 8進数字にマッチ
    pub fn oct_digit<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Self::satisfy(|ch| ('0'..='7').contains(&ch))
    }
    
    /// 入力の終端にマッチ
    pub fn eof<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, ()> {
        |input: Input<'a>| {
            if input.is_empty() {
                Ok((input, ()))
            } else {
                Err(Box::new(ParseError::new(
                    "Expected end of input".to_string(),
                    Span::new(0, 1)
                ).with_expected("end of input".to_string())
                .with_actual(input.chars().next().unwrap().to_string())))
            }
        }
    }
    
    /// 何もしないパーサー（常に成功）
    pub fn empty<'a, T>(value: T) -> impl Fn(Input<'a>) -> ParseResult<'a, T>
    where
        T: Clone,
    {
        move |input: Input<'a>| Ok((input, value.clone()))
    }
    
    /// 常に失敗するパーサー
    pub fn fail<'a, T>(message: &'static str) -> impl Fn(Input<'a>) -> ParseResult<'a, T> {
        move |_input: Input<'a>| {
            Err(Box::new(ParseError::new(
                message.to_string(),
                Span::new(0, 0)
            )))
        }
    }
}

/// Scheme言語特化のプリミティブ
pub struct SchemePreimitives;

impl SchemePreimitives {
    /// Scheme識別子開始文字
    pub fn identifier_start<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Primitives::satisfy(|ch| {
            ch.is_alphabetic() || 
            "!$%&*+-./:<=>?@^_~".contains(ch)
        })
    }
    
    /// Scheme識別子継続文字
    pub fn identifier_continue<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Primitives::satisfy(|ch| {
            ch.is_alphanumeric() || 
            "!$%&*+-./:<=>?@^_~".contains(ch)
        })
    }
    
    /// 左括弧類
    pub fn left_paren<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Primitives::satisfy(|ch| ch == '(' || ch == '[' || ch == '{')
    }
    
    /// 右括弧類
    pub fn right_paren<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Primitives::satisfy(|ch| ch == ')' || ch == ']' || ch == '}')
    }
    
    /// Scheme特殊文字
    pub fn scheme_delimiter<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Primitives::satisfy(|ch| {
            ch.is_whitespace() || 
            "()[]{}\"';#|\\".contains(ch)
        })
    }
    
    /// 文字列エスケープ文字
    pub fn string_escape<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, char> {
        Primitives::satisfy(|ch| "\\\"nrt".contains(ch))
    }
}

/// エラーハンドリング用ユーティリティ
pub struct ErrorUtils;

impl ErrorUtils {
    /// エラーにコンテキストを追加
    pub fn with_context<'a, T, P>(
        parser: P,
        context: &'static str,
    ) -> impl Fn(Input<'a>) -> ParseResult<'a, T>
    where
        P: Fn(Input<'a>) -> ParseResult<'a, T>,
    {
        move |input: Input<'a>| {
            parser(input).map_err(|err| Box::new(err.as_ref().clone().with_context(context.to_string())))
        }
    }
    
    /// カスタムエラーメッセージ
    pub fn custom_error<'a, T>(
        message: String,
        span: Span,
    ) -> ParseResult<'a, T> {
        Err(Box::new(ParseError::new(message, span)))
    }
    
    /// 期待される入力のエラー
    pub fn expected_error<'a, T>(
        expected: String,
        actual: String,
        span: Span,
    ) -> ParseResult<'a, T> {
        Err(Box::new(ParseError::new(
            format!("Expected {expected}, found {actual}"),
            span,
        ).with_expected(expected)
        .with_actual(actual)))
    }
    
    /// 詳細なエラー情報を提供するヘルパー
    pub fn detailed_error<'a, T>(
        message: String,
        span: Span,
        expected: Vec<String>,
        actual: String,
        context: Vec<String>,
    ) -> ParseResult<'a, T> {
        let mut error = ParseError::new(message, span).with_actual(actual);
        
        for exp in expected {
            error = error.with_expected(exp);
        }
        
        for ctx in context {
            error = error.with_context(ctx);
        }
        
        Err(Box::new(error))
    }
    
    /// 入力終了エラー
    pub fn eof_error<'a, T>(expected: String) -> ParseResult<'a, T> {
        Err(Box::new(ParseError::new(
            "Unexpected end of input".to_string(),
            Span::new(0, 0)
        ).with_expected(expected)))
    }
    
    /// 無効な文字エラー
    pub fn invalid_char_error<'a, T>(
        ch: char,
        position: usize,
        expected_description: String,
    ) -> ParseResult<'a, T> {
        Err(Box::new(ParseError::new(
            format!("Invalid character '{ch}'"),
            Span::new(position, position + ch.len_utf8())
        ).with_expected(expected_description)
        .with_actual(ch.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tag_success() {
        let parser = Primitives::tag("hello");
        let result = parser("hello world");
        assert!(result.is_ok());
        let (remaining, matched) = result.unwrap();
        assert_eq!(remaining, " world");
        assert_eq!(matched, "hello");
    }
    
    #[test]
    fn test_tag_failure() {
        let parser = Primitives::tag("hello");
        let result = parser("hi world");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_char_success() {
        let parser = Primitives::char('a');
        let result = parser("abc");
        assert!(result.is_ok());
        let (remaining, matched) = result.unwrap();
        assert_eq!(remaining, "bc");
        assert_eq!(matched, 'a');
    }
    
    #[test]
    fn test_satisfy() {
        let parser = Primitives::satisfy(|ch| ch.is_ascii_digit());
        let result = parser("123");
        assert!(result.is_ok());
        let (remaining, matched) = result.unwrap();
        assert_eq!(remaining, "23");
        assert_eq!(matched, '1');
    }
    
    #[test]
    fn test_scheme_identifier_start() {
        let parser = SchemePreimitives::identifier_start();
        
        // 正常ケース
        assert!(parser("abc").is_ok());
        assert!(parser("+123").is_ok());
        assert!(parser("$var").is_ok());
        
        // 異常ケース
        assert!(parser("123").is_err());
        assert!(parser("(").is_err());
    }
    
    #[test]
    fn test_parse_state() {
        let mut state = ParseState::new("hello\nworld");
        assert_eq!(state.line, 1);
        assert_eq!(state.column, 1);
        assert_eq!(state.position, 0);
        
        state.advance(6).unwrap(); // "hello\n"
        assert_eq!(state.line, 2);
        assert_eq!(state.column, 1);
        assert_eq!(state.position, 6);
        
        let span = state.current_span(5);
        assert_eq!(span.start, 6);
        assert_eq!(span.end, 11);
    }
}