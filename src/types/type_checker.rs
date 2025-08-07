use super::{TypeLevel, TypeEnv, TypeConstraint};
use crate::diagnostics::Error;
use super::substitution::Substitution;

/// Type checker state.
#[derive(Debug)]
pub struct TypeChecker {
    /// Current type checking level
    level: TypeLevel,
    /// Type environment
    env: TypeEnv,
    /// Current substitution
    #[allow(dead_code)]
    substitution: Substitution,
    /// Type constraints to solve
    #[allow(dead_code)]
    constraints: Vec<TypeConstraint>,
    /// Error accumulator
    errors: Vec<Error>,
}

impl TypeChecker {
    /// Creates a new type checker.
    pub fn new(level: TypeLevel) -> Self {
        Self {
            level,
            env: TypeEnv::new(),
            substitution: Substitution::empty(),
            constraints: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    /// Gets the current type level.
    pub fn level(&self) -> TypeLevel {
        self.level
    }
    
    /// Gets the current type environment.
    pub fn env(&self) -> &TypeEnv {
        &self.env
    }
    
    /// Gets a mutable reference to the type environment.
    pub fn env_mut(&mut self) -> &mut TypeEnv {
        &mut self.env
    }
    
    /// Adds a type error.
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }
    
    /// Gets all accumulated errors.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }
    
    /// Clears all errors.
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new(TypeLevel::Dynamic)
    }
}