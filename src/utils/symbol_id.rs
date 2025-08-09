//! Symbol identifier implementation.

/// Unique identifier for symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub usize);

impl SymbolId {
    /// Creates a new symbol ID.
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Gets the underlying ID.
    pub fn id(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // This is a placeholder - in practice you'd look up the string
        write!(f, "symbol-{}", self.0)
    }
}