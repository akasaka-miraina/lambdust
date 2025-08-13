//! Regular expression pattern parser for Thompson NFA construction.
//!
//! This module implements a recursive descent parser for regular expression
//! patterns, converting them into an Abstract Syntax Tree (AST) suitable
//! for Thompson NFA construction.
//!
//! ## Supported Syntax (Phase 1)
//!
//! **Basic Metacharacters:**
//! - `.` - Any character (except newline)
//! - `^` - Start of string/line
//! - `$` - End of string/line
//!
//! **Character Classes:**
//! - `[abc]` - Character set
//! - `[a-z]` - Character range
//! - `[^abc]` - Negated character set
//! - `\d` - ASCII digits [0-9]
//! - `\w` - Word characters [a-zA-Z0-9_]
//! - `\s` - Whitespace characters
//!
//! **Quantifiers:**
//! - `*` - Zero or more
//! - `+` - One or more  
//! - `?` - Zero or one
//!
//! **Grouping:**
//! - `(...)` - Basic grouping
//!
//! **Escapes:**
//! - `\\` - Literal backslash
//! - `\.` - Literal dot
//! - `\t` - Tab character
//! - `\n` - Newline character
//! - `\r` - Carriage return
//!
//! ## Grammar
//!
//! ```
//! Pattern     ::= Alternation
//! Alternation ::= Sequence ('|' Sequence)*  
//! Sequence    ::= Factor*
//! Factor      ::= Atom Quantifier?
//! Atom        ::= Char | CharClass | Group | Anchor | '.'
//! Quantifier  ::= '*' | '+' | '?'
//! Group       ::= '(' Alternation ')'
//! CharClass   ::= '[' ClassItems ']'
//! Anchor      ::= '^' | '$'
//! ```

use std::fmt;
use crate::regex::engine::CharClass;

/// Error type for pattern parsing.
#[derive(Debug, Clone)]
pub enum PatternError {
    /// Unexpected character in pattern
    UnexpectedChar(char, usize),
    /// Unexpected end of pattern
    UnexpectedEnd,
    /// Invalid character class
    InvalidCharClass(String),
    /// Invalid escape sequence
    InvalidEscape(char, usize),
    /// Unmatched parenthesis
    UnmatchedParen,
    /// Empty group
    EmptyGroup,
    /// Unsupported feature
    UnsupportedFeature(String),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternError::UnexpectedChar(ch, pos) => {
                write!(f, "Unexpected character '{ch}' at position {pos}")
            }
            PatternError::UnexpectedEnd => write!(f, "Unexpected end of pattern"),
            PatternError::InvalidCharClass(msg) => write!(f, "Invalid character class: {msg}"),
            PatternError::InvalidEscape(ch, pos) => {
                write!(f, "Invalid escape sequence '\\{ch}' at position {pos}")
            }
            PatternError::UnmatchedParen => write!(f, "Unmatched parenthesis"),
            PatternError::EmptyGroup => write!(f, "Empty group"),
            PatternError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {msg}"),
        }
    }
}

impl std::error::Error for PatternError {}

/// AST node representing a parsed regular expression pattern.
#[derive(Debug, Clone)]
pub enum PatternNode {
    /// Single character match
    Char(char),
    /// Character class match
    CharClass(CharClass),
    /// Any character (.)
    Any,
    /// Start of string/line anchor (^)
    Start,
    /// End of string/line anchor ($)
    End,
    /// Concatenation of patterns
    Concat(Vec<PatternNode>),
    /// Alternation of patterns (|)
    Alternate(Vec<PatternNode>),
    /// Zero or more repetitions (*)
    Star(Box<PatternNode>),
    /// One or more repetitions (+)
    Plus(Box<PatternNode>),
    /// Zero or one repetition (?)
    Question(Box<PatternNode>),
    /// Grouping (capturing group)
    Group(Box<PatternNode>),
}

/// Complete parsed pattern with metadata.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Root AST node
    pub root: PatternNode,
    /// Original pattern string
    pub source: String,
}

/// Regular expression pattern parser.
pub struct PatternParser<'p> {
    /// Pattern being parsed
    pattern: &'p str,
    /// Current position in pattern
    pos: usize,
    /// Characters of the pattern
    chars: Vec<char>,
}

impl<'p> PatternParser<'p> {
    /// Creates a new parser for the given pattern.
    pub fn new(pattern: &'p str) -> Self {
        Self {
            pattern,
            pos: 0,
            chars: pattern.chars().collect(),
        }
    }
    
    /// Parses the pattern into an AST.
    pub fn parse(mut self) -> Result<Pattern, PatternError> {
        let root = self.parse_alternation()?;
        
        if self.pos < self.chars.len() {
            return Err(PatternError::UnexpectedChar(self.chars[self.pos], self.pos));
        }
        
        Ok(Pattern {
            root,
            source: self.pattern.to_string(),
        })
    }
    
    /// Current character at position, if any.
    fn current_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    
    /// Peeks at character at offset from current position.
    fn peek_char(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }
    
    /// Advances to next character.
    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }
    
    /// Consumes expected character.
    fn expect(&mut self, expected: char) -> Result<(), PatternError> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(PatternError::UnexpectedChar(ch, self.pos - 1)),
            None => Err(PatternError::UnexpectedEnd),
        }
    }
    
    /// Parses alternation (|).
    fn parse_alternation(&mut self) -> Result<PatternNode, PatternError> {
        let mut alternatives = vec![self.parse_sequence()?];
        
        while self.current_char() == Some('|') {
            self.advance(); // consume '|'
            alternatives.push(self.parse_sequence()?);
        }
        
        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            Ok(PatternNode::Alternate(alternatives))
        }
    }
    
    /// Parses sequence (concatenation).
    fn parse_sequence(&mut self) -> Result<PatternNode, PatternError> {
        let mut factors = Vec::new();
        
        while let Some(ch) = self.current_char() {
            // Stop at alternation or group end
            if ch == '|' || ch == ')' {
                break;
            }
            
            factors.push(self.parse_factor()?);
        }
        
        if factors.is_empty() {
            // Empty sequence - create epsilon node
            Ok(PatternNode::Concat(vec![]))
        } else if factors.len() == 1 {
            Ok(factors.into_iter().next().unwrap())
        } else {
            Ok(PatternNode::Concat(factors))
        }
    }
    
    /// Parses factor (atom with optional quantifier).
    fn parse_factor(&mut self) -> Result<PatternNode, PatternError> {
        let atom = self.parse_atom()?;
        
        match self.current_char() {
            Some('*') => {
                self.advance();
                Ok(PatternNode::Star(Box::new(atom)))
            }
            Some('+') => {
                self.advance();
                Ok(PatternNode::Plus(Box::new(atom)))
            }
            Some('?') => {
                self.advance();
                Ok(PatternNode::Question(Box::new(atom)))
            }
            _ => Ok(atom),
        }
    }
    
    /// Parses atomic expressions.
    fn parse_atom(&mut self) -> Result<PatternNode, PatternError> {
        match self.current_char() {
            Some('.') => {
                self.advance();
                Ok(PatternNode::Any)
            }
            Some('^') => {
                self.advance();
                Ok(PatternNode::Start)
            }
            Some('$') => {
                self.advance();
                Ok(PatternNode::End)
            }
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('\\') => self.parse_escape(),
            Some(ch) if !is_meta_char(ch) => {
                self.advance();
                Ok(PatternNode::Char(ch))
            }
            Some(ch) => Err(PatternError::UnexpectedChar(ch, self.pos)),
            None => Err(PatternError::UnexpectedEnd),
        }
    }
    
    /// Parses grouped expressions.
    fn parse_group(&mut self) -> Result<PatternNode, PatternError> {
        self.expect('(')?;
        
        let inner = self.parse_alternation()?;
        
        self.expect(')')?;
        
        Ok(PatternNode::Group(Box::new(inner)))
    }
    
    /// Parses character classes [abc].
    fn parse_char_class(&mut self) -> Result<PatternNode, PatternError> {
        self.expect('[')?;
        
        let mut class = CharClass::new();
        let mut negated = false;
        
        // Check for negation
        if self.current_char() == Some('^') {
            self.advance();
            negated = true;
        }
        
        // Parse character class contents
        while let Some(ch) = self.current_char() {
            if ch == ']' {
                break;
            }
            
            self.advance();
            
            // Check for range
            if self.current_char() == Some('-') && self.peek_char(1) != Some(']') {
                self.advance(); // consume '-'
                
                let end_char = match self.advance() {
                    Some(end) => end,
                    None => return Err(PatternError::UnexpectedEnd),
                };
                
                if ch > end_char {
                    return Err(PatternError::InvalidCharClass(
                        format!("Invalid range {ch}-{end_char}: start > end")
                    ));
                }
                
                class.add_range(ch, end_char);
            } else {
                class.add_char(ch);
            }
        }
        
        self.expect(']')?;
        
        if negated {
            class = class.negate();
        }
        
        Ok(PatternNode::CharClass(class))
    }
    
    /// Parses escape sequences.
    fn parse_escape(&mut self) -> Result<PatternNode, PatternError> {
        self.expect('\\')?;
        
        match self.advance() {
            Some('\\') => Ok(PatternNode::Char('\\')),
            Some('.') => Ok(PatternNode::Char('.')),
            Some('*') => Ok(PatternNode::Char('*')),
            Some('+') => Ok(PatternNode::Char('+')),
            Some('?') => Ok(PatternNode::Char('?')),
            Some('(') => Ok(PatternNode::Char('(')),
            Some(')') => Ok(PatternNode::Char(')')),
            Some('[') => Ok(PatternNode::Char('[')),
            Some(']') => Ok(PatternNode::Char(']')),
            Some('^') => Ok(PatternNode::Char('^')),
            Some('$') => Ok(PatternNode::Char('$')),
            Some('|') => Ok(PatternNode::Char('|')),
            Some('t') => Ok(PatternNode::Char('\t')),
            Some('n') => Ok(PatternNode::Char('\n')),
            Some('r') => Ok(PatternNode::Char('\r')),
            Some('d') => Ok(PatternNode::CharClass(CharClass::builtin(
                crate::regex::engine::BuiltinClass::Digit
            ))),
            Some('w') => Ok(PatternNode::CharClass(CharClass::builtin(
                crate::regex::engine::BuiltinClass::Word
            ))),
            Some('s') => Ok(PatternNode::CharClass(CharClass::builtin(
                crate::regex::engine::BuiltinClass::Space
            ))),
            Some(ch) => Err(PatternError::InvalidEscape(ch, self.pos - 1)),
            None => Err(PatternError::UnexpectedEnd),
        }
    }
}

/// Tests if a character has special meaning in regex.
fn is_meta_char(ch: char) -> bool {
    matches!(ch, '.' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '|' | '\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char() {
        let pattern = PatternParser::new("a").parse().unwrap();
        match pattern.root {
            PatternNode::Char('a') => {}
            _ => panic!("Expected single character"),
        }
    }
    
    #[test]
    fn test_concatenation() {
        let pattern = PatternParser::new("abc").parse().unwrap();
        match pattern.root {
            PatternNode::Concat(nodes) => {
                assert_eq!(nodes.len(), 3);
                match (&nodes[0], &nodes[1], &nodes[2]) {
                    (PatternNode::Char('a'), PatternNode::Char('b'), PatternNode::Char('c')) => {}
                    _ => panic!("Expected abc concatenation"),
                }
            }
            _ => panic!("Expected concatenation"),
        }
    }
    
    #[test]
    fn test_alternation() {
        let pattern = PatternParser::new("a|b").parse().unwrap();
        match pattern.root {
            PatternNode::Alternate(alts) => {
                assert_eq!(alts.len(), 2);
                match (&alts[0], &alts[1]) {
                    (PatternNode::Char('a'), PatternNode::Char('b')) => {}
                    _ => panic!("Expected a|b alternation"),
                }
            }
            _ => panic!("Expected alternation"),
        }
    }
    
    #[test]
    fn test_quantifiers() {
        let star = PatternParser::new("a*").parse().unwrap();
        match star.root {
            PatternNode::Star(inner) => match inner.as_ref() {
                PatternNode::Char('a') => {}
                _ => panic!("Expected 'a' in star"),
            },
            _ => panic!("Expected star"),
        }
        
        let plus = PatternParser::new("a+").parse().unwrap();
        match plus.root {
            PatternNode::Plus(inner) => match inner.as_ref() {
                PatternNode::Char('a') => {}
                _ => panic!("Expected 'a' in plus"),
            },
            _ => panic!("Expected plus"),
        }
        
        let question = PatternParser::new("a?").parse().unwrap();
        match question.root {
            PatternNode::Question(inner) => match inner.as_ref() {
                PatternNode::Char('a') => {}
                _ => panic!("Expected 'a' in question"),
            },
            _ => panic!("Expected question"),
        }
    }
    
    #[test]
    fn test_character_class() {
        let pattern = PatternParser::new("[abc]").parse().unwrap();
        match pattern.root {
            PatternNode::CharClass(class) => {
                assert!(class.matches('a'));
                assert!(class.matches('b'));
                assert!(class.matches('c'));
                assert!(!class.matches('d'));
            }
            _ => panic!("Expected character class"),
        }
    }
    
    #[test]
    fn test_character_range() {
        let pattern = PatternParser::new("[a-z]").parse().unwrap();
        match pattern.root {
            PatternNode::CharClass(class) => {
                assert!(class.matches('a'));
                assert!(class.matches('m'));
                assert!(class.matches('z'));
                assert!(!class.matches('A'));
                assert!(!class.matches('0'));
            }
            _ => panic!("Expected character range"),
        }
    }
    
    #[test]
    fn test_negated_class() {
        let pattern = PatternParser::new("[^abc]").parse().unwrap();
        match pattern.root {
            PatternNode::CharClass(class) => {
                assert!(!class.matches('a'));
                assert!(!class.matches('b'));
                assert!(!class.matches('c'));
                assert!(class.matches('d'));
                assert!(class.matches('x'));
            }
            _ => panic!("Expected negated character class"),
        }
    }
    
    #[test]
    fn test_builtin_classes() {
        let digit = PatternParser::new(r"\d").parse().unwrap();
        match digit.root {
            PatternNode::CharClass(class) => {
                assert!(class.matches('5'));
                assert!(!class.matches('a'));
            }
            _ => panic!("Expected digit class"),
        }
        
        let word = PatternParser::new(r"\w").parse().unwrap();
        match word.root {
            PatternNode::CharClass(class) => {
                assert!(class.matches('a'));
                assert!(class.matches('5'));
                assert!(class.matches('_'));
                assert!(!class.matches('@'));
            }
            _ => panic!("Expected word class"),
        }
    }
    
    #[test]
    fn test_escapes() {
        let backslash = PatternParser::new(r"\\").parse().unwrap();
        match backslash.root {
            PatternNode::Char('\\') => {}
            _ => panic!("Expected literal backslash"),
        }
        
        let dot = PatternParser::new(r"\.").parse().unwrap();
        match dot.root {
            PatternNode::Char('.') => {}
            _ => panic!("Expected literal dot"),
        }
    }
    
    #[test]
    fn test_any_char() {
        let pattern = PatternParser::new(".").parse().unwrap();
        match pattern.root {
            PatternNode::Any => {}
            _ => panic!("Expected any character"),
        }
    }
    
    #[test]
    fn test_anchors() {
        let start = PatternParser::new("^").parse().unwrap();
        match start.root {
            PatternNode::Start => {}
            _ => panic!("Expected start anchor"),
        }
        
        let end = PatternParser::new("$").parse().unwrap();
        match end.root {
            PatternNode::End => {}
            _ => panic!("Expected end anchor"),
        }
    }
    
    #[test]
    fn test_groups() {
        let pattern = PatternParser::new("(abc)").parse().unwrap();
        match pattern.root {
            PatternNode::Group(inner) => match inner.as_ref() {
                PatternNode::Concat(nodes) => {
                    assert_eq!(nodes.len(), 3);
                }
                _ => panic!("Expected concatenation in group"),
            },
            _ => panic!("Expected group"),
        }
    }
    
    #[test]
    fn test_complex_pattern() {
        let pattern = PatternParser::new(r"\d+\.\d*").parse().unwrap();
        // Should parse as: concat(plus(digit), char('.'), star(digit))
        match pattern.root {
            PatternNode::Concat(nodes) => {
                assert_eq!(nodes.len(), 3);
                // Detailed structure testing would go here
            }
            _ => panic!("Expected concatenation"),
        }
    }
}