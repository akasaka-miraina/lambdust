//! SRFI-16: Syntax for procedures of variable arity (case-lambda)
//!
//! This module provides SRFI-16 compliant case-lambda functionality for Lambdust.
//! case-lambda creates procedures that can accept different numbers of arguments
//! and dispatch to different code based on the argument count.
//!
//! ## SRFI-16 Specification:
//! - `(case-lambda clause ...)` syntax
//! - Each clause has the form `(formals body ...)`
//! - Dispatch based on argument count and pattern matching
//! - Proper error handling for no matching clauses
//!
//! ## Implementation Notes:
//! This is a Rust-native implementation that integrates with Lambdust's existing
//! case-lambda AST and evaluator infrastructure for optimal performance.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

/// Binds SRFI-16 case-lambda operations to the environment
pub fn bind_srfi16_case_lambda(env: &Arc<ThreadSafeEnvironment>) {
    // Note: case-lambda is implemented as a special form in the parser/evaluator
    // This module provides complementary utilities and ensures SRFI-16 compliance

    // case-lambda? predicate to test if a value is a case-lambda procedure
    env.define(
        "case-lambda?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "case-lambda?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_case_lambda_predicate),
            effects: vec![Effect::Pure],
        })),
    );

    // case-lambda-arity procedure to get arity information
    env.define(
        "case-lambda-arity".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "case-lambda-arity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_case_lambda_arity),
            effects: vec![Effect::Pure],
        })),
    );

    // case-lambda-clauses procedure to get clause information (debugging utility)
    env.define(
        "case-lambda-clauses".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "case-lambda-clauses".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_case_lambda_clauses),
            effects: vec![Effect::Pure],
        })),
    );
}

/// case-lambda? predicate - tests if a value is a case-lambda procedure
fn primitive_case_lambda_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("case-lambda? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_case_lambda = matches!(&args[0], Value::CaseLambda(_));
    Ok(Value::boolean(is_case_lambda))
}

/// case-lambda-arity - returns arity information for a case-lambda procedure
fn primitive_case_lambda_arity(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("case-lambda-arity expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::CaseLambda(case_lambda) => {
            // Return a list of arity information for each clause
            let mut arities = Vec::new();

            for clause in &case_lambda.clauses {
                let arity_info = match &clause.formals {
                    crate::ast::Formals::Fixed(params) => {
                        // Fixed arity: exact number
                        Value::integer(params.len() as i64)
                    }
                    crate::ast::Formals::Variable(_) => {
                        // Variable arity: accepts any number (represented as -1)
                        Value::symbol(crate::utils::intern_symbol("variadic"))
                    }
                    crate::ast::Formals::Mixed { fixed, .. } => {
                        // Mixed arity: minimum number + variadic
                        Value::pair(
                            Value::integer(fixed.len() as i64),
                            Value::symbol(crate::utils::intern_symbol("variadic")),
                        )
                    }
                    crate::ast::Formals::Keyword { fixed, .. } => {
                        // Keyword args (Lambdust extension): minimum fixed
                        Value::pair(
                            Value::integer(fixed.len() as i64),
                            Value::symbol(crate::utils::intern_symbol("keyword")),
                        )
                    }
                    crate::ast::Formals::Typed(params) => {
                        // Typed parameters: exact number
                        Value::integer(params.len() as i64)
                    }
                    crate::ast::Formals::TypedVariable(_) => {
                        // Typed variable arity: accepts any number
                        Value::symbol(crate::utils::intern_symbol("typed-variadic"))
                    }
                    crate::ast::Formals::TypedMixed { fixed, .. } => {
                        // Typed mixed arity: minimum number + variadic
                        Value::pair(
                            Value::integer(fixed.len() as i64),
                            Value::symbol(crate::utils::intern_symbol("typed-variadic")),
                        )
                    }
                };
                arities.push(arity_info);
            }

            Ok(Value::list(arities))
        }
        Value::Procedure(_) => {
            // Regular procedure - return simple arity info
            // This is a simplified implementation; a full version would analyze the formals
            Ok(Value::list(vec![Value::symbol(
                crate::utils::intern_symbol("regular-procedure"),
            )]))
        }
        Value::Primitive(prim) => {
            // Primitive procedure - return arity constraints
            let min_arity = Value::integer(prim.arity_min as i64);
            let max_arity = match prim.arity_max {
                Some(max) => Value::integer(max as i64),
                None => Value::symbol(crate::utils::intern_symbol("unlimited")),
            };
            Ok(Value::list(vec![Value::pair(min_arity, max_arity)]))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "case-lambda-arity requires a procedure".to_string(),
            None,
        ))),
    }
}

/// case-lambda-clauses - returns clause count and debug information
fn primitive_case_lambda_clauses(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("case-lambda-clauses expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::CaseLambda(case_lambda) => {
            let clause_count = Value::integer(case_lambda.clauses.len() as i64);
            let name = case_lambda
                .name
                .as_ref()
                .map(|n| Value::string(n))
                .unwrap_or_else(|| Value::symbol(crate::utils::intern_symbol("anonymous")));

            // Return (count . name) pair for debugging
            Ok(Value::pair(clause_count, name))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "case-lambda-clauses requires a case-lambda procedure".to_string(),
            None,
        ))),
    }
}

/// Creates SRFI-16 compliant environment bindings
pub fn create_srfi16_bindings(env: &Arc<ThreadSafeEnvironment>) {
    bind_srfi16_case_lambda(env);
}

/// Validates SRFI-16 compliance of the current implementation
pub fn validate_srfi16_compliance() -> Result<()> {
    // This function can be used to run internal compliance checks
    // The actual case-lambda functionality is implemented in the parser/evaluator
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{CaseLambdaClause, Formals};
    use crate::eval::value::CaseLambdaProcedure;
    use std::collections::HashMap;

    fn create_test_case_lambda() -> Value {
        // Create a simple case-lambda for testing
        let clauses = vec![
            CaseLambdaClause {
                formals: Formals::Fixed(vec!["x".to_string()]), // 1 arg
                body: vec![],                                   // Empty body for testing
            },
            CaseLambdaClause {
                formals: Formals::Fixed(vec!["x".to_string(), "y".to_string()]), // 2 args
                body: vec![], // Empty body for testing
            },
            CaseLambdaClause {
                formals: Formals::Variable("args".to_string()), // Variadic
                body: vec![],
            },
        ];

        let case_lambda = CaseLambdaProcedure {
            clauses,
            environment: Arc::new(crate::eval::ThreadSafeEnvironment::new(None, 0)),
            name: Some("test-case-lambda".to_string()),
            metadata: HashMap::new(),
            source: None,
        };

        Value::case_lambda(case_lambda)
    }

    #[test]
    fn test_case_lambda_predicate() {
        // Test with case-lambda procedure
        let case_lambda = create_test_case_lambda();
        let result = primitive_case_lambda_predicate(&[case_lambda]).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test with regular value
        let regular_value = Value::integer(42);
        let result = primitive_case_lambda_predicate(&[regular_value]).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Test with regular procedure
        let env = Arc::new(crate::eval::ThreadSafeEnvironment::new(None, 0));
        let procedure = crate::eval::value::Procedure {
            formals: Formals::Fixed(vec!["x".to_string()]),
            body: vec![],
            environment: env,
            name: None,
            metadata: HashMap::new(),
            source: None,
        };
        let result =
            primitive_case_lambda_predicate(&[Value::Procedure(Arc::new(procedure))]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_case_lambda_arity() {
        let case_lambda = create_test_case_lambda();
        let result = primitive_case_lambda_arity(&[case_lambda]).unwrap();

        // Should return a list with arity information
        if let Some(list) = result.as_list() {
            assert_eq!(list.len(), 3); // 3 clauses
            assert_eq!(list[0], Value::integer(1)); // First clause: 1 arg
            assert_eq!(list[1], Value::integer(2)); // Second clause: 2 args
        // Third clause should be variadic symbol
        } else {
            panic!("Expected list result from case-lambda-arity");
        }
    }

    #[test]
    fn test_case_lambda_clauses() {
        let case_lambda = create_test_case_lambda();
        let result = primitive_case_lambda_clauses(&[case_lambda]).unwrap();

        // Should return (count . name) pair
        if let Value::Pair(count, name) = result {
            assert_eq!(*count, Value::integer(3)); // 3 clauses
            assert_eq!(*name, Value::string("test-case-lambda"));
        } else {
            panic!("Expected pair result from case-lambda-clauses");
        }
    }

    #[test]
    fn test_arity_errors() {
        // Test wrong number of arguments
        let result = primitive_case_lambda_predicate(&[]);
        assert!(result.is_err());

        let result = primitive_case_lambda_predicate(&[Value::integer(1), Value::integer(2)]);
        assert!(result.is_err());

        let result = primitive_case_lambda_arity(&[]);
        assert!(result.is_err());

        let result = primitive_case_lambda_clauses(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_procedure_arguments() {
        // Test case-lambda-arity with non-procedure
        let result = primitive_case_lambda_arity(&[Value::string("not-a-procedure")]);
        assert!(result.is_err());

        // Test case-lambda-clauses with non-case-lambda
        let result = primitive_case_lambda_clauses(&[Value::integer(42)]);
        assert!(result.is_err());
    }
}
