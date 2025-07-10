//! Built-in functions for pure semantic evaluator
//!
//! This module implements R7RS standard library functions within the
//! pure semantic evaluator framework, maintaining strict mathematical semantics.

use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::Value;

use super::semantic_core::SemanticEvaluator;

impl SemanticEvaluator {
    /// Apply built-in function in pure semantic evaluation
    pub(super) fn apply_builtin_pure(&self, name: &str, args: &[Value]) -> Result<Value> {
        match name {
            "+" => self.apply_builtin_add(args),
            "-" => self.apply_builtin_subtract(args),
            "*" => self.apply_builtin_multiply(args),
            "=" => self.apply_builtin_numeric_equal(args),
            "<" => self.apply_builtin_less_than(args),
            "car" => self.apply_builtin_car(args),
            "cdr" => self.apply_builtin_cdr(args),
            "cons" => self.apply_builtin_cons(args),
            "null?" => self.apply_builtin_null_p(args),
            "pair?" => self.apply_builtin_pair_p(args),
            _ => Err(LambdustError::runtime_error(format!(
                "Builtin function '{}' not implemented in pure evaluator",
                name
            ))),
        }
    }
    
    /// Arithmetic Operations
    
    /// Addition operator (+)
    fn apply_builtin_add(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(SchemeNumber::Integer(0)));
        }
        
        let mut sum = 0i64;
        for arg in args {
            match arg {
                Value::Number(n) => match n {
                    SchemeNumber::Integer(i) => sum += i,
                    _ => return Err(LambdustError::type_error("Addition expects integers")),
                },
                _ => return Err(LambdustError::type_error("Addition expects numbers")),
            }
        }
        
        Ok(Value::Number(SchemeNumber::Integer(sum)))
    }
    
    /// Subtraction operator (-)
    fn apply_builtin_subtract(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(LambdustError::arity_error(1, 0));
        }
        
        let first = match &args[0] {
            Value::Number(SchemeNumber::Integer(i)) => *i,
            _ => return Err(LambdustError::type_error("Subtraction expects integers")),
        };
        
        if args.len() == 1 {
            return Ok(Value::Number(SchemeNumber::Integer(-first)));
        }
        
        let mut result = first;
        for arg in &args[1..] {
            match arg {
                Value::Number(SchemeNumber::Integer(i)) => result -= i,
                _ => return Err(LambdustError::type_error("Subtraction expects integers")),
            }
        }
        
        Ok(Value::Number(SchemeNumber::Integer(result)))
    }
    
    /// Multiplication operator (*)
    fn apply_builtin_multiply(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(SchemeNumber::Integer(1)));
        }
        
        let mut product = 1i64;
        for arg in args {
            match arg {
                Value::Number(SchemeNumber::Integer(i)) => product *= i,
                _ => return Err(LambdustError::type_error("Multiplication expects integers")),
            }
        }
        
        Ok(Value::Number(SchemeNumber::Integer(product)))
    }
    
    /// Comparison Operations
    
    /// Numeric equality (=)
    fn apply_builtin_numeric_equal(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error_min(2, args.len()));
        }
        
        let first = match &args[0] {
            Value::Number(SchemeNumber::Integer(i)) => *i,
            _ => return Err(LambdustError::type_error("= expects numbers")),
        };
        
        for arg in &args[1..] {
            match arg {
                Value::Number(SchemeNumber::Integer(i)) => {
                    if *i != first {
                        return Ok(Value::Boolean(false));
                    }
                }
                _ => return Err(LambdustError::type_error("= expects numbers")),
            }
        }
        
        Ok(Value::Boolean(true))
    }
    
    /// Less than comparison (<)
    fn apply_builtin_less_than(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error_min(2, args.len()));
        }
        
        let mut prev = match &args[0] {
            Value::Number(SchemeNumber::Integer(i)) => *i,
            _ => return Err(LambdustError::type_error("< expects numbers")),
        };
        
        for arg in &args[1..] {
            match arg {
                Value::Number(SchemeNumber::Integer(i)) => {
                    if prev >= *i {
                        return Ok(Value::Boolean(false));
                    }
                    prev = *i;
                }
                _ => return Err(LambdustError::type_error("< expects numbers")),
            }
        }
        
        Ok(Value::Boolean(true))
    }
    
    /// List Operations
    
    /// Car operation
    fn apply_builtin_car(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        
        match &args[0] {
            Value::Pair(pair_ref) => {
                let pair_data = pair_ref.borrow();
                Ok(pair_data.car.clone())
            }
            _ => Err(LambdustError::type_error("car expects a pair")),
        }
    }
    
    /// Cdr operation
    fn apply_builtin_cdr(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        
        match &args[0] {
            Value::Pair(pair_ref) => {
                let pair_data = pair_ref.borrow();
                Ok(pair_data.cdr.clone())
            }
            _ => Err(LambdustError::type_error("cdr expects a pair")),
        }
    }
    
    /// Cons operation
    fn apply_builtin_cons(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }
        
        Ok(Value::cons(args[0].clone(), args[1].clone()))
    }
    
    /// Type Predicates
    
    /// Null predicate
    fn apply_builtin_null_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Nil)))
    }
    
    /// Pair predicate
    fn apply_builtin_pair_p(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        
        Ok(Value::Boolean(matches!(args[0], Value::Pair(_))))
    }
}