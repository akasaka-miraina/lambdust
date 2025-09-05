use super::Kind;

/// Type constructor information.
#[derive(Debug, Clone)]
pub struct TypeConstructor {
    /// The name of the type constructor
    pub name: String,
    /// The kind of the type constructor
    pub kind: Kind,
    /// The arity (number of type parameters) of the constructor
    pub arity: usize,
}
