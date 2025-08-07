//! SRFI-9 `define-record-type` macro implementation.
//!
//! This module provides the `define-record-type` macro that expands to
//! the appropriate primitive record operations and procedure definitions.

use crate::ast::Literal;
use crate::diagnostics::{Error, Result};
use crate::eval::value::{Value, ThreadSafeEnvironment};
use crate::macro_system::SyntaxRulesTransformer;
use crate::utils::symbol::intern_symbol;
use std::sync::Arc;

/// Installs the `define-record-type` macro into the given environment.
pub fn install_define_record_type_macro(env: &Arc<ThreadSafeEnvironment>) {
    // Create a simple macro transformer that handles define-record-type
    // For now, we'll use a procedural approach since syntax-rules might be complex
    
    // The macro will be installed as a special form that gets handled during evaluation
    // This is a simplified approach - in a full implementation, this would use syntax-rules
    
    // For demonstration, let's create a basic implementation
    env.define("define-record-type".to_string(), Value::Primitive(Arc::new(
        crate::eval::value::PrimitiveProcedure {
            name: "define-record-type".to_string(),
            arity_min: 3,
            arity_max: None,
            implementation: crate::eval::value::PrimitiveImpl::RustFn(expand_define_record_type),
            effects: vec![crate::effects::Effect::State],
        }
    )));
}

/// Expands a define-record-type form.
/// 
/// This is a simplified implementation that handles the basic case:
/// ```scheme
/// (define-record-type <type-name>
///   (constructor field1 field2 ...)
///   predicate
///   (field1 accessor1 [mutator1])
///   (field2 accessor2 [mutator2])
///   ...)
/// ```
fn expand_define_record_type(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(Error::runtime_error(
            "define-record-type requires at least 3 arguments".to_string(),
            None,
        ));
    }
    
    // Extract type name
    let type_name = extract_type_name(&args[0])?;
    
    // Extract constructor specification
    let (constructor_name, constructor_fields) = extract_constructor_spec(&args[1])?;
    
    // Extract predicate name
    let predicate_name = extract_predicate_name(&args[2])?;
    
    // Extract field specifications
    let field_specs = extract_field_specs(&args[3..])?;
    
    // Generate the expansion
    generate_record_type_expansion(
        type_name,
        constructor_name,
        constructor_fields,
        predicate_name,
        field_specs,
    )
}

/// Extracts the type name from a define-record-type form.
fn extract_type_name(value: &Value) -> Result<String> {
    match value {
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                Ok(name)
            } else {
                Err(Box::new(Error::runtime_error(
                    "Invalid symbol for record type name".to_string(),
                    None,
                ))
            }
        }
        Value::Literal(Literal::String(name)) => Ok(name.clone()),
        _ => Err(Box::new(Error::runtime_error(
            "Record type name must be a symbol or string".to_string(),
            None,
        )),
    }
}

/// Extracts constructor specification from a define-record-type form.
fn extract_constructor_spec(value: &Value) -> Result<(String, Vec<String>)> {
    let constructor_list = value.as_list().ok_or_else(|| {
        Error::runtime_error(
            "Constructor specification must be a list".to_string(),
            None,
        )
    })?;
    
    if constructor_list.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "Constructor specification cannot be empty".to_string(),
            None,
        ));
    }
    
    // First element is constructor name
    let constructor_name = match &constructor_list[0] {
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                name
            } else {
                return Err(Box::new(Error::runtime_error(
                    "Invalid symbol for constructor name".to_string(),
                    None,
                ));
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "Constructor name must be a symbol".to_string(),
            None,
        )),
    };
    
    // Remaining elements are field names
    let mut field_names = Vec::new();
    for field in &constructor_list[1..] {
        match field {
            Value::Symbol(sym_id) => {
                if let Some(name) = crate::utils::symbol_name(*sym_id) {
                    field_names.push(name);
                } else {
                    return Err(Box::new(Error::runtime_error(
                        "Invalid symbol for field name".to_string(),
                        None,
                    ));
                }
            }
            _ => return Err(Box::new(Error::runtime_error(
                "Field names must be symbols".to_string(),
                None,
            )),
        }
    }
    
    Ok((constructor_name, field_names))
}

/// Extracts predicate name from a define-record-type form.
fn extract_predicate_name(value: &Value) -> Result<String> {
    match value {
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                Ok(name)
            } else {
                Err(Box::new(Error::runtime_error(
                    "Invalid symbol for predicate name".to_string(),
                    None,
                ))
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "Predicate name must be a symbol".to_string(),
            None,
        )),
    }
}

/// Field specification from a define-record-type form.
#[derive(Debug, Clone)]
struct FieldSpec {
    name: String,
    accessor: String,
    mutator: Option<String>,
}

/// Extracts field specifications from a define-record-type form.
fn extract_field_specs(values: &[Value]) -> Result<Vec<FieldSpec>> {
    let mut field_specs = Vec::new();
    
    for value in values {
        let field_list = value.as_list().ok_or_else(|| {
            Error::runtime_error(
                "Field specification must be a list".to_string(),
                None,
            )
        })?;
        
        if field_list.len() < 2 || field_list.len() > 3 {
            return Err(Box::new(Error::runtime_error(
                "Field specification must have 2 or 3 elements: (field accessor [mutator])".to_string(),
                None,
            ));
        }
        
        // Extract field name
        let field_name = match &field_list[0] {
            Value::Symbol(sym_id) => {
                if let Some(name) = crate::utils::symbol_name(*sym_id) {
                    name
                } else {
                    return Err(Box::new(Error::runtime_error(
                        "Invalid symbol for field name".to_string(),
                        None,
                    ));
                }
            }
            _ => return Err(Box::new(Error::runtime_error(
                "Field name must be a symbol".to_string(),
                None,
            )),
        };
        
        // Extract accessor name
        let accessor_name = match &field_list[1] {
            Value::Symbol(sym_id) => {
                if let Some(name) = crate::utils::symbol_name(*sym_id) {
                    name
                } else {
                    return Err(Box::new(Error::runtime_error(
                        "Invalid symbol for accessor name".to_string(),
                        None,
                    ));
                }
            }
            _ => return Err(Box::new(Error::runtime_error(
                "Accessor name must be a symbol".to_string(),
                None,
            )),
        };
        
        // Extract optional mutator name
        let mutator_name = if field_list.len() == 3 {
            match &field_list[2] {
                Value::Symbol(sym_id) => {
                    if let Some(name) = crate::utils::symbol_name(*sym_id) {
                        Some(name)
                    } else {
                        return Err(Box::new(Error::runtime_error(
                            "Invalid symbol for mutator name".to_string(),
                            None,
                        ));
                    }
                }
                _ => return Err(Box::new(Error::runtime_error(
                    "Mutator name must be a symbol".to_string(),
                    None,
                )),
            }
        } else {
            None
        };
        
        field_specs.push(FieldSpec {
            name: field_name,
            accessor: accessor_name,
            mutator: mutator_name,
        });
    }
    
    Ok(field_specs)
}

/// Generates the expanded code for a define-record-type form.
fn generate_record_type_expansion(
    type_name: String,
    constructor_name: String,
    constructor_fields: Vec<String>,
    predicate_name: String,
    field_specs: Vec<FieldSpec>,
) -> Result<Value> {
    // This is a simplified implementation that returns a begin form
    // In a real implementation, this would generate proper Scheme code
    
    // For now, we'll create a simple procedure that sets up the record type
    // This is a placeholder - the actual implementation would need to integrate
    // with the macro system properly
    
    // Create a quoted list representing the expanded form
    let expansion = Value::list(vec![
        Value::symbol(intern_symbol("begin".to_string())),
        
        // Create the record type
        Value::list(vec![
            Value::symbol(intern_symbol("define".to_string())),
            Value::symbol(intern_symbol(format!("%{}-type", type_name))),
            Value::list(vec![
                Value::symbol(intern_symbol("make-record-type".to_string())),
                Value::string(&type_name),
                Value::list(constructor_fields.iter()
                    .map(|name| Value::string(name))
                    .collect()),
            ]),
        ]),
        
        // Create the constructor
        Value::list(vec![
            Value::symbol(intern_symbol("define".to_string())),
            Value::symbol(intern_symbol(constructor_name)),
            Value::list(vec![
                Value::symbol(intern_symbol("record-constructor".to_string())),
                Value::symbol(intern_symbol(format!("%{}-type", type_name))),
            ]),
        ]),
        
        // Create the predicate
        Value::list(vec![
            Value::symbol(intern_symbol("define".to_string())),
            Value::symbol(intern_symbol(predicate_name)),
            Value::list(vec![
                Value::symbol(intern_symbol("record-predicate".to_string())),
                Value::symbol(intern_symbol(format!("%{}-type", type_name))),
            ]),
        ]),
    ]);
    
    // Add field accessors and mutators
    let mut expanded_forms = expansion.as_list().unwrap_or_default();
    
    for field_spec in field_specs {
        // Add accessor
        expanded_forms.push(Value::list(vec![
            Value::symbol(intern_symbol("define".to_string())),
            Value::symbol(intern_symbol(field_spec.accessor)),
            Value::list(vec![
                Value::symbol(intern_symbol("record-accessor".to_string())),
                Value::symbol(intern_symbol(format!("%{}-type", type_name))),
                Value::string(&field_spec.name),
            ]),
        ]));
        
        // Add mutator if specified
        if let Some(mutator_name) = field_spec.mutator {
            expanded_forms.push(Value::list(vec![
                Value::symbol(intern_symbol("define".to_string())),
                Value::symbol(intern_symbol(mutator_name)),
                Value::list(vec![
                    Value::symbol(intern_symbol("record-mutator".to_string())),
                    Value::symbol(intern_symbol(format!("%{}-type", type_name))),
                    Value::string(&field_spec.name),
                ]),
            ]));
        }
    }
    
    Ok(Value::list(expanded_forms))
}

/// Creates a syntax-rules based implementation of define-record-type.
/// This is the proper way to implement it, but requires more macro system integration.
pub fn create_define_record_type_syntax_rules() -> SyntaxRulesTransformer {
    // This would be a proper syntax-rules implementation
    // For now, we use the procedural approach above
    todo!("Full syntax-rules implementation needs more macro system work")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::symbol::intern_symbol;

    #[test]
    fn test_type_name_extraction() {
        let symbol_val = Value::symbol(intern_symbol("point".to_string()));
        let result = extract_type_name(&symbol_val).unwrap();
        assert_eq!(result, "point");
        
        let string_val = Value::string("person");
        let result = extract_type_name(&string_val).unwrap();
        assert_eq!(result, "person");
    }
    
    #[test]
    fn test_constructor_spec_extraction() {
        let constructor_spec = Value::list(vec![
            Value::symbol(intern_symbol("make-point".to_string())),
            Value::symbol(intern_symbol("x".to_string())),
            Value::symbol(intern_symbol("y".to_string())),
        ]);
        
        let (name, fields) = extract_constructor_spec(&constructor_spec).unwrap();
        assert_eq!(name, "make-point");
        assert_eq!(fields, vec!["x", "y"]);
    }
    
    #[test]
    fn test_predicate_name_extraction() {
        let predicate_val = Value::symbol(intern_symbol("point?".to_string()));
        let result = extract_predicate_name(&predicate_val).unwrap();
        assert_eq!(result, "point?");
    }
    
    #[test]
    fn test_field_specs_extraction() {
        let field_specs = vec![
            Value::list(vec![
                Value::symbol(intern_symbol("x".to_string())),
                Value::symbol(intern_symbol("point-x".to_string())),
                Value::symbol(intern_symbol("point-x-set!".to_string())),
            ]),
            Value::list(vec![
                Value::symbol(intern_symbol("y".to_string())),
                Value::symbol(intern_symbol("point-y".to_string())),
            ]),
        ];
        
        let specs = extract_field_specs(&field_specs).unwrap();
        assert_eq!(specs.len(), 2);
        
        assert_eq!(specs[0].name, "x");
        assert_eq!(specs[0].accessor, "point-x");
        assert_eq!(specs[0].mutator, Some("point-x-set!".to_string()));
        
        assert_eq!(specs[1].name, "y");
        assert_eq!(specs[1].accessor, "point-y");
        assert_eq!(specs[1].mutator, None);
    }
}