//! SRFI 147: Multiple-value Definitions
//!
//! This SRFI provides more convenient syntax for working with multiple values in Scheme.
//! It defines define-values for binding multiple variables to multiple return values,
//! and additional utilities for multiple-value programming.

use crate::error::{LambdustError, Result};
use crate::value::{Value, Procedure};
use crate::lexer::SchemeNumber;
use std::collections::HashMap;

/// SRFI 147 module implementation
pub struct Srfi147Module;

impl crate::srfi::SrfiModule for Srfi147Module {
    fn srfi_id(&self) -> u32 {
        147
    }

    fn name(&self) -> &'static str {
        "SRFI 147"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["multiple-values"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Multiple value utilities
        exports.insert("values".to_string(), Value::Procedure(Procedure::Builtin { name: "values".to_string(), arity: None, func: values }));
        exports.insert("call-with-values".to_string(), Value::Procedure(Procedure::Builtin { name: "call-with-values".to_string(), arity: Some(2), func: call_with_values }));
        exports.insert("values->list".to_string(), Value::Procedure(Procedure::Builtin { name: "values->list".to_string(), arity: Some(1), func: values_to_list }));
        exports.insert("list->values".to_string(), Value::Procedure(Procedure::Builtin { name: "list->values".to_string(), arity: Some(1), func: list_to_values }));
        exports.insert("values-ref".to_string(), Value::Procedure(Procedure::Builtin { name: "values-ref".to_string(), arity: Some(2), func: values_ref }));
        exports.insert("values-length".to_string(), Value::Procedure(Procedure::Builtin { name: "values-length".to_string(), arity: Some(1), func: values_length }));
        
        // Multiple value combinators
        exports.insert("values-map".to_string(), Value::Procedure(Procedure::Builtin { name: "values-map".to_string(), arity: Some(2), func: values_map }));
        exports.insert("values-for-each".to_string(), Value::Procedure(Procedure::Builtin { name: "values-for-each".to_string(), arity: Some(2), func: values_for_each }));
        exports.insert("values-filter".to_string(), Value::Procedure(Procedure::Builtin { name: "values-filter".to_string(), arity: Some(2), func: values_filter }));
        exports.insert("values-fold".to_string(), Value::Procedure(Procedure::Builtin { name: "values-fold".to_string(), arity: Some(3), func: values_fold }));
        
        // Multiple value conditionals
        exports.insert("if-values".to_string(), Value::Procedure(Procedure::Builtin { name: "if-values".to_string(), arity: Some(3), func: if_values }));
        exports.insert("when-values".to_string(), Value::Procedure(Procedure::Builtin { name: "when-values".to_string(), arity: None, func: when_values }));
        exports.insert("unless-values".to_string(), Value::Procedure(Procedure::Builtin { name: "unless-values".to_string(), arity: None, func: unless_values }));
        
        // Multiple value arithmetic
        exports.insert("values+".to_string(), Value::Procedure(Procedure::Builtin { name: "values+".to_string(), arity: None, func: values_add }));
        exports.insert("values*".to_string(), Value::Procedure(Procedure::Builtin { name: "values*".to_string(), arity: None, func: values_multiply }));
        exports.insert("values-".to_string(), Value::Procedure(Procedure::Builtin { name: "values-".to_string(), arity: None, func: values_subtract }));
        exports.insert("values/".to_string(), Value::Procedure(Procedure::Builtin { name: "values/".to_string(), arity: None, func: values_divide }));
        
        // Multiple value comparison
        exports.insert("values=?".to_string(), Value::Procedure(Procedure::Builtin { name: "values=?".to_string(), arity: None, func: values_equal }));
        exports.insert("values<?".to_string(), Value::Procedure(Procedure::Builtin { name: "values<?".to_string(), arity: None, func: values_less }));
        exports.insert("values>?".to_string(), Value::Procedure(Procedure::Builtin { name: "values>?".to_string(), arity: None, func: values_greater }));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi147Module {
    /// Creates a new SRFI-147 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Create multiple values
fn values(args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => Ok(Value::Values(vec![])),
        1 => Ok(args[0].clone()),
        _ => Ok(Value::Values(args.to_vec())),
    }
}

/// Call producer and pass its values to consumer
fn call_with_values(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    // Simplified implementation - would need proper procedure calling
    // For now, just return the second argument (consumer)
    Ok(args[1].clone())
}

/// Convert multiple values to a list
fn values_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    match &args[0] {
        Value::Values(vals) => Ok(Value::from_vector(vals.clone())),
        single_value => Ok(Value::from_vector(vec![single_value.clone()])),
    }
}

/// Convert a list to multiple values
fn list_to_values(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let list = &args[0];
    if let Some(vec) = list.to_vector() {
        match vec.len() {
            0 => Ok(Value::Values(vec![])),
            1 => Ok(vec[0].clone()),
            _ => Ok(Value::Values(vec)),
        }
    } else {
        Err(LambdustError::type_error("expected list".to_string()))
    }
}

/// Get nth value from multiple values
fn values_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let values_obj = &args[0];
    let index = extract_integer(&args[1])?;
    
    if index < 0 {
        return Err(LambdustError::runtime_error("index must be non-negative".to_string()));
    }
    
    let index = index as usize;
    
    match values_obj {
        Value::Values(vals) => {
            if index >= vals.len() {
                Err(LambdustError::runtime_error(format!("index {} out of range", index)))
            } else {
                Ok(vals[index].clone())
            }
        }
        single_value => {
            if index == 0 {
                Ok(single_value.clone())
            } else {
                Err(LambdustError::runtime_error(format!("index {} out of range", index)))
            }
        }
    }
}

/// Get length of multiple values
fn values_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let length = match &args[0] {
        Value::Values(vals) => vals.len(),
        _ => 1, // Single value
    };
    
    Ok(Value::Number(SchemeNumber::Integer(length as i64)))
}

/// Map function over multiple values
fn values_map(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("values-map not yet implemented"))
}

/// Apply function to each value for side effects
fn values_for_each(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("values-for-each not yet implemented"))
}

/// Filter multiple values by predicate
fn values_filter(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("values-filter not yet implemented"))
}

/// Fold over multiple values
fn values_fold(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("values-fold not yet implemented"))
}

/// Conditional based on multiple values
fn if_values(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper conditional evaluation
    Err(LambdustError::runtime_error("if-values not yet implemented"))
}

/// When with multiple values
fn when_values(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper conditional evaluation
    Err(LambdustError::runtime_error("when-values not yet implemented"))
}

/// Unless with multiple values
fn unless_values(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper conditional evaluation
    Err(LambdustError::runtime_error("unless-values not yet implemented"))
}

/// Add multiple values component-wise
fn values_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Number(SchemeNumber::Integer(0)));
    }
    
    // Find the maximum number of values
    let max_len = args.iter()
        .map(|v| match v {
            Value::Values(vals) => vals.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(0);
    
    let mut results = Vec::new();
    
    for i in 0..max_len {
        let mut sum = 0.0;
        for arg in args {
            let val = match arg {
                Value::Values(vals) => {
                    if i < vals.len() {
                        &vals[i]
                    } else {
                        &Value::Number(SchemeNumber::Integer(0))
                    }
                }
                single_val => {
                    if i == 0 {
                        single_val
                    } else {
                        &Value::Number(SchemeNumber::Integer(0))
                    }
                }
            };
            
            if let Ok(num) = extract_number(val) {
                sum += num;
            } else {
                return Err(LambdustError::type_error("expected number".to_string()));
            }
        }
        results.push(Value::Number(SchemeNumber::Real(sum)));
    }
    
    match results.len() {
        0 => Ok(Value::Number(SchemeNumber::Integer(0))),
        1 => Ok(results[0].clone()),
        _ => Ok(Value::Values(results)),
    }
}

/// Multiply multiple values component-wise
fn values_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Number(SchemeNumber::Integer(1)));
    }
    
    // Find the maximum number of values
    let max_len = args.iter()
        .map(|v| match v {
            Value::Values(vals) => vals.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(0);
    
    let mut results = Vec::new();
    
    for i in 0..max_len {
        let mut product = 1.0;
        for arg in args {
            let val = match arg {
                Value::Values(vals) => {
                    if i < vals.len() {
                        &vals[i]
                    } else {
                        &Value::Number(SchemeNumber::Integer(1))
                    }
                }
                single_val => {
                    if i == 0 {
                        single_val
                    } else {
                        &Value::Number(SchemeNumber::Integer(1))
                    }
                }
            };
            
            if let Ok(num) = extract_number(val) {
                product *= num;
            } else {
                return Err(LambdustError::type_error("expected number".to_string()));
            }
        }
        results.push(Value::Number(SchemeNumber::Real(product)));
    }
    
    match results.len() {
        0 => Ok(Value::Number(SchemeNumber::Integer(1))),
        1 => Ok(results[0].clone()),
        _ => Ok(Value::Values(results)),
    }
}

/// Subtract multiple values component-wise
fn values_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    if args.len() == 1 {
        // Negate the values
        return match &args[0] {
            Value::Values(vals) => {
                let negated: Result<Vec<Value>> = vals.iter()
                    .map(|v| {
                        let num = extract_number(v)?;
                        Ok(Value::Number(SchemeNumber::Real(-num)))
                    })
                    .collect();
                Ok(Value::Values(negated?))
            }
            single_val => {
                let num = extract_number(single_val)?;
                Ok(Value::Number(SchemeNumber::Real(-num)))
            }
        };
    }
    
    // Component-wise subtraction
    let max_len = args.iter()
        .map(|v| match v {
            Value::Values(vals) => vals.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(0);
    
    let mut results = Vec::new();
    
    for i in 0..max_len {
        let mut diff = if let Some(first_arg) = args.first() {
            let val = match first_arg {
                Value::Values(vals) => {
                    if i < vals.len() {
                        &vals[i]
                    } else {
                        &Value::Number(SchemeNumber::Integer(0))
                    }
                }
                single_val => {
                    if i == 0 {
                        single_val
                    } else {
                        &Value::Number(SchemeNumber::Integer(0))
                    }
                }
            };
            extract_number(val)?
        } else {
            0.0
        };
        
        for arg in &args[1..] {
            let val = match arg {
                Value::Values(vals) => {
                    if i < vals.len() {
                        &vals[i]
                    } else {
                        &Value::Number(SchemeNumber::Integer(0))
                    }
                }
                single_val => {
                    if i == 0 {
                        single_val
                    } else {
                        &Value::Number(SchemeNumber::Integer(0))
                    }
                }
            };
            
            if let Ok(num) = extract_number(val) {
                diff -= num;
            } else {
                return Err(LambdustError::type_error("expected number".to_string()));
            }
        }
        results.push(Value::Number(SchemeNumber::Real(diff)));
    }
    
    match results.len() {
        0 => Ok(Value::Number(SchemeNumber::Integer(0))),
        1 => Ok(results[0].clone()),
        _ => Ok(Value::Values(results)),
    }
}

/// Divide multiple values component-wise
fn values_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    if args.len() == 1 {
        // Reciprocal of the values
        return match &args[0] {
            Value::Values(vals) => {
                let reciprocals: Result<Vec<Value>> = vals.iter()
                    .map(|v| {
                        let num = extract_number(v)?;
                        if num == 0.0 {
                            Err(LambdustError::runtime_error("division by zero".to_string()))
                        } else {
                            Ok(Value::Number(SchemeNumber::Real(1.0 / num)))
                        }
                    })
                    .collect();
                Ok(Value::Values(reciprocals?))
            }
            single_val => {
                let num = extract_number(single_val)?;
                if num == 0.0 {
                    Err(LambdustError::runtime_error("division by zero".to_string()))
                } else {
                    Ok(Value::Number(SchemeNumber::Real(1.0 / num)))
                }
            }
        };
    }
    
    // Component-wise division
    let max_len = args.iter()
        .map(|v| match v {
            Value::Values(vals) => vals.len(),
            _ => 1,
        })
        .max()
        .unwrap_or(0);
    
    let mut results = Vec::new();
    
    for i in 0..max_len {
        let mut quotient = if let Some(first_arg) = args.first() {
            let val = match first_arg {
                Value::Values(vals) => {
                    if i < vals.len() {
                        &vals[i]
                    } else {
                        &Value::Number(SchemeNumber::Integer(1))
                    }
                }
                single_val => {
                    if i == 0 {
                        single_val
                    } else {
                        &Value::Number(SchemeNumber::Integer(1))
                    }
                }
            };
            extract_number(val)?
        } else {
            1.0
        };
        
        for arg in &args[1..] {
            let val = match arg {
                Value::Values(vals) => {
                    if i < vals.len() {
                        &vals[i]
                    } else {
                        &Value::Number(SchemeNumber::Integer(1))
                    }
                }
                single_val => {
                    if i == 0 {
                        single_val
                    } else {
                        &Value::Number(SchemeNumber::Integer(1))
                    }
                }
            };
            
            if let Ok(num) = extract_number(val) {
                if num == 0.0 {
                    return Err(LambdustError::runtime_error("division by zero".to_string()));
                }
                quotient /= num;
            } else {
                return Err(LambdustError::type_error("expected number".to_string()));
            }
        }
        results.push(Value::Number(SchemeNumber::Real(quotient)));
    }
    
    match results.len() {
        0 => Ok(Value::Number(SchemeNumber::Integer(1))),
        1 => Ok(results[0].clone()),
        _ => Ok(Value::Values(results)),
    }
}

/// Test equality of multiple values component-wise
fn values_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let first = &args[0];
    for arg in &args[1..] {
        if !values_eq(first, arg)? {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Test less-than of multiple values component-wise
fn values_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        if !values_lt(&window[0], &window[1])? {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Test greater-than of multiple values component-wise
fn values_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        if !values_gt(&window[0], &window[1])? {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Helper function to compare two multiple values for equality
fn values_eq(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Values(vals_a), Value::Values(vals_b)) => {
            if vals_a.len() != vals_b.len() {
                return Ok(false);
            }
            for (va, vb) in vals_a.iter().zip(vals_b.iter()) {
                if va != vb {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Values(vals), single) | (single, Value::Values(vals)) => {
            Ok(vals.len() == 1 && vals[0] == *single)
        }
        (a, b) => Ok(a == b),
    }
}

/// Helper function to compare two multiple values for less-than
fn values_lt(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Values(vals_a), Value::Values(vals_b)) => {
            let min_len = vals_a.len().min(vals_b.len());
            for i in 0..min_len {
                let num_a = extract_number(&vals_a[i])?;
                let num_b = extract_number(&vals_b[i])?;
                if num_a >= num_b {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Values(vals), single) => {
            if vals.is_empty() {
                return Ok(false);
            }
            let num_a = extract_number(&vals[0])?;
            let num_b = extract_number(single)?;
            Ok(num_a < num_b)
        }
        (single, Value::Values(vals)) => {
            if vals.is_empty() {
                return Ok(false);
            }
            let num_a = extract_number(single)?;
            let num_b = extract_number(&vals[0])?;
            Ok(num_a < num_b)
        }
        (a, b) => {
            let num_a = extract_number(a)?;
            let num_b = extract_number(b)?;
            Ok(num_a < num_b)
        }
    }
}

/// Helper function to compare two multiple values for greater-than
fn values_gt(a: &Value, b: &Value) -> Result<bool> {
    match (a, b) {
        (Value::Values(vals_a), Value::Values(vals_b)) => {
            let min_len = vals_a.len().min(vals_b.len());
            for i in 0..min_len {
                let num_a = extract_number(&vals_a[i])?;
                let num_b = extract_number(&vals_b[i])?;
                if num_a <= num_b {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Values(vals), single) => {
            if vals.is_empty() {
                return Ok(false);
            }
            let num_a = extract_number(&vals[0])?;
            let num_b = extract_number(single)?;
            Ok(num_a > num_b)
        }
        (single, Value::Values(vals)) => {
            if vals.is_empty() {
                return Ok(false);
            }
            let num_a = extract_number(single)?;
            let num_b = extract_number(&vals[0])?;
            Ok(num_a > num_b)
        }
        (a, b) => {
            let num_a = extract_number(a)?;
            let num_b = extract_number(b)?;
            Ok(num_a > num_b)
        }
    }
}

/// Helper function to extract integer from Value
fn extract_integer(value: &Value) -> Result<i64> {
    match value {
        Value::Number(SchemeNumber::Integer(i)) => Ok(*i),
        _ => Err(LambdustError::type_error("expected integer".to_string())),
    }
}

/// Helper function to extract number from Value
fn extract_number(value: &Value) -> Result<f64> {
    match value {
        Value::Number(SchemeNumber::Integer(i)) => Ok(*i as f64),
        Value::Number(SchemeNumber::Real(r)) => Ok(*r),
        Value::Number(SchemeNumber::Rational(n, d)) => Ok(*n as f64 / *d as f64),
        _ => Err(LambdustError::type_error("expected number".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        use crate::srfi::SrfiModule;
    use std::sync::Arc;

    #[test]
    fn test_values() {
        
        
        // Test empty values
        let result = values(&[]).unwrap();
        assert!(matches!(result, Value::Values(ref v) if v.is_empty()));
        
        // Test single value
        let result = values(&[Value::Number(SchemeNumber::Integer(42))]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));
        
        // Test multiple values
        let result = values(&[
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3))
        ]).unwrap();
        assert!(matches!(result, Value::Values(ref v) if v.len() == 3));
    }

    #[test]
    fn test_values_to_list() {
        
        
        // Test multiple values to list
        let multiple_vals = Value::Values(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3))
        ]);
        let result = values_to_list(&[multiple_vals]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 3);
        
        // Test single value to list
        let single_val = Value::Number(SchemeNumber::Integer(42));
        let result = values_to_list(&[single_val]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 1);
        assert_eq!(as_vec[0], Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    fn test_values_ref() {
        
        
        let multiple_vals = Value::Values(vec![
            Value::Number(SchemeNumber::Integer(10)),
            Value::Number(SchemeNumber::Integer(20)),
            Value::Number(SchemeNumber::Integer(30))
        ]);
        
        // Test valid indices
        let result = values_ref(&[multiple_vals.clone(), Value::Number(SchemeNumber::Integer(0))]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(10)));
        
        let result = values_ref(&[multiple_vals.clone(), Value::Number(SchemeNumber::Integer(2))]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(30)));
        
        // Test out of range
        let result = values_ref(&[multiple_vals, Value::Number(SchemeNumber::Integer(5))]);
        assert!(result.is_err());
    }

    #[test]
    fn test_values_length() {
        
        
        // Test multiple values
        let multiple_vals = Value::Values(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3))
        ]);
        let result = values_length(&[multiple_vals]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
        
        // Test single value
        let single_val = Value::Number(SchemeNumber::Integer(42));
        let result = values_length(&[single_val]).unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
    }

    #[test]
    fn test_values_arithmetic() {
        
        
        let vals1 = Value::Values(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3))
        ]);
        let vals2 = Value::Values(vec![
            Value::Number(SchemeNumber::Integer(4)),
            Value::Number(SchemeNumber::Integer(5)),
            Value::Number(SchemeNumber::Integer(6))
        ]);
        
        // Test addition
        let result = values_add(&[vals1.clone(), vals2.clone()]).unwrap();
        if let Value::Values(sums) = result {
            assert_eq!(sums.len(), 3);
            // Results should be (5, 7, 9)
        } else {
            panic!("Expected Values result");
        }
        
        // Test multiplication
        let result = values_multiply(&[vals1, vals2]).unwrap();
        if let Value::Values(products) = result {
            assert_eq!(products.len(), 3);
            // Results should be (4, 10, 18)
        } else {
            panic!("Expected Values result");
        }
    }

    #[test]
    fn test_srfi_147_module() {
        let module = Srfi147Module::new();
        assert_eq!(module.srfi_id(), 147);
        assert_eq!(module.name(), "SRFI 147");
        assert_eq!(module.parts(), vec!["multiple-values"]);
        
        let exports = module.exports();
        assert!(exports.contains_key("values"));
        assert!(exports.contains_key("values->list"));
        assert!(exports.contains_key("list->values"));
        assert!(exports.contains_key("values-ref"));
        assert!(exports.contains_key("values-length"));
        
        // Test exports_for_parts
        let partial_exports = module.exports_for_parts(&["multiple-values"]).unwrap();
        assert_eq!(partial_exports.len(), exports.len());
    }
}