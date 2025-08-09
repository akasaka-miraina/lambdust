//! SRFI-39 Parameter objects implementation.
//!
//! This module provides parameter objects as defined in SRFI-39, which enables
//! dynamic binding in Scheme. Parameters are first-class objects that can be
//! called as procedures to retrieve their current value, and used with the
//! `parameterize` special form to establish dynamic bindings.

use crate::eval::{Value, Parameter};
use crate::diagnostics::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::cell::RefCell;

thread_local! {
    /// Thread-local parameter binding stack.
    ///
    /// Each thread maintains its own stack of parameter bindings that are
    /// established by `parameterize` forms. When a parameter is accessed,
    /// the thread-local stack is consulted first, falling back to the
    /// global default value if no binding exists.
    static PARAMETER_STACK: RefCell<Vec<ParameterFrame>> = const { RefCell::new(Vec::new()) };
}

/// A frame in the parameter binding stack.
///
/// Each frame represents a `parameterize` form and contains the parameter
/// bindings established by that form.
#[derive(Debug, Clone)]
pub struct ParameterFrame {
    /// Parameter bindings: parameter ID -> value
    pub bindings: HashMap<u64, Value>,
}

/// Parameter ID generator for unique parameter identification.
static PARAMETER_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Parameter {
    /// Creates a new parameter with the given initial value and optional converter.
    pub fn new(initial_value: Value, converter: Option<Value>) -> Self {
        let id = PARAMETER_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // Apply converter to initial value if provided
        let processed_value = if let Some(ref _conv) = converter {
            // For now, we store the converter but don't apply it during construction
            // The converter will be applied when the parameter is set
            initial_value
        } else {
            initial_value
        };
        
        Parameter {
            id,
            converter: converter.map(Arc::new),
            global_default: Arc::new(RwLock::new(processed_value)),
            name: None,
        }
    }

    /// Creates a new parameter with a name for debugging.
    pub fn with_name(initial_value: Value, converter: Option<Value>, name: String) -> Self {
        let mut param = Self::new(initial_value, converter);
        param.name = Some(name);
        param
    }

    /// Gets the current value of this parameter.
    ///
    /// First checks the thread-local parameter stack for any bindings,
    /// then falls back to the global default value.
    pub fn get(&self) -> Value {
        // Check thread-local bindings first
        let thread_local_value = PARAMETER_STACK.with(|stack| {
            let stack = stack.borrow();
            // Search from top of stack (most recent binding) to bottom
            for frame in stack.iter().rev() {
                if let Some(value) = frame.bindings.get(&self.id) {
                    return Some(value.clone());
                }
            }
            None
        });

        // Return thread-local value if found, otherwise global default
        thread_local_value.unwrap_or_else(|| {
            self.global_default.read().unwrap().clone()
        })
    }

    /// Sets the global default value of this parameter.
    ///
    /// This does not affect thread-local bindings, only the fallback value
    /// used when no thread-local binding exists.
    pub fn set_global(&self, value: Value) -> Result<()> {
        let processed_value = self.apply_converter(value)?;
        *self.global_default.write().unwrap() = processed_value;
        Ok(())
    }

    /// Applies the converter function to a value if one is defined.
    pub fn apply_converter(&self, value: Value) -> Result<Value> {
        if let Some(_converter) = &self.converter {
            // For now, we don't have a way to call the converter function
            // This would require access to the evaluator
            // TODO: Implement converter function calling
            Ok(value)
        } else {
            Ok(value)
        }
    }

    /// Gets the parameter ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the parameter name if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Checks if this parameter has a converter function.
    pub fn has_converter(&self) -> bool {
        self.converter.is_some()
    }
}

/// Parameter binding operations for thread-local stacks.
pub struct ParameterBinding;

impl ParameterBinding {
    /// Establishes parameter bindings for the duration of a closure.
    ///
    /// This is the core implementation of the `parameterize` special form.
    /// It pushes a new parameter frame onto the thread-local stack,
    /// executes the given closure, and then pops the frame.
    pub fn with_bindings<F, R>(bindings: HashMap<u64, Value>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Push new parameter frame
        PARAMETER_STACK.with(|stack| {
            stack.borrow_mut().push(ParameterFrame { bindings });
        });

        // Execute the closure
        let result = f();

        // Pop the parameter frame
        PARAMETER_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });

        result
    }

    /// Gets the current depth of the parameter stack.
    pub fn stack_depth() -> usize {
        PARAMETER_STACK.with(|stack| stack.borrow().len())
    }

    /// Clears all parameter bindings (used for testing).
    #[cfg(test)]
    pub fn clear_stack() {
        PARAMETER_STACK.with(|stack| {
            stack.borrow_mut().clear();
        });
    }
}

impl ParameterFrame {
    /// Creates a new parameter frame with the given bindings.
    pub fn new(bindings: HashMap<u64, Value>) -> Self {
        ParameterFrame { bindings }
    }

    /// Creates an empty parameter frame.
    pub fn empty() -> Self {
        ParameterFrame {
            bindings: HashMap::new(),
        }
    }

    /// Adds a binding to this frame.
    pub fn bind(&mut self, parameter_id: u64, value: Value) {
        self.bindings.insert(parameter_id, value);
    }

    /// Gets a binding from this frame.
    pub fn get(&self, parameter_id: u64) -> Option<&Value> {
        self.bindings.get(&parameter_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Value;

    #[test]
    fn test_parameter_creation() {
        let param = Parameter::new(Value::integer(42), None);
        assert_eq!(param.get().as_integer(), Some(42));
        assert!(!param.has_converter());
        assert!(param.name().is_none());
    }

    #[test]
    fn test_parameter_with_name() {
        let param = Parameter::with_name(
            Value::string("hello"),
            None,
            "test-param".to_string()
        );
        assert_eq!(param.get().as_string(), Some("hello"));
        assert_eq!(param.name(), Some("test-param"));
    }

    #[test]
    fn test_parameter_global_set() {
        let param = Parameter::new(Value::integer(1), None);
        assert_eq!(param.get().as_integer(), Some(1));
        
        param.set_global(Value::integer(2)).unwrap();
        assert_eq!(param.get().as_integer(), Some(2));
    }

    #[test]
    fn test_parameter_binding() {
        ParameterBinding::clear_stack();
        
        let param = Parameter::new(Value::integer(1), None);
        assert_eq!(param.get().as_integer(), Some(1));
        
        let mut bindings = HashMap::new();
        bindings.insert(param.id(), Value::integer(42));
        
        let result = ParameterBinding::with_bindings(bindings, || {
            assert_eq!(param.get().as_integer(), Some(42));
            param.get().as_integer().unwrap() * 2
        });
        
        // After the binding is removed, should return to global default
        assert_eq!(param.get().as_integer(), Some(1));
        assert_eq!(result, 84);
    }

    #[test]
    fn test_nested_parameter_bindings() {
        ParameterBinding::clear_stack();
        
        let param = Parameter::new(Value::integer(1), None);
        
        let mut bindings1 = HashMap::new();
        bindings1.insert(param.id(), Value::integer(10));
        
        let mut bindings2 = HashMap::new();
        bindings2.insert(param.id(), Value::integer(20));
        
        ParameterBinding::with_bindings(bindings1, || {
            assert_eq!(param.get().as_integer(), Some(10));
            
            ParameterBinding::with_bindings(bindings2, || {
                assert_eq!(param.get().as_integer(), Some(20));
            });
            
            // Should return to outer binding
            assert_eq!(param.get().as_integer(), Some(10));
        });
        
        // Should return to global default
        assert_eq!(param.get().as_integer(), Some(1));
    }

    #[test]
    fn test_stack_depth() {
        ParameterBinding::clear_stack();
        assert_eq!(ParameterBinding::stack_depth(), 0);
        
        let bindings = HashMap::new();
        ParameterBinding::with_bindings(bindings.clone(), || {
            assert_eq!(ParameterBinding::stack_depth(), 1);
            
            ParameterBinding::with_bindings(bindings, || {
                assert_eq!(ParameterBinding::stack_depth(), 2);
            });
            
            assert_eq!(ParameterBinding::stack_depth(), 1);
        });
        
        assert_eq!(ParameterBinding::stack_depth(), 0);
    }
}