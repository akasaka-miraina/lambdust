//! SRFI 148: Eager Comprehensions
//!
//! This SRFI provides syntax for list, vector, string, and bytevector comprehensions.
//! Comprehensions are a concise way to construct sequences by iterating over existing sequences
//! and applying transformations and filters.

use crate::error::{LambdustError, Result};
use crate::value::Value;
use crate::lexer::SchemeNumber;
use std::collections::HashMap;

/// SRFI 148 module implementation
pub struct Srfi148Module;

impl crate::srfi::SrfiModule for Srfi148Module {
    fn srfi_id(&self) -> u32 {
        148
    }

    fn name(&self) -> &'static str {
        "SRFI 148"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["comprehensions"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Helper function to create builtin procedures
        let make_builtin = |name: &str, arity: Option<usize>, func: fn(&[Value]) -> crate::Result<Value>| {
            Value::Procedure(crate::value::Procedure::Builtin {
                name: name.to_string(),
                arity,
                func,
            })
        };
        
        // List comprehensions
        exports.insert("list-ec".to_string(), make_builtin("list-ec", None, list_ec));
        exports.insert("append-ec".to_string(), make_builtin("append-ec", None, append_ec));
        exports.insert("string-ec".to_string(), make_builtin("string-ec", None, string_ec));
        exports.insert("string-append-ec".to_string(), make_builtin("string-append-ec", None, string_append_ec));
        exports.insert("vector-ec".to_string(), make_builtin("vector-ec", None, vector_ec));
        exports.insert("vector-of-length-ec".to_string(), make_builtin("vector-of-length-ec", None, vector_of_length_ec));
        
        // Bytevector comprehensions
        exports.insert("bytevector-ec".to_string(), make_builtin("bytevector-ec", None, bytevector_ec));
        
        // Aggregate comprehensions
        exports.insert("sum-ec".to_string(), make_builtin("sum-ec", None, sum_ec));
        exports.insert("product-ec".to_string(), make_builtin("product-ec", None, product_ec));
        exports.insert("min-ec".to_string(), make_builtin("min-ec", None, min_ec));
        exports.insert("max-ec".to_string(), make_builtin("max-ec", None, max_ec));
        exports.insert("any?-ec".to_string(), make_builtin("any?-ec", None, any_ec));
        exports.insert("every?-ec".to_string(), make_builtin("every?-ec", None, every_ec));
        exports.insert("first-ec".to_string(), make_builtin("first-ec", None, first_ec));
        exports.insert("last-ec".to_string(), make_builtin("last-ec", None, last_ec));
        exports.insert("fold-ec".to_string(), make_builtin("fold-ec", None, fold_ec));
        exports.insert("fold3-ec".to_string(), make_builtin("fold3-ec", None, fold3_ec));
        
        // Generator procedures
        exports.insert(":range".to_string(), make_builtin(":range", None, range_generator));
        exports.insert(":real-range".to_string(), make_builtin(":real-range", None, real_range_generator));
        exports.insert(":char-range".to_string(), make_builtin(":char-range", None, char_range_generator));
        exports.insert(":list".to_string(), make_builtin(":list", None, list_generator));
        exports.insert(":string".to_string(), make_builtin(":string", None, string_generator));
        exports.insert(":vector".to_string(), make_builtin(":vector", None, vector_generator));
        exports.insert(":integers".to_string(), make_builtin(":integers", None, integers_generator));
        exports.insert(":port".to_string(), make_builtin(":port", None, port_generator));
        
        // Conditional generators
        exports.insert(":while".to_string(), make_builtin(":while", None, while_generator));
        exports.insert(":until".to_string(), make_builtin(":until", None, until_generator));
        
        // Multi-value generators
        exports.insert(":parallel".to_string(), make_builtin(":parallel", None, parallel_generator));
        exports.insert(":do".to_string(), make_builtin(":do", None, do_generator));
        exports.insert(":let".to_string(), make_builtin(":let", None, let_generator));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi148Module {
    /// Creates a new SRFI-148 module instance
    pub fn new() -> Self {
        Self
    }
}

/// List comprehension
fn list_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need full comprehension syntax parsing
    // For now, create a simple list
    Ok(Value::from_vector(vec![]))
}

/// Append comprehension
fn append_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation
    Ok(Value::from_vector(vec![]))
}

/// String comprehension
fn string_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation
    Ok(Value::String(String::new()))
}

/// String append comprehension
fn string_append_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation
    Ok(Value::String(String::new()))
}

/// Vector comprehension
fn vector_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation
    Ok(Value::Vector(vec![]))
}

/// Vector of specific length comprehension
fn vector_of_length_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation
    Ok(Value::Vector(vec![]))
}

/// Bytevector comprehension
fn bytevector_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need bytevector support
    Ok(Value::Vector(vec![]))
}

/// Sum comprehension
fn sum_ec(args: &[Value]) -> Result<Value> {
    // Simplified implementation - sum all numeric arguments
    let mut sum = 0.0;
    for arg in args {
        match arg {
            Value::Number(SchemeNumber::Integer(i)) => sum += *i as f64,
            Value::Number(SchemeNumber::Real(r)) => sum += r,
            Value::Number(SchemeNumber::Rational(n, d)) => sum += *n as f64 / *d as f64,
            _ => {} // Skip non-numeric values
        }
    }
    Ok(Value::Number(SchemeNumber::Real(sum)))
}

/// Product comprehension
fn product_ec(args: &[Value]) -> Result<Value> {
    // Simplified implementation - multiply all numeric arguments
    let mut product = 1.0;
    for arg in args {
        match arg {
            Value::Number(SchemeNumber::Integer(i)) => product *= *i as f64,
            Value::Number(SchemeNumber::Real(r)) => product *= r,
            Value::Number(SchemeNumber::Rational(n, d)) => product *= *n as f64 / *d as f64,
            _ => {} // Skip non-numeric values
        }
    }
    Ok(Value::Number(SchemeNumber::Real(product)))
}

/// Min comprehension
fn min_ec(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut min_val: Option<f64> = None;
    for arg in args {
        let val = match arg {
            Value::Number(SchemeNumber::Integer(i)) => *i as f64,
            Value::Number(SchemeNumber::Real(r)) => *r,
            Value::Number(SchemeNumber::Rational(n, d)) => *n as f64 / *d as f64,
            _ => continue, // Skip non-numeric values
        };
        
        min_val = Some(match min_val {
            None => val,
            Some(current_min) => current_min.min(val),
        });
    }
    
    match min_val {
        Some(val) => Ok(Value::Number(SchemeNumber::Real(val))),
        None => Err(LambdustError::runtime_error("no numeric values found")),
    }
}

/// Max comprehension
fn max_ec(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut max_val: Option<f64> = None;
    for arg in args {
        let val = match arg {
            Value::Number(SchemeNumber::Integer(i)) => *i as f64,
            Value::Number(SchemeNumber::Real(r)) => *r,
            Value::Number(SchemeNumber::Rational(n, d)) => *n as f64 / *d as f64,
            _ => continue, // Skip non-numeric values
        };
        
        max_val = Some(match max_val {
            None => val,
            Some(current_max) => current_max.max(val),
        });
    }
    
    match max_val {
        Some(val) => Ok(Value::Number(SchemeNumber::Real(val))),
        None => Err(LambdustError::runtime_error("no numeric values found")),
    }
}

/// Any? comprehension
fn any_ec(args: &[Value]) -> Result<Value> {
    for arg in args {
        match arg {
            Value::Boolean(false) => continue,
            _ => return Ok(Value::Boolean(true)), // Any non-#f value is truthy
        }
    }
    Ok(Value::Boolean(false))
}

/// Every? comprehension
fn every_ec(args: &[Value]) -> Result<Value> {
    for arg in args {
        match arg {
            Value::Boolean(false) => return Ok(Value::Boolean(false)),
            _ => continue, // Any non-#f value is truthy
        }
    }
    Ok(Value::Boolean(true))
}

/// First comprehension
fn first_ec(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        Err(LambdustError::runtime_error("no values in comprehension"))
    } else {
        Ok(args[0].clone())
    }
}

/// Last comprehension
fn last_ec(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        Err(LambdustError::runtime_error("no values in comprehension"))
    } else {
        Ok(args[args.len() - 1].clone())
    }
}

/// Fold comprehension
fn fold_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("fold-ec not yet implemented"))
}

/// Fold3 comprehension
fn fold3_ec(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper procedure calling
    Err(LambdustError::runtime_error("fold3-ec not yet implemented"))
}

/// Range generator
fn range_generator(args: &[Value]) -> Result<Value> {
    match args.len() {
        1 => {
            // (:range n) -> 0, 1, ..., n-1
            let n = extract_integer(&args[0])?;
            let range: Vec<Value> = (0..n)
                .map(|i| Value::Number(SchemeNumber::Integer(i)))
                .collect();
            Ok(Value::from_vector(range))
        }
        2 => {
            // (:range start end) -> start, start+1, ..., end-1
            let start = extract_integer(&args[0])?;
            let end = extract_integer(&args[1])?;
            let range: Vec<Value> = (start..end)
                .map(|i| Value::Number(SchemeNumber::Integer(i)))
                .collect();
            Ok(Value::from_vector(range))
        }
        3 => {
            // (:range start end step) -> start, start+step, start+2*step, ...
            let start = extract_integer(&args[0])?;
            let end = extract_integer(&args[1])?;
            let step = extract_integer(&args[2])?;
            
            if step == 0 {
                return Err(LambdustError::runtime_error("step cannot be zero"));
            }
            
            let mut range = Vec::new();
            let mut current = start;
            
            if step > 0 {
                while current < end {
                    range.push(Value::Number(SchemeNumber::Integer(current)));
                    current += step;
                }
            } else {
                while current > end {
                    range.push(Value::Number(SchemeNumber::Integer(current)));
                    current += step;
                }
            }
            
            Ok(Value::from_vector(range))
        }
        _ => Err(LambdustError::runtime_error("expected 1, 2, or 3 arguments".to_string())),
    }
}

/// Real range generator
fn real_range_generator(args: &[Value]) -> Result<Value> {
    match args.len() {
        2 => {
            // (:real-range start end) with default step 1.0
            let start = extract_real(&args[0])?;
            let end = extract_real(&args[1])?;
            let step = 1.0;
            
            let mut range = Vec::new();
            let mut current = start;
            
            while current < end {
                range.push(Value::Number(SchemeNumber::Real(current)));
                current += step;
            }
            
            Ok(Value::from_vector(range))
        }
        3 => {
            // (:real-range start end step)
            let start = extract_real(&args[0])?;
            let end = extract_real(&args[1])?;
            let step = extract_real(&args[2])?;
            
            if step == 0.0 {
                return Err(LambdustError::runtime_error("step cannot be zero"));
            }
            
            let mut range = Vec::new();
            let mut current = start;
            
            if step > 0.0 {
                while current < end {
                    range.push(Value::Number(SchemeNumber::Real(current)));
                    current += step;
                }
            } else {
                while current > end {
                    range.push(Value::Number(SchemeNumber::Real(current)));
                    current += step;
                }
            }
            
            Ok(Value::from_vector(range))
        }
        _ => Err(LambdustError::runtime_error("expected 2 or 3 arguments".to_string())),
    }
}

/// Character range generator
fn char_range_generator(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let start_char = extract_character(&args[0])?;
    let end_char = extract_character(&args[1])?;
    
    let start_code = start_char as u32;
    let end_code = end_char as u32;
    
    let mut range = Vec::new();
    for code in start_code..=end_code {
        if let Some(ch) = char::from_u32(code) {
            range.push(Value::Character(ch));
        }
    }
    
    Ok(Value::from_vector(range))
}

/// List generator
fn list_generator(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    // Return the list as-is for iteration
    Ok(args[0].clone())
}

/// String generator
fn string_generator(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    match &args[0] {
        Value::String(s) => {
            let chars: Vec<Value> = s.chars()
                .map(|c| Value::Character(c))
                .collect();
            Ok(Value::from_vector(chars))
        }
        _ => Err(LambdustError::type_error("expected string".to_string())),
    }
}

/// Vector generator
fn vector_generator(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    // Return the vector as a list for iteration
    match &args[0] {
        Value::Vector(v) => Ok(Value::from_vector(v.clone())),
        _ => Err(LambdustError::type_error("expected vector".to_string())),
    }
}

/// Integers generator (infinite sequence)
fn integers_generator(args: &[Value]) -> Result<Value> {
    let start = if args.is_empty() {
        0
    } else {
        extract_integer(&args[0])?
    };
    
    // Return a finite but large range for practical purposes
    let range: Vec<Value> = (start..start + 1000)
        .map(|i| Value::Number(SchemeNumber::Integer(i)))
        .collect();
    Ok(Value::from_vector(range))
}

/// Port generator
fn port_generator(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper port handling
    Err(LambdustError::runtime_error(":port generator not yet implemented"))
}

/// While generator
fn while_generator(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper condition evaluation
    Err(LambdustError::runtime_error(":while generator not yet implemented"))
}

/// Until generator
fn until_generator(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper condition evaluation
    Err(LambdustError::runtime_error(":until generator not yet implemented"))
}

/// Parallel generator
fn parallel_generator(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper parallel iteration
    Err(LambdustError::runtime_error(":parallel generator not yet implemented"))
}

/// Do generator
fn do_generator(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper do-loop semantics
    Err(LambdustError::runtime_error(":do generator not yet implemented"))
}

/// Let generator
fn let_generator(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - would need proper let binding
    Err(LambdustError::runtime_error(":let generator not yet implemented"))
}

/// Helper function to extract integer from Value
fn extract_integer(value: &Value) -> Result<i64> {
    match value {
        Value::Number(SchemeNumber::Integer(i)) => Ok(*i),
        _ => Err(LambdustError::type_error("expected integer".to_string())),
    }
}

/// Helper function to extract real number from Value
fn extract_real(value: &Value) -> Result<f64> {
    match value {
        Value::Number(SchemeNumber::Real(r)) => Ok(*r),
        Value::Number(SchemeNumber::Integer(i)) => Ok(*i as f64),
        Value::Number(SchemeNumber::Rational(n, d)) => Ok(*n as f64 / *d as f64),
        _ => Err(LambdustError::type_error("expected real number".to_string())),
    }
}

/// Helper function to extract character from Value
fn extract_character(value: &Value) -> Result<char> {
    match value {
        Value::Character(c) => Ok(*c),
        _ => Err(LambdustError::type_error("expected character".to_string())),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_range_generator() {
        // Test single argument (:range 5) -> 0,1,2,3,4
        let result = range_generator(&[Value::Number(SchemeNumber::Integer(5))]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 5);
        assert_eq!(as_vec[0], Value::Number(SchemeNumber::Integer(0)));
        assert_eq!(as_vec[4], Value::Number(SchemeNumber::Integer(4)));
        
        // Test two arguments (:range 2 7) -> 2,3,4,5,6
        let result = range_generator(&[
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(7))
        ]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 5);
        assert_eq!(as_vec[0], Value::Number(SchemeNumber::Integer(2)));
        assert_eq!(as_vec[4], Value::Number(SchemeNumber::Integer(6)));
        
        // Test three arguments with step (:range 0 10 2) -> 0,2,4,6,8
        let result = range_generator(&[
            Value::Number(SchemeNumber::Integer(0)),
            Value::Number(SchemeNumber::Integer(10)),
            Value::Number(SchemeNumber::Integer(2))
        ]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 5);
        assert_eq!(as_vec[0], Value::Number(SchemeNumber::Integer(0)));
        assert_eq!(as_vec[2], Value::Number(SchemeNumber::Integer(4)));
    }

    #[test]
    fn test_char_range_generator() {
        
        
        let result = char_range_generator(&[
            Value::Character('a'),
            Value::Character('e')
        ]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 5); // a, b, c, d, e
        assert_eq!(as_vec[0], Value::Character('a'));
        assert_eq!(as_vec[4], Value::Character('e'));
    }

    #[test]
    fn test_string_generator() {
        
        
        let result = string_generator(&[Value::String("hello".to_string())]).unwrap();
        let as_vec = result.to_vector().unwrap();
        assert_eq!(as_vec.len(), 5);
        assert_eq!(as_vec[0], Value::Character('h'));
        assert_eq!(as_vec[4], Value::Character('o'));
    }

    #[test]
    fn test_aggregate_comprehensions() {
        
        
        let nums = vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
            Value::Number(SchemeNumber::Integer(5)),
        ];
        
        // Test sum
        let result = sum_ec(&nums).unwrap();
        if let Value::Number(SchemeNumber::Real(sum)) = result {
            assert_eq!(sum, 15.0);
        } else {
            panic!("Expected real number result");
        }
        
        // Test product
        let result = product_ec(&nums).unwrap();
        if let Value::Number(SchemeNumber::Real(product)) = result {
            assert_eq!(product, 120.0);
        } else {
            panic!("Expected real number result");
        }
        
        // Test min
        let result = min_ec(&nums).unwrap();
        if let Value::Number(SchemeNumber::Real(min_val)) = result {
            assert_eq!(min_val, 1.0);
        } else {
            panic!("Expected real number result");
        }
        
        // Test max
        let result = max_ec(&nums).unwrap();
        if let Value::Number(SchemeNumber::Real(max_val)) = result {
            assert_eq!(max_val, 5.0);
        } else {
            panic!("Expected real number result");
        }
    }

    #[test]
    fn test_boolean_comprehensions() {
        
        
        let mixed_values = vec![
            Value::Boolean(true),
            Value::Number(SchemeNumber::Integer(42)),
            Value::String("hello".to_string()),
        ];
        
        // Test any? - should be true (all values are truthy)
        let result = any_ec(&mixed_values).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test every? - should be true (no #f values)
        let result = every_ec(&mixed_values).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        let with_false = vec![
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Number(SchemeNumber::Integer(42)),
        ];
        
        // Test every? with false - should be false
        let result = every_ec(&with_false).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_srfi_148_module() {
        let module = Srfi148Module::new();
        assert_eq!(module.srfi_id(), 148);
        assert_eq!(module.name(), "SRFI 148");
        assert_eq!(module.parts(), vec!["comprehensions"]);
        
        let exports = module.exports();
        assert!(exports.contains_key("list-ec"));
        assert!(exports.contains_key(":range"));
        assert!(exports.contains_key("sum-ec"));
        assert!(exports.contains_key("any?-ec"));
        
        // Test exports_for_parts
        let partial_exports = module.exports_for_parts(&["comprehensions"]).unwrap();
        assert_eq!(partial_exports.len(), exports.len());
    }
}