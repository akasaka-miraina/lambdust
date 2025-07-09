//! SRFI 139: Syntax Parameters
//!
//! This SRFI defines syntax parameters, which are to the expansion
//! process of a Scheme program what parameters are to the evaluation
//! process of a Scheme program.
//!
//! NOTE: This is a simplified implementation that provides the basic
//! runtime support for syntax parameters without full macro integration.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// Define a syntax parameter (placeholder implementation)
fn define_syntax_parameter_proc(_args: &[Value]) -> Result<Value> {
    // For now, just return success to indicate the parameter was "defined"
    // In a full implementation, this would register with the macro system
    Ok(Value::Symbol("syntax-parameter-defined".to_string()))
}

/// Check if a syntax parameter is defined (placeholder)
fn syntax_parameter_defined_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;

    match &args[0] {
        Value::Symbol(_) => {
            // For demonstration, return true for any symbol
            // In a full implementation, this would check the macro registry
            Ok(Value::Boolean(true))
        }
        _ => Err(LambdustError::type_error("Expected symbol".to_string())),
    }
}

/// Get syntax parameter value (placeholder)
fn syntax_parameter_value_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;

    match &args[0] {
        Value::Symbol(name) => {
            // Return a placeholder value
            Ok(Value::String(format!("placeholder-value-for-{}", name)))
        }
        _ => Err(LambdustError::type_error("Expected symbol".to_string())),
    }
}

/// Syntax parameterize (simplified implementation)
fn syntax_parameterize_proc(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    // For now, just execute the body (last argument)
    // In a full implementation, this would set up parameter bindings
    Ok(args[args.len() - 1].clone())
}

/// List all syntax parameters (placeholder)
fn syntax_parameter_list_proc(_args: &[Value]) -> Result<Value> {
    // Return empty list for now
    Ok(Value::Vector(Vec::new()))
}

/// Create a syntax parameter transformer (placeholder)
fn make_syntax_parameter_transformer_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;

    // Return the input value as a simple transformer
    Ok(args[0].clone())
}

/// SRFI 139: Syntax Parameters implementation
pub struct Srfi139;

impl super::SrfiModule for Srfi139 {
    fn srfi_id(&self) -> u32 {
        139
    }

    fn name(&self) -> &'static str {
        "Syntax Parameters"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["syntax", "parameters"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Core syntax parameter procedures (placeholder implementations)
        exports.insert(
            "define-syntax-parameter".to_string(),
            make_builtin_procedure(
                "define-syntax-parameter",
                Some(2),
                define_syntax_parameter_proc,
            ),
        );

        exports.insert(
            "syntax-parameter-value".to_string(),
            make_builtin_procedure(
                "syntax-parameter-value",
                Some(1),
                syntax_parameter_value_proc,
            ),
        );

        exports.insert(
            "syntax-parameter-defined?".to_string(),
            make_builtin_procedure(
                "syntax-parameter-defined?",
                Some(1),
                syntax_parameter_defined_proc,
            ),
        );

        exports.insert(
            "syntax-parameterize".to_string(),
            make_builtin_procedure("syntax-parameterize", None, syntax_parameterize_proc),
        );

        exports.insert(
            "syntax-parameter-list".to_string(),
            make_builtin_procedure("syntax-parameter-list", Some(0), syntax_parameter_list_proc),
        );

        exports.insert(
            "make-syntax-parameter-transformer".to_string(),
            make_builtin_procedure(
                "make-syntax-parameter-transformer",
                Some(1),
                make_syntax_parameter_transformer_proc,
            ),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of part
        Ok(self.exports())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_srfi_139_info() {
        let srfi = Srfi139;
        assert_eq!(srfi.srfi_id(), 139);
        assert_eq!(srfi.name(), "Syntax Parameters");
        assert!(srfi.parts().contains(&"syntax"));
        assert!(srfi.parts().contains(&"parameters"));
    }

    #[test]
    fn test_define_syntax_parameter_proc() {
        let name = Value::Symbol("test-param".to_string());
        let default_val = Value::Number(crate::lexer::SchemeNumber::Integer(42));

        let result = define_syntax_parameter_proc(&[name, default_val]).unwrap();
        assert_eq!(
            result,
            Value::Symbol("syntax-parameter-defined".to_string())
        );
    }

    #[test]
    fn test_syntax_parameter_defined_proc() {
        let name = Value::Symbol("test-param".to_string());
        let result = syntax_parameter_defined_proc(&[name]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_syntax_parameter_value_proc() {
        let name = Value::Symbol("test-param".to_string());
        let result = syntax_parameter_value_proc(&[name]).unwrap();
        assert_eq!(
            result,
            Value::String("placeholder-value-for-test-param".to_string())
        );
    }

    #[test]
    fn test_syntax_parameterize_proc() {
        let bindings = Value::Vector(vec![Value::Vector(vec![
            Value::Symbol("param".to_string()),
            Value::Number(crate::lexer::SchemeNumber::Integer(123)),
        ])]);
        let body = Value::String("body-result".to_string());

        let result = syntax_parameterize_proc(&[bindings, body.clone()]).unwrap();
        assert_eq!(result, body);
    }

    #[test]
    fn test_syntax_parameter_list_proc() {
        let result = syntax_parameter_list_proc(&[]).unwrap();
        assert_eq!(result, Value::Vector(Vec::new()));
    }

    #[test]
    fn test_make_syntax_parameter_transformer_proc() {
        let input = Value::Symbol("transformer-input".to_string());
        let result = make_syntax_parameter_transformer_proc(&[input.clone()]).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_srfi_139_exports() {
        let srfi = Srfi139;
        let exports = srfi.exports();

        assert!(exports.contains_key("define-syntax-parameter"));
        assert!(exports.contains_key("syntax-parameter-value"));
        assert!(exports.contains_key("syntax-parameter-defined?"));
        assert!(exports.contains_key("syntax-parameterize"));
        assert!(exports.contains_key("syntax-parameter-list"));
        assert!(exports.contains_key("make-syntax-parameter-transformer"));
    }

    #[test]
    fn test_error_cases() {
        // Test invalid argument types
        let result = syntax_parameter_defined_proc(&[Value::Number(
            crate::lexer::SchemeNumber::Integer(42),
        )]);
        assert!(result.is_err());

        let result = syntax_parameter_value_proc(&[Value::Boolean(true)]);
        assert!(result.is_err());

        // Test arity errors
        let result = syntax_parameterize_proc(&[Value::Symbol("only-one-arg".to_string())]);
        assert!(result.is_err());
    }
}
