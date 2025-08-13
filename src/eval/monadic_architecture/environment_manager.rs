//! Environment manager trait (interface for environment operations)

use crate::eval::{Value, Environment};
use crate::diagnostics::Result;
use std::rc::Rc;
use std::collections::HashMap;

/// Environment manager trait (interface for environment operations)
pub trait EnvironmentManager: std::fmt::Debug {
    /// Create a new environment
    fn create_environment(&self, parent: Option<Rc<Environment>>) -> Rc<Environment>;
    
    /// Clone an environment
    fn clone_environment(&self, env: &Rc<Environment>) -> Rc<Environment>;
    
    /// Extend an environment with new bindings
    fn extend_environment(
        &self, 
        env: &Rc<Environment>, 
        bindings: HashMap<String, Value>
    ) -> Rc<Environment>;
    
    /// Lookup a value in an environment
    fn lookup(&self, env: &Rc<Environment>, name: &str) -> Option<Value>;
    
    /// Update a binding in an environment
    fn update(&self, env: &mut Rc<Environment>, name: String, value: Value) -> Result<()>;
}