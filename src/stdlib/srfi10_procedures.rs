//! SRFI-10 runtime procedures implementation.
//!
//! This module implements the runtime procedures for SRFI-10 external forms:
//! - `define-reader-ctor`: Register a constructor procedure for a tag
//! - `reader-ctor?`: Check if a tag has a registered constructor  
//! - `reader-ctor-ref`: Get the constructor procedure for a tag
//!
//! These procedures provide the Scheme-level interface to the external
//! form registry.

use crate::eval::{Environment, Value};
use crate::stdlib::srfi10_external_forms::global_registry;
use crate::utils::{SymbolId, intern_symbol};
use std::rc::Rc;

/// Implement `define-reader-ctor` procedure.
///
/// # Syntax
/// ```scheme
/// (define-reader-ctor tag constructor-procedure)
/// ```
///
/// # Arguments
/// - `tag`: Symbol identifying the external form constructor
/// - `constructor-procedure`: Procedure to call when the external form is encountered
///
/// # Returns
/// Unspecified value (typically #<unspecified>).
///
/// # Errors
/// - If tag is not a symbol
/// - If constructor-procedure is not a procedure
/// - If registry operations fail
pub fn define_reader_ctor(args: &[Value], _env: &Environment) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "define-reader-ctor: expected 2 arguments, got {}",
            args.len()
        ));
    }

    // Extract and validate tag argument
    let tag_symbol = match &args[0] {
        Value::Symbol(name) => intern_symbol(name),
        Value::Identifier(name) => intern_symbol(name),
        _ => {
            return Err(format!(
                "define-reader-ctor: tag must be a symbol, got: {:?}",
                args[0]
            ));
        }
    };

    // Extract and validate constructor argument
    let constructor = args[1].clone();
    match &constructor {
        Value::Primitive { .. } | Value::Procedure { .. } | Value::CaseLambda { .. } => {
            // Valid procedure types
        }
        _ => {
            return Err(format!(
                "define-reader-ctor: constructor must be a procedure, got: {:?}",
                constructor
            ));
        }
    }

    // Register the constructor
    let registry = global_registry();
    registry.define_constructor(tag_symbol, constructor)?;

    // Return unspecified value
    Ok(Value::Unspecified)
}

/// Implement `reader-ctor?` procedure.
///
/// # Syntax
/// ```scheme
/// (reader-ctor? tag)
/// ```
///
/// # Arguments  
/// - `tag`: Symbol to check for registered constructor
///
/// # Returns
/// `#t` if the tag has a registered constructor, `#f` otherwise.
///
/// # Errors
/// - If tag is not a symbol
/// - If registry operations fail
pub fn reader_ctor_predicate(args: &[Value], _env: &Environment) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "reader-ctor?: expected 1 argument, got {}",
            args.len()
        ));
    }

    // Extract and validate tag argument
    let tag_symbol = match &args[0] {
        Value::Symbol(name) => intern_symbol(name),
        Value::Identifier(name) => intern_symbol(name),
        _ => {
            return Err(format!(
                "reader-ctor?: tag must be a symbol, got: {:?}",
                args[0]
            ));
        }
    };

    // Check if constructor is registered
    let registry = global_registry();
    let has_constructor = registry.has_constructor(tag_symbol)?;

    Ok(Value::Boolean(has_constructor))
}

/// Implement `reader-ctor-ref` procedure.
///
/// # Syntax
/// ```scheme
/// (reader-ctor-ref tag)
/// ```
///
/// # Arguments
/// - `tag`: Symbol to look up constructor for
///
/// # Returns
/// The constructor procedure if registered.
///
/// # Errors
/// - If tag is not a symbol
/// - If no constructor is registered for the tag
/// - If registry operations fail
pub fn reader_ctor_ref(args: &[Value], _env: &Environment) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "reader-ctor-ref: expected 1 argument, got {}",
            args.len()
        ));
    }

    // Extract and validate tag argument
    let tag_symbol = match &args[0] {
        Value::Symbol(name) => intern_symbol(name),
        Value::Identifier(name) => intern_symbol(name),
        _ => {
            return Err(format!(
                "reader-ctor-ref: tag must be a symbol, got: {:?}",
                args[0]
            ));
        }
    };

    // Look up the constructor
    let registry = global_registry();
    match registry.get_constructor(tag_symbol)? {
        Some(constructor) => Ok(constructor),
        None => Err(format!(
            "reader-ctor-ref: no constructor registered for tag '{:?}'",
            tag_symbol
        )),
    }
}

/// Register all SRFI-10 procedures in an environment.
///
/// This function adds all the SRFI-10 procedures to the given environment
/// so they can be called from Scheme code.
///
/// # Arguments
/// - `env`: Environment to register procedures in
pub fn register_srfi10_procedures(env: &mut Environment) -> Result<(), String> {
    // Register define-reader-ctor
    env.define(
        "define-reader-ctor".to_string(),
        Value::Primitive {
            name: "define-reader-ctor".to_string(),
            arity: 2,
            pointer: define_reader_ctor as *const (),
        },
    )?;

    // Register reader-ctor?
    env.define(
        "reader-ctor?".to_string(),
        Value::Primitive {
            name: "reader-ctor?".to_string(),
            arity: 1,
            pointer: reader_ctor_predicate as *const (),
        },
    )?;

    // Register reader-ctor-ref
    env.define(
        "reader-ctor-ref".to_string(),
        Value::Primitive {
            name: "reader-ctor-ref".to_string(),
            arity: 1,
            pointer: reader_ctor_ref as *const (),
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{Environment, Value};

    fn create_test_env() -> Environment {
        let mut env = Environment::new();
        register_srfi10_procedures(&mut env).unwrap();
        env
    }

    #[test]
    fn test_define_reader_ctor_basic() {
        let env = create_test_env();
        let tag = Value::Symbol("test-tag".to_string());
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 2,
            pointer: 0 as *const (),
        };

        let args = vec![tag, constructor];
        let result = define_reader_ctor(&args, &env);
        
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Unspecified));
    }

    #[test]
    fn test_define_reader_ctor_invalid_args() {
        let env = create_test_env();

        // Test wrong number of arguments
        let result = define_reader_ctor(&[Value::integer(42)], &env);
        assert!(result.is_err());

        // Test invalid tag type
        let invalid_tag = Value::integer(42);
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 1,
            pointer: 0 as *const (),
        };
        let result = define_reader_ctor(&[invalid_tag, constructor], &env);
        assert!(result.is_err());

        // Test invalid constructor type
        let tag = Value::Symbol("test-tag".to_string());
        let invalid_constructor = Value::integer(42);
        let result = define_reader_ctor(&[tag, invalid_constructor], &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_reader_ctor_predicate() {
        let env = create_test_env();
        let tag = Value::Symbol("test-tag".to_string());

        // Initially should return false
        let result = reader_ctor_predicate(&[tag.clone()], &env);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(false)));

        // Register a constructor
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 1,
            pointer: 0 as *const (),
        };
        define_reader_ctor(&[tag.clone(), constructor], &env).unwrap();

        // Now should return true
        let result = reader_ctor_predicate(&[tag], &env);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Boolean(true)));
    }

    #[test]
    fn test_reader_ctor_ref() {
        let env = create_test_env();
        let tag = Value::Symbol("test-tag".to_string());
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 1,
            pointer: 0 as *const (),
        };

        // Initially should fail
        let result = reader_ctor_ref(&[tag.clone()], &env);
        assert!(result.is_err());

        // Register a constructor
        define_reader_ctor(&[tag.clone(), constructor.clone()], &env).unwrap();

        // Now should return the constructor
        let result = reader_ctor_ref(&[tag], &env);
        assert!(result.is_ok());
        // Note: exact equality check would require implementing PartialEq for Value
    }

    #[test]
    fn test_register_procedures() {
        let mut env = Environment::new();
        assert!(register_srfi10_procedures(&mut env).is_ok());

        // Check that all procedures were registered
        assert!(env.lookup("define-reader-ctor").is_some());
        assert!(env.lookup("reader-ctor?").is_some());
        assert!(env.lookup("reader-ctor-ref").is_some());
    }
}