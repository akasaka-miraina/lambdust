//! High-performance Parser monad implementation for Lambdust.
//!
//! This module provides a zero-cost abstraction for parser combinators that integrates
//! with Lambdust's Value system. The implementation focuses on performance, error
//! reporting, and composability while maintaining the mathematical properties
//! of the Parser monad.
//!
//! The Parser monad represents computations that consume input and either succeed
//! with a result and remaining input, or fail with an error. This is ideal for
//! building parsers, lexers, and other input processing systems.

#![allow(missing_docs)]

use crate::eval::value::Value;
use crate::diagnostics::{Error, Result, Span};
use crate::effects::list_monad::List;
use std::sync::Arc;
use std::fmt;
use std::collections::HashMap;

/// Input position for tracking parser state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    /// Byte offset in input
    pub offset: usize,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl Position {
    /// Create a new position at the start of input
    pub fn start() -> Self {
        Position {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Advance position by one character
    pub fn advance(mut self, ch: char) -> Self {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self
    }
    
    /// Advance position by a string
    pub fn advance_str(mut self, s: &str) -> Self {
        for ch in s.chars() {
            self = self.advance(ch);
        }
        self
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.offset, self.line, self.column)
    }
}

/// Parser input state
#[derive(Debug, Clone)]
pub struct Input {
    /// The input text
    pub text: Arc<str>,
    /// Current position
    pub position: Position,
    /// User state (for stateful parsing)
    pub user_state: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl Input {
    /// Create new input from text
    pub fn new(text: &str) -> Self {
        Input {
            text: Arc::from(text),
            position: Position::start(),
            user_state: None,
        }
    }
    
    /// Create input with user state
    pub fn with_state<T: std::any::Any + Send + Sync>(text: &str, state: T) -> Self {
        Input {
            text: Arc::from(text),
            position: Position::start(),
            user_state: Some(Arc::new(state)),
        }
    }
    
    /// Get remaining input text
    pub fn remaining(&self) -> &str {
        &self.text[self.position.offset..]
    }
    
    /// Check if at end of input
    pub fn is_empty(&self) -> bool {
        self.position.offset >= self.text.len()
    }
    
    /// Advance input by consuming a character
    pub fn advance_char(mut self) -> Option<(char, Self)> {
        let mut chars = self.remaining().chars();
        if let Some(ch) = chars.next() {
            self.position = self.position.advance(ch);
            Some((ch, self))
        } else {
            None
        }
    }
    
    /// Advance input by consuming a string
    pub fn advance_str(mut self, s: &str) -> Option<Self> {
        if self.remaining().starts_with(s) {
            self.position = self.position.advance_str(s);
            Some(self)
        } else {
            None
        }
    }
    
    /// Set user state
    pub fn set_state<T: std::any::Any + Send + Sync>(mut self, state: T) -> Self {
        self.user_state = Some(Arc::new(state));
        self
    }
    
    /// Get user state
    pub fn get_state<T: std::any::Any + Send + Sync>(&self) -> Option<&T> {
        self.user_state
            .as_ref()
            .and_then(|s| s.downcast_ref::<T>())
    }
}

/// Parser error with position information
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Error position
    pub position: Position,
    /// Expected items (for better error messages)
    pub expected: Vec<String>,
    /// Actual item found
    pub found: Option<String>,
    /// Error message
    pub message: String,
}

impl ParseError {
    /// Create a new parse error
    pub fn new(position: Position, message: String) -> Self {
        ParseError {
            position,
            expected: Vec::new(),
            found: None,
            message,
        }
    }
    
    /// Add expected item
    pub fn expect(mut self, item: String) -> Self {
        self.expected.push(item);
        self
    }
    
    /// Set found item
    pub fn found(mut self, item: String) -> Self {
        self.found = Some(item);
        self
    }
    
    /// Combine two parse errors (prefer the one that consumed more input)
    pub fn combine(self, other: ParseError) -> ParseError {
        if self.position.offset > other.position.offset {
            self
        } else if other.position.offset > self.position.offset {
            other
        } else {
            // Same position, combine expected items
            ParseError {
                position: self.position,
                expected: {
                    let mut combined = self.expected;
                    combined.extend(other.expected);
                    combined.sort();
                    combined.dedup();
                    combined
                },
                found: self.found.or(other.found),
                message: if self.message.len() > other.message.len() {
                    self.message
                } else {
                    other.message
                },
            }
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.position, self.message)?;
        if !self.expected.is_empty() {
            write!(f, " (expected: {})", self.expected.join(", "))?;
        }
        if let Some(ref found) = self.found {
            write!(f, " (found: {found})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Parser result
pub type ParseResult<T> = std::result::Result<(T, Input), ParseError>;

/// High-performance Parser monad
pub struct Parser<T> {
    /// The parser function
    parser: Arc<dyn Fn(Input) -> ParseResult<T> + Send + Sync + 'static>,
    /// Parser ID for debugging
    id: u64,
    /// Parser name for error reporting
    name: Option<String>,
}

impl<T> Clone for Parser<T> {
    fn clone(&self) -> Self {
        Parser {
            parser: self.parser.clone(),
            id: self.id,
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for Parser<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "Parser({}: {})", self.id, name)
        } else {
            write!(f, "Parser({})", self.id)
        }
    }
}

/// Counter for generating unique parser IDs
static PARSER_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<T> Parser<T> {
    /// Create a new parser
    pub fn new<F>(parser: F) -> Self
    where
        F: Fn(Input) -> ParseResult<T> + Send + Sync + 'static,
    {
        let id = PARSER_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Parser {
            parser: Arc::new(parser),
            id,
            name: None,
        }
    }
    
    /// Create a parser with a name
    pub fn named<F>(name: String, parser: F) -> Self
    where
        F: Fn(Input) -> ParseResult<T> + Send + Sync + 'static,
    {
        let id = PARSER_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Parser {
            parser: Arc::new(parser),
            id,
            name: Some(name),
        }
    }
    
    /// Run the parser on input
    pub fn parse(&self, input: Input) -> ParseResult<T> {
        (self.parser)(input)
    }
    
    /// Run the parser on a string
    pub fn parse_str(&self, text: &str) -> std::result::Result<T, ParseError> {
        let input = Input::new(text);
        match self.parse(input) {
            Ok((result, remaining)) => {
                if remaining.is_empty() {
                    Ok(result)
                } else {
                    let pos = remaining.position.clone();
                    let found_char = remaining.remaining().chars().next().unwrap_or('\0');
                    Err(ParseError::new(
                        pos,
                        "Unexpected input after parsing".to_string(),
                    ).found(found_char.to_string()))
                }
            }
            Err(error) => Err(error),
        }
    }
    
    /// Set parser name (for debugging and error reporting)
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

/// Core parser combinators
impl<T> Parser<T> {
    /// Pure/return - always succeeds with the given value
    pub fn pure(value: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        Parser::new(move |input| Ok((value.clone(), input)))
    }
    
    /// Fail - always fails with the given error message
    pub fn fail(message: String) -> Self {
        Parser::new(move |input| {
            Err(ParseError::new(input.position, message.clone()))
        })
    }
    
    /// Monadic bind operation
    pub fn bind<U, F>(self, f: F) -> Parser<U>
    where
        F: Fn(T) -> Parser<U> + Send + Sync + 'static + Clone,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        Parser::new(move |input| {
            match self.parse(input) {
                Ok((result, remaining)) => f(result).parse(remaining),
                Err(error) => Err(error),
            }
        })
    }
    
    /// Functor map operation
    pub fn map<U, F>(self, f: F) -> Parser<U>
    where
        F: Fn(T) -> U + Send + Sync + 'static + Clone,
        T: Send + Sync + 'static,
        U: Clone + Send + Sync + 'static,
    {
        self.bind(move |value| Parser::pure(f(value)))
    }
    
    /// Choice operation - try this parser, if it fails, try the other
    pub fn choice(self, other: Parser<T>) -> Parser<T>
    where
        T: Send + Sync + 'static,
    {
        Parser::new(move |input| {
            // Clone input state for backtracking only if needed
            let original_position = input.position.clone();
            match self.parse(input.clone()) {
                Ok(result) => Ok(result),
                Err(error1) => {
                    // Create new input with reset position for backtracking
                    let mut reset_input = input;
                    reset_input.position = original_position;
                    match other.parse(reset_input) {
                        Ok(result) => Ok(result),
                        Err(error2) => Err(error1.combine(error2)),
                    }
                }
            }
        })
    }
    
    /// Try - if this parser fails without consuming input, backtrack
    pub fn try_parse(self) -> Parser<T>
    where
        T: Send + Sync + 'static,
    {
        Parser::new(move |input| {
            let original_position = input.position.clone();
            match self.parse(input) {
                Ok(result) => Ok(result),
                Err(mut error) => {
                    // Reset position to allow backtracking
                    error.position = original_position;
                    Err(error)
                }
            }
        })
    }
    
    /// Optional - make parser optional (returns Option<T>)
    pub fn optional(self) -> Parser<Option<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.map(Some).choice(Parser::pure(None))
    }
    
    /// Many - parse zero or more occurrences
    pub fn many(self) -> Parser<Vec<T>>
    where
        T: Send + Sync + 'static + Clone,
    {
        self.clone().some().choice(Parser::pure(Vec::new()))
    }
    
    /// Some - parse one or more occurrences
    pub fn some(self) -> Parser<Vec<T>>
    where
        T: Send + Sync + 'static + Clone,
    {
        let parser_for_many = self.clone();
        self.bind(move |first| {
            parser_for_many.clone().many().map(move |mut rest| {
                rest.insert(0, first.clone());
                rest
            })
        })
    }
    
    /// Chain left - parse with left-associative binary operators
    pub fn chain_left<F>(self, op: Parser<F>) -> Parser<T>
    where
        T: Send + Sync + 'static + Clone,
        F: Fn(T, T) -> T + Send + Sync + 'static + Clone,
    {
        let self_clone = self.clone();
        self.bind(move |first| {
            let op_clone = op.clone();
            let self_clone2 = self_clone.clone();
            let op_and_operand = op_clone.bind(move |f| {
                self_clone2.clone().map(move |second| (f.clone(), second))
            });
            
            op_and_operand.many().map(move |ops| {
                ops.into_iter().fold(first.clone(), |acc, (op_func, operand)| op_func(acc, operand))
            })
        })
    }
    
    /// Separated by - parse items separated by a separator
    pub fn separated_by<S>(self, sep: Parser<S>) -> Parser<Vec<T>>
    where
        T: Send + Sync + 'static + Clone,
        S: Send + Sync + 'static,
    {
        let parser_clone = self.clone();
        self.bind(move |first| {
            let sep_clone = sep.clone();
            let parser_clone2 = parser_clone.clone();
            sep_clone.then(parser_clone2).many().map(move |mut rest| {
                rest.insert(0, first.clone());
                rest
            })
        }).choice(Parser::pure(Vec::new()))
    }
    
    /// Then - sequence parsers, ignoring the result of this one
    pub fn then<U>(self, other: Parser<U>) -> Parser<U>
    where
        T: Send + Sync + 'static,
        U: Send + Sync + 'static,
    {
        self.bind(move |_| other.clone())
    }
    
    /// Skip - sequence parsers, ignoring the result of the second one
    pub fn skip<U>(self, other: Parser<U>) -> Parser<T>
    where
        T: Send + Sync + 'static + Clone,
        U: Send + Sync + 'static,
    {
        self.bind(move |value| {
            other.clone().map(move |_| value.clone())
        })
    }
    
    /// Between - parse between two other parsers
    pub fn between<L, R>(self, left: Parser<L>, right: Parser<R>) -> Parser<T>
    where
        T: Send + Sync + 'static + Clone,
        L: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        left.then(self).skip(right)
    }
}

/// Primitive parsers
impl Parser<char> {
    /// Parse any character
    pub fn any_char() -> Self {
        Parser::named("any_char".to_string(), |input| {
            let pos = input.position.clone();
            match input.advance_char() {
                Some((ch, remaining)) => Ok((ch, remaining)),
                None => Err(ParseError::new(
                    pos,
                    "Unexpected end of input".to_string(),
                ).expect("character".to_string())),
            }
        })
    }
    
    /// Parse a specific character
    pub fn char(expected: char) -> Self {
        Parser::named(format!("char('{expected}')"), move |input| {
            let pos = input.position.clone();
            match input.advance_char() {
                Some((ch, remaining)) if ch == expected => Ok((ch, remaining)),
                Some((ch, _)) => Err(ParseError::new(
                    pos,
                    format!("Expected '{expected}', found '{ch}'"),
                ).expect(expected.to_string()).found(ch.to_string())),
                None => Err(ParseError::new(
                    pos,
                    "Unexpected end of input".to_string(),
                ).expect(expected.to_string())),
            }
        })
    }
    
    /// Parse a character satisfying a predicate
    pub fn satisfy<P>(predicate: P) -> Self
    where
        P: Fn(char) -> bool + Send + Sync + 'static,
    {
        Parser::named("satisfy".to_string(), move |input| {
            let pos = input.position.clone();
            match input.advance_char() {
                Some((ch, remaining)) if predicate(ch) => Ok((ch, remaining)),
                Some((ch, _)) => Err(ParseError::new(
                    pos,
                    format!("Character '{ch}' does not satisfy predicate"),
                ).found(ch.to_string())),
                None => Err(ParseError::new(
                    pos,
                    "Unexpected end of input".to_string(),
                )),
            }
        })
    }
    
    /// Parse a digit
    pub fn digit() -> Self {
        Parser::satisfy(|ch| ch.is_ascii_digit())
            .with_name("digit".to_string())
    }
    
    /// Parse a letter
    pub fn letter() -> Self {
        Parser::satisfy(|ch| ch.is_alphabetic())
            .with_name("letter".to_string())
    }
    
    /// Parse an alphanumeric character
    pub fn alpha_num() -> Self {
        Parser::satisfy(|ch| ch.is_alphanumeric())
            .with_name("alphanumeric".to_string())
    }
    
    /// Parse whitespace
    pub fn whitespace() -> Self {
        Parser::satisfy(|ch| ch.is_whitespace())
            .with_name("whitespace".to_string())
    }
}

impl Parser<String> {
    /// Parse a specific string
    pub fn string(expected: String) -> Self {
        Parser::named(format!("string(\"{expected}\")"), move |mut input| {
            let pos = input.position.clone();
            let input_remaining = input.remaining().to_string();
            match input.clone().advance_str(&expected) {
                Some(remaining) => Ok((expected.clone(), remaining)),
                None => {
                    let found = input_remaining.chars().take(expected.len()).collect::<String>();
                    Err(ParseError::new(
                        pos,
                        format!("Expected \"{expected}\", found \"{found}\""),
                    ).expect(expected.clone()).found(found))
                }
            }
        })
    }
    
    /// Parse whitespace (one or more)
    pub fn whitespaces() -> Self {
        Parser::char(' ')
            .choice(Parser::char('\t'))
            .choice(Parser::char('\n'))
            .choice(Parser::char('\r'))
            .some()
            .map(|chars| chars.into_iter().collect())
            .with_name("whitespaces".to_string())
    }
    
    /// Parse an identifier (letter followed by letters, digits, or underscores)
    pub fn identifier() -> Self {
        Parser::letter()
            .bind(|first| {
                Parser::alpha_num()
                    .choice(Parser::char('_'))
                    .many()
                    .map(move |rest| {
                        let mut result = String::new();
                        result.push(first);
                        result.extend(rest);
                        result
                    })
            })
            .with_name("identifier".to_string())
    }
}

impl Parser<f64> {
    /// Parse a number
    pub fn number() -> Self {
        let integer_part = Parser::digit().some().map(|digits| digits.into_iter().collect::<String>());
        
        let fractional_part = Parser::char('.')
            .then(Parser::digit().some().map(|digits| digits.into_iter().collect::<String>()))
            .optional();
        
        integer_part
            .bind(move |int_part| {
                fractional_part.clone().map(move |frac_part| {
                    let mut number_str = int_part.clone();
                    if let Some(frac) = frac_part {
                        number_str.push('.');
                        number_str.push_str(&frac);
                    }
                    number_str.parse::<f64>().unwrap_or(0.0)
                })
            })
            .with_name("number".to_string())
    }
}

/// Utility parsers
impl<T: Clone> Parser<T> {
    /// Skip whitespace before this parser
    pub fn skip_whitespace(self) -> Parser<T>
    where
        T: Send + Sync + 'static,
    {
        Parser::whitespaces().optional().then(self)
    }
    
    /// Parse with trailing whitespace
    pub fn lexeme(self) -> Parser<T>
    where
        T: Send + Sync + 'static + Clone,
    {
        self.skip(Parser::whitespaces().optional())
    }
}

/// Integration with Lambdust Value system
impl Parser<Value> {
    /// Parse a Lambdust literal value
    pub fn value() -> Self {
        Parser::number().map(|n| Value::Literal(crate::ast::Literal::Number(n)))
            .choice(
                Parser::string("\"".to_string())
                    .then(Parser::satisfy(|ch| ch != '"').many())
                    .skip(Parser::char('"'))
                    .map(|chars| Value::Literal(crate::ast::Literal::String(chars.into_iter().collect())))
            )
            .choice(
                Parser::string("#t".to_string()).map(|_| Value::Literal(crate::ast::Literal::Boolean(true)))
            )
            .choice(
                Parser::string("#f".to_string()).map(|_| Value::Literal(crate::ast::Literal::Boolean(false)))
            )
            .with_name("value".to_string())
    }
    
    /// Parse a symbol
    pub fn symbol() -> Self {
        Parser::identifier()
            .map(|name| {
                // This is a simplified implementation
                // In practice, you'd use the symbol interning system
                Value::Symbol(crate::utils::SymbolId::new(0)) // Placeholder
            })
            .with_name("symbol".to_string())
    }
}

/// Thread safety
unsafe impl<T: Send> Send for Parser<T> {}
unsafe impl<T: Sync> Sync for Parser<T> {}

/// Memoization support for performance
pub struct ParserCache {
    cache: std::sync::RwLock<HashMap<(u64, Position), ParseResult<Value>>>,
}

impl ParserCache {
    pub fn new() -> Self {
        ParserCache {
            cache: std::sync::RwLock::new(HashMap::new()),
        }
    }
    
    pub fn memoize<T>(&self, parser_id: u64, position: Position, result: ParseResult<T>) 
    where
        T: Into<Value>,
    {
        if let Ok(mut cache) = self.cache.write() {
            let value_result = result.map(|(v, input)| (v.into(), input));
            cache.insert((parser_id, position), value_result);
        }
    }
    
    pub fn lookup(&self, parser_id: u64, position: Position) -> Option<ParseResult<Value>> {
        if let Ok(cache) = self.cache.read() {
            cache.get(&(parser_id, position)).cloned()
        } else {
            None
        }
    }
}

impl Default for ParserCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_monad_laws() {
        // Left identity: return(a) >>= f ≡ f(a)
        let a = 42;
        let f = |x: i32| Parser::pure(x * 2);
        let input = Input::new("test");
        
        let left = Parser::pure(a).bind(f);
        let right = f(a);
        
        let left_result = left.parse(input.clone()).unwrap();
        let right_result = right.parse(input).unwrap();
        
        assert_eq!(left_result.0, right_result.0);

        // Right identity: m >>= return ≡ m
        let m = Parser::pure(42);
        let left = m.clone().bind(Parser::pure);
        let input = Input::new("test");
        
        assert_eq!(
            left.parse(input.clone()).unwrap().0,
            m.parse(input).unwrap().0
        );
    }

    #[test]
    fn test_char_parser() {
        let parser = Parser::char('a');
        let input = Input::new("abc");
        let result = parser.parse(input).unwrap();
        
        assert_eq!(result.0, 'a');
        assert_eq!(result.1.remaining(), "bc");
    }

    #[test]
    fn test_string_parser() {
        let parser = Parser::string("hello".to_string());
        let input = Input::new("hello world");
        let result = parser.parse(input).unwrap();
        
        assert_eq!(result.0, "hello");
        assert_eq!(result.1.remaining(), " world");
    }

    #[test]
    fn test_choice_parser() {
        let parser = Parser::char('a').choice(Parser::char('b'));
        
        let input1 = Input::new("abc");
        let result1 = parser.parse(input1).unwrap();
        assert_eq!(result1.0, 'a');
        
        let input2 = Input::new("bcd");
        let result2 = parser.parse(input2).unwrap();
        assert_eq!(result2.0, 'b');
        
        let input3 = Input::new("cde");
        let result3 = parser.parse(input3);
        assert!(result3.is_err());
    }

    #[test]
    fn test_many_parser() {
        let parser = Parser::char('a').many();
        
        let input = Input::new("aaabbb");
        let result = parser.parse(input).unwrap();
        assert_eq!(result.0, vec!['a', 'a', 'a']);
        assert_eq!(result.1.remaining(), "bbb");
    }

    #[test]
    fn test_number_parser() {
        let parser = Parser::number();
        
        let input = Input::new("123.45");
        let result = parser.parse(input).unwrap();
        assert_eq!(result.0, 123.45);
        
        let input2 = Input::new("42");
        let result2 = parser.parse(input2).unwrap();
        assert_eq!(result2.0, 42.0);
    }

    #[test]
    fn test_separated_by_parser() {
        let item = Parser::char('a');
        let sep = Parser::char(',');
        let parser = item.separated_by(sep);
        
        let input = Input::new("a,a,a");
        let result = parser.parse(input).unwrap();
        assert_eq!(result.0, vec!['a', 'a', 'a']);
    }

    #[test]
    fn test_error_combining() {
        let error1 = ParseError::new(Position::start(), "Error 1".to_string())
            .expect("expected1".to_string());
        let error2 = ParseError::new(Position::start(), "Error 2".to_string())
            .expect("expected2".to_string());
        
        let combined = error1.combine(error2);
        assert_eq!(combined.expected.len(), 2);
        assert!(combined.expected.contains(&"expected1".to_string()));
        assert!(combined.expected.contains(&"expected2".to_string()));
    }

    #[test]
    fn test_position_tracking() {
        let mut pos = Position::start();
        pos = pos.advance('a');
        assert_eq!(pos.offset, 1);
        assert_eq!(pos.column, 2);
        
        pos = pos.advance('\n');
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }
}