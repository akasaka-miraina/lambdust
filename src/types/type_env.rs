use super::{TypeScheme, TypeConstructor};
use super::type_classes::TypeClassInstance;
use std::collections::HashMap;

/// Type environment for type inference.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Variable bindings
    pub bindings: HashMap<String, TypeScheme>,
    /// Type class instances
    pub instances: HashMap<String, Vec<TypeClassInstance>>,
    /// Type constructors
    pub constructors: HashMap<String, TypeConstructor>,
}

impl TypeEnv {
    /// Creates a new empty type environment.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            instances: HashMap::new(),
            constructors: HashMap::new(),
        }
    }
    
    /// Looks up a variable in the environment.
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }
    
    /// Adds a binding to the environment.
    pub fn bind(&mut self, name: String, scheme: TypeScheme) {
        self.bindings.insert(name, scheme);
    }
    
    /// Extends the environment with new bindings.
    pub fn extend(&self, bindings: HashMap<String, TypeScheme>) -> Self {
        let mut new_env = self.clone());
        new_env.bindings.extend(bindings);
        new_env
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}