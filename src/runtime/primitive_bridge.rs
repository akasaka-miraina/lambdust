#![allow(unreachable_patterns)]
//! Minimal Primitive Bridge System
//!
//! This module defines the absolute minimum set of Rust primitives required to support
//! R7RS and SRFI functionality implemented in Scheme. The goal is to minimize the 
//! Rust surface area while providing a solid foundation for pure Scheme implementations.
//!
//! ## Design Principles
//! 1. **Minimalism**: Only include operations that CANNOT be implemented in Scheme
//! 2. **Completeness**: Ensure these primitives can support all R7RS-small + SRFI features
//! 3. **Efficiency**: Provide optimized implementations for performance-critical operations
//! 4. **Orthogonality**: Each primitive should be independent and composable

use crate::diagnostics::{Result, Error};
use crate::eval::{Value, ThreadSafeEnvironment, PrimitiveProcedure, PrimitiveImpl};
use crate::effects::Effect;
use crate::ast::Literal;
use std::sync::Arc;
use std::collections::HashMap;

/// Categories of minimal primitives required for bootstrap
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MinimalPrimitiveCategory {
    /// Core evaluation and control flow
    Evaluation,
    /// Memory operations (allocation, mutation)
    Memory,
    /// Type predicates and inspection
    Types,
    /// Input/output operations
    IO,
    /// String manipulation
    Strings,
    /// Vector operations
    Vectors,
    /// Bytevector operations
    Bytevectors,
    /// Symbol operations
    Symbols,
    /// Arithmetic operations
    Arithmetic,
    /// System interface
    System,
}

/// Registry of minimal primitives organized by category
#[derive(Debug)]
pub struct MinimalPrimitiveRegistry {
    primitives: HashMap<String, MinimalPrimitive>,
    categories: HashMap<MinimalPrimitiveCategory, Vec<String>>,
}

/// A minimal primitive procedure with metadata
#[derive(Debug, Clone)]
pub struct MinimalPrimitive {
    pub name: String,
    pub category: MinimalPrimitiveCategory,
    pub implementation: fn(&[Value]) -> Result<Value>,
    pub arity_min: usize,
    pub arity_max: Option<usize>,
    pub documentation: String,
    pub r7rs_required: bool,
}

impl MinimalPrimitiveRegistry {
    /// Creates a new registry with all minimal primitives
    pub fn new() -> Self {
        let mut registry = Self {
            primitives: HashMap::new(),
            categories: HashMap::new(),
        };
        registry.register_all_primitives();
        registry
    }

    /// Registers all minimal primitives required for R7RS + SRFI support
    fn register_all_primitives(&mut self) {
        // ============= EVALUATION PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%apply".to_string(),
            category: MinimalPrimitiveCategory::Evaluation,
            implementation: primitive_apply,
            arity_min: 2,
            arity_max: None,
            documentation: "Apply procedure to arguments list".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%eval".to_string(),
            category: MinimalPrimitiveCategory::Evaluation,
            implementation: primitive_eval,
            arity_min: 1,
            arity_max: Some(2),
            documentation: "Evaluate expression in environment".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%call/cc".to_string(),
            category: MinimalPrimitiveCategory::Evaluation,
            implementation: primitive_call_cc,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Call with current continuation".to_string(),
            r7rs_required: true,
        });

        // ============= MEMORY PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%cons".to_string(),
            category: MinimalPrimitiveCategory::Memory,
            implementation: primitive_cons,
            arity_min: 2,
            arity_max: Some(2),
            documentation: "Construct a pair".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%car".to_string(),
            category: MinimalPrimitiveCategory::Memory,
            implementation: primitive_car,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "First element of pair".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%cdr".to_string(),
            category: MinimalPrimitiveCategory::Memory,
            implementation: primitive_cdr,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Rest of pair".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%set-car!".to_string(),
            category: MinimalPrimitiveCategory::Memory,
            implementation: primitive_set_car,
            arity_min: 2,
            arity_max: Some(2),
            documentation: "Mutate first element of pair".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%set-cdr!".to_string(),
            category: MinimalPrimitiveCategory::Memory,
            implementation: primitive_set_cdr,
            arity_min: 2,
            arity_max: Some(2),
            documentation: "Mutate rest of pair".to_string(),
            r7rs_required: true,
        });

        // ============= TYPE PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%null?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_null_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for null value".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%pair?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_pair_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for pair".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%symbol?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_symbol_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for symbol".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%number?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_number_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for number".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%string?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_string_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for string".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%char?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_char_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for character".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%vector?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_vector_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for vector".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%procedure?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_procedure_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for procedure".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%port?".to_string(),
            category: MinimalPrimitiveCategory::Types,
            implementation: primitive_port_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for port".to_string(),
            r7rs_required: true,
        });

        // ============= I/O PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%read-char".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_read_char,
            arity_min: 0,
            arity_max: Some(1),
            documentation: "Read character from port".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%write-char".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_write_char,
            arity_min: 1,
            arity_max: Some(2),
            documentation: "Write character to port".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%peek-char".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_peek_char,
            arity_min: 0,
            arity_max: Some(1),
            documentation: "Peek character from port".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%eof-object?".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_eof_object_p,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Test for EOF object".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%open-input-file".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_open_input_file,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Open file for input".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%open-output-file".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_open_output_file,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Open file for output".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%close-port".to_string(),
            category: MinimalPrimitiveCategory::IO,
            implementation: primitive_close_port,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Close port".to_string(),
            r7rs_required: true,
        });

        // ============= STRING PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%string-length".to_string(),
            category: MinimalPrimitiveCategory::Strings,
            implementation: primitive_string_length,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Get string length".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%string-ref".to_string(),
            category: MinimalPrimitiveCategory::Strings,
            implementation: primitive_string_ref,
            arity_min: 2,
            arity_max: Some(2),
            documentation: "Get character at string index".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%string-set!".to_string(),
            category: MinimalPrimitiveCategory::Strings,
            implementation: primitive_string_set,
            arity_min: 3,
            arity_max: Some(3),
            documentation: "Set character at string index".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%make-string".to_string(),
            category: MinimalPrimitiveCategory::Strings,
            implementation: primitive_make_string,
            arity_min: 1,
            arity_max: Some(2),
            documentation: "Create string of specified length".to_string(),
            r7rs_required: true,
        });

        // ============= VECTOR PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%vector-length".to_string(),
            category: MinimalPrimitiveCategory::Vectors,
            implementation: primitive_vector_length,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Get vector length".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%vector-ref".to_string(),
            category: MinimalPrimitiveCategory::Vectors,
            implementation: primitive_vector_ref,
            arity_min: 2,
            arity_max: Some(2),
            documentation: "Get element at vector index".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%vector-set!".to_string(),
            category: MinimalPrimitiveCategory::Vectors,
            implementation: primitive_vector_set,
            arity_min: 3,
            arity_max: Some(3),
            documentation: "Set element at vector index".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%make-vector".to_string(),
            category: MinimalPrimitiveCategory::Vectors,
            implementation: primitive_make_vector,
            arity_min: 1,
            arity_max: Some(2),
            documentation: "Create vector of specified length".to_string(),
            r7rs_required: true,
        });

        // ============= SYMBOL PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%string->symbol".to_string(),
            category: MinimalPrimitiveCategory::Symbols,
            implementation: primitive_string_to_symbol,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Convert string to symbol".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%symbol->string".to_string(),
            category: MinimalPrimitiveCategory::Symbols,
            implementation: primitive_symbol_to_string,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Convert symbol to string".to_string(),
            r7rs_required: true,
        });

        // ============= ARITHMETIC PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%+".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_add,
            arity_min: 0,
            arity_max: None,
            documentation: "Add numbers".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%-".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_subtract,
            arity_min: 1,
            arity_max: None,
            documentation: "Subtract numbers".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%*".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_multiply,
            arity_min: 0,
            arity_max: None,
            documentation: "Multiply numbers".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%/".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_divide,
            arity_min: 1,
            arity_max: None,
            documentation: "Divide numbers".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%=".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_numeric_equal,
            arity_min: 2,
            arity_max: None,
            documentation: "Numeric equality".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%<".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_less_than,
            arity_min: 2,
            arity_max: None,
            documentation: "Numeric less than".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%>".to_string(),
            category: MinimalPrimitiveCategory::Arithmetic,
            implementation: primitive_greater_than,
            arity_min: 2,
            arity_max: None,
            documentation: "Numeric greater than".to_string(),
            r7rs_required: true,
        });

        // ============= SYSTEM PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%error".to_string(),
            category: MinimalPrimitiveCategory::System,
            implementation: primitive_error,
            arity_min: 1,
            arity_max: None,
            documentation: "Signal error".to_string(),
            r7rs_required: true,
        });

        self.register(MinimalPrimitive {
            name: "%gc".to_string(),
            category: MinimalPrimitiveCategory::System,
            implementation: primitive_gc,
            arity_min: 0,
            arity_max: Some(0),
            documentation: "Force garbage collection".to_string(),
            r7rs_required: false,
        });
    }

    /// Registers a primitive in the registry
    pub fn register(&mut self, primitive: MinimalPrimitive) {
        let name = primitive.name.clone());
        let category = primitive.category.clone());
        
        self.primitives.insert(name.clone()), primitive);
        self.categories.entry(category)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Installs all minimal primitives in the given environment
    pub fn install_in_environment(&self, env: &Arc<ThreadSafeEnvironment>) {
        for (name, primitive) in &self.primitives {
            let value = Value::Primitive(Arc::new(PrimitiveProcedure {
                name: name.clone()),
                arity_min: primitive.arity_min,
                arity_max: primitive.arity_max,
                implementation: PrimitiveImpl::RustFn(primitive.implementation),
                effects: vec![Effect::Pure], // Most primitives are pure
            }));
            
            env.define(name.clone()), value);
        }
    }

    /// Gets primitives by category
    pub fn primitives_in_category(&self, category: &MinimalPrimitiveCategory) -> Vec<&MinimalPrimitive> {
        if let Some(names) = self.categories.get(category) {
            names.iter()
                .filter_map(|name| self.primitives.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets all R7RS required primitives
    pub fn r7rs_required_primitives(&self) -> Vec<&MinimalPrimitive> {
        self.primitives.values()
            .filter(|p| p.r7rs_required)
            .collect()
    }

    /// Gets a primitive by name
    pub fn get_primitive(&self, name: &str) -> Option<&MinimalPrimitive> {
        self.primitives.get(name)
    }

    /// Gets primitives by category (alias for primitives_in_category)
    pub fn get_primitives_by_category(&self, category: &MinimalPrimitiveCategory) -> Vec<&MinimalPrimitive> {
        self.primitives_in_category(category)
    }

    /// Gets primitive count by category
    pub fn count_by_category(&self) -> HashMap<MinimalPrimitiveCategory, usize> {
        let mut counts = HashMap::new();
        for primitive in self.primitives.values() {
            *counts.entry(primitive.category.clone()).or_insert(0) += 1;
        }
        counts
    }
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

// Evaluation primitives
fn primitive_apply(_args: &[Value]) -> Result<Value> {
    // This requires evaluator integration - simplified for now
    Err(Box::new(Error::runtime_error("apply requires evaluator integration".to_string(), None))
}

fn primitive_eval(_args: &[Value]) -> Result<Value> {
    // This requires evaluator integration - simplified for now
    Err(Box::new(Error::runtime_error("eval requires evaluator integration".to_string(), None))
}

fn primitive_call_cc(_args: &[Value]) -> Result<Value> {
    // This requires evaluator integration - simplified for now
    Err(Box::new(Error::runtime_error("call/cc requires evaluator integration".to_string(), None))
}

// Memory primitives
fn primitive_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("cons requires exactly 2 arguments".to_string(), None));
    }
    Ok(Value::pair(args[0].clone()), args[1].clone()))
}

fn primitive_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("car requires exactly 1 argument".to_string(), None));
    }
    
    match &args[0] {
        Value::Pair(car, _cdr) => Ok((**car).clone()),
        _ => Err(Box::new(Error::runtime_error("car expects a pair".to_string(), None)),
    }
}

fn primitive_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("cdr requires exactly 1 argument".to_string(), None));
    }
    
    match &args[0] {
        Value::Pair(_car, cdr) => Ok((**cdr).clone()),
        _ => Err(Box::new(Error::runtime_error("cdr expects a pair".to_string(), None)),
    }
}

fn primitive_set_car(_args: &[Value]) -> Result<Value> {
    // This requires mutable pairs - simplified for now
    Err(Box::new(Error::runtime_error("set-car! requires mutable pair support".to_string(), None))
}

fn primitive_set_cdr(_args: &[Value]) -> Result<Value> {
    // This requires mutable pairs - simplified for now
    Err(Box::new(Error::runtime_error("set-cdr! requires mutable pair support".to_string(), None))
}

// Type predicates
fn primitive_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("null? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Nil)))
}

fn primitive_pair_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("pair? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Pair(_, _))))
}

fn primitive_symbol_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("symbol? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Symbol(_))))
}

fn primitive_number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("number? requires exactly 1 argument".to_string(), None));
    }
    let is_number = matches!(args[0], 
        Value::Literal(Literal::Number(_))
    );
    Ok(Value::boolean(is_number))
}

fn primitive_string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Literal(Literal::String(_)))))
}

fn primitive_char_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("char? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Literal(Literal::Character(_)))))
}

fn primitive_vector_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("vector? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Vector(_))))
}

fn primitive_procedure_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("procedure? requires exactly 1 argument".to_string(), None));
    }
    let is_procedure = matches!(args[0], 
        Value::Procedure(_) | 
        Value::Primitive(_) |
        Value::CaseLambda(_)
    );
    Ok(Value::boolean(is_procedure))
}

fn primitive_port_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("port? requires exactly 1 argument".to_string(), None));
    }
    Ok(Value::boolean(matches!(args[0], Value::Port(_))))
}

// I/O primitives (simplified implementations)
fn primitive_read_char(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("read-char requires I/O system integration".to_string(), None))
}

fn primitive_write_char(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("write-char requires I/O system integration".to_string(), None))
}

fn primitive_peek_char(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("peek-char requires I/O system integration".to_string(), None))
}

fn primitive_eof_object_p(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("eof-object? requires I/O system integration".to_string(), None))
}

fn primitive_open_input_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("open-input-file requires I/O system integration".to_string(), None))
}

fn primitive_open_output_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("open-output-file requires I/O system integration".to_string(), None))
}

fn primitive_close_port(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("close-port requires I/O system integration".to_string(), None))
}

// String primitives (simplified implementations)
fn primitive_string_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string-length requires exactly 1 argument".to_string(), None));
    }
    
    match &args[0] {
        Value::Literal(Literal::String(s)) => Ok(Value::integer(s.len() as i64)),
        _ => Err(Box::new(Error::runtime_error("string-length expects a string".to_string(), None)),
    }
}

fn primitive_string_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("string-ref requires exactly 2 arguments".to_string(), None));
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(Literal::String(s)), Value::Literal(Literal::Number(i))) if i.fract() == 0.0 => {
            if *i < 0.0 || *i as usize >= s.len() {
                return Err(Box::new(Error::runtime_error("string-ref index out of bounds".to_string(), None));
            }
            if let Some(ch) = s.chars().nth(*i as usize) {
                Ok(Value::Literal(Literal::Character(ch)))
            } else {
                Err(Box::new(Error::runtime_error("string-ref index invalid".to_string(), None))
            }
        }
        _ => Err(Box::new(Error::runtime_error("string-ref expects string and integer".to_string(), None)),
    }
}

fn primitive_string_set(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("string-set! requires mutable string support".to_string(), None))
}

fn primitive_make_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error("make-string requires 1 or 2 arguments".to_string(), None));
    }
    
    let length = match &args[0] {
        Value::Literal(Literal::Number(n)) if n.fract() == 0.0 && *n >= 0.0 => *n as usize,
        _ => return Err(Box::new(Error::runtime_error("make-string expects non-negative integer length".to_string(), None)),
    };
    
    let fill_char = if args.len() == 2 {
        match &args[1] {
            Value::Literal(Literal::Character(c)) => *c,
            _ => return Err(Box::new(Error::runtime_error("make-string fill must be a character".to_string(), None)),
        }
    } else {
        '\0'
    };
    
    let s = fill_char.to_string().repeat(length);
    Ok(Value::string(s))
}

// Vector primitives (simplified implementations)
fn primitive_vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("vector-length requires exactly 1 argument".to_string(), None));
    }
    
    match &args[0] {
        Value::Vector(vec) => {
            let vec_guard = vec.read().map_err(|_| Error::runtime_error("Failed to read vector".to_string(), None))?;
            Ok(Value::integer(vec_guard.len() as i64))
        }
        _ => Err(Box::new(Error::runtime_error("vector-length expects a vector".to_string(), None)),
    }
}

fn primitive_vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("vector-ref requires exactly 2 arguments".to_string(), None));
    }
    
    match (&args[0], &args[1]) {
        (Value::Vector(vec), Value::Literal(Literal::Number(i))) if i.fract() == 0.0 => {
            let vec_guard = vec.read().map_err(|_| Error::runtime_error("Failed to read vector".to_string(), None))?;
            if *i < 0.0 || *i as usize >= vec_guard.len() {
                return Err(Box::new(Error::runtime_error("vector-ref index out of bounds".to_string(), None));
            }
            Ok(vec_guard[*i as usize].clone())
        }
        _ => Err(Box::new(Error::runtime_error("vector-ref expects vector and integer".to_string(), None)),
    }
}

fn primitive_vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error("vector-set! requires exactly 3 arguments".to_string(), None));
    }
    
    match (&args[0], &args[1]) {
        (Value::Vector(vec), Value::Literal(Literal::Number(i))) if i.fract() == 0.0 => {
            let mut vec_guard = vec.write().map_err(|_| Error::runtime_error("Failed to write vector".to_string(), None))?;
            if *i < 0.0 || *i as usize >= vec_guard.len() {
                return Err(Box::new(Error::runtime_error("vector-set! index out of bounds".to_string(), None));
            }
            vec_guard[*i as usize] = args[2].clone());
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(Error::runtime_error("vector-set! expects vector and integer".to_string(), None)),
    }
}

fn primitive_make_vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error("make-vector requires 1 or 2 arguments".to_string(), None));
    }
    
    let length = match &args[0] {
        Value::Literal(Literal::Number(n)) if n.fract() == 0.0 && *n >= 0.0 => *n as usize,
        _ => return Err(Box::new(Error::runtime_error("make-vector expects non-negative integer length".to_string(), None)),
    };
    
    let fill_value = if args.len() == 2 {
        args[1].clone())
    } else {
        Value::Unspecified
    };
    
    let vec = vec![fill_value; length];
    Ok(Value::vector(vec))
}

// Symbol primitives
fn primitive_string_to_symbol(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string->symbol requires exactly 1 argument".to_string(), None));
    }
    
    match &args[0] {
        Value::Literal(Literal::String(s)) => {
            use crate::utils::intern_symbol;
            let symbol_id = intern_symbol(s.clone());
            Ok(Value::symbol(symbol_id))
        }
        _ => Err(Box::new(Error::runtime_error("string->symbol expects a string".to_string(), None)),
    }
}

fn primitive_symbol_to_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("symbol->string requires exactly 1 argument".to_string(), None));
    }
    
    match &args[0] {
        Value::Symbol(symbol_id) => {
            use crate::utils::symbol::symbol_name;
            if let Some(name) = symbol_name(*symbol_id) {
                Ok(Value::string(name))
            } else {
                Err(Box::new(Error::runtime_error("Invalid symbol ID".to_string(), None))
            }
        }
        _ => Err(Box::new(Error::runtime_error("symbol->string expects a symbol".to_string(), None)),
    }
}

// Arithmetic primitives
fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = 0i64;
    for arg in args {
        match arg {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => result += *n as i64,
            _ => return Err(Box::new(Error::runtime_error("+ expects numeric arguments".to_string(), None)),
        }
    }
    Ok(Value::integer(result))
}

fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("- requires at least one argument".to_string(), None));
    }
    
    if args.len() == 1 {
        match &args[0] {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => Ok(Value::integer(-(*n as i64))),
            _ => Err(Box::new(Error::runtime_error("- expects numeric arguments".to_string(), None)),
        }
    } else {
        let mut result = match &args[0] {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => *n as i64,
            _ => return Err(Box::new(Error::runtime_error("- expects numeric arguments".to_string(), None)),
        };
        
        for arg in &args[1..] {
            match arg {
                Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => result -= *n as i64,
                _ => return Err(Box::new(Error::runtime_error("- expects numeric arguments".to_string(), None)),
            }
        }
        Ok(Value::integer(result))
    }
}

fn primitive_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }
    
    let mut result = 1i64;
    for arg in args {
        match arg {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => result *= *n as i64,
            _ => return Err(Box::new(Error::runtime_error("* expects numeric arguments".to_string(), None)),
        }
    }
    Ok(Value::integer(result))
}

fn primitive_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("/ requires at least one argument".to_string(), None));
    }
    
    if args.len() == 1 {
        match &args[0] {
            Value::Literal(Literal::Number(n)) if n.fract() == 0.0 => {
                if *n == 0.0 {
                    return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None));
                }
                Ok(Value::number(1.0 / *n))
            }
            _ => Err(Box::new(Error::runtime_error("/ expects numeric arguments".to_string(), None)),
        }
    } else {
        let mut result = match &args[0] {
            Value::Literal(Literal::Number(n)) => *n as f64,
            _ => return Err(Box::new(Error::runtime_error("/ expects numeric arguments".to_string(), None)),
        };
        
        for arg in &args[1..] {
            match arg {
                Value::Literal(Literal::Number(n)) => {
                    if *n == 0.0 {
                        return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None));
                    }
                    result /= *n as f64;
                }
                _ => return Err(Box::new(Error::runtime_error("/ expects numeric arguments".to_string(), None)),
            }
        }
        Ok(Value::number(result))
    }
}

fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("= requires at least 2 arguments".to_string(), None));
    }
    
    let first_val = match &args[0] {
        Value::Literal(Literal::Number(n)) => *n as f64,
        Value::Literal(Literal::Number(r)) => *r,
        _ => return Err(Box::new(Error::runtime_error("= expects numeric arguments".to_string(), None)),
    };
    
    for arg in &args[1..] {
        let val = match arg {
            Value::Literal(Literal::Number(n)) => *n as f64,
            Value::Literal(Literal::Number(r)) => *r,
            _ => return Err(Box::new(Error::runtime_error("= expects numeric arguments".to_string(), None)),
        };
        
        if (first_val - val).abs() > f64::EPSILON {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("< requires at least 2 arguments".to_string(), None));
    }
    
    for i in 0..args.len() - 1 {
        let current = match &args[i] {
            Value::Literal(Literal::Number(n)) => *n as f64,
            Value::Literal(Literal::Number(r)) => *r,
            _ => return Err(Box::new(Error::runtime_error("< expects numeric arguments".to_string(), None)),
        };
        
        let next = match &args[i + 1] {
            Value::Literal(Literal::Number(n)) => *n as f64,
            Value::Literal(Literal::Number(r)) => *r,
            _ => return Err(Box::new(Error::runtime_error("< expects numeric arguments".to_string(), None)),
        };
        
        if current >= next {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

fn primitive_greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("> requires at least 2 arguments".to_string(), None));
    }
    
    for i in 0..args.len() - 1 {
        let current = match &args[i] {
            Value::Literal(Literal::Number(n)) => *n as f64,
            Value::Literal(Literal::Number(r)) => *r,
            _ => return Err(Box::new(Error::runtime_error("> expects numeric arguments".to_string(), None)),
        };
        
        let next = match &args[i + 1] {
            Value::Literal(Literal::Number(n)) => *n as f64,
            Value::Literal(Literal::Number(r)) => *r,
            _ => return Err(Box::new(Error::runtime_error("> expects numeric arguments".to_string(), None)),
        };
        
        if current <= next {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

// System primitives
fn primitive_error(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("error requires at least 1 argument".to_string(), None));
    }
    
    let message = match &args[0] {
        Value::Literal(Literal::String(s)) => s.clone()),
        _ => format!("{}", args[0]),
    };
    
    Err(Box::new(Error::runtime_error(message, None).boxed())
}

fn primitive_gc(_args: &[Value]) -> Result<Value> {
    // Force garbage collection if available
    // In Rust, we can't force GC, but we can suggest it
    Ok(Value::Unspecified)
}

impl Default for MinimalPrimitiveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = MinimalPrimitiveRegistry::new();
        assert!(!registry.primitives.is_empty());
        
        // Verify all categories are represented
        let categories = registry.count_by_category();
        assert!(categories.contains_key(&MinimalPrimitiveCategory::Evaluation));
        assert!(categories.contains_key(&MinimalPrimitiveCategory::Memory));
        assert!(categories.contains_key(&MinimalPrimitiveCategory::Types));
    }

    #[test]
    fn test_primitive_arithmetic() {
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let result = primitive_add(&args).unwrap();
        assert_eq!(result, Value::integer(6));
        
        let args = vec![Value::integer(10), Value::integer(3)];
        let result = primitive_subtract(&args).unwrap();
        assert_eq!(result, Value::integer(7));
    }

    #[test]
    fn test_primitive_memory() {
        let args = vec![Value::integer(1), Value::integer(2)];
        let result = primitive_cons(&args).unwrap();
        assert!(matches!(result, Value::Pair(_, _)));
        
        let pair = Value::pair(Value::integer(1), Value::integer(2));
        let result = primitive_car(&[pair]).unwrap();
        assert_eq!(result, Value::integer(1));
    }

    #[test]
    fn test_primitive_types() {
        let result = primitive_null_p(&[Value::Nil]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_pair_p(&[Value::pair(Value::integer(1), Value::integer(2))]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_string_p(&[Value::string("hello")]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_r7rs_required_primitives() {
        let registry = MinimalPrimitiveRegistry::new();
        let r7rs_prims = registry.r7rs_required_primitives();
        
        // Ensure we have all essential R7RS primitives
        assert!(!r7rs_prims.is_empty());
        assert!(r7rs_prims.iter().any(|p| p.name == "%cons"));
        assert!(r7rs_prims.iter().any(|p| p.name == "%car"));
        assert!(r7rs_prims.iter().any(|p| p.name == "%+"));
    }
}