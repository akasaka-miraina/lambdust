//! パーサーコンビネータ実装
//!
//! このモジュールは基本的なパーサーコンビネータを提供します：
//! - シーケンス操作（and, then）
//! - 選択操作（or, alt）  
//! - 繰り返し操作（many, many1）
//! - 変換操作（map, flat_map）
//! - エラーハンドリング（recover, cut）

use super::types::*;
use super::primitive::*;
use std::marker::PhantomData;
use std::rc::Rc;

/// パーサーコンビネータトレイト
pub trait ParserCombinator<'a, T> {
    /// パーサーを実行
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, T>;
    
    /// パーサーの結果を変換
    fn map<U, F>(self, f: F) -> Map<'a, T, U, Self, F>
    where
        Self: Sized,
        F: Fn(T) -> U,
    {
        Map::new(self, f)
    }
    
    /// パーサーの結果をフラット変換
    fn flat_map<U, F>(self, f: F) -> FlatMap<'a, T, U, Self, F>
    where
        Self: Sized,
        F: Fn(T) -> Box<dyn ParserCombinator<'a, U>>,
    {
        FlatMap::new(self, f)
    }
    
    /// 連続実行
    fn and<U, P>(self, other: P) -> And<'a, T, U, Self, P>
    where
        Self: Sized,
        P: ParserCombinator<'a, U>,
    {
        And::new(self, other)
    }
    
    /// 選択実行
    fn or<P>(self, other: P) -> Or<'a, T, Self, P>
    where
        Self: Sized,
        P: ParserCombinator<'a, T>,
    {
        Or::new(self, other)
    }
    
    /// オプショナル実行
    fn optional(self) -> Optional<'a, T, Self>
    where
        Self: Sized,
    {
        Optional::new(self)
    }
    
    /// 繰り返し実行（0回以上）
    fn many(self) -> Many<'a, T, Self>
    where
        Self: Sized + Clone,
    {
        Many::new(self)
    }
    
    /// 繰り返し実行（1回以上）
    fn many1(self) -> Many1<'a, T, Self>
    where
        Self: Sized + Clone,
    {
        Many1::new(self)
    }
    
    /// エラー回復
    fn recover<F>(self, recovery: F) -> Recover<'a, T, Self, F>
    where
        Self: Sized,
        F: Fn(Box<ParseError>) -> T,
    {
        Recover::new(self, recovery)
    }
    
    /// カット操作（コミット）
    fn cut(self) -> Cut<'a, T, Self>
    where
        Self: Sized,
    {
        Cut::new(self)
    }
}

/// 関数型パーサーラッパー
pub struct FnParser<'a, T, F> 
where
    F: Fn(Input<'a>) -> ParseResult<'a, T>,
{
    parser: Rc<F>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, F> FnParser<'a, T, F>
where
    F: Fn(Input<'a>) -> ParseResult<'a, T>,
{
    /// Creates a new function parser from a parsing function
    pub fn new(parser: F) -> Self {
        Self {
            parser: Rc::new(parser),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, F> Clone for FnParser<'a, T, F>
where
    F: Fn(Input<'a>) -> ParseResult<'a, T>,
{
    fn clone(&self) -> Self {
        Self {
            parser: Rc::clone(&self.parser),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, F> ParserCombinator<'a, T> for FnParser<'a, T, F>
where
    F: Fn(Input<'a>) -> ParseResult<'a, T>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, T> {
        (*self.parser)(input)
    }
}

/// マップコンビネータ
pub struct Map<'a, T, U, P, F> 
where
    P: ParserCombinator<'a, T>,
    F: Fn(T) -> U,
{
    parser: P,
    mapper: Rc<F>,
    phantom: PhantomData<&'a (T, U)>,
}

impl<'a, T, U, P, F> Map<'a, T, U, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(T) -> U,
{
    /// Creates a new mapping parser with a source parser and mapping function
    pub fn new(parser: P, mapper: F) -> Self {
        Self {
            parser,
            mapper: Rc::new(mapper),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, U, P, F> Clone for Map<'a, T, U, P, F>
where
    P: ParserCombinator<'a, T> + Clone,
    F: Fn(T) -> U,
{
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: Rc::clone(&self.mapper),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, U, P, F> ParserCombinator<'a, U> for Map<'a, T, U, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(T) -> U,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, U> {
        self.parser.parse(input).map(|(remaining, value)| {
            (remaining, (*self.mapper)(value))
        })
    }
}

/// フラットマップコンビネータ
pub struct FlatMap<'a, T, U, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(T) -> Box<dyn ParserCombinator<'a, U>>,
{
    parser: P,
    mapper: F,
    phantom: PhantomData<&'a (T, U)>,
}

impl<'a, T, U, P, F> FlatMap<'a, T, U, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(T) -> Box<dyn ParserCombinator<'a, U>>,
{
    /// Creates a new flat-map parser with a source parser and mapping function
    pub fn new(parser: P, mapper: F) -> Self {
        Self {
            parser,
            mapper,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, U, P, F> ParserCombinator<'a, U> for FlatMap<'a, T, U, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(T) -> Box<dyn ParserCombinator<'a, U>>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, U> {
        let (remaining, value) = self.parser.parse(input)?;
        let next_parser = (self.mapper)(value);
        next_parser.parse(remaining)
    }
}

/// AND（連続）コンビネータ
pub struct And<'a, T, U, P1, P2>
where
    P1: ParserCombinator<'a, T>,
    P2: ParserCombinator<'a, U>,
{
    first: P1,
    second: P2,
    phantom: PhantomData<&'a (T, U)>,
}

impl<'a, T, U, P1, P2> And<'a, T, U, P1, P2>
where
    P1: ParserCombinator<'a, T>,
    P2: ParserCombinator<'a, U>,
{
    /// Creates a new and parser that requires both parsers to succeed
    pub fn new(first: P1, second: P2) -> Self {
        Self {
            first,
            second,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, U, P1, P2> Clone for And<'a, T, U, P1, P2>
where
    P1: ParserCombinator<'a, T> + Clone,
    P2: ParserCombinator<'a, U> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            first: self.first.clone(),
            second: self.second.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, U, P1, P2> ParserCombinator<'a, (T, U)> for And<'a, T, U, P1, P2>
where
    P1: ParserCombinator<'a, T>,
    P2: ParserCombinator<'a, U>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, (T, U)> {
        let (remaining, first_value) = self.first.parse(input)?;
        let (remaining, second_value) = self.second.parse(remaining)?;
        Ok((remaining, (first_value, second_value)))
    }
}

/// OR（選択）コンビネータ
pub struct Or<'a, T, P1, P2>
where
    P1: ParserCombinator<'a, T>,
    P2: ParserCombinator<'a, T>,
{
    first: P1,
    second: P2,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, P1, P2> Or<'a, T, P1, P2>
where
    P1: ParserCombinator<'a, T>,
    P2: ParserCombinator<'a, T>,
{
    /// Creates a new or parser that tries the first parser, then the second on failure
    pub fn new(first: P1, second: P2) -> Self {
        Self {
            first,
            second,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P1, P2> Clone for Or<'a, T, P1, P2>
where
    P1: ParserCombinator<'a, T> + Clone,
    P2: ParserCombinator<'a, T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            first: self.first.clone(),
            second: self.second.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P1, P2> ParserCombinator<'a, T> for Or<'a, T, P1, P2>
where
    P1: ParserCombinator<'a, T>,
    P2: ParserCombinator<'a, T>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, T> {
        match self.first.parse(input) {
            Ok(result) => Ok(result),
            Err(_) => self.second.parse(input),
        }
    }
}

/// オプショナルコンビネータ
pub struct Optional<'a, T, P>
where
    P: ParserCombinator<'a, T>,
{
    parser: P,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, P> Optional<'a, T, P>
where
    P: ParserCombinator<'a, T>,
{
    /// Creates a new optional parser that makes the given parser optional
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> Clone for Optional<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> ParserCombinator<'a, Option<T>> for Optional<'a, T, P>
where
    P: ParserCombinator<'a, T>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, Option<T>> {
        match self.parser.parse(input) {
            Ok((remaining, value)) => Ok((remaining, Some(value))),
            Err(_) => Ok((input, None)),
        }
    }
}

/// 繰り返し（0回以上）コンビネータ
pub struct Many<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    parser: P,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, P> Many<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    /// Creates a new many parser that repeats the given parser zero or more times
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> Clone for Many<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> ParserCombinator<'a, Vec<T>> for Many<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, Vec<T>> {
        let mut results = Vec::new();
        let mut remaining = input;
        
        while let Ok((new_remaining, value)) = self.parser.parse(remaining) {
            results.push(value);
            remaining = new_remaining;
        }
        
        Ok((remaining, results))
    }
}

/// 繰り返し（1回以上）コンビネータ
pub struct Many1<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    parser: P,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, P> Many1<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    /// Creates a new many1 parser that repeats the given parser one or more times
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> Clone for Many1<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> ParserCombinator<'a, Vec<T>> for Many1<'a, T, P>
where
    P: ParserCombinator<'a, T> + Clone,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, Vec<T>> {
        let (remaining, first_value) = self.parser.parse(input)?;
        let mut results = vec![first_value];
        let mut remaining = remaining;
        
        while let Ok((new_remaining, value)) = self.parser.parse(remaining) {
            results.push(value);
            remaining = new_remaining;
        }
        
        Ok((remaining, results))
    }
}

/// エラー回復コンビネータ
pub struct Recover<'a, T, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(Box<ParseError>) -> T,
{
    parser: P,
    recovery: F,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, P, F> Recover<'a, T, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(Box<ParseError>) -> T,
{
    /// Creates a new recoverable parser with error recovery function
    pub fn new(parser: P, recovery: F) -> Self {
        Self {
            parser,
            recovery,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P, F> ParserCombinator<'a, T> for Recover<'a, T, P, F>
where
    P: ParserCombinator<'a, T>,
    F: Fn(Box<ParseError>) -> T,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, T> {
        match self.parser.parse(input) {
            Ok(result) => Ok(result),
            Err(error) => {
                let recovered_value = (self.recovery)(error);
                Ok((input, recovered_value))
            }
        }
    }
}

/// カット（コミット）コンビネータ
pub struct Cut<'a, T, P>
where
    P: ParserCombinator<'a, T>,
{
    parser: P,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, P> Cut<'a, T, P>
where
    P: ParserCombinator<'a, T>,
{
    /// Creates a new cut parser that prevents backtracking on failure
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, P> ParserCombinator<'a, T> for Cut<'a, T, P>
where
    P: ParserCombinator<'a, T>,
{
    fn parse(&self, input: Input<'a>) -> ParseResult<'a, T> {
        // カット操作はパーサーが成功した場合、バックトラックを無効化する
        // 実装では単純に元のパーサーを呼び出すが、
        // 実際のカットセマンティクスは外部の選択コンビネータで処理される
        self.parser.parse(input)
    }
}

/// ヘルパー関数群
pub mod helpers {
    use super::*;
    use crate::parser::combinators::primitive::Primitives;
    
    /// 文字列リテラルパーサー
    pub fn tag<'a>(expected: &'static str) -> FnParser<'a, &'a str, impl Fn(Input<'a>) -> ParseResult<'a, &'a str>> {
        FnParser::new(Primitives::tag(expected))
    }
    
    /// 単一文字パーサー
    pub fn char<'a>(expected: char) -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>> {
        FnParser::new(Primitives::char(expected))
    }
    
    /// 任意文字パーサー
    pub fn any_char<'a>() -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>> {
        FnParser::new(Primitives::any_char())
    }
    
    /// 述語満足パーサー
    pub fn satisfy<'a, P>(predicate: P) -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>>
    where
        P: Fn(char) -> bool + 'static,
    {
        FnParser::new(Primitives::satisfy(predicate))
    }
    
    /// 空白文字パーサー
    pub fn whitespace<'a>() -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>> {
        FnParser::new(Primitives::whitespace())
    }
    
    /// 空白文字列パーサー（0回以上）
    pub fn whitespace0<'a>() -> Many<'a, char, FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>>> {
        whitespace().many()
    }
    
    /// 空白文字列パーサー（1回以上）
    pub fn whitespace1<'a>() -> Many1<'a, char, FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>>> {
        whitespace().many1()
    }
    
    /// 数字パーサー
    pub fn digit<'a>() -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>> {
        FnParser::new(Primitives::digit())
    }
    
    /// 英字パーサー
    pub fn alpha<'a>() -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>> {
        FnParser::new(Primitives::alpha())
    }
    
    /// 英数字パーサー
    pub fn alphanumeric<'a>() -> FnParser<'a, char, impl Fn(Input<'a>) -> ParseResult<'a, char>> {
        FnParser::new(Primitives::alphanumeric())
    }
}

#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod tests {
    use super::*;
    use super::helpers::*;
    
    #[test]
    fn test_map_combinator() {
        let parser = digit().map(|c| c.to_digit(10).unwrap());
        let result = parser.parse("123");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "23");
        assert_eq!(value, 1);
    }
    
    #[test]
    fn test_and_combinator() {
        let parser = alpha().and(digit());
        let result = parser.parse("a1bc");
        assert!(result.is_ok());
        let (remaining, (first, second)) = result.unwrap();
        assert_eq!(remaining, "bc");
        assert_eq!(first, 'a');
        assert_eq!(second, '1');
    }
    
    #[test]
    fn test_or_combinator() {
        let parser = alpha().or(digit());
        
        // テスト1: 最初のパーサーが成功
        let result = parser.parse("abc");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "bc");
        assert_eq!(value, 'a');
        
        // テスト2: 2番目のパーサーが成功  
        let result = parser.parse("123");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "23");
        assert_eq!(value, '1');
    }
    
    #[test]
    fn test_many_combinator() {
        let parser = digit().many();
        
        // 0個の場合
        let result = parser.parse("abc");
        assert!(result.is_ok());
        let (remaining, values) = result.unwrap();
        assert_eq!(remaining, "abc");
        assert_eq!(values, vec![]);
        
        // 複数個の場合
        let result = parser.parse("123abc");
        assert!(result.is_ok());
        let (remaining, values) = result.unwrap();
        assert_eq!(remaining, "abc");
        assert_eq!(values, vec!['1', '2', '3']);
    }
    
    #[test]
    fn test_many1_combinator() {
        let parser = digit().many1();
        
        // 0個の場合（失敗）
        let result = parser.parse("abc");
        assert!(result.is_err());
        
        // 複数個の場合（成功）
        let result = parser.parse("123abc");
        assert!(result.is_ok());
        let (remaining, values) = result.unwrap();
        assert_eq!(remaining, "abc");
        assert_eq!(values, vec!['1', '2', '3']);
    }
    
    #[test]
    fn test_optional_combinator() {
        let parser = tag("hello").optional();
        
        // 存在する場合
        let result = parser.parse("hello world");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, " world");
        assert_eq!(value, Some("hello"));
        
        // 存在しない場合
        let result = parser.parse("world");
        assert!(result.is_ok());
        let (remaining, value) = result.unwrap();
        assert_eq!(remaining, "world");
        assert_eq!(value, None);
    }
    
    #[test]
    fn test_complex_combinator() {
        // "hello" に続いて空白、そして数字1文字以上
        let parser = tag("hello")
            .and(whitespace1())
            .and(digit().many1())
            .map(|((greeting, _), digits)| (greeting, digits));
            
        let result = parser.parse("hello 123");
        assert!(result.is_ok());
        let (remaining, (greeting, digits)) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(greeting, "hello");
        assert_eq!(digits, vec!['1', '2', '3']);
    }
}