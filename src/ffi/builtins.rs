//! Built-in FFI functions for common operations.
//!
//! This module provides a comprehensive set of FFI functions that are
//! automatically registered with the FFI system, covering arithmetic,
//! string manipulation, list operations, type checking, and I/O.

#![allow(missing_docs)]

use super::*;
use crate::eval::Value;

// ============= ARITHMETIC FUNCTIONS =============

pub struct AddFunction;

impl FfiFunction for AddFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "add".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Adds two numbers together.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        Ok((a + b).to_lambdust())
    }
}

pub struct SubtractFunction;

impl FfiFunction for SubtractFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "subtract".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Subtracts the second number from the first.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        Ok((a - b).to_lambdust())
    }
}

pub struct MultiplyFunction;

impl FfiFunction for MultiplyFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "multiply".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Multiplies two numbers.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        Ok((a * b).to_lambdust())
    }
}

pub struct DivideFunction;

impl FfiFunction for DivideFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "divide".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["number".to_string(), "number".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Divides the first number by the second.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let a = f64::from_lambdust(&args[0])?;
        let b = f64::from_lambdust(&args[1])?;
        if b == 0.0 {
            Ok(f64::INFINITY.to_lambdust())
        } else {
            Ok((a / b).to_lambdust())
        }
    }
}

// ============= STRING FUNCTIONS =============

pub struct StringLengthFunction;

impl FfiFunction for StringLengthFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-length".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["string".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Returns the length of a string.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let s = String::from_lambdust(&args[0])?;
        Ok((s.chars().count() as i64).to_lambdust())
    }
}

pub struct StringConcatFunction;

impl FfiFunction for StringConcatFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-concat".to_string(),
            arity: AritySpec::AtLeast(0),
            parameter_types: vec!["string".to_string()],
            return_type: "string".to_string(),
            documentation: Some("Concatenates multiple strings together.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let mut result = String::new();
        for arg in args {
            if let Some(s) = arg.as_string() {
                result.push_str(s);
            }
        }
        Ok(result.to_lambdust())
    }
}

pub struct StringUpperFunction;

impl FfiFunction for StringUpperFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-upper".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["string".to_string()],
            return_type: "string".to_string(),
            documentation: Some("Converts a string to uppercase.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let s = String::from_lambdust(&args[0])?;
        Ok(s.to_uppercase().to_lambdust())
    }
}

pub struct StringLowerFunction;

impl FfiFunction for StringLowerFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string-lower".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["string".to_string()],
            return_type: "string".to_string(),
            documentation: Some("Converts a string to lowercase.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let s = String::from_lambdust(&args[0])?;
        Ok(s.to_lowercase().to_lambdust())
    }
}

// ============= LIST FUNCTIONS =============

pub struct ListLengthFunction;

impl FfiFunction for ListLengthFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list-length".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["list".to_string()],
            return_type: "number".to_string(),
            documentation: Some("Returns the length of a list.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let list = Vec::<Value>::from_lambdust(&args[0])?;
        Ok((list.len() as i64).to_lambdust())
    }
}

pub struct ListMapFunction;

impl FfiFunction for ListMapFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list-map".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["procedure".to_string(), "list".to_string()],
            return_type: "list".to_string(),
            documentation: Some("Maps a function over a list.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        // For now, return the list unchanged
        // In a real implementation, this would apply the function to each element
        let list = Vec::<Value>::from_lambdust(&args[1])?;
        Ok(list.to_lambdust())
    }
}

pub struct ListFilterFunction;

impl FfiFunction for ListFilterFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list-filter".to_string(),
            arity: AritySpec::Exact(2),
            parameter_types: vec!["procedure".to_string(), "list".to_string()],
            return_type: "list".to_string(),
            documentation: Some("Filters a list using a predicate function.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        // For now, return the list unchanged
        // In a real implementation, this would filter elements based on the predicate
        let list = Vec::<Value>::from_lambdust(&args[1])?;
        Ok(list.to_lambdust())
    }
}

// ============= TYPE CHECKING FUNCTIONS =============

pub struct IsNumberFunction;

impl FfiFunction for IsNumberFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "number?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a number.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        Ok(args[0].is_number().to_lambdust())
    }
}

pub struct IsStringFunction;

impl FfiFunction for IsStringFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "string?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a string.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        Ok(args[0].is_string().to_lambdust())
    }
}

pub struct IsListFunction;

impl FfiFunction for IsListFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "list?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a list.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        Ok(args[0].is_list().to_lambdust())
    }
}

pub struct IsBooleanFunction;

impl FfiFunction for IsBooleanFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "boolean?".to_string(),
            arity: AritySpec::Exact(1),
            parameter_types: vec!["any".to_string()],
            return_type: "boolean".to_string(),
            documentation: Some("Checks if a value is a boolean.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        let is_bool = matches!(args[0], Value::Literal(crate::ast::Literal::Boolean(_)));
        Ok(is_bool.to_lambdust())
    }
}

// ============= I/O FUNCTIONS =============

pub struct PrintFunction;

impl FfiFunction for PrintFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "print".to_string(),
            arity: AritySpec::AtLeast(0),
            parameter_types: vec!["any".to_string()],
            return_type: "unspecified".to_string(),
            documentation: Some("Prints values to standard output.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{arg}");
        }
        Ok(Value::Unspecified)
    }
}

pub struct PrintlnFunction;

impl FfiFunction for PrintlnFunction {
    fn signature(&self) -> &FfiSignature {
        use std::sync::OnceLock;
        static SIGNATURE: OnceLock<FfiSignature> = OnceLock::new();
        SIGNATURE.get_or_init(|| FfiSignature {
            name: "println".to_string(),
            arity: AritySpec::AtLeast(0),
            parameter_types: vec!["any".to_string()],
            return_type: "unspecified".to_string(),
            documentation: Some("Prints values to standard output followed by a newline.".to_string()),
        })
    }
    
    fn call(&self, args: &[Value]) -> std::result::Result<Value, FfiError> {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{arg}");
        }
        println!();
        Ok(Value::Unspecified)
    }
}

/// Module for registering all built-in FFI functions.
pub struct BuiltinFfiModule;

impl FfiModule for BuiltinFfiModule {
    fn register_functions(registry: &FfiRegistry) -> std::result::Result<(), FfiError> {
        // Arithmetic functions
        registry.register(AddFunction)?;
        registry.register(SubtractFunction)?;
        registry.register(MultiplyFunction)?;
        registry.register(DivideFunction)?;
        
        // String functions
        registry.register(StringLengthFunction)?;
        registry.register(StringConcatFunction)?;
        registry.register(StringUpperFunction)?;
        registry.register(StringLowerFunction)?;
        
        // List functions
        registry.register(ListLengthFunction)?;
        registry.register(ListMapFunction)?;
        registry.register(ListFilterFunction)?;
        
        // Type checking functions
        registry.register(IsNumberFunction)?;
        registry.register(IsStringFunction)?;
        registry.register(IsListFunction)?;
        registry.register(IsBooleanFunction)?;
        
        // I/O functions
        registry.register(PrintFunction)?;
        registry.register(PrintlnFunction)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_functions() {
        let add_func = AddFunction;
        let args = vec![Value::number(2.0), Value::number(3.0)];
        let result = add_func.call(&args).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);
        
        let div_func = DivideFunction;
        let args = vec![Value::number(10.0), Value::number(2.0)];
        let result = div_func.call(&args).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_string_functions() {
        let len_func = StringLengthFunction;
        let args = vec![Value::string("hello")];
        let result = len_func.call(&args).unwrap();
        assert_eq!(result.as_integer().unwrap(), 5);
        
        let upper_func = StringUpperFunction;
        let args = vec![Value::string("hello")];
        let result = upper_func.call(&args).unwrap();
        assert_eq!(result.as_string().unwrap(), "HELLO");
    }

    #[test]
    fn test_type_checking_functions() {
        let is_num_func = IsNumberFunction;
        let args = vec![Value::number(42.0)];
        let result = is_num_func.call(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::string("hello")];
        let result = is_num_func.call(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_list_functions() {
        let len_func = ListLengthFunction;
        let list = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let args = vec![Value::list(list)];
        let result = len_func.call(&args).unwrap();
        assert_eq!(result.as_integer().unwrap(), 3);
    }

    #[test]
    fn test_builtin_module_registration() {
        let registry = FfiRegistry::new();
        assert!(BuiltinFfiModule::register_functions(&registry).is_ok());
        
        let functions = registry.list_functions();
        assert!(functions.contains(&"add".to_string()));
        assert!(functions.contains(&"string-length".to_string()));
        assert!(functions.contains(&"number?".to_string()));
    }
}