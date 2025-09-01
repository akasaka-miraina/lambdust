//! SRFI-11: let-values - Multiple value binding forms
//!
//! This module implements SRFI-11, which provides `let-values` and `let*-values`
//! constructs for binding multiple values returned by expressions to variables.
//!
//! ## R7RS Compliance
//!
//! SRFI-11 is included in R7RS-large and provides essential syntax for working
//! with multiple values in Scheme. This implementation supports all standard
//! formal parameter patterns and integrates seamlessly with SRFI-8.
//!
//! ## Syntax
//!
//! ```scheme
//! (let-values ([(formals1 producer1) ...]) body ...)
//! (let*-values ([(formals1 producer1) ...]) body ...)
//! ```
//!
//! Where:
//! - `formals` follows standard lambda formal parameter patterns
//! - `producer` expressions may return multiple values
//! - Bindings in `let-values` are evaluated in parallel (independent)
//! - Bindings in `let*-values` are evaluated sequentially (dependent)
//!
//! ## Examples
//!
//! ```scheme
//! (let-values ([(a b) (values 1 2)]
//!              [(c d e) (values 3 4 5)])
//!   (+ a b c d e))  ; => 15
//!
//! (let*-values ([(a b) (values 1 2)]
//!               [(c d) (values a b)])
//!   (+ a b c d))  ; => 6
//!
//! (let-values ([(first . rest) (values 'x 'y 'z)])
//!   (cons first rest))  ; => (x y z)
//! ```
//!
//! ## Formal Parameter Support
//!
//! Supports all R7RS formal parameter patterns:
//! - **Fixed**: `(a b c)` - exactly 3 values
//! - **Variable**: `args` - any number of values as list
//! - **Mixed**: `(a b . rest)` - at least 2 values, rest as list
//!
//! ## Implementation Strategy
//!
//! The implementation uses macro expansion to leverage existing infrastructure:
//! 1. `let-values` expands to nested `call-with-values` forms
//! 2. `let*-values` expands to nested `let-values` forms
//! 3. Builds upon SRFI-8's multiple value support
//! 4. Provides comprehensive error reporting for arity mismatches

use crate::ast::Formals;
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::srfi8_receive;
use crate::utils::intern_symbol;
use std::sync::Arc;

/// Creates SRFI-11 let-values bindings for the standard library.
pub fn create_srfi11_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // let-values: parallel multiple value bindings
    env.define(
        "let-values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "let-values".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(primitive_let_values),
            effects: vec![Effect::Pure],
        })),
    );

    // let*-values: sequential multiple value bindings
    env.define(
        "let*-values".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "let*-values".to_string(),
            arity_min: 2,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(primitive_let_star_values),
            effects: vec![Effect::Pure],
        })),
    );

    // Helper for macro expansion validation
    env.define(
        "let-values?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "let-values?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_let_values_p),
            effects: vec![Effect::Pure],
        })),
    );
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// Primitive implementation of let-values (parallel bindings).
fn primitive_let_values(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "let-values requires at least 2 arguments: bindings and body".to_string(),
            None,
        )));
    }

    let bindings = &args[0];
    let body = &args[1..];

    // Parse bindings list
    let binding_pairs = parse_bindings(bindings, "let-values")?;

    // Expand to call-with-values forms
    expand_let_values(&binding_pairs, body)
}

/// Primitive implementation of let*-values (sequential bindings).
fn primitive_let_star_values(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "let*-values requires at least 2 arguments: bindings and body".to_string(),
            None,
        )));
    }

    let bindings = &args[0];
    let body = &args[1..];

    // Parse bindings list
    let binding_pairs = parse_bindings(bindings, "let*-values")?;

    // Expand to nested let-values forms
    expand_let_star_values(&binding_pairs, body)
}

/// Predicate to check if a form is a let-values binding.
fn primitive_let_values_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("let-values? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    // Check if the form looks like a let-values binding
    let is_let_values = if args[0].is_list() {
        if let Some(elements) = args[0].as_list() {
            if let Some(first) = elements.first() {
                if let Value::Symbol(sym_id) = first {
                    use crate::utils::symbol::symbol_name;
                    symbol_name(*sym_id)
                        .map(|name| name == "let-values" || name == "let*-values")
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    Ok(Value::boolean(is_let_values))
}

// ============= BINDING PARSING =============

/// Represents a single binding in let-values.
#[derive(Debug, Clone)]
pub struct LetValuesBinding {
    /// The formal parameters pattern
    pub formals: Formals,
    /// The producer expression
    pub producer: Value,
}

/// Parses bindings from a list of binding pairs.
fn parse_bindings(bindings: &Value, context: &str) -> Result<Vec<LetValuesBinding>> {
    let binding_list = bindings.as_list().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            format!("{}: bindings must be a list", context),
            None,
        ))
    })?;

    let mut parsed_bindings = Vec::new();

    for (i, binding) in binding_list.iter().enumerate() {
        let binding_pair = binding.as_list().ok_or_else(|| {
            Box::new(DiagnosticError::runtime_error(
                format!("{}: binding {} must be a list", context, i),
                None,
            ))
        })?;

        if binding_pair.len() != 2 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "{}: binding {} must have exactly 2 elements (formals producer), got {}",
                    context,
                    i,
                    binding_pair.len()
                ),
                None,
            )));
        }

        let formals = parse_formals(&binding_pair[0], context)?;
        let producer = binding_pair[1].clone();

        parsed_bindings.push(LetValuesBinding { formals, producer });
    }

    Ok(parsed_bindings)
}

/// Parses formal parameters from a value.
fn parse_formals(formals_value: &Value, context: &str) -> Result<Formals> {
    match formals_value {
        // Fixed formals: (a b c)
        _ if formals_value.is_list() => {
            let params = formals_value.as_list().unwrap();
            let param_names: Result<Vec<String>> = params
                .iter()
                .map(|param| {
                    let sym_id = param.as_symbol().ok_or_else(|| {
                        Box::new(DiagnosticError::runtime_error(
                            format!("{}: formal parameter must be a symbol", context),
                            None,
                        ))
                    })?;
                    use crate::utils::symbol::symbol_name;
                    symbol_name(sym_id).ok_or_else(|| {
                        Box::new(DiagnosticError::runtime_error(
                            format!("{}: invalid symbol in formals", context),
                            None,
                        ))
                    })
                })
                .collect();

            Ok(Formals::Fixed(param_names?))
        }
        // Variable formals: args
        Value::Symbol(sym_id) => {
            use crate::utils::symbol::symbol_name;
            let param_name = symbol_name(*sym_id).ok_or_else(|| {
                Box::new(DiagnosticError::runtime_error(
                    format!("{}: invalid symbol in formals", context),
                    None,
                ))
            })?;
            Ok(Formals::Variable(param_name))
        }
        // Mixed formals: (a b . rest) - represented as improper list
        Value::Pair(car, cdr) => parse_improper_formals(car, cdr, context),
        Value::Nil => Ok(Formals::Fixed(vec![])), // Empty parameter list
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{}: invalid formals syntax", context),
            None,
        ))),
    }
}

/// Parses improper list formals (a b . rest).
fn parse_improper_formals(car: &Value, cdr: &Value, context: &str) -> Result<Formals> {
    let mut fixed_params = Vec::new();
    let mut current_car = car.clone();
    let mut current_cdr = cdr.clone();

    loop {
        // Get the car (parameter)
        let param_id = current_car.as_symbol().ok_or_else(|| {
            Box::new(DiagnosticError::runtime_error(
                format!("{}: formal parameter must be a symbol", context),
                None,
            ))
        })?;
        use crate::utils::symbol::symbol_name;
        let param = symbol_name(param_id).ok_or_else(|| {
            Box::new(DiagnosticError::runtime_error(
                format!("{}: invalid symbol in formals", context),
                None,
            ))
        })?;
        fixed_params.push(param);

        // Check the cdr
        match current_cdr {
            Value::Nil => {
                // Proper list ending - convert to Fixed formals
                return Ok(Formals::Fixed(fixed_params));
            }
            Value::Symbol(rest_id) => {
                // Improper list ending - Mixed formals
                use crate::utils::symbol::symbol_name;
                let rest_name = symbol_name(rest_id).ok_or_else(|| {
                    Box::new(DiagnosticError::runtime_error(
                        format!("{}: invalid rest parameter symbol", context),
                        None,
                    ))
                })?;
                return Ok(Formals::Mixed {
                    fixed: fixed_params,
                    rest: rest_name,
                });
            }
            Value::Pair(next_car, next_cdr) => {
                // Continue with the cdr
                current_car = (*next_car).clone();
                current_cdr = (*next_cdr).clone();
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("{}: malformed improper list in formals", context),
                    None,
                )));
            }
        }
    }
}

// ============= MACRO EXPANSION =============

/// Expands let-values to nested call-with-values forms.
pub fn expand_let_values(bindings: &[LetValuesBinding], body: &[Value]) -> Result<Value> {
    if bindings.is_empty() {
        // No bindings - just evaluate body
        if body.len() == 1 {
            return Ok(body[0].clone());
        } else {
            // Multiple expressions - wrap in begin
            return Ok(create_begin_form(body.to_vec()));
        }
    }

    // Create nested call-with-values structure
    expand_bindings_to_call_with_values(bindings, body, 0)
}

/// Recursively expands bindings to call-with-values forms.
fn expand_bindings_to_call_with_values(
    bindings: &[LetValuesBinding],
    body: &[Value],
    index: usize,
) -> Result<Value> {
    if index >= bindings.len() {
        // All bindings processed - execute body
        if body.len() == 1 {
            Ok(body[0].clone())
        } else {
            Ok(create_begin_form(body.to_vec()))
        }
    } else {
        let binding = &bindings[index];

        // Create producer thunk
        let producer_thunk = create_producer_thunk(&binding.producer)?;

        // Create consumer lambda for this binding
        let remaining_body = if index + 1 < bindings.len() {
            // More bindings to process
            vec![expand_bindings_to_call_with_values(
                bindings,
                body,
                index + 1,
            )?]
        } else {
            // Last binding - use actual body
            body.to_vec()
        };

        let consumer_lambda = create_consumer_lambda(&binding.formals, &remaining_body)?;

        // Create call-with-values form
        Ok(Value::list(vec![
            Value::symbol(intern_symbol("call-with-values")),
            producer_thunk,
            consumer_lambda,
        ]))
    }
}

/// Expands let*-values to nested let-values forms.
pub fn expand_let_star_values(bindings: &[LetValuesBinding], body: &[Value]) -> Result<Value> {
    if bindings.is_empty() {
        // No bindings - just evaluate body
        if body.len() == 1 {
            return Ok(body[0].clone());
        } else {
            return Ok(create_begin_form(body.to_vec()));
        }
    }

    // Expand from right to left (innermost to outermost)
    expand_sequential_bindings(bindings, body, bindings.len() - 1)
}

/// Recursively expands sequential bindings (let*-values).
fn expand_sequential_bindings(
    bindings: &[LetValuesBinding],
    body: &[Value],
    index: usize,
) -> Result<Value> {
    let binding = &bindings[index];

    let inner_body = if index == 0 {
        // Last binding (innermost) - use actual body
        body.to_vec()
    } else {
        // More bindings - create nested let-values
        vec![expand_sequential_bindings(bindings, body, index - 1)?]
    };

    // Create single let-values form for this binding
    let single_binding = vec![binding.clone()];
    expand_let_values(&single_binding, &inner_body)
}

// ============= UTILITY FUNCTIONS =============

/// Creates a producer thunk that evaluates the producer expression.
fn create_producer_thunk(producer: &Value) -> Result<Value> {
    Ok(Value::list(vec![
        Value::symbol(intern_symbol("lambda")),
        Value::Nil, // No parameters
        producer.clone(),
    ]))
}

/// Creates a consumer lambda from formals and body.
fn create_consumer_lambda(formals: &Formals, body: &[Value]) -> Result<Value> {
    let formals_list = srfi8_receive::formals_to_list(formals)?;

    let mut lambda_parts = vec![Value::symbol(intern_symbol("lambda")), formals_list];

    lambda_parts.extend(body.iter().cloned());

    Ok(Value::list(lambda_parts))
}

/// Creates a begin form for multiple expressions.
fn create_begin_form(expressions: Vec<Value>) -> Value {
    let mut begin_form = vec![Value::symbol(intern_symbol("begin"))];
    begin_form.extend(expressions);
    Value::list(begin_form)
}

// ============= VALIDATION UTILITIES =============

/// Validates let-values binding syntax.
pub fn validate_let_values_syntax(bindings: &Value) -> Result<()> {
    let binding_list = bindings.as_list().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            "let-values: bindings must be a list".to_string(),
            None,
        ))
    })?;

    for (i, binding) in binding_list.iter().enumerate() {
        let binding_pair = binding.as_list().ok_or_else(|| {
            Box::new(DiagnosticError::runtime_error(
                format!("let-values: binding {} must be a list", i),
                None,
            ))
        })?;

        if binding_pair.len() != 2 {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "let-values: binding {} must have exactly 2 elements, got {}",
                    i,
                    binding_pair.len()
                ),
                None,
            )));
        }

        // Validate formals syntax
        validate_formals_syntax(&binding_pair[0], &format!("let-values binding {}", i))?;
    }

    Ok(())
}

/// Validates formal parameter syntax.
fn validate_formals_syntax(formals: &Value, context: &str) -> Result<()> {
    if formals.is_list() {
        if let Some(params) = formals.as_list() {
            // Fixed formals - all must be symbols
            for (j, param) in params.iter().enumerate() {
                if !matches!(param, Value::Symbol(_)) {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("{}: parameter {} must be a symbol", context, j),
                        None,
                    )));
                }
            }
        }
    } else if matches!(formals, Value::Symbol(_)) {
        // Variable formals - valid
    } else if matches!(formals, Value::Pair(_, _)) {
        // Mixed formals - validate structure
        validate_improper_list_formals(formals, context)?;
    } else if matches!(formals, Value::Nil) {
        // Empty formals - valid
    } else {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("{}: invalid formals syntax", context),
            None,
        )));
    }

    Ok(())
}

/// Validates improper list formals structure.
fn validate_improper_list_formals(formals: &Value, context: &str) -> Result<()> {
    let mut current = formals.clone();

    loop {
        match current {
            Value::Pair(car, cdr) => {
                // Validate car is a symbol
                if !matches!(car.as_ref(), Value::Symbol(_)) {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("{}: formal parameter must be a symbol", context),
                        None,
                    )));
                }

                // Check cdr
                match cdr.as_ref() {
                    Value::Nil => break,                 // Proper list ending
                    Value::Symbol(_) => break,           // Improper list ending
                    _ => current = cdr.as_ref().clone(), // Continue
                }
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("{}: malformed improper list in formals", context),
                    None,
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::ThreadSafeEnvironment;

    fn create_test_env() -> Arc<ThreadSafeEnvironment> {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        create_srfi11_bindings(&env);
        // Also need SRFI-8 for call-with-values
        crate::stdlib::srfi8_receive::create_srfi8_bindings(&env);
        env
    }

    #[test]
    fn test_parse_fixed_formals() {
        let formals = Value::list(vec![
            Value::symbol(intern_symbol("a")),
            Value::symbol(intern_symbol("b")),
        ]);

        let result = parse_formals(&formals, "test").unwrap();
        assert!(matches!(result, Formals::Fixed(ref params) if params.len() == 2));

        if let Formals::Fixed(params) = result {
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
        }
    }

    #[test]
    fn test_parse_variable_formals() {
        let formals = Value::symbol(intern_symbol("args"));

        let result = parse_formals(&formals, "test").unwrap();
        assert!(matches!(result, Formals::Variable(ref name) if name == "args"));
    }

    #[test]
    fn test_parse_mixed_formals() {
        // Create (a b . rest)
        let rest = Value::symbol(intern_symbol("rest"));
        let b_pair = Value::cons(Value::symbol(intern_symbol("b")), rest);
        let formals = Value::cons(Value::symbol(intern_symbol("a")), b_pair);

        let result = parse_formals(&formals, "test").unwrap();
        assert!(matches!(result, Formals::Mixed { .. }));

        if let Formals::Mixed { fixed, rest } = result {
            assert_eq!(fixed.len(), 2);
            assert_eq!(fixed[0], "a");
            assert_eq!(fixed[1], "b");
            assert_eq!(rest, "rest");
        }
    }

    #[test]
    fn test_parse_empty_formals() {
        let formals = Value::Nil;

        let result = parse_formals(&formals, "test").unwrap();
        assert!(matches!(result, Formals::Fixed(ref params) if params.is_empty()));
    }

    #[test]
    fn test_parse_bindings() {
        let binding1 = Value::list(vec![
            Value::list(vec![Value::symbol(intern_symbol("a"))]),
            Value::integer(1),
        ]);

        let binding2 = Value::list(vec![
            Value::list(vec![
                Value::symbol(intern_symbol("x")),
                Value::symbol(intern_symbol("y")),
            ]),
            Value::list(vec![
                Value::symbol(intern_symbol("values")),
                Value::integer(2),
                Value::integer(3),
            ]),
        ]);

        let bindings_list = Value::list(vec![binding1, binding2]);

        let result = parse_bindings(&bindings_list, "test").unwrap();
        assert_eq!(result.len(), 2);

        // First binding: (a) <- 1
        assert!(matches!(result[0].formals, Formals::Fixed(ref params) if params.len() == 1));
        assert_eq!(result[0].producer, Value::integer(1));

        // Second binding: (x y) <- (values 2 3)
        assert!(matches!(result[1].formals, Formals::Fixed(ref params) if params.len() == 2));
    }

    #[test]
    fn test_let_values_predicate() {
        let let_values_form = Value::list(vec![
            Value::symbol(intern_symbol("let-values")),
            Value::Nil,
            Value::integer(42),
        ]);

        let result = primitive_let_values_p(&[let_values_form]).unwrap();
        assert_eq!(result, Value::boolean(true));

        let not_let_values = Value::list(vec![
            Value::symbol(intern_symbol("let")),
            Value::Nil,
            Value::integer(42),
        ]);

        let result = primitive_let_values_p(&[not_let_values]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_validate_let_values_syntax() {
        // Valid syntax
        let valid_bindings = Value::list(vec![Value::list(vec![
            Value::list(vec![Value::symbol(intern_symbol("a"))]),
            Value::integer(1),
        ])]);

        assert!(validate_let_values_syntax(&valid_bindings).is_ok());

        // Invalid syntax - not a list
        let invalid_bindings = Value::integer(42);
        assert!(validate_let_values_syntax(&invalid_bindings).is_err());

        // Invalid syntax - binding not a list
        let invalid_bindings = Value::list(vec![Value::integer(42)]);
        assert!(validate_let_values_syntax(&invalid_bindings).is_err());

        // Invalid syntax - wrong binding length
        let invalid_bindings = Value::list(vec![Value::list(vec![Value::integer(1)])]);
        assert!(validate_let_values_syntax(&invalid_bindings).is_err());
    }

    #[test]
    fn test_create_producer_thunk() {
        let producer = Value::list(vec![
            Value::symbol(intern_symbol("values")),
            Value::integer(1),
            Value::integer(2),
        ]);

        let thunk = create_producer_thunk(&producer).unwrap();

        if let Some(thunk_list) = thunk.as_list() {
            assert_eq!(thunk_list.len(), 3);
            assert_eq!(thunk_list[0], Value::symbol(intern_symbol("lambda")));
            assert_eq!(thunk_list[1], Value::Nil);
            assert_eq!(thunk_list[2], producer);
        } else {
            panic!("Expected thunk to be a list");
        }
    }

    #[test]
    fn test_expand_let_values_empty() {
        let bindings = vec![];
        let body = vec![Value::integer(42)];

        let result = expand_let_values(&bindings, &body).unwrap();
        assert_eq!(result, Value::integer(42));
    }

    #[test]
    fn test_expand_let_values_single_binding() {
        let binding = LetValuesBinding {
            formals: Formals::Fixed(vec!["a".to_string()]),
            producer: Value::integer(1),
        };
        let bindings = vec![binding];
        let body = vec![Value::symbol(intern_symbol("a"))];

        let result = expand_let_values(&bindings, &body).unwrap();

        // Should be a call-with-values form
        if let Some(result_list) = result.as_list() {
            assert_eq!(
                result_list[0],
                Value::symbol(intern_symbol("call-with-values"))
            );
            assert_eq!(result_list.len(), 3);
        } else {
            panic!("Expected result to be a list");
        }
    }

    #[test]
    fn test_expand_let_star_values_empty() {
        let bindings = vec![];
        let body = vec![Value::integer(42)];

        let result = expand_let_star_values(&bindings, &body).unwrap();
        assert_eq!(result, Value::integer(42));
    }

    #[test]
    fn test_create_begin_form() {
        let expressions = vec![Value::integer(1), Value::integer(2), Value::integer(3)];

        let result = create_begin_form(expressions.clone());

        if let Some(result_list) = result.as_list() {
            assert_eq!(result_list[0], Value::symbol(intern_symbol("begin")));
            assert_eq!(result_list.len(), 4);
            assert_eq!(result_list[1], Value::integer(1));
            assert_eq!(result_list[2], Value::integer(2));
            assert_eq!(result_list[3], Value::integer(3));
        } else {
            panic!("Expected result to be a list");
        }
    }

    #[test]
    fn test_validate_formals_syntax() {
        // Valid fixed formals
        let fixed = Value::list(vec![Value::symbol(intern_symbol("a"))]);
        assert!(validate_formals_syntax(&fixed, "test").is_ok());

        // Valid variable formals
        let variable = Value::symbol(intern_symbol("args"));
        assert!(validate_formals_syntax(&variable, "test").is_ok());

        // Valid empty formals
        assert!(validate_formals_syntax(&Value::Nil, "test").is_ok());

        // Invalid formals - non-symbol in list
        let invalid = Value::list(vec![Value::integer(42)]);
        assert!(validate_formals_syntax(&invalid, "test").is_err());
    }
}
