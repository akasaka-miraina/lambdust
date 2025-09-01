//! Abstract Syntax Tree for SRFI-115 Symbolic Regular Expressions
//!
//! This module defines the internal representation of SRE patterns after parsing.
//! The AST is designed to be:
//! - Easy to traverse and transform during compilation
//! - Efficient to construct during parsing
//! - Rich enough to preserve all SRE semantics
//! - Suitable for optimization passes

use crate::utils::SymbolId;
use std::collections::HashMap;
use std::fmt;

/// Complete AST representation of an SRE pattern
#[derive(Debug, Clone, PartialEq)]
pub struct SreAst {
    /// Root node of the AST
    pub root: SreNode,
    /// Named groups defined in this pattern (name -> group_index)
    pub named_groups: HashMap<String, usize>,
    /// Total number of capture groups (including unnamed)
    pub total_groups: usize,
    /// Compilation flags that affect the entire pattern
    pub flags: AstFlags,
}

/// Compilation flags embedded in AST
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AstFlags {
    pub case_insensitive: bool,
    pub multiline: bool,
    pub dotall: bool,
    pub unicode: bool,
    pub extended: bool,
}

/// Node types in the SRE AST
#[derive(Debug, Clone, PartialEq)]
pub enum SreNode {
    /// String literal: "hello"
    Literal(String),

    /// Single character literal: #\a
    Character(char),

    /// Character set: [abc], [^abc], [:alpha:]
    CharacterSet(CharacterSet),

    /// Sequence: (: a b c) or implicit sequence
    Sequence(Vec<SreNode>),

    /// Alternation: (or a b c)
    Alternation(Vec<SreNode>),

    /// Zero or more: (* pattern)
    ZeroOrMore(Box<SreNode>),

    /// One or more: (+ pattern)
    OneOrMore(Box<SreNode>),

    /// Optional: (? pattern)
    Optional(Box<SreNode>),

    /// Exact repetition: (= n pattern)
    Exactly {
        count: usize,
        node: Box<SreNode>,
    },

    /// Bounded repetition: (>= n pattern), (<= n pattern), (** m n pattern)
    Repeat {
        min: usize,
        max: Option<usize>, // None means unbounded
        node: Box<SreNode>,
    },

    /// Capture group: ($ pattern) or (submatch pattern)
    Group {
        index: usize,
        name: Option<String>,
        node: Box<SreNode>,
    },

    /// Non-capturing group: (: pattern)
    NonCapturingGroup(Box<SreNode>),

    /// Backreference: (backref n) or (backref "name")
    Backreference {
        target: BackrefTarget,
    },

    /// Anchors: bos, eos, bol, eol, bow, eow, nwb
    Anchor(AnchorType),

    /// Lookahead: (?= pattern), (?! pattern)
    Lookahead {
        positive: bool,
        node: Box<SreNode>,
    },

    /// Lookbehind: (?<= pattern), (?<! pattern)
    Lookbehind {
        positive: bool,
        node: Box<SreNode>,
    },

    /// Conditional: (if condition then-pattern else-pattern)
    Conditional {
        condition: Box<SreNode>,
        then_branch: Box<SreNode>,
        else_branch: Option<Box<SreNode>>,
    },

    /// Word boundary assertions
    WordBoundary(BoundaryType),

    /// Unicode property: (/ char-set property)
    UnicodeProperty(UnicodePropertySet),

    /// Named character class: digit, word, space, etc.
    NamedClass(NamedCharacterClass),

    /// Atomic group (possessive matching): (atomic pattern)
    AtomicGroup(Box<SreNode>),

    /// Empty pattern (matches empty string)
    Empty,
}

/// Character set specification
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSet {
    /// Individual characters and ranges
    pub ranges: Vec<CharRange>,
    /// Named character classes included
    pub classes: Vec<NamedCharacterClass>,
    /// Unicode properties included
    pub unicode_properties: Vec<UnicodePropertySet>,
    /// Whether this is a negated set [^...]
    pub negated: bool,
    /// Case-insensitive matching for this set
    pub case_insensitive: bool,
}

/// A character range: 'a' or 'a'-'z'
#[derive(Debug, Clone, PartialEq)]
pub enum CharRange {
    /// Single character
    Single(char),
    /// Character range (inclusive)
    Range(char, char),
}

/// Named character classes from SRFI-115
#[derive(Debug, Clone, PartialEq)]
pub enum NamedCharacterClass {
    /// Any character except newline
    Any,
    /// Alphabetic characters
    Alphabetic,
    /// Alphanumeric characters  
    Alphanumeric,
    /// ASCII characters
    Ascii,
    /// Blank characters (space and tab)
    Blank,
    /// Control characters
    Control,
    /// Decimal digits
    Digit,
    /// Graphical characters
    Graph,
    /// Lowercase letters
    Lower,
    /// Newline characters
    Newline,
    /// Non-digit characters
    NonDigit,
    /// Non-word characters
    NonWord,
    /// Non-whitespace characters
    NonWhitespace,
    /// Numeric characters
    Numeric,
    /// Printable characters
    Print,
    /// Punctuation characters
    Punctuation,
    /// Whitespace characters
    Space,
    /// Uppercase letters
    Upper,
    /// Word characters (letters, digits, underscore)
    Word,
    /// Hexadecimal digit characters
    Hex,
    /// POSIX character class by name
    Posix(String),
}

/// Unicode property sets
#[derive(Debug, Clone, PartialEq)]
pub enum UnicodePropertySet {
    /// General Category: Ll, Lu, Nd, etc.
    GeneralCategory(String),
    /// Script: Latin, Greek, Cyrillic, etc.
    Script(String),
    /// Block: BasicLatin, LatinExtendedA, etc.
    Block(String),
    /// Binary property: Alphabetic, Whitespace, etc.
    Binary(String),
    /// Age property: 1.1, 2.0, etc.
    Age(String),
    /// Canonical combining class
    CanonicalCombiningClass(u8),
    /// Bidi class: L, R, EN, etc.
    BidiClass(String),
    /// East Asian Width: F, H, W, Na, etc.
    EastAsianWidth(String),
}

/// Anchor types for position assertions
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorType {
    /// Beginning of string
    BeginningOfString,
    /// End of string  
    EndOfString,
    /// Beginning of line (after newline or start)
    BeginningOfLine,
    /// End of line (before newline or end)
    EndOfLine,
    /// Beginning of word
    BeginningOfWord,
    /// End of word
    EndOfWord,
    /// Not word boundary
    NotWordBoundary,
}

/// Word boundary types
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryType {
    /// Standard word boundary
    Word,
    /// Not word boundary
    NotWord,
    /// Beginning of word
    BeginWord,
    /// End of word
    EndWord,
    /// Grapheme boundary
    Grapheme,
    /// Sentence boundary
    Sentence,
    /// Line boundary
    Line,
}

/// Backreference target
#[derive(Debug, Clone, PartialEq)]
pub enum BackrefTarget {
    /// Reference by number (1-based)
    Index(usize),
    /// Reference by name
    Name(String),
}

impl SreAst {
    /// Create a new empty AST
    pub fn new(root: SreNode) -> Self {
        Self {
            root,
            named_groups: HashMap::new(),
            total_groups: 0,
            flags: AstFlags::default(),
        }
    }

    /// Create AST with flags
    pub fn with_flags(root: SreNode, flags: AstFlags) -> Self {
        Self {
            root,
            named_groups: HashMap::new(),
            total_groups: 0,
            flags,
        }
    }

    /// Add a named group to the AST
    pub fn add_named_group(&mut self, name: String, index: usize) {
        self.named_groups.insert(name, index);
        if index >= self.total_groups {
            self.total_groups = index + 1;
        }
    }

    /// Get the index of a named group
    pub fn get_named_group(&self, name: &str) -> Option<usize> {
        self.named_groups.get(name).copied()
    }

    /// Increment total groups counter
    pub fn increment_groups(&mut self) -> usize {
        let index = self.total_groups;
        self.total_groups += 1;
        index
    }

    /// Check if the AST is empty (matches empty string)
    pub fn is_empty(&self) -> bool {
        matches!(self.root, SreNode::Empty)
    }

    /// Check if the AST contains only literals
    pub fn is_literal_only(&self) -> bool {
        self.is_node_literal_only(&self.root)
    }

    /// Recursively check if a node contains only literals
    fn is_node_literal_only(&self, node: &SreNode) -> bool {
        match node {
            SreNode::Literal(_) | SreNode::Character(_) | SreNode::Empty => true,
            SreNode::Sequence(nodes) | SreNode::Alternation(nodes) => {
                nodes.iter().all(|n| self.is_node_literal_only(n))
            }
            SreNode::NonCapturingGroup(inner) | SreNode::AtomicGroup(inner) => {
                self.is_node_literal_only(inner)
            }
            _ => false,
        }
    }

    /// Extract literal text if the AST represents a simple literal pattern
    pub fn as_literal(&self) -> Option<String> {
        self.node_as_literal(&self.root)
    }

    /// Extract literal text from a node if it's a simple literal
    fn node_as_literal(&self, node: &SreNode) -> Option<String> {
        match node {
            SreNode::Literal(s) => Some(s.clone()),
            SreNode::Character(c) => Some(c.to_string()),
            SreNode::Empty => Some(String::new()),
            SreNode::Sequence(nodes) => {
                let mut result = String::new();
                for node in nodes {
                    result.push_str(&self.node_as_literal(node)?);
                }
                Some(result)
            }
            SreNode::NonCapturingGroup(inner) => self.node_as_literal(inner),
            _ => None,
        }
    }
}

impl CharacterSet {
    /// Create a new character set
    pub fn new() -> Self {
        Self {
            ranges: Vec::new(),
            classes: Vec::new(),
            unicode_properties: Vec::new(),
            negated: false,
            case_insensitive: false,
        }
    }

    /// Create a negated character set
    pub fn negated() -> Self {
        Self {
            ranges: Vec::new(),
            classes: Vec::new(),
            unicode_properties: Vec::new(),
            negated: true,
            case_insensitive: false,
        }
    }

    /// Add a single character
    pub fn add_char(mut self, c: char) -> Self {
        self.ranges.push(CharRange::Single(c));
        self
    }

    /// Add a character range
    pub fn add_range(mut self, start: char, end: char) -> Self {
        self.ranges.push(CharRange::Range(start, end));
        self
    }

    /// Add a named character class
    pub fn add_class(mut self, class: NamedCharacterClass) -> Self {
        self.classes.push(class);
        self
    }

    /// Add a Unicode property
    pub fn add_unicode_property(mut self, property: UnicodePropertySet) -> Self {
        self.unicode_properties.push(property);
        self
    }

    /// Set case insensitive flag
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    /// Check if the character set is empty
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty() && self.classes.is_empty() && self.unicode_properties.is_empty()
    }

    /// Check if this represents a simple single-character set
    pub fn is_single_char(&self) -> Option<char> {
        if self.negated || self.case_insensitive {
            return None;
        }
        
        if !self.classes.is_empty() || !self.unicode_properties.is_empty() {
            return None;
        }

        if self.ranges.len() == 1 {
            match &self.ranges[0] {
                CharRange::Single(c) => Some(*c),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Default for CharacterSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common AST nodes
impl SreNode {
    /// Create a literal node
    pub fn literal(text: impl Into<String>) -> Self {
        Self::Literal(text.into())
    }

    /// Create a character node
    pub fn character(c: char) -> Self {
        Self::Character(c)
    }

    /// Create a sequence node
    pub fn sequence(nodes: Vec<SreNode>) -> Self {
        match nodes.len() {
            0 => Self::Empty,
            1 => nodes.into_iter().next().unwrap(),
            _ => Self::Sequence(nodes),
        }
    }

    /// Create an alternation node
    pub fn alternation(nodes: Vec<SreNode>) -> Self {
        match nodes.len() {
            0 => Self::Empty,
            1 => nodes.into_iter().next().unwrap(),
            _ => Self::Alternation(nodes),
        }
    }

    /// Create a zero-or-more node
    pub fn zero_or_more(node: SreNode) -> Self {
        Self::ZeroOrMore(Box::new(node))
    }

    /// Create a one-or-more node
    pub fn one_or_more(node: SreNode) -> Self {
        Self::OneOrMore(Box::new(node))
    }

    /// Create an optional node
    pub fn optional(node: SreNode) -> Self {
        Self::Optional(Box::new(node))
    }

    /// Create an exactly node
    pub fn exactly(count: usize, node: SreNode) -> Self {
        Self::Exactly {
            count,
            node: Box::new(node),
        }
    }

    /// Create a repeat node
    pub fn repeat(min: usize, max: Option<usize>, node: SreNode) -> Self {
        Self::Repeat {
            min,
            max,
            node: Box::new(node),
        }
    }

    /// Create a capture group node
    pub fn group(index: usize, name: Option<String>, node: SreNode) -> Self {
        Self::Group {
            index,
            name,
            node: Box::new(node),
        }
    }

    /// Create a non-capturing group
    pub fn non_capturing_group(node: SreNode) -> Self {
        Self::NonCapturingGroup(Box::new(node))
    }

    /// Check if this node can match empty string
    pub fn can_match_empty(&self) -> bool {
        match self {
            SreNode::Empty => true,
            SreNode::Literal(s) => s.is_empty(),
            SreNode::Character(_) => false,
            SreNode::CharacterSet(_) => false,
            SreNode::Sequence(nodes) => nodes.iter().all(|n| n.can_match_empty()),
            SreNode::Alternation(nodes) => nodes.iter().any(|n| n.can_match_empty()),
            SreNode::ZeroOrMore(_) | SreNode::Optional(_) => true,
            SreNode::OneOrMore(inner) => inner.can_match_empty(),
            SreNode::Exactly { count: 0, .. } => true,
            SreNode::Exactly { count, node } => *count == 0 || node.can_match_empty(),
            SreNode::Repeat { min: 0, .. } => true,
            SreNode::Repeat { min, node, .. } => *min == 0 || node.can_match_empty(),
            SreNode::Group { node, .. } => node.can_match_empty(),
            SreNode::NonCapturingGroup(inner) => inner.can_match_empty(),
            SreNode::AtomicGroup(inner) => inner.can_match_empty(),
            SreNode::Anchor(_) => true,
            SreNode::WordBoundary(_) => true,
            SreNode::Lookahead { .. } | SreNode::Lookbehind { .. } => true,
            SreNode::Conditional { then_branch, else_branch, .. } => {
                then_branch.can_match_empty() || 
                else_branch.as_ref().map_or(true, |n| n.can_match_empty())
            }
            SreNode::Backreference { .. } => true, // Conservative
            SreNode::UnicodeProperty(_) => false,
            SreNode::NamedClass(_) => false,
        }
    }
}

/// Display implementation for debugging
impl fmt::Display for SreNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SreNode::Literal(s) => write!(f, "\"{}\"", s),
            SreNode::Character(c) => write!(f, "#\\{}", c),
            SreNode::CharacterSet(cs) => write!(f, "[{}]", 
                if cs.negated { "^..." } else { "..." }),
            SreNode::Sequence(nodes) => {
                write!(f, "(:")?;
                for node in nodes {
                    write!(f, " {}", node)?;
                }
                write!(f, ")")
            }
            SreNode::Alternation(nodes) => {
                write!(f, "(or")?;
                for node in nodes {
                    write!(f, " {}", node)?;
                }
                write!(f, ")")
            }
            SreNode::ZeroOrMore(node) => write!(f, "(* {})", node),
            SreNode::OneOrMore(node) => write!(f, "(+ {})", node),
            SreNode::Optional(node) => write!(f, "(? {})", node),
            SreNode::Exactly { count, node } => write!(f, "(= {} {})", count, node),
            SreNode::Repeat { min, max, node } => {
                match max {
                    Some(max_val) => write!(f, "(** {} {} {})", min, max_val, node),
                    None => write!(f, "(>= {} {})", min, node),
                }
            }
            SreNode::Group { index, name, node } => {
                if let Some(name) = name {
                    write!(f, "(=> {} {})", name, node)
                } else {
                    write!(f, "($ {})", node)
                }
            }
            SreNode::NonCapturingGroup(node) => write!(f, "(: {})", node),
            SreNode::Backreference { target } => {
                match target {
                    BackrefTarget::Index(i) => write!(f, "(backref {})", i),
                    BackrefTarget::Name(name) => write!(f, "(backref \"{}\")", name),
                }
            }
            SreNode::Anchor(anchor) => write!(f, "{:?}", anchor),
            SreNode::WordBoundary(boundary) => write!(f, "{:?}", boundary),
            SreNode::Lookahead { positive, node } => {
                write!(f, "({}?= {})", if *positive { "" } else { "!" }, node)
            }
            SreNode::Lookbehind { positive, node } => {
                write!(f, "({}?<= {})", if *positive { "" } else { "!" }, node)
            }
            SreNode::NamedClass(class) => write!(f, "{:?}", class),
            SreNode::UnicodeProperty(prop) => write!(f, "{:?}", prop),
            SreNode::AtomicGroup(node) => write!(f, "(atomic {})", node),
            SreNode::Empty => write!(f, "''"),
            SreNode::Conditional { condition, then_branch, else_branch } => {
                write!(f, "(if {} {}", condition, then_branch)?;
                if let Some(else_node) = else_branch {
                    write!(f, " {}", else_node)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        let ast = SreAst::new(SreNode::literal("hello"));
        assert_eq!(ast.total_groups, 0);
        assert!(ast.named_groups.is_empty());
    }

    #[test]
    fn test_literal_only_detection() {
        let ast = SreAst::new(SreNode::literal("hello"));
        assert!(ast.is_literal_only());

        let ast2 = SreAst::new(SreNode::one_or_more(SreNode::literal("a")));
        assert!(!ast2.is_literal_only());
    }

    #[test]
    fn test_as_literal() {
        let ast = SreAst::new(SreNode::literal("hello"));
        assert_eq!(ast.as_literal(), Some("hello".to_string()));

        let seq = SreAst::new(SreNode::sequence(vec![
            SreNode::literal("hello"),
            SreNode::character(' '),
            SreNode::literal("world"),
        ]));
        assert_eq!(seq.as_literal(), Some("hello world".to_string()));
    }

    #[test]
    fn test_character_set() {
        let charset = CharacterSet::new()
            .add_char('a')
            .add_range('0', '9')
            .add_class(NamedCharacterClass::Word);

        assert!(!charset.is_empty());
        assert_eq!(charset.is_single_char(), None); // Complex set

        let single = CharacterSet::new().add_char('x');
        assert_eq!(single.is_single_char(), Some('x'));
    }

    #[test]
    fn test_can_match_empty() {
        assert!(SreNode::Empty.can_match_empty());
        assert!(SreNode::literal("").can_match_empty());
        assert!(!SreNode::literal("a").can_match_empty());
        assert!(SreNode::zero_or_more(SreNode::literal("a")).can_match_empty());
        assert!(!SreNode::one_or_more(SreNode::literal("a")).can_match_empty());
        assert!(SreNode::optional(SreNode::literal("a")).can_match_empty());
    }

    #[test]
    fn test_named_groups() {
        let mut ast = SreAst::new(SreNode::Empty);
        ast.add_named_group("test".to_string(), 1);
        
        assert_eq!(ast.get_named_group("test"), Some(1));
        assert_eq!(ast.get_named_group("missing"), None);
        assert_eq!(ast.total_groups, 2); // 0-based, so group 1 means 2 total
    }

    #[test]
    fn test_node_display() {
        let node = SreNode::sequence(vec![
            SreNode::literal("hello"),
            SreNode::one_or_more(SreNode::character('a')),
        ]);
        let display = format!("{}", node);
        assert!(display.contains("hello"));
        assert!(display.contains("(+"));
    }

    #[test]
    fn test_sequence_optimization() {
        // Single element sequence should be optimized away
        let seq = SreNode::sequence(vec![SreNode::literal("hello")]);
        assert!(matches!(seq, SreNode::Literal(_)));

        // Empty sequence should become Empty
        let empty_seq = SreNode::sequence(vec![]);
        assert!(matches!(empty_seq, SreNode::Empty));
    }
}