//! Type operations for the Lambdust standard library.
//!
//! This module implements Lambdust-specific type operations including
//! type checking, type manipulation, and gradual typing support.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Creates type operation bindings for the standard library.
pub fn create_type_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Type queries
    bind_type_queries(env);
    
    // Type operations
    bind_type_operations(env);
    
    // Type checking
    bind_type_checking(env);
    
    // Gradual typing support
    bind_gradual_typing(env);
}

/// Binds type query operations.
fn bind_type_queries(env: &Arc<ThreadSafeEnvironment>) {
    // type-of
    env.define("type-of".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-of".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_type_of),
        effects: vec![Effect::Pure],
    })));
    
    // type?
    env.define("type?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_type_p),
        effects: vec![Effect::Pure],
    })));
    
    // type-name
    env.define("type-name".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-name".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_type_name),
        effects: vec![Effect::Pure],
    })));
}

/// Binds type operation functions.
fn bind_type_operations(env: &Arc<ThreadSafeEnvironment>) {
    // type-union
    env.define("type-union".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-union".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_type_union),
        effects: vec![Effect::Pure],
    })));
    
    // type-intersection
    env.define("type-intersection".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-intersection".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_type_intersection),
        effects: vec![Effect::Pure],
    })));
    
    // type-difference
    env.define("type-difference".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-difference".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_type_difference),
        effects: vec![Effect::Pure],
    })));
    
    // subtype?
    env.define("subtype?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "subtype?".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_subtype_p),
        effects: vec![Effect::Pure],
    })));
    
    // type-equivalent?
    env.define("type-equivalent?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-equivalent?".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_type_equivalent_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds type checking operations.
fn bind_type_checking(env: &Arc<ThreadSafeEnvironment>) {
    // type-check
    env.define("type-check".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-check".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_type_check),
        effects: vec![Effect::Pure],
    })));
    
    // type-assert
    env.define("type-assert".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-assert".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_type_assert),
        effects: vec![Effect::Error], // Can throw type errors
    })));
    
    // type-cast
    env.define("type-cast".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "type-cast".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_type_cast),
        effects: vec![Effect::Pure],
    })));
}

/// Binds gradual typing support.
fn bind_gradual_typing(env: &Arc<ThreadSafeEnvironment>) {
    // any-type
    env.define("any-type".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "any-type".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_any_type),
        effects: vec![Effect::Pure],
    })));
    
    // unknown-type
    env.define("unknown-type".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unknown-type".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_unknown_type),
        effects: vec![Effect::Pure],
    })));
    
    // make-function-type
    env.define("make-function-type".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-function-type".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_function_type),
        effects: vec![Effect::Pure],
    })));
    
    // function-type?
    env.define("function-type?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "function-type?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_function_type_p),
        effects: vec![Effect::Pure],
    })));
    
    // function-parameter-types
    env.define("function-parameter-types".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "function-parameter-types".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_function_parameter_types),
        effects: vec![Effect::Pure],
    })));
    
    // function-return-type
    env.define("function-return-type".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "function-return-type".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_function_return_type),
        effects: vec![Effect::Pure],
    })));
}

// ============= IMPLEMENTATIONS =============

/// type-of procedure
fn primitive_type_of(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("type-of expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let type_name = get_value_type_name(&args[0]);
    Ok(Value::string(type_name))
}

/// type? predicate
fn primitive_type_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("type? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    // Check if the value is a type representation
    let is_type = matches!(args[0], Value::Type(_));
    Ok(Value::boolean(is_type))
}

/// type-name procedure
fn primitive_type_name(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("type-name expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Type(type_val) => {
            let name = match type_val.as_ref() {
                crate::eval::value::TypeValue::Base(name) => name.clone()),
                crate::eval::value::TypeValue::Function { .. } => "function".to_string(),
                crate::eval::value::TypeValue::Union(_) => "union".to_string(),
                crate::eval::value::TypeValue::Intersection(_) => "intersection".to_string(),
                crate::eval::value::TypeValue::Variable(name) => name.clone()),
            };
            Ok(Value::string(name))
        }
        _ => Err(DiagnosticError::runtime_error(
            "type-name requires a type argument".to_string(),
            None,
        )),
    }
}

/// type-union procedure
fn primitive_type_union(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "type-union requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// type-intersection procedure
fn primitive_type_intersection(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "type-intersection requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// type-difference procedure
fn primitive_type_difference(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "type-difference requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// subtype? predicate
fn primitive_subtype_p(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "subtype? requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// type-equivalent? predicate
fn primitive_type_equivalent_p(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "type-equivalent? requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// type-check procedure
fn primitive_type_check(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("type-check expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // For now, just return whether the value matches the basic type
    let value = &args[0];
    let expected_type = args[1].as_string().unwrap_or("unknown");
    let actual_type = get_value_type_name(value);
    
    Ok(Value::boolean(actual_type == expected_type))
}

/// type-assert procedure
fn primitive_type_assert(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("type-assert expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let type_check_result = primitive_type_check(args)?;
    
    if type_check_result.is_truthy() {
        Ok(args[0].clone())
    } else {
        Err(DiagnosticError::runtime_error(
            format!("Type assertion failed: expected {}, got {}", 
                    args[1], get_value_type_name(&args[0])),
            None,
        ))
    }
}

/// type-cast procedure
fn primitive_type_cast(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("type-cast expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // For now, just return the value unchanged
    // In a full implementation, this would perform type conversion
    Ok(args[0].clone())
}

/// any-type procedure
fn primitive_any_type(_args: &[Value]) -> Result<Value> {
    // Return a representation of the "any" type
    Ok(Value::Type(Arc::new(crate::eval::value::TypeValue::Base("any".to_string()))))
}

/// unknown-type procedure
fn primitive_unknown_type(_args: &[Value]) -> Result<Value> {
    // Return a representation of the "unknown" type
    Ok(Value::Type(Arc::new(crate::eval::value::TypeValue::Base("unknown".to_string()))))
}

/// make-function-type procedure
fn primitive_make_function_type(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("make-function-type expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "make-function-type requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// function-type? predicate
fn primitive_function_type_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("function-type? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Type(type_val) => {
            let is_function = matches!(type_val.as_ref(), crate::eval::value::TypeValue::Function { .. });
            Ok(Value::boolean(is_function))
        }
        _ => Ok(Value::boolean(false)),
    }
}

/// function-parameter-types procedure
fn primitive_function_parameter_types(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "function-parameter-types requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

/// function-return-type procedure
fn primitive_function_return_type(_args: &[Value]) -> Result<Value> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "function-return-type requires type system integration (not yet implemented)".to_string(),
        None,
    ))
}

// ============= HELPER FUNCTIONS =============

/// Gets the type name of a value.
fn get_value_type_name(value: &Value) -> String {
    match value {
        Value::Literal(lit) => match lit {
            crate::ast::Literal::Number(_) => "number".to_string(),
            crate::ast::Literal::Rational { .. } => "rational".to_string(),
            crate::ast::Literal::Complex { .. } => "complex".to_string(),
            crate::ast::Literal::String(_) => "string".to_string(),
            crate::ast::Literal::Character(_) => "character".to_string(),
            crate::ast::Literal::Boolean(_) => "boolean".to_string(),
            crate::ast::Literal::Bytevector(_) => "bytevector".to_string(),
            crate::ast::Literal::Nil => "null".to_string(),
            crate::ast::Literal::Unspecified => "unspecified".to_string(),
        },
        Value::Symbol(_) => "symbol".to_string(),
        Value::Keyword(_) => "keyword".to_string(),
        Value::Nil => "null".to_string(),
        Value::Unspecified => "unspecified".to_string(),
        Value::Pair(_, _) => "pair".to_string(),
        Value::Vector(_) => "vector".to_string(),
        Value::Hashtable(_) => "hashtable".to_string(),
        Value::Procedure(_) => "procedure".to_string(),
        Value::Primitive(_) => "primitive".to_string(),
        Value::Continuation(_) => "continuation".to_string(),
        Value::Syntax(_) => "syntax".to_string(),
        Value::Port(_) => "port".to_string(),
        Value::Promise(_) => "promise".to_string(),
        Value::Type(_) => "type".to_string(),
        Value::Foreign(obj) => obj.type_name.clone()),
        Value::ErrorObject(_) => "error".to_string(),
        Value::CharSet(_) => "char-set".to_string(),
        Value::Parameter(_) => "parameter".to_string(),
        Value::CaseLambda(_) => "procedure".to_string(), // case-lambda is a type of procedure
        Value::Record(_) => "record".to_string(),
        // Advanced container types
        Value::AdvancedHashTable(_) => "advanced-hash-table".to_string(),
        Value::Ideque(_) => "ideque".to_string(),
        Value::PriorityQueue(_) => "priority-queue".to_string(),
        Value::OrderedSet(_) => "ordered-set".to_string(),
        Value::ListQueue(_) => "list-queue".to_string(),
        Value::RandomAccessList(_) => "random-access-list".to_string(),
        // Concurrency types
        Value::Future(_) => "future".to_string(),
        Value::Channel(_) => "channel".to_string(),
        Value::Mutex(_) => "mutex".to_string(),
        Value::Semaphore(_) => "semaphore".to_string(),
        Value::AtomicCounter(_) => "atomic-counter".to_string(),
        Value::DistributedNode(_) => "distributed-node".to_string(),
        Value::Opaque(_) => "opaque".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_of() {
        let args = vec![Value::integer(42)];
        let result = primitive_type_of(&args).unwrap();
        assert_eq!(result, Value::string("number"));
        
        let args = vec![Value::string("hello")];
        let result = primitive_type_of(&args).unwrap();
        assert_eq!(result, Value::string("string"));
        
        let args = vec![Value::boolean(true)];
        let result = primitive_type_of(&args).unwrap();
        assert_eq!(result, Value::string("boolean"));
    }
    
    #[test]
    fn test_type_predicate() {
        let not_type = Value::integer(42);
        let result = primitive_type_p(&[not_type]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        let type_val = Value::Type(Arc::new(crate::eval::value::TypeValue::Base("number".to_string())));
        let result = primitive_type_p(&[type_val]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
    
    #[test]
    fn test_type_check() {
        let args = vec![Value::integer(42), Value::string("number")];
        let result = primitive_type_check(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::string("hello"), Value::string("number")];
        let result = primitive_type_check(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_type_assert() {
        let args = vec![Value::integer(42), Value::string("number")];
        let result = primitive_type_assert(&args).unwrap();
        assert_eq!(result, Value::integer(42));
        
        let args = vec![Value::string("hello"), Value::string("number")];
        let result = primitive_type_assert(&args);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_any_type() {
        let result = primitive_any_type(&[]).unwrap();
        assert!(matches!(result, Value::Type(_)));
    }
}