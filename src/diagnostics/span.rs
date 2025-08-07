//! Source location information for error reporting.

/// A span represents a location in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Span {
    /// Starting byte position
    pub start: usize,
    /// Length in bytes
    pub len: usize,
    /// Optional source file name
    pub file_id: Option<usize>,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// A wrapper that adds span information to any type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Spanned<T> {
    /// The wrapped value
    pub inner: T,
    /// The source location span
    pub span: Span,
}

impl Span {
    /// Creates a new span.
    pub fn new(start: usize, len: usize) -> Self {
        Self { 
            start, 
            len,
            file_id: None,
            line: 1,
            column: 1,
        }
    }

    /// Creates a new span with file information.
    pub fn with_file(start: usize, len: usize, file_id: usize) -> Self {
        Self {
            start,
            len,
            file_id: Some(file_id),
            line: 1,
            column: 1,
        }
    }

    /// Returns the end position of the span.
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    /// Combines this span with another to create a span that covers both.
    pub fn combine(self, other: Span) -> Span {
        let start = self.start.min(other.start);
        let end = self.end().max(other.end());
        Span {
            start,
            len: end - start,
            file_id: self.file_id.or(other.file_id),
            line: self.line.min(other.line),
            column: if self.line == other.line { self.column.min(other.column) } else { self.column },
        }
    }

    /// Returns true if this span contains the given position.
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end()
    }

    /// Returns true if this span overlaps with another.
    pub fn overlaps(&self, other: Span) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    /// Creates a zero-width span at the given position.
    pub fn at(pos: usize) -> Self {
        Self::new(pos, 0)
    }

    /// Returns true if this is a zero-width span.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a new span with position information (line and column).
    /// This is primarily used for testing and error reporting.
    pub fn with_position(start: usize, len: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            len,
            file_id: None,
            line,
            column,
        }
    }
}

impl<T> Spanned<T> {
    /// Creates a new spanned value.
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    /// Returns a reference to the inner value.
    pub fn as_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner value.
    pub fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Unwraps the spanned value, returning the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Maps the inner value to a new type, preserving the span.
    pub fn map<U, F>(self, f: F) -> Spanned<U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned::new(f(self.inner), self.span)
    }

    /// Maps the inner value to a new result type, preserving the span.
    pub fn try_map<U, E, F>(self, f: F) -> Result<Spanned<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match f(self.inner) {
            Ok(value) => Ok(Spanned::new(value, self.span)),
            Err(err) => Err(err),
        }
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(span: Span) -> Self {
        miette::SourceSpan::new(span.start.into(), span.len)
    }
}

/// Helper function to create a spanned value.
pub fn spanned<T>(inner: T, span: Span) -> Spanned<T> {
    Spanned::new(inner, span)
}

/// Converts a range to a Span.
impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self::new(range.start, range.end - range.start)
    }
}

/// Convenience function to convert range to span
pub fn range_to_span(range: std::ops::Range<usize>) -> Span {
    range.into())
}