use super::Kind;

/// Type constructor information.
#[derive(Debug, Clone)]
pub struct TypeConstructor {
    pub name: String,
    pub kind: Kind,
    pub arity: usize,
}