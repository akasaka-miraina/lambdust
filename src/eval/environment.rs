//! Environment and variable binding implementation.
//!
//! This module provides the environment system for Lambdust, which handles
//! lexical scoping, variable binding, and proper closure semantics.

use super::{Generation, Value};
use crate::diagnostics::{Error, Result};
use std::sync::Arc;
use std::rc::Rc;

/// A lexical environment that maintains variable bindings.
///
/// Environments form a chain through parent pointers, implementing
/// proper lexical scoping. Each environment has a generation number
/// for garbage collection optimization.
pub type Environment = super::value::Environment;

/// Builder for creating environments with standard bindings.
pub struct EnvironmentBuilder {
    environment: Rc<Environment>,
    #[allow(dead_code)]
    generation: Generation,
}

impl EnvironmentBuilder {
    /// Creates a new environment builder.
    pub fn new(generation: Generation) -> Self {
        let env = Rc::new(Environment::new(None, generation));
        Self {
            environment: env,
            generation,
        }
    }

    /// Creates a new environment builder with a parent.
    pub fn with_parent(parent: Rc<Environment>, generation: Generation) -> Self {
        let env = Rc::new(Environment::new(Some(parent), generation));
        Self {
            environment: env,
            generation,
        }
    }

    /// Adds a binding to the environment.
    pub fn bind(self, name: impl Into<String>, value: Value) -> Self {
        self.environment.define(name.into(), value);
        self
    }

    /// Adds a primitive procedure binding.
    pub fn bind_primitive(
        self, 
        name: impl Into<String>, 
        proc: super::value::PrimitiveProcedure
    ) -> Self {
        let value = Value::Primitive(Arc::new(proc));
        self.bind(name, value)
    }

    /// Builds the final environment.
    pub fn build(self) -> Rc<Environment> {
        self.environment
    }
}

/// Gets or creates the global environment.
/// Each thread gets its own copy since Rc<Environment> is not thread-safe.
pub fn global_environment() -> Rc<Environment> {
    thread_local! {
        static GLOBAL_ENV: std::cell::OnceCell<Rc<Environment>> = const { std::cell::OnceCell::new() };
    }
    
    GLOBAL_ENV.with(|cell| {
        cell.get_or_init(create_global_environment).clone()
    })
}

/// Creates the global environment with standard bindings.
fn create_global_environment() -> Rc<Environment> {
    // Start with an empty environment
    let env = Rc::new(Environment::new(None, 0));
    
    // Add basic values first - these are essential and should never fail
    env.define("true".to_string(), Value::t());
    env.define("false".to_string(), Value::f());
    env.define("null".to_string(), Value::Nil);
    
    // Add R7RS-small special forms as identifiers for macro expansion
    // These are needed for macro templates to reference basic syntax
    bind_special_forms_as_identifiers(&env);
    
    // Convert to ThreadSafeEnvironment to use StandardLibrary
    let thread_safe_env = env.to_thread_safe();
    
    // Populate with complete standard library (same as multithreaded runtime)
    // This ensures both single-threaded and multi-threaded paths have identical environments
    let stdlib = crate::stdlib::StandardLibrary::new();
    stdlib.populate_environment(&thread_safe_env);
    
    // Convert back to legacy Environment for single-threaded use
    thread_safe_env.to_legacy()
}

/// Binds R7RS-small special forms as identifiers in the environment.
/// This allows macros to reference basic syntax elements like 'lambda', 'if', etc.
/// These are not callable procedures but syntactic identifiers for macro expansion.
fn bind_special_forms_as_identifiers(env: &Rc<Environment>) {
    use crate::utils::intern_symbol;
    
    // Core special forms required for macro expansion
    let special_forms = [
        "lambda", "if", "define", "set!", "quote", "quasiquote", "unquote", "unquote-splicing",
        "begin", "let", "let*", "letrec", "cond", "case", "and", "or",
        "when", "unless", "do", "delay", "force", "case-lambda",
        "make-promise", "promise-force", // for delay/force implementation
    ];
    
    for &form_name in &special_forms {
        // Create a special syntax value that indicates this is a special form identifier
        let symbol_id = intern_symbol(form_name.to_string());
        let syntax_value = Value::Symbol(symbol_id);
        env.define(form_name.to_string(), syntax_value);
    }
    
    // Also bind some essential procedures that macros might need to reference
    // These will be overwritten by actual implementations later if they exist
    env.define("apply".to_string(), Value::Symbol(intern_symbol("apply".to_string())));
    env.define("list".to_string(), Value::Symbol(intern_symbol("list".to_string())));
    env.define("cons".to_string(), Value::Symbol(intern_symbol("cons".to_string())));
    env.define("car".to_string(), Value::Symbol(intern_symbol("car".to_string())));
    env.define("cdr".to_string(), Value::Symbol(intern_symbol("cdr".to_string())));
    env.define("null?".to_string(), Value::Symbol(intern_symbol("null?".to_string())));
    env.define("length".to_string(), Value::Symbol(intern_symbol("length".to_string())));
    env.define("error".to_string(), Value::Symbol(intern_symbol("error".to_string())));
    env.define("not".to_string(), Value::Symbol(intern_symbol("not".to_string())));
    env.define("memv".to_string(), Value::Symbol(intern_symbol("memv".to_string())));
}


// ============= PRIMITIVE IMPLEMENTATIONS =============

/// Addition primitive (+).
pub fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }

    let mut result = 0.0;
    for arg in args {
        match arg.as_number() {
            Some(n) => result += n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {arg}"),
                None,
            ))),
        }
    }

    Ok(Value::number(result))
}

/// Subtraction primitive (-).
#[allow(dead_code)]
fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "- requires at least one argument".to_string(),
            None,
        )));
    }

    if args.len() == 1 {
        // Unary minus
        match args[0].as_number() {
            Some(n) => Ok(Value::number(-n)),
            None => Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", args[0]),
                None,
            ))),
        }
    } else {
        // Binary and n-ary minus
        let mut result = match args[0].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", args[0]),
                None,
            ))),
        };

        for arg in &args[1..] {
            match arg.as_number() {
                Some(n) => result -= n,
                None => return Err(Box::new(Error::runtime_error(
                    format!("Expected number, got {arg}"),
                    None,
                ))),
            }
        }

        Ok(Value::number(result))
    }
}

/// Multiplication primitive (*).
#[allow(dead_code)]
fn primitive_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }

    let mut result = 1.0;
    for arg in args {
        match arg.as_number() {
            Some(n) => result *= n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {arg}"),
                None,
            ))),
        }
    }

    Ok(Value::number(result))
}

/// Division primitive (/).
#[allow(dead_code)]
fn primitive_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "/ requires at least one argument".to_string(),
            None,
        )));
    }

    if args.len() == 1 {
        // Reciprocal
        match args[0].as_number() {
            Some(n) => {
                if n == 0.0 {
                    Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)))
                } else {
                    Ok(Value::number(1.0 / n))
                }
            }
            None => Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", args[0]),
                None,
            ))),
        }
    } else {
        // Binary and n-ary division
        let mut result = match args[0].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", args[0]),
                None,
            ))),
        };

        for arg in &args[1..] {
            match arg.as_number() {
                Some(n) => {
                    if n == 0.0 {
                        return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)));
                    }
                    result /= n;
                }
                None => return Err(Box::new(Error::runtime_error(
                    format!("Expected number, got {arg}"),
                    None,
                ))),
            }
        }

        Ok(Value::number(result))
    }
}

/// Numeric equality primitive (=).
#[allow(dead_code)]
fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            "= requires at least two arguments".to_string(),
            None,
        )));
    }

    let first = match args[0].as_number() {
        Some(n) => n,
        None => return Err(Box::new(Error::runtime_error(
            format!("Expected number, got {}", args[0]),
            None,
        ))),
    };

    for arg in &args[1..] {
        match arg.as_number() {
            Some(n) => {
                if (first - n).abs() > f64::EPSILON {
                    return Ok(Value::f());
                }
            }
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {arg}"),
                None,
            ))),
        }
    }

    Ok(Value::t())
}

/// Less than primitive (<).
#[allow(dead_code)]
fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            "< requires at least two arguments".to_string(),
            None,
        )));
    }

    for window in args.windows(2) {
        let a = match window[0].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[0]),
                None,
            ))),
        };

        let b = match window[1].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[1]),
                None,
            ))),
        };

        if a >= b {
            return Ok(Value::f());
        }
    }

    Ok(Value::t())
}

/// Greater than primitive (>).
#[allow(dead_code)]
fn primitive_greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            "> requires at least two arguments".to_string(),
            None,
        )));
    }

    for window in args.windows(2) {
        let a = match window[0].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[0]),
                None,
            ))),
        };

        let b = match window[1].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[1]),
                None,
            ))),
        };

        if a <= b {
            return Ok(Value::f());
        }
    }

    Ok(Value::t())
}

/// Cons primitive (cons).
#[allow(dead_code)]
fn primitive_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("cons expects 2 arguments, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

/// Car primitive (car).
#[allow(dead_code)]
fn primitive_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("car expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Pair(car, _) => Ok((**car).clone()),
        _ => Err(Box::new(Error::runtime_error(
            format!("car expects a pair, got {}", args[0]),
            None,
        ))),
    }
}

/// Cdr primitive (cdr).
#[allow(dead_code)]
fn primitive_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("cdr expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Pair(_, cdr) => Ok((**cdr).clone()),
        _ => Err(Box::new(Error::runtime_error(
            format!("cdr expects a pair, got {}", args[0]),
            None,
        ))),
    }
}

// Type predicate primitives

#[allow(dead_code)]
fn primitive_number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("number? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(args[0].is_number()))
}

#[allow(dead_code)]
fn primitive_string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("string? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(args[0].is_string()))
}

#[allow(dead_code)]
fn primitive_symbol_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("symbol? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(args[0].is_symbol()))
}

#[allow(dead_code)]
fn primitive_pair_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("pair? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(args[0].is_pair()))
}

#[allow(dead_code)]
fn primitive_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("null? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(args[0].is_nil()))
}

#[allow(dead_code)]
fn primitive_procedure_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("procedure? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(args[0].is_procedure()))
}

// I/O primitives

#[allow(dead_code)]
fn primitive_display(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("display expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let value = &args[0];
    
    // For now, just print to stdout
    // TODO: Handle optional port argument
    print!("{value}");
    
    Ok(Value::Unspecified)
}

#[allow(dead_code)]
fn primitive_newline(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(Error::runtime_error(
            format!("newline expects 0 or 1 arguments, got {}", args.len()),
            None,
        )));
    }

    // For now, just print to stdout
    // TODO: Handle optional port argument
    println!();
    
    Ok(Value::Unspecified)
}

// Additional comparison primitives

#[allow(dead_code)]
fn primitive_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            "<= requires at least two arguments".to_string(),
            None,
        )));
    }
    for window in args.windows(2) {
        let a = match window[0].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[0]),
                None,
            ))),
        };
        let b = match window[1].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[1]),
                None,
            ))),
        };
        if a > b {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

#[allow(dead_code)]
fn primitive_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            ">= requires at least two arguments".to_string(),
            None,
        )));
    }
    for window in args.windows(2) {
        let a = match window[0].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[0]),
                None,
            ))),
        };
        let b = match window[1].as_number() {
            Some(n) => n,
            None => return Err(Box::new(Error::runtime_error(
                format!("Expected number, got {}", window[1]),
                None,
            ))),
        };
        if a < b {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

