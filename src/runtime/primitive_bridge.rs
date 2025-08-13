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
    /// Bitwise operations (SRFI-151)
    Bitwise,
    /// System interface
    System,
}

/// Helper function to extract numeric values from literals in primitive operations
fn extract_numeric_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Literal(literal) => literal.to_f64(),
        _ => None,
    }
}

/// Helper function to extract integer values from literals in primitive operations
fn extract_numeric_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Literal(literal) => literal.to_i64(),
        _ => None,
    }
}

/// Helper function to extract exact integer values for bitwise operations
fn extract_exact_integer(value: &Value) -> Option<i64> {
    match value {
        Value::Literal(literal) => {
            if literal.is_exact() && literal.is_integer() {
                literal.to_i64()
            } else {
                None
            }
        }
        _ => None,
    }
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
    /// Name of the primitive procedure
    pub name: String,
    /// Category this primitive belongs to
    pub category: MinimalPrimitiveCategory,
    /// Function implementation of the primitive
    pub implementation: fn(&[Value]) -> Result<Value>,
    /// Minimum number of arguments required
    pub arity_min: usize,
    /// Maximum number of arguments allowed (None for unlimited)
    pub arity_max: Option<usize>,
    /// Human-readable documentation string
    pub documentation: String,
    /// Whether this primitive is required by R7RS standard
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

        // ============= BITWISE PRIMITIVES =============
        self.register(MinimalPrimitive {
            name: "%bitwise-and".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_bitwise_and,
            arity_min: 0,
            arity_max: None,
            documentation: "Bitwise AND operation on exact integers".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%bitwise-ior".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_bitwise_ior,
            arity_min: 0,
            arity_max: None,
            documentation: "Bitwise inclusive OR operation on exact integers".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%bitwise-xor".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_bitwise_xor,
            arity_min: 0,
            arity_max: None,
            documentation: "Bitwise exclusive OR operation on exact integers".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%bitwise-not".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_bitwise_not,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Bitwise NOT operation on exact integer".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%arithmetic-shift".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_arithmetic_shift,
            arity_min: 2,
            arity_max: Some(2),
            documentation: "Arithmetic shift of exact integer by count positions".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%bit-count".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_bit_count,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Count number of 1 bits in exact integer's two's complement representation".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%integer-length".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_integer_length,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Find minimum bits needed to represent exact integer in two's complement".to_string(),
            r7rs_required: false,
        });

        self.register(MinimalPrimitive {
            name: "%first-set-bit".to_string(),
            category: MinimalPrimitiveCategory::Bitwise,
            implementation: primitive_first_set_bit,
            arity_min: 1,
            arity_max: Some(1),
            documentation: "Find position of first set bit (rightmost 1) in exact integer".to_string(),
            r7rs_required: false,
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
        let name = primitive.name.clone();
        let category = primitive.category.clone();
        
        self.primitives.insert(name.clone(), primitive);
        self.categories.entry(category)
            .or_default()
            .push(name);
    }

    /// Installs all minimal primitives in the given environment
    pub fn install_in_environment(&self, env: &Arc<ThreadSafeEnvironment>) {
        for (name, primitive) in &self.primitives {
            let value = Value::Primitive(Arc::new(PrimitiveProcedure {
                name: name.clone(),
                arity_min: primitive.arity_min,
                arity_max: primitive.arity_max,
                implementation: PrimitiveImpl::RustFn(primitive.implementation),
                effects: vec![Effect::Pure], // Most primitives are pure
            }));
            
            env.define(name.clone(), value);
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
    Err(Box::new(Error::runtime_error("apply requires evaluator integration".to_string(), None)))
}

fn primitive_eval(_args: &[Value]) -> Result<Value> {
    // This requires evaluator integration - simplified for now
    Err(Box::new(Error::runtime_error("eval requires evaluator integration".to_string(), None)))
}

fn primitive_call_cc(_args: &[Value]) -> Result<Value> {
    // This requires evaluator integration - simplified for now
    Err(Box::new(Error::runtime_error("call/cc requires evaluator integration".to_string(), None)))
}

// Memory primitives
fn primitive_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("cons requires exactly 2 arguments".to_string(), None)));
    }
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

fn primitive_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("car requires exactly 1 argument".to_string(), None)));
    }
    
    match &args[0] {
        Value::Pair(car, _cdr) => Ok((**car).clone()),
        _ => Err(Box::new(Error::runtime_error("car expects a pair".to_string(), None))),
    }
}

fn primitive_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("cdr requires exactly 1 argument".to_string(), None)));
    }
    
    match &args[0] {
        Value::Pair(_car, cdr) => Ok((**cdr).clone()),
        _ => Err(Box::new(Error::runtime_error("cdr expects a pair".to_string(), None))),
    }
}

fn primitive_set_car(_args: &[Value]) -> Result<Value> {
    // This requires mutable pairs - simplified for now
    Err(Box::new(Error::runtime_error("set-car! requires mutable pair support".to_string(), None)))
}

fn primitive_set_cdr(_args: &[Value]) -> Result<Value> {
    // This requires mutable pairs - simplified for now
    Err(Box::new(Error::runtime_error("set-cdr! requires mutable pair support".to_string(), None)))
}

// Type predicates
fn primitive_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("null? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Nil)))
}

fn primitive_pair_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("pair? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Pair(_, _))))
}

fn primitive_symbol_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("symbol? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Symbol(_))))
}

fn primitive_number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("number? requires exactly 1 argument".to_string(), None)));
    }
    let is_number = matches!(&args[0], 
        Value::Literal(literal) if literal.is_number()
    );
    Ok(Value::boolean(is_number))
}

fn primitive_string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Literal(Literal::String(_)))))
}

fn primitive_char_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("char? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Literal(Literal::Character(_)))))
}

fn primitive_vector_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("vector? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Vector(_))))
}

fn primitive_procedure_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("procedure? requires exactly 1 argument".to_string(), None)));
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
        return Err(Box::new(Error::runtime_error("port? requires exactly 1 argument".to_string(), None)));
    }
    Ok(Value::boolean(matches!(args[0], Value::Port(_))))
}

// I/O primitives (simplified implementations)
fn primitive_read_char(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("read-char requires I/O system integration".to_string(), None)))
}

fn primitive_write_char(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("write-char requires I/O system integration".to_string(), None)))
}

fn primitive_peek_char(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("peek-char requires I/O system integration".to_string(), None)))
}

fn primitive_eof_object_p(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("eof-object? requires I/O system integration".to_string(), None)))
}

fn primitive_open_input_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("open-input-file requires I/O system integration".to_string(), None)))
}

fn primitive_open_output_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("open-output-file requires I/O system integration".to_string(), None)))
}

fn primitive_close_port(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("close-port requires I/O system integration".to_string(), None)))
}

// String primitives (simplified implementations)
fn primitive_string_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string-length requires exactly 1 argument".to_string(), None)));
    }
    
    match &args[0] {
        Value::Literal(Literal::String(s)) => Ok(Value::integer(s.len() as i64)),
        _ => Err(Box::new(Error::runtime_error("string-length expects a string".to_string(), None))),
    }
}

fn primitive_string_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("string-ref requires exactly 2 arguments".to_string(), None)));
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(Literal::String(s)), Value::Literal(literal)) if literal.is_number() => {
            if let Some(i_f64) = literal.to_f64() {
                if i_f64 < 0.0 || i_f64.fract() != 0.0 || i_f64 as usize >= s.len() {
                    return Err(Box::new(Error::runtime_error("string-ref index out of bounds".to_string(), None)));
                }
                let i = i_f64 as usize;
                if let Some(ch) = s.chars().nth(i) {
                    Ok(Value::Literal(Literal::Character(ch)))
                } else {
                    Err(Box::new(Error::runtime_error("string-ref index invalid".to_string(), None)))
                }
            } else {
                Err(Box::new(Error::runtime_error("string-ref expects integer index".to_string(), None)))
            }
        }
        _ => Err(Box::new(Error::runtime_error("string-ref expects string and integer".to_string(), None))),
    }
}

fn primitive_string_set(_args: &[Value]) -> Result<Value> {
    Err(Box::new(Error::runtime_error("string-set! requires mutable string support".to_string(), None)))
}

fn primitive_make_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error("make-string requires 1 or 2 arguments".to_string(), None)));
    }
    
    let length = match &args[0] {
        Value::Literal(literal) if literal.is_number() => {
            if let Some(n) = literal.to_f64() {
                if n.fract() == 0.0 && n >= 0.0 {
                    n as usize
                } else {
                    return Err(Box::new(Error::runtime_error("make-string expects non-negative integer length".to_string(), None)));
                }
            } else {
                return Err(Box::new(Error::runtime_error("make-string expects non-negative integer length".to_string(), None)));
            }
        }
        _ => return Err(Box::new(Error::runtime_error("make-string expects non-negative integer length".to_string(), None))),
    };
    
    let fill_char = if args.len() == 2 {
        match &args[1] {
            Value::Literal(Literal::Character(c)) => *c,
            _ => return Err(Box::new(Error::runtime_error("make-string fill must be a character".to_string(), None))),
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
        return Err(Box::new(Error::runtime_error("vector-length requires exactly 1 argument".to_string(), None)));
    }
    
    match &args[0] {
        Value::Vector(vec) => {
            let vec_guard = vec.read().map_err(|_| Error::runtime_error("Failed to read vector".to_string(), None))?;
            Ok(Value::integer(vec_guard.len() as i64))
        }
        _ => Err(Box::new(Error::runtime_error("vector-length expects a vector".to_string(), None))),
    }
}

fn primitive_vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("vector-ref requires exactly 2 arguments".to_string(), None)));
    }
    
    match (&args[0], &args[1]) {
        (Value::Vector(vec), value) if extract_numeric_i64(value).is_some() => {
            let i = extract_numeric_i64(value).unwrap();
            let vec_guard = vec.read().map_err(|_| Error::runtime_error("Failed to read vector".to_string(), None))?;
            if i < 0 || i as usize >= vec_guard.len() {
                return Err(Box::new(Error::runtime_error("vector-ref index out of bounds".to_string(), None)));
            }
            Ok(vec_guard[i as usize].clone())
        }
        _ => Err(Box::new(Error::runtime_error("vector-ref expects vector and integer".to_string(), None))),
    }
}

fn primitive_vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error("vector-set! requires exactly 3 arguments".to_string(), None)));
    }
    
    match (&args[0], &args[1]) {
        (Value::Vector(vec), value) if extract_numeric_i64(value).is_some() => {
            let i = extract_numeric_i64(value).unwrap();
            let mut vec_guard = vec.write().map_err(|_| Error::runtime_error("Failed to write vector".to_string(), None))?;
            if i < 0 || i as usize >= vec_guard.len() {
                return Err(Box::new(Error::runtime_error("vector-set! index out of bounds".to_string(), None)));
            }
            vec_guard[i as usize] = args[2].clone();
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(Error::runtime_error("vector-set! expects vector and integer".to_string(), None))),
    }
}

fn primitive_make_vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error("make-vector requires 1 or 2 arguments".to_string(), None)));
    }
    
    let length = match &args[0] {
        value if extract_numeric_i64(value).is_some() => {
            let n = extract_numeric_i64(value).unwrap();
            if n >= 0 {
                n as usize
            } else {
                return Err(Box::new(Error::runtime_error("make-vector expects non-negative integer length".to_string(), None)));
            }
        }
        _ => return Err(Box::new(Error::runtime_error("make-vector expects non-negative integer length".to_string(), None))),
    };
    
    let fill_value = if args.len() == 2 {
        args[1].clone()
    } else {
        Value::Unspecified
    };
    
    let vec = vec![fill_value; length];
    Ok(Value::vector(vec))
}

// Symbol primitives
fn primitive_string_to_symbol(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string->symbol requires exactly 1 argument".to_string(), None)));
    }
    
    match &args[0] {
        Value::Literal(Literal::String(s)) => {
            use crate::utils::intern_symbol;
            let symbol_id = intern_symbol(s.clone());
            Ok(Value::symbol(symbol_id))
        }
        _ => Err(Box::new(Error::runtime_error("string->symbol expects a string".to_string(), None))),
    }
}

fn primitive_symbol_to_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("symbol->string requires exactly 1 argument".to_string(), None)));
    }
    
    match &args[0] {
        Value::Symbol(symbol_id) => {
            use crate::utils::symbol::symbol_name;
            if let Some(name) = symbol_name(*symbol_id) {
                Ok(Value::string(name))
            } else {
                Err(Box::new(Error::runtime_error("Invalid symbol ID".to_string(), None)))
            }
        }
        _ => Err(Box::new(Error::runtime_error("symbol->string expects a symbol".to_string(), None))),
    }
}

// Arithmetic primitives
fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = 0i64;
    for arg in args {
        if let Some(n) = extract_numeric_i64(arg) {
            result += n;
        } else {
            return Err(Box::new(Error::runtime_error("+ expects numeric arguments".to_string(), None)));
        }
    }
    Ok(Value::integer(result))
}

fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("- requires at least one argument".to_string(), None)));
    }
    
    if args.len() == 1 {
        if let Some(n) = extract_numeric_i64(&args[0]) {
            Ok(Value::integer(-n))
        } else {
            Err(Box::new(Error::runtime_error("- expects numeric arguments".to_string(), None)))
        }
    } else {
        let mut result = if let Some(n) = extract_numeric_i64(&args[0]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("- expects numeric arguments".to_string(), None)));
        };
        
        for arg in &args[1..] {
            if let Some(n) = extract_numeric_i64(arg) {
                result -= n;
            } else {
                return Err(Box::new(Error::runtime_error("- expects numeric arguments".to_string(), None)));
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
        if let Some(n) = extract_numeric_i64(arg) {
            result *= n;
        } else {
            return Err(Box::new(Error::runtime_error("* expects numeric arguments".to_string(), None)));
        }
    }
    Ok(Value::integer(result))
}

fn primitive_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("/ requires at least one argument".to_string(), None)));
    }
    
    if args.len() == 1 {
        if let Some(n) = extract_numeric_f64(&args[0]) {
            if n == 0.0 {
                return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)));
            }
            Ok(Value::number(1.0 / n))
        } else {
            Err(Box::new(Error::runtime_error("/ expects numeric arguments".to_string(), None)))
        }
    } else {
        let mut result = if let Some(n) = extract_numeric_f64(&args[0]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("/ expects numeric arguments".to_string(), None)));
        };
        
        for arg in &args[1..] {
            if let Some(n) = extract_numeric_f64(arg) {
                if n == 0.0 {
                    return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)));
                }
                result /= n;
            } else {
                return Err(Box::new(Error::runtime_error("/ expects numeric arguments".to_string(), None)));
            }
        }
        Ok(Value::number(result))
    }
}

fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("= requires at least 2 arguments".to_string(), None)));
    }
    
    let first_val = if let Some(n) = extract_numeric_f64(&args[0]) {
        n
    } else {
        return Err(Box::new(Error::runtime_error("= expects numeric arguments".to_string(), None)));
    };
    
    for arg in &args[1..] {
        let val = if let Some(n) = extract_numeric_f64(arg) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("= expects numeric arguments".to_string(), None)));
        };
        
        if (first_val - val).abs() > f64::EPSILON {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("< requires at least 2 arguments".to_string(), None)));
    }
    
    for i in 0..args.len() - 1 {
        let current = if let Some(n) = extract_numeric_f64(&args[i]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("< expects numeric arguments".to_string(), None)));
        };
        
        let next = if let Some(n) = extract_numeric_f64(&args[i + 1]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("< expects numeric arguments".to_string(), None)));
        };
        
        if current >= next {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

fn primitive_greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("> requires at least 2 arguments".to_string(), None)));
    }
    
    for i in 0..args.len() - 1 {
        let current = if let Some(n) = extract_numeric_f64(&args[i]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("> expects numeric arguments".to_string(), None)));
        };
        
        let next = if let Some(n) = extract_numeric_f64(&args[i + 1]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("> expects numeric arguments".to_string(), None)));
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
        return Err(Box::new(Error::runtime_error("error requires at least 1 argument".to_string(), None)));
    }
    
    let message = match &args[0] {
        Value::Literal(Literal::String(s)) => s.clone(),
        _ => format!("{}", args[0]),
    };
    
    Err(Error::runtime_error(message, None).boxed())
}

fn primitive_gc(_args: &[Value]) -> Result<Value> {
    // Force garbage collection if available
    // In Rust, we can't force GC, but we can suggest it
    Ok(Value::Unspecified)
}

// Bitwise primitives (SRFI-151)
fn primitive_bitwise_and(args: &[Value]) -> Result<Value> {
    // bitwise-and with no arguments returns -1 (all bits set)
    if args.is_empty() {
        return Ok(Value::integer(-1));
    }
    
    let mut result = -1i64; // Identity element for AND (all bits set)
    for arg in args {
        if let Some(n) = extract_exact_integer(arg) {
            result &= n;
        } else {
            return Err(Box::new(Error::runtime_error(
                "bitwise-and expects exact integer arguments".to_string(), 
                None
            )));
        }
    }
    Ok(Value::integer(result))
}

fn primitive_bitwise_ior(args: &[Value]) -> Result<Value> {
    // bitwise-ior with no arguments returns 0 (no bits set)
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = 0i64; // Identity element for OR (no bits set)
    for arg in args {
        if let Some(n) = extract_exact_integer(arg) {
            result |= n;
        } else {
            return Err(Box::new(Error::runtime_error(
                "bitwise-ior expects exact integer arguments".to_string(), 
                None
            )));
        }
    }
    Ok(Value::integer(result))
}

fn primitive_bitwise_xor(args: &[Value]) -> Result<Value> {
    // bitwise-xor with no arguments returns 0 (identity element for XOR)
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = 0i64; // Identity element for XOR (no bits set)
    for arg in args {
        if let Some(n) = extract_exact_integer(arg) {
            result ^= n;
        } else {
            return Err(Box::new(Error::runtime_error(
                "bitwise-xor expects exact integer arguments".to_string(), 
                None
            )));
        }
    }
    Ok(Value::integer(result))
}

fn primitive_bitwise_not(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "bitwise-not requires exactly 1 argument".to_string(), 
            None
        )));
    }
    
    if let Some(n) = extract_exact_integer(&args[0]) {
        Ok(Value::integer(!n))
    } else {
        Err(Box::new(Error::runtime_error(
            "bitwise-not expects an exact integer argument".to_string(), 
            None
        )))
    }
}

fn primitive_arithmetic_shift(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            "arithmetic-shift requires exactly 2 arguments".to_string(), 
            None
        )));
    }
    
    let n = if let Some(val) = extract_exact_integer(&args[0]) {
        val
    } else {
        return Err(Box::new(Error::runtime_error(
            "arithmetic-shift expects an exact integer as first argument".to_string(), 
            None
        )));
    };
    
    let count = if let Some(val) = extract_exact_integer(&args[1]) {
        val
    } else {
        return Err(Box::new(Error::runtime_error(
            "arithmetic-shift expects an exact integer as second argument".to_string(), 
            None
        )));
    };
    
    // Limit shift count to prevent overflow
    if count.abs() >= 64 {
        if count > 0 {
            // Left shift by large amount - result depends on sign
            if n == 0 {
                Ok(Value::integer(0))
            } else if n > 0 {
                Ok(Value::integer(i64::MAX))
            } else {
                Ok(Value::integer(i64::MIN))
            }
        } else {
            // Right shift by large amount - result is 0 or -1
            if n >= 0 {
                Ok(Value::integer(0))
            } else {
                Ok(Value::integer(-1))
            }
        }
    } else if count >= 0 {
        // Left shift (positive count)
        // Check for overflow
        match n.checked_shl(count as u32) {
            Some(result) => Ok(Value::integer(result)),
            None => {
                // Overflow - return appropriate extreme value
                if n >= 0 {
                    Ok(Value::integer(i64::MAX))
                } else {
                    Ok(Value::integer(i64::MIN))
                }
            }
        }
    } else {
        // Right shift (negative count)
        Ok(Value::integer(n >> (-count as u32)))
    }
}

fn primitive_bit_count(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "bit-count requires exactly 1 argument".to_string(), 
            None
        )));
    }
    
    if let Some(n) = extract_exact_integer(&args[0]) {
        let count = if n >= 0 {
            // For non-negative integers, count the 1 bits directly
            n.count_ones() as i64
        } else {
            // For negative integers in two's complement, 
            // bit-count returns the count of 0 bits in the absolute value
            // This is equivalent to: (bitwidth - popcount(abs(n)))
            // For SRFI-151, this is defined as the number of 1 bits that would 
            // be needed in the two's complement representation
            64 - ((!n).count_ones() as i64)
        };
        Ok(Value::integer(count))
    } else {
        Err(Box::new(Error::runtime_error(
            "bit-count expects an exact integer argument".to_string(), 
            None
        )))
    }
}

fn primitive_integer_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "integer-length requires exactly 1 argument".to_string(), 
            None
        )));
    }
    
    if let Some(n) = extract_exact_integer(&args[0]) {
        let length = if n == 0 {
            // Special case: integer-length of 0 is 0
            0
        } else if n > 0 {
            // For positive integers, find the position of the most significant bit
            // This is equivalent to floor(log2(n)) + 1
            64 - n.leading_zeros() as i64
        } else {
            // For negative integers, integer-length is the number of bits needed
            // to represent the number in two's complement form.
            // This is equivalent to the integer-length of -(n+1) 
            // (the positive number with the same bit pattern)
            let abs_minus_one = (-n - 1) as u64;
            if abs_minus_one == 0 {
                1 // Special case for -1
            } else {
                64 - abs_minus_one.leading_zeros() as i64
            }
        };
        Ok(Value::integer(length))
    } else {
        Err(Box::new(Error::runtime_error(
            "integer-length expects an exact integer argument".to_string(), 
            None
        )))
    }
}

fn primitive_first_set_bit(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "first-set-bit requires exactly 1 argument".to_string(), 
            None
        )));
    }
    
    if let Some(n) = extract_exact_integer(&args[0]) {
        let position = if n == 0 {
            // Special case: first-set-bit of 0 is -1 (no bits set)
            -1
        } else {
            // Find the position of the rightmost set bit (0-indexed)
            // trailing_zeros() gives us exactly what we need
            n.trailing_zeros() as i64
        };
        Ok(Value::integer(position))
    } else {
        Err(Box::new(Error::runtime_error(
            "first-set-bit expects an exact integer argument".to_string(), 
            None
        )))
    }
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

    #[test]
    fn test_bitwise_primitives() {
        // Test bitwise-and
        let args = vec![Value::integer(5), Value::integer(3)]; // 101 & 011 = 001
        let result = primitive_bitwise_and(&args).unwrap();
        assert_eq!(result, Value::integer(1));

        // Test bitwise-and with no args (identity: -1)
        let result = primitive_bitwise_and(&[]).unwrap();
        assert_eq!(result, Value::integer(-1));

        // Test bitwise-ior
        let args = vec![Value::integer(5), Value::integer(3)]; // 101 | 011 = 111
        let result = primitive_bitwise_ior(&args).unwrap();
        assert_eq!(result, Value::integer(7));

        // Test bitwise-ior with no args (identity: 0)
        let result = primitive_bitwise_ior(&[]).unwrap();
        assert_eq!(result, Value::integer(0));

        // Test bitwise-xor
        let args = vec![Value::integer(5), Value::integer(3)]; // 101 ^ 011 = 110
        let result = primitive_bitwise_xor(&args).unwrap();
        assert_eq!(result, Value::integer(6));

        // Test bitwise-not
        let args = vec![Value::integer(5)]; // ~5 = -6 in two's complement
        let result = primitive_bitwise_not(&args).unwrap();
        assert_eq!(result, Value::integer(-6));
    }

    #[test]
    fn test_arithmetic_shift() {
        // Test left shift
        let args = vec![Value::integer(5), Value::integer(2)]; // 5 << 2 = 20
        let result = primitive_arithmetic_shift(&args).unwrap();
        assert_eq!(result, Value::integer(20));

        // Test right shift
        let args = vec![Value::integer(20), Value::integer(-2)]; // 20 >> 2 = 5
        let result = primitive_arithmetic_shift(&args).unwrap();
        assert_eq!(result, Value::integer(5));

        // Test right shift with negative number
        let args = vec![Value::integer(-8), Value::integer(-2)]; // -8 >> 2 = -2
        let result = primitive_arithmetic_shift(&args).unwrap();
        assert_eq!(result, Value::integer(-2));
    }

    #[test]
    fn test_bit_analysis_primitives() {
        // Test bit-count
        let args = vec![Value::integer(7)]; // 111 has 3 bits set
        let result = primitive_bit_count(&args).unwrap();
        assert_eq!(result, Value::integer(3));

        // Test bit-count with zero
        let args = vec![Value::integer(0)];
        let result = primitive_bit_count(&args).unwrap();
        assert_eq!(result, Value::integer(0));

        // Test integer-length
        let args = vec![Value::integer(7)]; // 111 requires 3 bits
        let result = primitive_integer_length(&args).unwrap();
        assert_eq!(result, Value::integer(3));

        // Test integer-length with zero
        let args = vec![Value::integer(0)];
        let result = primitive_integer_length(&args).unwrap();
        assert_eq!(result, Value::integer(0));

        // Test first-set-bit
        let args = vec![Value::integer(8)]; // 1000, first set bit at position 3
        let result = primitive_first_set_bit(&args).unwrap();
        assert_eq!(result, Value::integer(3));

        // Test first-set-bit with zero
        let args = vec![Value::integer(0)];
        let result = primitive_first_set_bit(&args).unwrap();
        assert_eq!(result, Value::integer(-1));
    }

    #[test]
    fn test_bitwise_category() {
        let registry = MinimalPrimitiveRegistry::new();
        let bitwise_prims = registry.primitives_in_category(&MinimalPrimitiveCategory::Bitwise);
        
        // Ensure we have all 8 bitwise primitives
        assert_eq!(bitwise_prims.len(), 8);
        assert!(bitwise_prims.iter().any(|p| p.name == "%bitwise-and"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%bitwise-ior"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%bitwise-xor"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%bitwise-not"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%arithmetic-shift"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%bit-count"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%integer-length"));
        assert!(bitwise_prims.iter().any(|p| p.name == "%first-set-bit"));
    }
}