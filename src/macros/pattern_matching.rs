//! Pattern and template definitions for syntax-rules

/// Pattern for syntax-rules
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal pattern (must match exactly)
    Literal(String),
    /// Variable pattern (binds to any expression)
    Variable(String),
    /// List pattern
    List(Vec<Pattern>),
    /// Ellipsis pattern (matches zero or more)
    Ellipsis(Box<Pattern>),
    /// Dotted pattern
    Dotted(Vec<Pattern>, Box<Pattern>),
    /// Nested ellipsis pattern (SRFI 46 extension)
    NestedEllipsis(Box<Pattern>, usize), // pattern with nesting level
    /// Vector pattern (SRFI 46 extension)
    Vector(Vec<Pattern>),
}

/// Template for syntax-rules
#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    /// Literal template
    Literal(String),
    /// Variable reference
    Variable(String),
    /// List template
    List(Vec<Template>),
    /// Ellipsis template (expands pattern variables)
    Ellipsis(Box<Template>),
    /// Dotted template
    Dotted(Vec<Template>, Box<Template>),
    /// Nested ellipsis template (SRFI 46 extension)
    NestedEllipsis(Box<Template>, usize), // template with nesting level
    /// Vector template (SRFI 46 extension)
    Vector(Vec<Template>),
}

/// Syntax rule (pattern -> template)
///
/// Represents a single transformation rule in a syntax-rules macro definition.
/// Each rule consists of a pattern that matches input expressions and a template
/// that specifies how to transform the matched input.
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    /// The pattern to match against input expressions
    pub pattern: Pattern,
    /// The template for generating the output expression
    pub template: Template,
}