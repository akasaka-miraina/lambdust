//! Environment and variable binding management

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Environment for variable bindings
#[derive(Debug, Clone)]
pub struct Environment {
    /// Current frame of bindings
    bindings: Rc<RefCell<HashMap<String, Value>>>,
    /// Parent environment (for lexical scoping)
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Create a new global environment
    pub fn new() -> Self {
        Environment {
            bindings: Rc::new(RefCell::new(HashMap::new())),
            parent: None,
        }
    }

    /// Create a new environment with a parent
    pub fn with_parent(parent: Rc<Environment>) -> Self {
        Environment {
            bindings: Rc::new(RefCell::new(HashMap::new())),
            parent: Some(parent),
        }
    }

    /// Create a new environment with initial bindings
    pub fn with_bindings(bindings: HashMap<String, Value>) -> Self {
        Environment {
            bindings: Rc::new(RefCell::new(bindings)),
            parent: None,
        }
    }

    /// Define a variable in the current environment
    pub fn define(&self, name: String, value: Value) {
        self.bindings.borrow_mut().insert(name, value);
    }

    /// Set a variable (must already exist in this environment or a parent)
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        // Try to set in current environment first
        if self.bindings.borrow().contains_key(name) {
            self.bindings.borrow_mut().insert(name.to_string(), value);
            return Ok(());
        }

        // Try parent environments
        if let Some(ref parent) = self.parent {
            parent.set(name, value)
        } else {
            Err(LambdustError::undefined_variable(name.to_string()))
        }
    }

    /// Get a variable from this environment or a parent
    pub fn get(&self, name: &str) -> Result<Value> {
        // Try current environment first
        if let Some(value) = self.bindings.borrow().get(name) {
            return Ok(value.clone());
        }

        // Try parent environments
        if let Some(ref parent) = self.parent {
            parent.get(name)
        } else {
            Err(LambdustError::undefined_variable(name.to_string()))
        }
    }

    /// Check if a variable exists in this environment or a parent
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.borrow().contains_key(name)
            || self.parent.as_ref().is_some_and(|p| p.contains(name))
    }

    /// Create a new child environment
    pub fn extend(&self) -> Environment {
        Environment::with_parent(Rc::new(self.clone()))
    }

    /// Create a new environment with parameter bindings
    pub fn bind_parameters(
        &self,
        params: &[String],
        args: &[Value],
        variadic: bool,
    ) -> Result<Environment> {
        let mut bindings = HashMap::new();

        if variadic {
            // Last parameter collects remaining arguments
            if params.is_empty() {
                return Err(LambdustError::arity_error(0, args.len()));
            }

            let required_params = params.len() - 1;
            if args.len() < required_params {
                return Err(LambdustError::arity_error(required_params, args.len()));
            }

            // Bind required parameters
            for (i, param) in params[..required_params].iter().enumerate() {
                bindings.insert(param.clone(), args[i].clone());
            }

            // Bind variadic parameter to remaining arguments as a list
            let rest_args = args[required_params..].to_vec();
            let rest_list = Value::from_vector(rest_args);
            bindings.insert(params[required_params].clone(), rest_list);
        } else {
            // Fixed arity
            if args.len() != params.len() {
                return Err(LambdustError::arity_error(params.len(), args.len()));
            }

            for (param, arg) in params.iter().zip(args.iter()) {
                bindings.insert(param.clone(), arg.clone());
            }
        }

        Ok(Environment {
            bindings: Rc::new(RefCell::new(bindings)),
            parent: Some(Rc::new(self.clone())),
        })
    }

    /// Get all bindings in the current frame (for debugging)
    pub fn current_bindings(&self) -> HashMap<String, Value> {
        self.bindings.borrow().clone()
    }

    /// Get the global environment (root of the chain)
    pub fn global(&self) -> Rc<Environment> {
        match &self.parent {
            Some(parent) => parent.global(),
            None => Rc::new(self.clone()),
        }
    }

    /// Create a new environment with built-in procedures
    pub fn with_builtins() -> Self {
        let env = Environment::new();

        // Load all built-in procedures from the builtins module
        let builtins = crate::builtins::create_builtins();
        for (name, value) in builtins {
            env.define(name, value);
        }

        env
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        // Compare by reference since environments are unique
        Rc::ptr_eq(&self.bindings, &other.bindings)
    }
}
