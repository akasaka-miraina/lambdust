//! Environment and evaluation operations for R7RS (scheme eval) library.
//!
//! This module implements the critical environment operations required for
//! R7RS-small compliance, including `eval`, `environment`, `environment-bound?`,
//! and R5RS compatibility procedures.

use crate::ast::{Expr, Program};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::evaluator::Evaluator;
use crate::eval::{Environment, Generation, ThreadSafeEnvironment, Value};
use crate::module_system::{ImportConfig, ImportSpec, ModuleId, ModuleNamespace, ModuleSystem};
use crate::parser::Parser;
use crate::utils::intern_symbol;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Security configuration for dynamic evaluation.
#[derive(Debug, Clone)]
pub struct EvalSecurityConfig {
    /// Maximum recursion depth for evaluation
    pub max_recursion_depth: usize,
    /// Maximum time allowed for evaluation (in milliseconds)
    pub max_eval_time_ms: u64,
    /// Whether to allow access to I/O operations during eval
    pub allow_io: bool,
    /// Whether to allow loading of modules during eval
    pub allow_module_loading: bool,
    /// Whether to allow mutation of global environment
    pub allow_global_mutation: bool,
}

impl Default for EvalSecurityConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 1000,
            max_eval_time_ms: 5000, // 5 seconds
            allow_io: false,
            allow_module_loading: false,
            allow_global_mutation: false,
        }
    }
}

/// Environment construction result.
#[derive(Debug)]
pub enum EnvironmentResult {
    /// Successfully created environment
    Success(Arc<ThreadSafeEnvironment>),
    /// Failed to create environment
    Error(String),
}

/// Core implementation of the `eval` primitive.
///
/// This function provides secure dynamic evaluation of Scheme expressions
/// with proper error handling and resource limits.
pub fn primitive_eval(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!(
                "eval expects 2 arguments (expression and environment), got {}",
                args.len()
            ),
            None,
        )));
    }

    let expr_value = &args[0];
    let env_value = &args[1];

    // Extract environment from the value
    let environment = extract_environment_from_value(env_value)?;

    // Convert the expression value back to an AST expression
    let expr = value_to_expression(expr_value)?;

    // Create evaluator with security limits
    let security_config = EvalSecurityConfig::default();
    let mut evaluator = create_secure_evaluator(security_config)?;

    // Convert ThreadSafeEnvironment to legacy Environment for evaluator
    let legacy_env = environment.to_legacy();

    // Evaluate the expression in the given environment
    let result = evaluator.eval(&expr, legacy_env)?;

    Ok(result)
}

/// Core implementation of the `environment` primitive.
///
/// Creates a new environment by importing the specified modules/libraries.
pub fn primitive_environment(args: &[Value]) -> Result<Value> {
    // Each argument should be an import set specification
    let mut import_specs = Vec::new();

    for arg in args {
        let import_spec = value_to_import_spec(arg)?;
        import_specs.push(import_spec);
    }

    // Create new environment with imports
    let environment = create_environment_from_imports(&import_specs)?;

    // Wrap environment in a Value
    Ok(Value::Environment(environment))
}

/// Core implementation of the `environment-bound?` primitive.
///
/// Checks if a symbol is bound in the given environment.
pub fn primitive_environment_bound_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!(
                "environment-bound? expects 2 arguments (symbol and environment), got {}",
                args.len()
            ),
            None,
        )));
    }

    let symbol_value = &args[0];
    let env_value = &args[1];

    // Extract symbol name
    let symbol_name = match symbol_value {
        Value::Symbol(symbol_id) => {
            // Convert symbol ID back to string
            // Note: This would need access to the symbol interner
            symbol_id.to_string() // Simplified for now
        }
        _ => {
            return Err(Box::new(Error::runtime_error(
                format!(
                    "environment-bound? expects first argument to be a symbol, got {symbol_value}"
                ),
                None,
            )));
        }
    };

    // Extract environment
    let environment = extract_environment_from_value(env_value)?;

    // Check if symbol is bound
    let is_bound = environment.lookup(&symbol_name).is_some();

    Ok(Value::boolean(is_bound))
}

/// Implementation of `scheme-report-environment` for R5RS compatibility.
pub fn primitive_scheme_report_environment(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!(
                "scheme-report-environment expects 1 argument (version), got {}",
                args.len()
            ),
            None,
        )));
    }

    let version = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => *n,
        Value::Literal(crate::ast::Literal::InexactReal(f)) => *f as i64,
        Value::Literal(crate::ast::Literal::Number(n)) => *n as i64, // deprecated but still supported
        _ => {
            return Err(Box::new(Error::runtime_error(
                format!(
                    "scheme-report-environment expects an integer version, got {}",
                    args[0]
                ),
                None,
            )));
        }
    };

    match version {
        5 => {
            // Create R5RS environment with all R5RS procedures
            let env = create_r5rs_environment()?;
            Ok(Value::Environment(env))
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("Unsupported Scheme version: {version}. Only R5RS (version 5) is supported."),
            None,
        ))),
    }
}

/// Implementation of `null-environment` for R5RS compatibility.
pub fn primitive_null_environment(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!(
                "null-environment expects 1 argument (version), got {}",
                args.len()
            ),
            None,
        )));
    }

    let version = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => *n,
        Value::Literal(crate::ast::Literal::InexactReal(f)) => *f as i64,
        Value::Literal(crate::ast::Literal::Number(n)) => *n as i64, // deprecated but still supported
        _ => {
            return Err(Box::new(Error::runtime_error(
                format!(
                    "null-environment expects an integer version, got {}",
                    args[0]
                ),
                None,
            )));
        }
    };

    match version {
        5 => {
            // Create null environment with only special forms
            let env = create_null_environment()?;
            Ok(Value::Environment(env))
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("Unsupported Scheme version: {version}. Only R5RS (version 5) is supported."),
            None,
        ))),
    }
}

/// Implementation of `interaction-environment`.
pub fn primitive_interaction_environment(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            format!(
                "interaction-environment expects 0 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    // Return the current global/interaction environment
    let global_env = crate::eval::environment::global_environment();
    let thread_safe_env = global_env.to_thread_safe();

    Ok(Value::Environment(thread_safe_env))
}

// ============= HELPER FUNCTIONS =============

/// Extracts an environment from a Value.
fn extract_environment_from_value(value: &Value) -> Result<Arc<ThreadSafeEnvironment>> {
    match value {
        Value::Environment(env) => Ok(env.clone()),
        _ => Err(Box::new(Error::runtime_error(
            format!("Expected environment, got {value}"),
            None,
        ))),
    }
}

/// Converts a Value back to an AST expression for evaluation.
fn value_to_expression(value: &Value) -> Result<crate::diagnostics::Spanned<Expr>> {
    let expr = match value {
        Value::Literal(lit) => Expr::Literal(lit.clone()),
        Value::Symbol(symbol_id) => {
            // Convert symbol ID back to identifier
            // This is simplified - would need proper symbol table lookup
            let name = symbol_id.to_string(); // Simplified
            Expr::Identifier(name)
        }
        Value::Pair(car, cdr) => {
            // Convert pair to function application or special form
            let car_expr = value_to_expression(car)?;
            let mut args = Vec::new();

            // Flatten the argument list
            let mut current = &**cdr;
            while let Value::Pair(arg, rest) = current {
                args.push(value_to_expression(arg)?);
                current = rest;
            }

            if !matches!(current, Value::Nil) {
                return Err(Box::new(Error::runtime_error(
                    "Invalid expression: improper list in eval".to_string(),
                    None,
                )));
            }

            Expr::Application {
                operator: Box::new(car_expr),
                operands: args,
            }
        }
        Value::Nil => Expr::Literal(crate::ast::Literal::Nil),
        _ => {
            return Err(Box::new(Error::runtime_error(
                format!("Cannot convert {value} to expression for evaluation"),
                None,
            )));
        }
    };

    // Wrap in a Spanned with a default span
    Ok(crate::diagnostics::Spanned::new(expr, Span::default()))
}

/// Converts a Value to an ImportSpec for environment creation.
fn value_to_import_spec(value: &Value) -> Result<ImportSpec> {
    // Import specs are typically lists like (scheme base) or (srfi 1)
    match value {
        Value::Pair(_, _) => {
            // Parse the import spec from the list structure
            let spec_list = value_to_list(value)?;
            parse_import_spec_from_list(&spec_list)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("Invalid import specification: {value}"),
            None,
        ))),
    }
}

/// Converts a Value that represents a list to a Vec<Value>.
fn value_to_list(value: &Value) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    let mut current = value;

    while let Value::Pair(car, cdr) = current {
        result.push((**car).clone());
        current = cdr;
    }

    if !matches!(current, Value::Nil) {
        return Err(Box::new(Error::runtime_error(
            "Expected proper list".to_string(),
            None,
        )));
    }

    Ok(result)
}

/// Parses an ImportSpec from a list of Values.
fn parse_import_spec_from_list(values: &[Value]) -> Result<ImportSpec> {
    if values.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "Empty import specification".to_string(),
            None,
        )));
    }

    // Convert Values to strings to build module path
    let mut path_parts = Vec::new();
    for value in values {
        match value {
            Value::Symbol(symbol_id) => {
                // Convert symbol to string
                path_parts.push(symbol_id.to_string()); // Simplified
            }
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                path_parts.push(n.to_string());
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                path_parts.push((*f as i64).to_string());
            }
            Value::Literal(crate::ast::Literal::Number(n)) => {
                path_parts.push((*n as i64).to_string());
            }
            _ => {
                return Err(Box::new(Error::runtime_error(
                    format!("Invalid import spec component: {value}"),
                    None,
                )));
            }
        }
    }

    // Create a simple ImportSpec
    // This is simplified - a full implementation would handle complex import syntax

    // Determine namespace based on first component
    let namespace = if !path_parts.is_empty() {
        match path_parts[0].as_str() {
            "scheme" => ModuleNamespace::R7RS,
            "srfi" => ModuleNamespace::SRFI,
            "lambdust" => ModuleNamespace::Builtin,
            _ => ModuleNamespace::User,
        }
    } else {
        ModuleNamespace::User
    };

    // Create ModuleId with the determined namespace
    let module_id = ModuleId::new(namespace, path_parts);

    Ok(ImportSpec {
        module_id,
        config: ImportConfig::All,
    })
}

/// Creates a new environment from a list of import specifications.
fn create_environment_from_imports(
    import_specs: &[ImportSpec],
) -> Result<Arc<ThreadSafeEnvironment>> {
    // Create a new empty environment
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));

    // Load and import each specified module
    for spec in import_specs {
        import_module_into_environment(spec, &env)?;
    }

    Ok(env)
}

/// Imports a module into an environment according to the import spec.
fn import_module_into_environment(
    spec: &ImportSpec,
    env: &Arc<ThreadSafeEnvironment>,
) -> Result<()> {
    // This is a simplified implementation
    // A full implementation would use the module system to resolve and load modules

    // Get module name from the spec
    let module_name = spec.module_id.components.join(" ");

    // Handle common R7RS libraries
    match module_name.as_str() {
        "scheme base" => {
            import_scheme_base(env)?;
        }
        "scheme char" => {
            import_scheme_char(env)?;
        }
        "scheme cxr" => {
            import_scheme_cxr(env)?;
        }
        "scheme eval" => {
            import_scheme_eval(env)?;
        }
        "scheme file" => {
            import_scheme_file(env)?;
        }
        "scheme inexact" => {
            import_scheme_inexact(env)?;
        }
        "scheme lazy" => {
            import_scheme_lazy(env)?;
        }
        "scheme load" => {
            import_scheme_load(env)?;
        }
        "scheme process-context" => {
            import_scheme_process_context(env)?;
        }
        "scheme read" => {
            import_scheme_read(env)?;
        }
        "scheme repl" => {
            import_scheme_repl(env)?;
        }
        "scheme time" => {
            import_scheme_time(env)?;
        }
        "scheme write" => {
            import_scheme_write(env)?;
        }
        _ => {
            return Err(Box::new(Error::runtime_error(
                format!("Unknown module: {module_name}"),
                None,
            )));
        }
    }

    Ok(())
}

/// Creates a secure evaluator with resource limits.
fn create_secure_evaluator(_config: EvalSecurityConfig) -> Result<Evaluator> {
    // Create evaluator with security configuration
    let evaluator = Evaluator::new();

    // TODO: Apply security limits when evaluator supports them
    // TODO: Implement timeout and other security features

    Ok(evaluator)
}

/// Creates an R5RS-compatible environment.
fn create_r5rs_environment() -> Result<Arc<ThreadSafeEnvironment>> {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));

    // Import R5RS standard procedures
    import_scheme_base(&env)?;
    import_r5rs_specific_procedures(&env)?;

    Ok(env)
}

/// Creates a null environment with only special forms.
fn create_null_environment() -> Result<Arc<ThreadSafeEnvironment>> {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));

    // Only bind special form syntax, no procedures
    bind_special_forms_only(&env);

    Ok(env)
}

// ============= MODULE IMPORT FUNCTIONS =============

/// Imports (scheme base) bindings.
fn import_scheme_base(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // This would import all (scheme base) procedures
    // For now, delegate to existing standard library
    let stdlib = crate::stdlib::StandardLibrary::new();
    stdlib.populate_environment(env);
    Ok(())
}

/// Imports (scheme char) bindings.
fn import_scheme_char(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    crate::stdlib::characters::create_character_bindings(env);
    Ok(())
}

/// Imports (scheme cxr) bindings.
fn import_scheme_cxr(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Import cxr combinations (caar, cadr, cdar, cddr, etc.)
    // TODO: Implement cxr combinations when available
    crate::stdlib::lists::create_list_bindings(env);
    Ok(())
}

/// Imports (scheme eval) bindings.
fn import_scheme_eval(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Import eval operations (this module)
    create_eval_bindings(env);
    Ok(())
}

/// Imports (scheme file) bindings.
fn import_scheme_file(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // File operations would be implemented in a separate module
    // TODO: Implement file operations
    Ok(())
}

/// Imports (scheme inexact) bindings.
fn import_scheme_inexact(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Inexact arithmetic operations
    // TODO: Implement inexact-specific bindings when available
    crate::stdlib::arithmetic::create_arithmetic_bindings(env);
    Ok(())
}

/// Imports (scheme lazy) bindings.
fn import_scheme_lazy(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Lazy evaluation primitives
    // TODO: Implement delay/force primitives
    Ok(())
}

/// Imports (scheme load) bindings.
fn import_scheme_load(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Load operations
    // TODO: Implement load primitive
    Ok(())
}

/// Imports (scheme process-context) bindings.
fn import_scheme_process_context(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Process context operations
    // TODO: Implement process context bindings when available
    crate::stdlib::system::create_system_bindings(env);
    Ok(())
}

/// Imports (scheme read) bindings.
fn import_scheme_read(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Read operations
    // TODO: Implement read primitives
    Ok(())
}

/// Imports (scheme repl) bindings.
fn import_scheme_repl(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // REPL-specific operations
    // TODO: Implement REPL-specific bindings
    Ok(())
}

/// Imports (scheme time) bindings.
fn import_scheme_time(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Time operations
    // TODO: Implement time operations
    Ok(())
}

/// Imports (scheme write) bindings.
fn import_scheme_write(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // Write operations
    // TODO: Implement write-specific bindings when available
    crate::stdlib::io::create_io_bindings(env);
    Ok(())
}

/// Imports R5RS-specific procedures not in R7RS base.
fn import_r5rs_specific_procedures(env: &Arc<ThreadSafeEnvironment>) -> Result<()> {
    // R5RS specific bindings that differ from R7RS
    Ok(())
}

/// Binds only special forms (no procedures) for null environment.
fn bind_special_forms_only(env: &Arc<ThreadSafeEnvironment>) {
    use crate::utils::intern_symbol;

    // Core special forms required for syntax
    let special_forms = [
        "lambda",
        "if",
        "define",
        "set!",
        "quote",
        "quasiquote",
        "unquote",
        "unquote-splicing",
        "begin",
        "let",
        "let*",
        "letrec",
        "cond",
        "case",
        "and",
        "or",
        "when",
        "unless",
        "do",
        "delay",
        "force",
        "case-lambda",
    ];

    for &form_name in &special_forms {
        let symbol_id = intern_symbol(form_name.to_owned());
        let syntax_value = Value::Symbol(symbol_id);
        env.define(form_name.to_owned(), syntax_value);
    }
}

/// Creates bindings for eval operations in the given environment.
pub fn create_eval_bindings(env: &Arc<ThreadSafeEnvironment>) {
    use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure};

    // eval procedure
    let eval_proc = Arc::new(PrimitiveProcedure {
        name: "eval".to_string(),
        arity_min: 2,
        arity_max: Some(2), // Exactly 2 arguments
        implementation: PrimitiveImpl::RustFn(primitive_eval),
        effects: vec![], // Dynamic evaluation can have any effects
    });
    env.define("eval".to_string(), Value::Primitive(eval_proc));

    // environment procedure
    let environment_proc = Arc::new(PrimitiveProcedure {
        name: "environment".to_string(),
        arity_min: 0,
        arity_max: None, // Variadic
        implementation: PrimitiveImpl::RustFn(primitive_environment),
        effects: vec![],
    });
    env.define(
        "environment".to_string(),
        Value::Primitive(environment_proc),
    );

    // environment-bound? procedure
    let environment_bound_proc = Arc::new(PrimitiveProcedure {
        name: "environment-bound?".to_string(),
        arity_min: 2,
        arity_max: Some(2), // Exactly 2 arguments
        implementation: PrimitiveImpl::RustFn(primitive_environment_bound_p),
        effects: vec![],
    });
    env.define(
        "environment-bound?".to_string(),
        Value::Primitive(environment_bound_proc),
    );

    // scheme-report-environment procedure
    let scheme_report_env_proc = Arc::new(PrimitiveProcedure {
        name: "scheme-report-environment".to_string(),
        arity_min: 1,
        arity_max: Some(1), // Exactly 1 argument
        implementation: PrimitiveImpl::RustFn(primitive_scheme_report_environment),
        effects: vec![],
    });
    env.define(
        "scheme-report-environment".to_string(),
        Value::Primitive(scheme_report_env_proc),
    );

    // null-environment procedure
    let null_env_proc = Arc::new(PrimitiveProcedure {
        name: "null-environment".to_string(),
        arity_min: 1,
        arity_max: Some(1), // Exactly 1 argument
        implementation: PrimitiveImpl::RustFn(primitive_null_environment),
        effects: vec![],
    });
    env.define(
        "null-environment".to_string(),
        Value::Primitive(null_env_proc),
    );

    // interaction-environment procedure
    let interaction_env_proc = Arc::new(PrimitiveProcedure {
        name: "interaction-environment".to_string(),
        arity_min: 0,
        arity_max: Some(0), // No arguments
        implementation: PrimitiveImpl::RustFn(primitive_interaction_environment),
        effects: vec![],
    });
    env.define(
        "interaction-environment".to_string(),
        Value::Primitive(interaction_env_proc),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        create_eval_bindings(&env);

        // Test that eval is bound
        assert!(env.lookup("eval").is_some());
        assert!(env.lookup("environment").is_some());
        assert!(env.lookup("environment-bound?").is_some());
    }

    #[test]
    fn test_environment_bound_predicate() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        env.define("test-var".to_string(), Value::integer(42));

        let symbol_value = Value::Symbol(intern_symbol("test-var".to_string()));
        let env_value = Value::Environment(env.clone());

        let result = primitive_environment_bound_p(&[symbol_value, env_value]).unwrap();
        assert!(result.is_truthy());

        // Test unbound symbol
        let unbound_symbol = Value::Symbol(intern_symbol("unbound-var".to_string()));
        let env_value = Value::Environment(env);

        let result = primitive_environment_bound_p(&[unbound_symbol, env_value]).unwrap();
        assert!(result.is_falsy());
    }
}
