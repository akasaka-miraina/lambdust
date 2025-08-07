//! Line and column position information.

use super::Span;

/// A position in source code (line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset from start of source
    pub offset: usize,
}

impl Position {
    /// Creates a new position.
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    /// Creates a position at the start of a file.
    pub fn start() -> Self {
        Self::new(1, 1, 0)
    }

    /// Creates a position from a span (basic implementation).
    pub fn from_span(span: &Span) -> Self {
        Self {
            line: 1, // TODO: Calculate from source
            column: 1, // TODO: Calculate from source
            offset: span.start,
        }
    }
}