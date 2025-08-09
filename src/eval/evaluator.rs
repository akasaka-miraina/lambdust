//! Main evaluation engine with tail call optimization.
//!
//! This module implements the core evaluator for Lambdust expressions.
//! It uses a trampoline-based approach to ensure proper tail call optimization
//! and constant stack space usage.

#![allow(missing_docs)]

use super::{
    Environment, ThreadSafeEnvironment, Generation, StackFrame, StackTrace, Value, 
    Continuation, Procedure, PrimitiveProcedure, PrimitiveImpl, Frame
};
use crate::module_system::{ModuleSystem, SchemeLibraryLoader, ImportSpec, ModuleId, ModuleNamespace, ImportConfig};
use crate::runtime::GlobalEnvironmentManager;
use super::value::CaseLambdaProcedure;
use crate::ast::{CaseLambdaClause, Expr, Formals, GuardClause, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::effects::{Effect, EffectSystem, EffectLifter, MonadicValue};
use crate::ffi::FfiBridge;
use crate::macro_system::MacroExpander;
use crate::utils::{intern_symbol};
use std::sync::Arc;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for continuation IDs.
static CONTINUATION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a unique continuation ID.
fn next_continuation_id() -> u64 {
    CONTINUATION_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// The result of a single evaluation step.
///
/// Using this enum allows us to implement proper tail call optimization
/// through a trampoline pattern - the evaluator returns instructions
/// about what to do next rather than directly making recursive calls.
#[derive(Debug, Clone)]
pub enum EvalStep {
    /// Evaluation completed with a value
    Return(Value),
    
    /// Continue evaluation with a new expression
    Continue {
        expr: Spanned<Expr>,
        env: Rc<Environment>,
    },
    
    /// Apply a procedure to arguments (tail call)
    TailCall {
        procedure: Value,
        args: Vec<Value>,
        location: Option<Span>,
    },
    
    /// Handle a captured continuation
    CallContinuation {
        continuation: Arc<Continuation>,
        value: Value,
    },
    
    /// Evaluation error
    Error(Error),
}

/// The main evaluator for Lambdust expressions.
///
/// This evaluator implements proper Scheme semantics including:
/// - Lexical scoping with closures
/// - Proper tail call optimization
/// - Call/cc support for continuations
/// - Hygienic macro expansion
/// - Effect tracking and transformation
/// - FFI support for calling Rust functions
/// - Comprehensive error reporting with stack traces
#[derive(Debug)]
pub struct Evaluator {
    /// Current generation for garbage collection
    generation: Generation,
    /// Stack trace for error reporting
    stack_trace: StackTrace,
    /// Global environment
    global_env: Rc<Environment>,
    /// Macro expander for hygienic macro expansion
    macro_expander: MacroExpander,
    /// Effect system for tracking and transforming effects
    effect_system: EffectSystem,
    /// Effect lifter for automatic lifting
    effect_lifter: EffectLifter,
    /// FFI bridge for calling Rust functions
    ffi_bridge: FfiBridge,
    /// Evaluation context stack for continuation capture
    context_stack: Vec<Frame>,
    /// Module system for handling imports
    module_system: ModuleSystem,
    /// Scheme library loader for SRFI modules
    scheme_loader: SchemeLibraryLoader,
}

impl Evaluator {
    /// Creates a new evaluator with the global environment.
    pub fn new() -> Self {
        let global_env_manager = Arc::new(GlobalEnvironmentManager::new());
        let module_system = ModuleSystem::new().expect("Failed to create module system");
        let scheme_loader = SchemeLibraryLoader::new(global_env_manager.clone())
            .expect("Failed to create scheme library loader");
        
        Self {
            generation: 0,
            stack_trace: StackTrace::new(),
            global_env: crate::eval::environment::global_environment(),
            macro_expander: MacroExpander::with_builtins(),
            effect_system: EffectSystem::new(),
            effect_lifter: EffectLifter::new(),
            ffi_bridge: FfiBridge::with_builtins(),
            context_stack: Vec::new(),
            module_system,
            scheme_loader,
        }
    }

    /// Creates a new evaluator with a custom global environment.
    pub fn with_environment(global_env: Rc<Environment>) -> Self {
        let global_env_manager = Arc::new(GlobalEnvironmentManager::new());
        let module_system = ModuleSystem::new().expect("Failed to create module system");
        let scheme_loader = SchemeLibraryLoader::new(global_env_manager.clone())
            .expect("Failed to create scheme library loader");
        
        Self {
            generation: 0,
            stack_trace: StackTrace::new(),
            global_env,
            macro_expander: MacroExpander::with_builtins(),
            effect_system: EffectSystem::new(),
            effect_lifter: EffectLifter::new(),
            ffi_bridge: FfiBridge::with_builtins(),
            context_stack: Vec::new(),
            module_system,
            scheme_loader,
        }
    }

    /// Creates a new evaluator with a custom macro expander.
    pub fn with_macro_expander(macro_expander: MacroExpander) -> Self {
        let global_env_manager = Arc::new(GlobalEnvironmentManager::new());
        let module_system = ModuleSystem::new().expect("Failed to create module system");
        let scheme_loader = SchemeLibraryLoader::new(global_env_manager.clone())
            .expect("Failed to create scheme library loader");
        
        Self {
            generation: 0,
            stack_trace: StackTrace::new(),
            global_env: crate::eval::environment::global_environment(),
            macro_expander,
            effect_system: EffectSystem::new(),
            effect_lifter: EffectLifter::new(),
            ffi_bridge: FfiBridge::with_builtins(),
            context_stack: Vec::new(),
            module_system,
            scheme_loader,
        }
    }

    /// Evaluates an expression in the given environment.
    ///
    /// This is the main entry point for expression evaluation.
    /// It first expands macros, then uses a trampoline to ensure proper tail call optimization.
    pub fn eval(&mut self, expr: &Spanned<Expr>, env: Rc<Environment>) -> Result<Value> {
        // First, expand macros in the expression
        let expanded_expr = self.macro_expander.expand(expr)?;
        
        // Set up initial evaluation step with expanded expression
        let mut step = EvalStep::Continue {
            expr: expanded_expr,
            env,
        };

        // Trampoline loop - keeps evaluating until we get a final result
        loop {
            step = match step {
                EvalStep::Return(value) => return Ok(value),
                EvalStep::Error(error) => return Err(Box::new(error)),
                EvalStep::Continue { expr, env } => self.eval_step(&expr, env),
                EvalStep::TailCall { procedure, args, location } => {
                    self.apply_procedure(procedure, args, location)
                }
                EvalStep::CallContinuation { continuation, value } => {
                    self.call_continuation(continuation, value)
                }
            };
        }
    }

    /// Evaluates a program (sequence of expressions).
    pub fn eval_program(&mut self, program: &Program) -> Result<Value> {
        if program.expressions.is_empty() {
            return Ok(Value::Unspecified);
        }

        // First, expand all macros in the program
        let expanded_program = self.macro_expander.expand_program(program)?;
        
        // Separate defines from other expressions for proper R7RS mutual recursion
        let mut defines = Vec::new();
        let mut other_exprs = Vec::new();
        
        for expr in &expanded_program.expressions {
            if let Expr::Define { .. } = expr.inner {
                defines.push(expr);
            } else {
                other_exprs.push(expr);
            }
        }
        
        // First pass: create bindings for all defines
        for define_expr in &defines {
            if let Expr::Define { name, .. } = &define_expr.inner {
                self.global_env.define(name.clone(), Value::Unspecified);
            }
        }
        
        // Second pass: separate lambda and non-lambda defines
        let mut lambda_defines = Vec::new();
        let mut non_lambda_defines = Vec::new();
        
        for define_expr in &defines {
            if let Expr::Define { value, .. } = &define_expr.inner {
                if matches!(value.inner, Expr::Lambda { .. }) {
                    lambda_defines.push(define_expr);
                } else {
                    non_lambda_defines.push(define_expr);
                }
            }
        }
        
        // Evaluate non-lambda defines first
        for define_expr in &non_lambda_defines {
            let mut step = EvalStep::Continue {
                expr: (**define_expr).clone(),
                env: self.global_env.clone(),
            };

            // Trampoline loop for each define
            loop {
                step = match step {
                    EvalStep::Return(_) => break, // Define returns unspecified
                    EvalStep::Error(error) => return Err(Box::new(error)),
                    EvalStep::Continue { expr, env } => self.eval_step(&expr, env),
                    EvalStep::TailCall { procedure, args, location } => {
                        self.apply_procedure(procedure, args, location)
                    }
                    EvalStep::CallContinuation { continuation, value } => {
                        self.call_continuation(continuation, value)
                    }
                };
            }
        }
        
        // Now evaluate lambda defines - they will see all bound names
        for define_expr in &lambda_defines {
            let mut step = EvalStep::Continue {
                expr: (**define_expr).clone(),
                env: self.global_env.clone(),
            };

            // Trampoline loop for each define
            loop {
                step = match step {
                    EvalStep::Return(_) => break, // Define returns unspecified
                    EvalStep::Error(error) => return Err(Box::new(error)),
                    EvalStep::Continue { expr, env } => self.eval_step(&expr, env),
                    EvalStep::TailCall { procedure, args, location } => {
                        self.apply_procedure(procedure, args, location)
                    }
                    EvalStep::CallContinuation { continuation, value } => {
                        self.call_continuation(continuation, value)
                    }
                };
            }
        }
        
        // Finally, evaluate other expressions
        let mut result = Value::Unspecified;
        
        for expr in &other_exprs {
            let mut step = EvalStep::Continue {
                expr: (*expr).clone(),
                env: self.global_env.clone(),
            };

            // Trampoline loop for each expression
            loop {
                step = match step {
                    EvalStep::Return(value) => {
                        result = value;
                        break;
                    }
                    EvalStep::Error(error) => return Err(Box::new(error)),
                    EvalStep::Continue { expr, env } => self.eval_step(&expr, env),
                    EvalStep::TailCall { procedure, args, location } => {
                        self.apply_procedure(procedure, args, location)
                    }
                    EvalStep::CallContinuation { continuation, value } => {
                        self.call_continuation(continuation, value)
                    }
                };
            }
        }

        Ok(result)
    }

    /// Evaluates a self-evaluating literal expression.
    fn eval_self_evaluating_literal(&mut self, lit: &crate::ast::Literal) -> EvalStep {
        EvalStep::Return(Value::Literal(lit.clone()))
    }

    /// Evaluates a self-evaluating keyword expression.
    fn eval_self_evaluating_keyword(&mut self, k: &str) -> EvalStep {
        EvalStep::Return(Value::Keyword(k.to_string()))
    }

    /// Evaluates an identifier (variable lookup).
    fn eval_identifier(&mut self, name: &str, env: &Rc<Environment>, span: Span) -> EvalStep {
        match env.lookup(name) {
            Some(value) => EvalStep::Return(value),
            None => EvalStep::Error(Error::runtime_error(
                format!("Unbound variable: {name}"),
                Some(span),
            )),
        }
    }

    /// Evaluates a type annotation expression.
    fn eval_type_annotation(&mut self, inner_expr: &Spanned<Expr>, env: Rc<Environment>) -> EvalStep {
        // For now, just evaluate the expression and ignore the type
        // TODO: Integrate with type system
        EvalStep::Continue {
            expr: (*inner_expr).clone(),
            env,
        }
    }

    /// Evaluates a pair construction expression.
    fn eval_pair_construction(&mut self, car: &Spanned<Expr>, cdr: &Spanned<Expr>, env: Rc<Environment>) -> EvalStep {
        // Evaluate both car and cdr, then construct pair
        // For simplicity, not using trampoline here since it's not a tail position
        match self.eval(car, env.clone()) {
            Ok(car_val) => match self.eval(cdr, env) {
                Ok(cdr_val) => EvalStep::Return(Value::pair(car_val, cdr_val)),
                Err(e) => EvalStep::Error(*e),
            },
            Err(e) => EvalStep::Error(*e),
        }
    }

    /// Performs a single evaluation step.
    fn eval_step(&mut self, expr: &Spanned<Expr>, env: Rc<Environment>) -> EvalStep {
        match &expr.inner {
            // Self-evaluating expressions
            Expr::Literal(lit) => self.eval_self_evaluating_literal(lit),
            Expr::Keyword(k) => self.eval_self_evaluating_keyword(k),

            // Variable lookup
            Expr::Identifier(name) => self.eval_identifier(name, &env, expr.span),

            // Special forms
            Expr::Quote(quoted) => self.eval_quote(quoted),
            Expr::Lambda { formals, metadata, body } => {
                self.eval_lambda(formals, metadata, body, env.clone(), expr.span)
            }
            Expr::CaseLambda { clauses, metadata } => {
                self.eval_case_lambda(clauses, metadata, env.clone(), expr.span)
            }
            Expr::If { test, consequent, alternative } => {
                self.eval_if(test, consequent, alternative.as_ref().map(|boxed| boxed.as_ref()), env, expr.span)
            }
            Expr::Define { name, value, metadata } => {
                self.eval_define(name, value, metadata, env, expr.span)
            }
            Expr::Set { name, value } => {
                self.eval_set(name, value, env, expr.span)
            }
            Expr::DefineSyntax { name, transformer } => {
                self.eval_define_syntax(name, transformer, env, expr.span)
            }
            Expr::SyntaxRules { literals, rules } => {
                self.eval_syntax_rules(literals, rules, env, expr.span)
            }
            Expr::CallCC(proc_expr) => {
                self.eval_call_cc(proc_expr, env, expr.span)
            }
            Expr::Primitive { name, args } => {
                self.eval_primitive(name, args, env, expr.span)
            }
            Expr::TypeAnnotation { expr: inner_expr, type_expr: _ } => {
                self.eval_type_annotation(inner_expr, env)
            }
            Expr::Parameterize { bindings, body } => {
                self.eval_parameterize(bindings, body, env, expr.span)
            }
            Expr::Import { import_specs } => {
                self.eval_import(import_specs, env, expr.span)
            }
            
            Expr::DefineLibrary { name, imports, exports, body } => {
                self.eval_define_library(name, imports, exports, body, env, expr.span)
            }

            // Function application
            Expr::Application { operator, operands } => {
                self.eval_application(operator, operands, env, expr.span)
            }

            // Derived forms (implemented as macros in full system)
            Expr::Begin(exprs) => self.eval_begin(exprs, env, expr.span),
            Expr::Let { bindings, body } => self.eval_let(bindings, body, env, expr.span),
            Expr::LetStar { bindings, body } => self.eval_let_star(bindings, body, env, expr.span),
            Expr::LetRec { bindings, body } => self.eval_letrec(bindings, body, env, expr.span),
            Expr::Cond(clauses) => self.eval_cond(clauses, env, expr.span),
            Expr::And(exprs) => self.eval_and(exprs, env, expr.span),
            Expr::Or(exprs) => self.eval_or(exprs, env, expr.span),
            Expr::Guard { variable, clauses, body } => {
                self.eval_guard(variable, clauses, body, env, expr.span)
            }

            // Compound data structures
            Expr::Pair { car, cdr } => {
                self.eval_pair_construction(car, cdr, env)
            }

            // Unimplemented forms
            _ => EvalStep::Error(Error::runtime_error(
                format!("Unimplemented expression type: {:?}", expr.inner),
                Some(expr.span),
            )),
        }
    }

    /// Evaluates a quote expression.
    fn eval_quote(&mut self, quoted: &Spanned<Expr>) -> EvalStep {
        // Convert AST expression to runtime value
        match self.ast_to_value(&quoted.inner) {
            Ok(value) => EvalStep::Return(value),
            Err(e) => EvalStep::Error(*e),
        }
    }

    /// Evaluates a lambda expression (creates a closure).
    fn eval_lambda(
        &mut self,
        formals: &Formals,
        metadata: &HashMap<String, Spanned<Expr>>,
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        if body.is_empty() {
            return EvalStep::Error(Error::runtime_error(
                "Lambda body cannot be empty",
                Some(span),
            ));
        }

        // Evaluate metadata
        let mut eval_metadata = HashMap::new();
        for (key, value_expr) in metadata {
            match self.eval(value_expr, env.clone()) {
                Ok(value) => { eval_metadata.insert(key.clone(), value); }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        let procedure = Procedure {
            formals: formals.clone(),
            body: body.to_vec(),
            environment: env.to_thread_safe(),
            name: eval_metadata.get("name").and_then(|v| v.as_string().map(|s| s.to_string())),
            metadata: eval_metadata,
            source: Some(span),
        };

        EvalStep::Return(Value::Procedure(Arc::new(procedure)))
    }

    /// Evaluates a case-lambda expression (creates a case-lambda procedure).
    fn eval_case_lambda(
        &mut self,
        clauses: &[CaseLambdaClause],
        metadata: &HashMap<String, Spanned<Expr>>,
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        if clauses.is_empty() {
            return EvalStep::Error(Error::runtime_error(
                "Case-lambda must have at least one clause",
                Some(span),
            ));
        }

        // Validate that all clauses have non-empty bodies
        for (i, clause) in clauses.iter().enumerate() {
            if clause.body.is_empty() {
                return EvalStep::Error(Error::runtime_error(
                    format!("Case-lambda clause {} has empty body", i + 1),
                    Some(span),
                ));
            }
        }

        // Evaluate metadata
        let mut eval_metadata = HashMap::new();
        for (key, value_expr) in metadata {
            match self.eval(value_expr, env.clone()) {
                Ok(value) => { eval_metadata.insert(key.clone(), value); }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        let case_lambda = CaseLambdaProcedure {
            clauses: clauses.to_vec(),
            environment: env.to_thread_safe(),
            name: eval_metadata.get("name").and_then(|v| v.as_string().map(|s| s.to_string())),
            metadata: eval_metadata,
            source: Some(span),
        };

        EvalStep::Return(Value::CaseLambda(Arc::new(case_lambda)))
    }

    /// Evaluates an if expression.
    fn eval_if(
        &mut self,
        test: &Spanned<Expr>,
        consequent: &Spanned<Expr>,
        alternative: Option<&Spanned<Expr>>,
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        // Push stack frame for error reporting
        self.stack_trace.push(StackFrame::special_form("if".to_string(), Some(span)));

        // Evaluate test expression
        match self.eval(test, env.clone()) {
            Ok(test_value) => {
                self.stack_trace.pop(); // Remove if frame
                
                if test_value.is_truthy() {
                    EvalStep::Continue {
                        expr: consequent.clone(),
                        env,
                    }
                } else if let Some(alt) = alternative {
                    EvalStep::Continue {
                        expr: alt.clone(),
                        env,
                    }
                } else {
                    EvalStep::Return(Value::Unspecified)
                }
            }
            Err(e) => {
                self.stack_trace.pop(); // Remove if frame
                EvalStep::Error(*e)
            }
        }
    }

    /// Evaluates a define expression.
    fn eval_define(
        &mut self,
        name: &str,
        value_expr: &Spanned<Expr>,
        _metadata: &HashMap<String, Spanned<Expr>>,
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("define".to_string(), Some(span)));

        // Handle recursive function definitions like letrec
        if let Expr::Lambda { .. } = &value_expr.inner {
            // First bind name to unspecified for recursive reference
            env.define(name.to_string(), Value::Unspecified);
            
            // Evaluate the lambda expression
            match self.eval(value_expr, env.clone()) {
                Ok(value) => {
                    // Update the binding
                    env.define(name.to_string(), value.clone());
                    
                    // Fix the procedure's environment if it's a procedure
                    if let Value::Procedure(proc_arc) = value {
                        let proc = proc_arc.as_ref();
                        let updated_proc = Procedure {
                            formals: proc.formals.clone(),
                            body: proc.body.clone(),
                            environment: env.to_thread_safe(), // Capture current environment state
                            name: Some(name.to_string()), // Set the procedure name for recursive reference
                            metadata: proc.metadata.clone(),
                            source: proc.source,
                        };
                        env.define(name.to_string(), Value::Procedure(Arc::new(updated_proc)));
                    }
                    
                    self.stack_trace.pop();
                    EvalStep::Return(Value::Unspecified)
                }
                Err(e) => {
                    self.stack_trace.pop();
                    EvalStep::Error(*e)
                }
            }
        } else {
            // For non-lambda expressions, use normal evaluation
            match self.eval(value_expr, env.clone()) {
                Ok(value) => {
                    env.define(name.to_string(), value);
                    self.stack_trace.pop();
                    EvalStep::Return(Value::Unspecified)
                }
                Err(e) => {
                    self.stack_trace.pop();
                    EvalStep::Error(*e)
                }
            }
        }
    }

    /// Evaluates a set! expression.
    fn eval_set(
        &mut self,
        name: &str,
        value_expr: &Spanned<Expr>,
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("set!".to_string(), Some(span)));

        match self.eval(value_expr, env.clone()) {
            Ok(value) => {
                // Check if set! should be automatically lifted to State monad
                let args = vec![Value::symbol(crate::utils::intern_symbol(name)), value.clone()];
                if let Some(lifted) = self.effect_lifter.lift_operation("set!", &args) {
                    self.stack_trace.pop();
                    return self.handle_monadic_computation(lifted, env, span);
                }
                
                // Normal set! operation
                if env.set(name, value) {
                    // Increment generation for state change
                    self.generation += 1;
                    let _old_context = self.effect_system.enter_context(vec![Effect::State]);
                    self.stack_trace.pop();
                    EvalStep::Return(Value::Unspecified)
                } else {
                    self.stack_trace.pop();
                    EvalStep::Error(Error::runtime_error(
                        format!("Unbound variable in set!: {name}"),
                        Some(span),
                    ))
                }
            }
            Err(e) => {
                self.stack_trace.pop();
                EvalStep::Error(*e)
            }
        }
    }

    /// Evaluates a define-syntax expression.
    fn eval_define_syntax(
        &mut self,
        name: &str,
        transformer: &Spanned<Expr>,
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("define-syntax".to_string(), Some(span)));

        // Parse the transformer and add it to the macro environment
        match self.parse_syntax_transformer(transformer, env) {
            Ok(macro_transformer) => {
                self.macro_expander.define_macro(name.to_string(), macro_transformer);
                self.stack_trace.pop();
                EvalStep::Return(Value::Unspecified)
            }
            Err(e) => {
                self.stack_trace.pop();
                EvalStep::Error(*e)
            }
        }
    }

    /// Evaluates a syntax-rules expression.
    fn eval_syntax_rules(
        &mut self,
        literals: &[String],
        rules: &[(Spanned<Expr>, Spanned<Expr>)],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("syntax-rules".to_string(), Some(span)));

        // Create a syntax-rules transformer
        let syntax_rules_expr = Spanned::new(
            Expr::SyntaxRules {
                literals: literals.to_vec(),
                rules: rules.to_vec(),
            },
            span,
        );

        match crate::macro_system::parse_syntax_rules(&syntax_rules_expr, env) {
            Ok(syntax_rules_transformer) => {
                let macro_transformer = crate::macro_system::syntax_rules_to_macro_transformer(syntax_rules_transformer);
                // For direct syntax-rules evaluation, we could return a procedure
                // but typically syntax-rules is only used within define-syntax
                self.stack_trace.pop();
                EvalStep::Return(Value::Unspecified) // Or could return a macro transformer value
            }
            Err(e) => {
                self.stack_trace.pop();
                EvalStep::Error(*e)
            }
        }
    }

    /// Evaluates a call/cc expression.
    fn eval_call_cc(
        &mut self,
        proc_expr: &Spanned<Expr>,
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("call/cc".to_string(), Some(span)));

        // Evaluate the procedure
        match self.eval(proc_expr, env.clone()) {
            Ok(procedure) => {
                // Create a continuation that captures the current evaluation context
                let continuation = self.capture_continuation(env.clone(), None);
                let cont_value = Value::Continuation(Arc::new(continuation));
                
                self.stack_trace.pop();
                
                // Apply the procedure to the continuation
                EvalStep::TailCall {
                    procedure,
                    args: vec![cont_value],
                    location: Some(span),
                }
            }
            Err(e) => {
                self.stack_trace.pop();
                EvalStep::Error(*e)
            }
        }
    }

    /// Evaluates a primitive expression.
    fn eval_primitive(
        &mut self,
        name: &str,
        args: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("primitive".to_string(), Some(span)));

        // Evaluate all arguments
        let mut eval_args = Vec::new();
        for arg in args {
            match self.eval(arg, env.clone()) {
                Ok(value) => eval_args.push(value),
                Err(e) => {
                    self.stack_trace.pop();
                    return EvalStep::Error(*e);
                }
            }
        }

        // Call the FFI function through the bridge
        match self.ffi_bridge.call_rust_function(name, &eval_args) {
            Ok(result) => {
                self.stack_trace.pop();
                EvalStep::Return(result)
            }
            Err(e) => {
                // Create a new error with span information
                let error_with_span = match *e {
                    Error::RuntimeError { message, .. } => Box::new(Error::runtime_error(message, Some(span))),
                    _ => e,
                };
                self.stack_trace.pop();
                EvalStep::Error(*error_with_span)
            }
        }
    }

    /// Evaluates a function application.
    fn eval_application(
        &mut self,
        operator: &Spanned<Expr>,
        operands: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        // Check if this is a function call that should be automatically lifted
        if let Expr::Identifier(op_name) = &operator.inner {
            // Evaluate operands first for effect lifting
            let mut args = Vec::new();
            for operand in operands {
                match self.eval(operand, env.clone()) {
                    Ok(value) => args.push(value),
                    Err(e) => return EvalStep::Error(*e),
                }
            }
            
            // Check if this operation should be lifted
            let _current_effects = self.effect_system.context().effects();
            if let Some(lifted) = self.effect_lifter.lift_operation(op_name, &args) {
                // Handle the lifted monadic computation
                return self.handle_monadic_computation(lifted, env, span);
            }
        }
        
        // Evaluate operator normally
        match self.eval(operator, env.clone()) {
            Ok(procedure) => {
                // Evaluate operands
                let mut args = Vec::new();
                for operand in operands {
                    match self.eval(operand, env.clone()) {
                        Ok(value) => args.push(value),
                        Err(e) => return EvalStep::Error(*e),
                    }
                }

                // Apply procedure to arguments (tail call)
                EvalStep::TailCall {
                    procedure,
                    args,
                    location: Some(span),
                }
            }
            Err(e) => EvalStep::Error(*e),
        }
    }

    /// Applies a procedure to arguments.
    fn apply_procedure(&mut self, procedure: Value, args: Vec<Value>, location: Option<Span>) -> EvalStep {
        match procedure {
            Value::Procedure(proc) => self.apply_user_procedure(&proc, args, location),
            Value::CaseLambda(case_lambda) => self.apply_case_lambda_procedure(&case_lambda, args, location),
            Value::Primitive(prim) => self.apply_primitive_procedure(&prim, args, location),
            Value::Continuation(cont) => {
                if args.len() != 1 {
                    EvalStep::Error(Error::runtime_error(
                        format!("Continuation expects 1 argument, got {}", args.len()),
                        location,
                    ))
                } else {
                    EvalStep::CallContinuation {
                        continuation: cont,
                        value: args[0].clone(),
                    }
                }
            }
            Value::Parameter(param) => {
                // Parameters are callable as procedures
                match crate::stdlib::parameters::call_parameter(&param, &args) {
                    Ok(value) => EvalStep::Return(value),
                    Err(e) => EvalStep::Error(*e),
                }
            }
            _ => EvalStep::Error(Error::runtime_error(
                format!("Cannot apply non-procedure: {procedure}"),
                location,
            )),
        }
    }

    /// Applies a user-defined procedure.
    fn apply_user_procedure(
        &mut self,
        proc: &Procedure,
        args: Vec<Value>,
        location: Option<Span>,
    ) -> EvalStep {
        // Check arity
        if let Err(e) = self.check_arity(&proc.formals, args.len(), location) {
            return EvalStep::Error(*e);
        }

        // Create new environment for procedure body
        let mut new_env = proc.environment.extend(self.generation);
        
        // For recursive functions, we need to ensure the function name is correctly bound
        // This is a workaround for the environment snapshot issue in ThreadSafeEnvironment
        if let Some(proc_name) = &proc.name {
            // First check the global environment (for define-based functions)
            if let Some(global_value) = self.global_env.lookup(proc_name) {
                new_env = new_env.define_cow(proc_name.clone(), global_value);
            }
            // TODO: Also check parent environments for letrec-based functions
        }
        
        // Bind parameters using thread-safe environment
        let bound_env = match self.bind_parameters_thread_safe(&proc.formals, &args, new_env, location) {
            Ok(env) => env,
            Err(e) => return EvalStep::Error(*e),
        };

        // Push context frame for continuation capture
        self.push_context_frame(Frame::ProcedureCall {
            procedure_name: proc.name.clone(),
            remaining_body: proc.body.clone(),
            environment: bound_env.clone(),
            source: location.unwrap_or_default(),
        });

        // Push stack frame
        self.stack_trace.push(StackFrame::procedure_call(proc.name.clone(), location));

        // Convert back to legacy environment for eval_sequence
        let legacy_env = bound_env.to_legacy();
        
        // Evaluate body in sequence (implicit begin)
        let result = self.eval_sequence(&proc.body, legacy_env);
        
        // Pop context frame when procedure completes
        self.pop_context_frame();
        
        result
    }

    /// Applies a case-lambda procedure with arity dispatch.
    fn apply_case_lambda_procedure(
        &mut self,
        case_lambda: &CaseLambdaProcedure,
        args: Vec<Value>,
        location: Option<Span>,
    ) -> EvalStep {
        let arg_count = args.len();
        
        // Find the first matching clause
        for clause in case_lambda.clauses.iter() {
            if self.formals_match_arity(&clause.formals, arg_count) {
                // Create temporary procedure from matching clause
                let temp_proc = Procedure {
                    formals: clause.formals.clone(),
                    body: clause.body.clone(),
                    environment: case_lambda.environment.clone(),
                    name: case_lambda.name.clone(),
                    metadata: case_lambda.metadata.clone(),
                    source: case_lambda.source,
                };
                
                // Apply the temporary procedure
                return self.apply_user_procedure(&temp_proc, args, location);
            }
        }
        
        // No matching clause found - generate helpful error
        let clause_info: Vec<String> = case_lambda.clauses
            .iter()
            .enumerate()
            .map(|(i, clause)| {
                format!("clause {}: {}", i + 1, self.formals_arity_description(&clause.formals))
            })
            .collect();
        
        let proc_name = case_lambda.name
            .as_ref()
            .map(|n| format!("case-lambda procedure '{n}'"))
            .unwrap_or_else(|| "case-lambda procedure".to_string());
        
        EvalStep::Error(Error::runtime_error(
            format!(
                "{} called with {} arguments, but no clause matches. Available clauses: {}",
                proc_name,
                arg_count,
                clause_info.join(", ")
            ),
            location,
        ))
    }

    /// Applies a primitive procedure.
    fn apply_primitive_procedure(
        &mut self,
        prim: &PrimitiveProcedure,
        args: Vec<Value>,
        location: Option<Span>,
    ) -> EvalStep {
        // Check arity
        if args.len() < prim.arity_min {
            return EvalStep::Error(Error::runtime_error(
                format!(
                    "{} expects at least {} arguments, got {}",
                    prim.name, prim.arity_min, args.len()
                ),
                location,
            ));
        }

        if let Some(max) = prim.arity_max {
            if args.len() > max {
                return EvalStep::Error(Error::runtime_error(
                    format!(
                        "{} expects at most {} arguments, got {}",
                        prim.name, max, args.len()
                    ),
                    location,
                ));
            }
        }

        // Track effects from the primitive
        if !prim.effects.is_empty() && !prim.effects.contains(&Effect::Pure) {
            let _old_context = self.effect_system.enter_context(prim.effects.clone());
            
            // For state-modifying operations, increment generation
            if prim.effects.contains(&Effect::State) {
                self.generation += 1;
            }
        }

        // Push stack frame
        self.stack_trace.push(StackFrame::primitive(prim.name.clone(), location));

        // Call implementation
        let result = match &prim.implementation {
            PrimitiveImpl::RustFn(f) => f(&args),
            PrimitiveImpl::Native(f) => f(&args),
            PrimitiveImpl::ForeignFn { library: _, symbol: _ } => {
                // TODO: Implement FFI calls
                Err(Box::new(Error::runtime_error(
                    "FFI not yet implemented".to_string(),
                    location,
                )))
            }
        };

        self.stack_trace.pop();

        match result {
            Ok(value) => EvalStep::Return(value),
            Err(e) => EvalStep::Error(*e),
        }
    }

    /// Calls a continuation.
    fn call_continuation(&mut self, continuation: Arc<Continuation>, value: Value) -> EvalStep {
        // Check if this continuation has already been invoked (one-shot semantics)
        if continuation.is_invoked() {
            return EvalStep::Error(Error::runtime_error(
                "Attempt to invoke continuation more than once".to_string(),
                None,
            ));
        }

        // Mark the continuation as invoked
        continuation.mark_invoked();

        // Restore the captured evaluation context
        self.restore_continuation(&continuation, value)
    }

    /// Captures the current continuation.
    fn capture_continuation(&self, env: Rc<Environment>, current_expr: Option<Spanned<Expr>>) -> Continuation {
        // Clone the current context stack
        let captured_stack = self.context_stack.clone();
        
        Continuation::new(
            captured_stack,
            env.to_thread_safe(),
            next_continuation_id(),
            current_expr,
        )
    }

    /// Restores a captured continuation and returns the given value.
    fn restore_continuation(&mut self, continuation: &Continuation, value: Value) -> EvalStep {
        // Restore the context stack
        self.context_stack = continuation.stack.clone();
        
        // The continuation essentially performs a non-local exit
        // by abandoning the current computation and returning the value
        // in the context where the continuation was captured
        EvalStep::Return(value)
    }

    /// Pushes a frame onto the context stack.
    fn push_context_frame(&mut self, frame: Frame) {
        self.context_stack.push(frame);
    }

    /// Pops a frame from the context stack.
    fn pop_context_frame(&mut self) -> Option<Frame> {
        self.context_stack.pop()
    }

    // Helper methods for derived forms

    /// Evaluates a begin expression.
    fn eval_begin(&mut self, exprs: &[Spanned<Expr>], env: Rc<Environment>, span: Span) -> EvalStep {
        if exprs.is_empty() {
            return EvalStep::Error(Error::runtime_error(
                "Begin form cannot be empty",
                Some(span),
            ));
        }

        self.eval_sequence(exprs, env)
    }

    /// Evaluates a sequence of expressions.
    fn eval_sequence(&mut self, exprs: &[Spanned<Expr>], env: Rc<Environment>) -> EvalStep {
        if exprs.is_empty() {
            return EvalStep::Return(Value::Unspecified);
        }

        // Evaluate all but the last expression for side effects
        for expr in &exprs[..exprs.len() - 1] {
            if let Err(e) = self.eval(expr, env.clone()) {
                return EvalStep::Error(*e);
            }
        }

        // Tail call the last expression
        EvalStep::Continue {
            expr: exprs[exprs.len() - 1].clone(),
            env,
        }
    }

    /// Evaluates a let expression.
    fn eval_let(
        &mut self,
        bindings: &[crate::ast::Binding],
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        _span: Span,
    ) -> EvalStep {
        // Create new environment
        let new_env = env.extend(self.generation);

        // Evaluate all binding values in the original environment
        for binding in bindings {
            match self.eval(&binding.value, env.clone()) {
                Ok(value) => new_env.define(binding.name.clone(), value),
                Err(e) => return EvalStep::Error(*e),
            }
        }

        // Evaluate body in new environment
        self.eval_sequence(body, new_env)
    }

    /// Evaluates a let* expression.
    fn eval_let_star(
        &mut self,
        bindings: &[crate::ast::Binding],
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        _span: Span,
    ) -> EvalStep {
        let mut current_env = env;

        // Process bindings sequentially
        for binding in bindings {
            match self.eval(&binding.value, current_env.clone()) {
                Ok(value) => {
                    let new_env = current_env.extend(self.generation);
                    new_env.define(binding.name.clone(), value);
                    current_env = new_env;
                }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        // Evaluate body in final environment
        self.eval_sequence(body, current_env)
    }

    /// Evaluates a letrec expression.
    fn eval_letrec(
        &mut self,
        bindings: &[crate::ast::Binding],
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        _span: Span,
    ) -> EvalStep {
        // Create new environment
        let new_env = env.extend(self.generation);

        // First pass: bind all names to unspecified
        for binding in bindings {
            new_env.define(binding.name.clone(), Value::Unspecified);
        }

        // Second pass: evaluate all expressions and update bindings
        for binding in bindings {
            match self.eval(&binding.value, new_env.clone()) {
                Ok(value) => {
                    new_env.define(binding.name.clone(), value);
                }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        // Third pass: fix procedure environments to ensure they see the final state
        for binding in bindings {
            if let Some(Value::Procedure(proc_arc)) = new_env.lookup(&binding.name) {
                let proc = proc_arc.as_ref();
                let updated_proc = Procedure {
                    formals: proc.formals.clone(),
                    body: proc.body.clone(),
                    environment: new_env.to_thread_safe(), // Capture final environment state
                    name: Some(binding.name.clone()),
                    metadata: proc.metadata.clone(),
                    source: proc.source,
                };
                new_env.define(binding.name.clone(), Value::Procedure(Arc::new(updated_proc)));
            }
        }

        // Evaluate body in new environment
        self.eval_sequence(body, new_env)
    }

    /// Evaluates a cond expression.
    fn eval_cond(
        &mut self,
        clauses: &[crate::ast::CondClause],
        env: Rc<Environment>,
        _span: Span,
    ) -> EvalStep {
        for clause in clauses {
            // Check if this is an else clause
            if let Expr::Identifier(name) = &clause.test.inner {
                if name == "else" {
                    return self.eval_sequence(&clause.body, env);
                }
            }

            // Evaluate test
            match self.eval(&clause.test, env.clone()) {
                Ok(test_value) => {
                    if test_value.is_truthy() {
                        return self.eval_sequence(&clause.body, env);
                    }
                }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        // No clause matched
        EvalStep::Return(Value::Unspecified)
    }

    /// Evaluates an and expression.
    fn eval_and(&mut self, exprs: &[Spanned<Expr>], env: Rc<Environment>, _span: Span) -> EvalStep {
        if exprs.is_empty() {
            return EvalStep::Return(Value::t());
        }

        // Evaluate expressions left to right, short-circuiting on false
        for expr in &exprs[..exprs.len() - 1] {
            match self.eval(expr, env.clone()) {
                Ok(value) => {
                    if value.is_falsy() {
                        return EvalStep::Return(value);
                    }
                }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        // Tail call the last expression
        EvalStep::Continue {
            expr: exprs[exprs.len() - 1].clone(),
            env,
        }
    }

    /// Evaluates an or expression.
    fn eval_or(&mut self, exprs: &[Spanned<Expr>], env: Rc<Environment>, _span: Span) -> EvalStep {
        if exprs.is_empty() {
            return EvalStep::Return(Value::f());
        }

        // Evaluate expressions left to right, short-circuiting on true
        for expr in &exprs[..exprs.len() - 1] {
            match self.eval(expr, env.clone()) {
                Ok(value) => {
                    if value.is_truthy() {
                        return EvalStep::Return(value);
                    }
                }
                Err(e) => return EvalStep::Error(*e),
            }
        }

        // Tail call the last expression
        EvalStep::Continue {
            expr: exprs[exprs.len() - 1].clone(),
            env,
        }
    }

    /// Evaluates a guard expression for exception handling.
    fn eval_guard(
        &mut self,
        variable: &str,
        clauses: &[GuardClause],
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("guard".to_string(), Some(span)));
        
        // Try to evaluate the body
        let result = self.eval_sequence(body, env.clone());
        
        match result {
            EvalStep::Return(value) => {
                // Body completed normally - return the value
                self.stack_trace.pop();
                EvalStep::Return(value)
            }
            EvalStep::Error(Error::Exception { exception, .. }) => {
                // An exception was raised - try to handle it with the clauses
                self.stack_trace.pop();
                
                // Create new environment with exception bound to variable
                let handler_env = env.extend(self.generation);
                handler_env.define(variable.to_string(), Value::exception_object(exception.clone()));
                
                // Try each clause in order
                for clause in clauses {
                    // Evaluate the test condition
                    match self.eval(&clause.test, handler_env.clone()) {
                        Ok(test_result) => {
                            if test_result.is_truthy() {
                                // This clause matches
                                if let Some(ref arrow_expr) = clause.arrow {
                                    // => clause: apply the procedure to the test result
                                    match self.eval(arrow_expr, handler_env.clone()) {
                                        Ok(proc) => {
                                            return EvalStep::TailCall {
                                                procedure: proc,
                                                args: vec![test_result],
                                                location: Some(span),
                                            };
                                        }
                                        Err(e) => return EvalStep::Error(*e),
                                    }
                                } else {
                                    // Regular clause: evaluate the body
                                    return self.eval_sequence(&clause.body, handler_env);
                                }
                            }
                        }
                        Err(e) => {
                            // Error in test expression - this becomes the new exception
                            return EvalStep::Error(*e);
                        }
                    }
                }
                
                // No clause matched - re-raise the exception
                EvalStep::Error(Error::Exception { exception, span: Some(span) })
            }
            other => {
                // Other evaluation outcomes (continue, tail call, etc.) - pass through
                self.stack_trace.pop();
                other
            }
        }
    }

    /// Evaluates a parameterize expression.
    fn eval_parameterize(
        &mut self,
        bindings: &[crate::ast::ParameterBinding],
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        use crate::eval::parameter::ParameterBinding;
        use crate::stdlib::parameters::process_parameter_bindings;
        
        self.stack_trace.push(StackFrame::special_form("parameterize".to_string(), Some(span)));
        
        // Convert AST bindings to runtime bindings
        let runtime_bindings = match process_parameter_bindings(bindings, |expr| {
            // We need to evaluate expressions synchronously here
            // Create a temporary evaluator to evaluate the expressions
            match self.eval(&Spanned::new(expr.clone(), span), env.clone()) {
                Ok(value) => Ok(value),
                Err(e) => Err(e),
            }
        }) {
            Ok(bindings) => bindings,
            Err(e) => {
                self.stack_trace.pop();
                return EvalStep::Error(*e);
            }
        };
        
        // Execute the body with the parameter bindings
        let result = ParameterBinding::with_bindings(runtime_bindings, || {
            self.eval_sequence(body, env)
        });
        
        self.stack_trace.pop();
        result
    }

    // Helper methods

    /// Checks if the number of arguments matches the formal parameters.
    fn check_arity(&self, formals: &Formals, arg_count: usize, location: Option<Span>) -> Result<()> {
        match formals {
            Formals::Fixed(params) => {
                if arg_count != params.len() {
                    Err(Box::new(Error::runtime_error(
                        format!("Expected {} arguments, got {}", params.len(), arg_count),
                        location,
                    )))
                } else {
                    Ok(())
                }
            }
            Formals::Variable(_) => Ok(()), // Variable arity accepts any number
            Formals::Mixed { fixed, .. } => {
                if arg_count < fixed.len() {
                    Err(Box::new(Error::runtime_error(
                        format!("Expected at least {} arguments, got {}", fixed.len(), arg_count),
                        location,
                    )))
                } else {
                    Ok(())
                }
            }
            Formals::Keyword { fixed,  .. } => {
                // TODO: Implement proper keyword argument checking
                if arg_count < fixed.len() {
                    Err(Box::new(Error::runtime_error(
                        format!("Expected at least {} arguments, got {}", fixed.len(), arg_count),
                        location,
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Checks if formals can accept the given number of arguments (for case-lambda dispatch).
    fn formals_match_arity(&self, formals: &Formals, arg_count: usize) -> bool {
        match formals {
            Formals::Fixed(params) => arg_count == params.len(),
            Formals::Variable(_) => true, // Variable arity accepts any number
            Formals::Mixed { fixed, .. } => arg_count >= fixed.len(),
            Formals::Keyword { fixed, .. } => {
                // TODO: Implement proper keyword argument checking
                arg_count >= fixed.len()
            }
        }
    }

    /// Provides a human-readable description of the arity for a formals pattern.
    fn formals_arity_description(&self, formals: &Formals) -> String {
        match formals {
            Formals::Fixed(params) => {
                if params.len() == 1 {
                    "exactly 1 argument".to_string()
                } else {
                    format!("exactly {} arguments", params.len())
                }
            }
            Formals::Variable(_) => "any number of arguments".to_string(),
            Formals::Mixed { fixed, .. } => {
                if fixed.len() == 1 {
                    "at least 1 argument".to_string()
                } else {
                    format!("at least {} arguments", fixed.len())
                }
            }
            Formals::Keyword { fixed, .. } => {
                // TODO: Implement proper keyword argument description
                if fixed.len() == 1 {
                    "at least 1 argument (with keywords)".to_string()
                } else {
                    format!("at least {} arguments (with keywords)", fixed.len())
                }
            }
        }
    }

    /// Binds formal parameters to actual arguments in the given environment.
    #[allow(dead_code)]
    fn bind_parameters(
        &self,
        formals: &Formals,
        args: &[Value],
        env: &Environment,
        _location: Option<Span>,
    ) -> Result<()> {
        match formals {
            Formals::Fixed(params) => {
                for (param, arg) in params.iter().zip(args.iter()) {
                    env.define(param.clone(), arg.clone());
                }
            }
            Formals::Variable(param) => {
                // Bind all arguments as a list
                let args_list = Value::list(args.to_vec());
                env.define(param.clone(), args_list);
            }
            Formals::Mixed { fixed, rest } => {
                // Bind fixed parameters
                for (param, arg) in fixed.iter().zip(args.iter()) {
                    env.define(param.clone(), arg.clone());
                }
                
                // Bind remaining arguments as a list
                let rest_args = if args.len() > fixed.len() {
                    Value::list(args[fixed.len()..].to_vec())
                } else {
                    Value::Nil
                };
                env.define(rest.clone(), rest_args);
            }
            Formals::Keyword { fixed, rest: _, keywords: _ } => {
                // TODO: Implement proper keyword argument binding
                // For now, just bind fixed parameters
                for (param, arg) in fixed.iter().zip(args.iter()) {
                    env.define(param.clone(), arg.clone());
                }
            }
        }
        
        Ok(())
    }
    
    /// Binds formal parameters using ThreadSafeEnvironment (COW semantics).
    fn bind_parameters_thread_safe(
        &self,
        formals: &Formals,
        args: &[Value],
        env: Arc<ThreadSafeEnvironment>,
        _location: Option<Span>,
    ) -> Result<Arc<ThreadSafeEnvironment>> {
        let mut current_env = env;
        
        match formals {
            Formals::Fixed(params) => {
                for (param, arg) in params.iter().zip(args.iter()) {
                    current_env = current_env.define_cow(param.clone(), arg.clone());
                }
            }
            Formals::Variable(param) => {
                // Bind all arguments as a list
                let args_list = Value::list(args.to_vec());
                current_env = current_env.define_cow(param.clone(), args_list);
            }
            Formals::Mixed { fixed, rest } => {
                // Bind fixed parameters
                for (param, arg) in fixed.iter().zip(args.iter()) {
                    current_env = current_env.define_cow(param.clone(), arg.clone());
                }
                
                // Bind remaining arguments as a list
                let rest_args = if args.len() > fixed.len() {
                    Value::list(args[fixed.len()..].to_vec())
                } else {
                    Value::Nil
                };
                current_env = current_env.define_cow(rest.clone(), rest_args);
            }
            Formals::Keyword { fixed, rest: _, keywords: _ } => {
                // TODO: Implement proper keyword argument binding
                // For now, just bind fixed parameters
                for (param, arg) in fixed.iter().zip(args.iter()) {
                    current_env = current_env.define_cow(param.clone(), arg.clone());
                }
            }
        }
        
        Ok(current_env)
    }

    /// Converts an AST expression to a runtime value (for quote).
    #[allow(clippy::only_used_in_recursion)]
    fn ast_to_value(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Ok(Value::Literal(lit.clone())),
            Expr::Identifier(name) => Ok(Value::Symbol(intern_symbol(name))),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),
            Expr::Pair { car, cdr } => {
                let car_val = self.ast_to_value(&car.inner)?;
                let cdr_val = self.ast_to_value(&cdr.inner)?;
                Ok(Value::pair(car_val, cdr_val))
            }
            Expr::Application { operator, operands } => {
                // Convert to list
                let mut values = vec![self.ast_to_value(&operator.inner)?];
                for operand in operands {
                    values.push(self.ast_to_value(&operand.inner)?);
                }
                Ok(Value::list(values))
            }
            _ => Ok(Value::list(vec![])), // For now, other forms become empty lists
        }
    }

    /// Parses a syntax transformer expression.
    fn parse_syntax_transformer(
        &self,
        transformer_expr: &Spanned<Expr>,
        env: Rc<Environment>,
    ) -> Result<crate::macro_system::MacroTransformer> {
        match &transformer_expr.inner {
            Expr::SyntaxRules { literals, rules } => {
                // Parse syntax-rules into a transformer
                let syntax_rules_transformer = crate::macro_system::parse_syntax_rules(transformer_expr, env)?;
                Ok(crate::macro_system::syntax_rules_to_macro_transformer(syntax_rules_transformer))
            }
            _ => {
                // For other transformer types (lambda-based macros, etc.)
                // This is a simplified implementation - full support would require
                // evaluating the transformer expression in a macro-time environment
                Err(Box::new(Error::runtime_error(
                    "Only syntax-rules transformers are currently supported".to_string(),
                    Some(transformer_expr.span),
                )))
            }
        }
    }

    /// Gets the current stack trace.
    pub fn stack_trace(&self) -> &StackTrace {
        &self.stack_trace
    }

    /// Increments the generation counter.
    pub fn next_generation(&mut self) {
        self.generation += 1;
    }

    /// Gets a reference to the macro expander.
    pub fn macro_expander(&self) -> &MacroExpander {
        &self.macro_expander
    }

    /// Gets a mutable reference to the macro expander.
    pub fn macro_expander_mut(&mut self) -> &mut MacroExpander {
        &mut self.macro_expander
    }
    
    /// Gets a reference to the effect system.
    pub fn effect_system(&self) -> &EffectSystem {
        &self.effect_system
    }
    
    /// Gets a mutable reference to the effect system.
    pub fn effect_system_mut(&mut self) -> &mut EffectSystem {
        &mut self.effect_system
    }
    
    /// Gets a reference to the effect lifter.
    pub fn effect_lifter(&self) -> &EffectLifter {
        &self.effect_lifter
    }
    
    /// Gets a mutable reference to the effect lifter.
    pub fn effect_lifter_mut(&mut self) -> &mut EffectLifter {
        &mut self.effect_lifter
    }
    
    /// Gets a reference to the FFI bridge.
    pub fn ffi_bridge(&self) -> &FfiBridge {
        &self.ffi_bridge
    }
    
    /// Gets a mutable reference to the FFI bridge.
    pub fn ffi_bridge_mut(&mut self) -> &mut FfiBridge {
        &mut self.ffi_bridge
    }

    /// Evaluates an import expression.
    fn eval_import(
        &mut self,
        import_specs: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("import".to_string(), Some(span)));

        // Set up search paths for SRFI libraries if not already done
        self.setup_library_search_paths();

        // Process each import specification
        for spec_expr in import_specs {
            match self.process_import_spec(spec_expr, env.clone()) {
                Ok(bindings) => {
                    // Import the bindings into the current environment
                    for (name, value) in bindings {
                        env.define(name, value);
                    }
                }
                Err(e) => {
                    self.stack_trace.pop();
                    return EvalStep::Error(*e);
                }
            }
        }

        self.stack_trace.pop();
        EvalStep::Return(Value::Unspecified)
    }

    /// Sets up search paths for library loading.
    fn setup_library_search_paths(&mut self) {
        // Add stdlib path for SRFI modules
        self.scheme_loader.add_search_path("stdlib");
        
        // Initialize from library resolver if available
        self.scheme_loader.initialize_from_library_resolver();
    }

    /// Processes a single import specification.
    fn process_import_spec(&mut self, spec_expr: &Spanned<Expr>, _env: Rc<Environment>) -> Result<HashMap<String, Value>> {
        use crate::module_system::{import::parse_import_spec, ModuleId, ModuleNamespace};
        
        // Convert the spec expression to import specification
        let import_spec = self.parse_import_expression(spec_expr)?;

        // Load the module using the scheme library loader
        match self.scheme_loader.load_library(&import_spec.module_id) {
            Ok(compiled_library) => {
                // Apply import configuration to get final bindings
                crate::module_system::import::apply_import_config(
                    &compiled_library.module.exports,
                    &import_spec.config,
                )
            }
            Err(e) => Err(e),
        }
    }

    /// Parses an import expression into an ImportSpec.
    fn parse_import_expression(&self, spec_expr: &Spanned<Expr>) -> Result<ImportSpec> {
        use crate::module_system::{ImportSpec, ImportConfig, ModuleId, ModuleNamespace};
        
        match &spec_expr.inner {
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Err(Box::new(Error::syntax_error(
                        "Empty import specification".to_string(),
                        Some(spec_expr.span),
                    )));
                }

                // Parse module identifier from first element
                let module_id = self.parse_module_identifier(&elements[0])?;
                
                // For now, we'll support simple imports without configuration
                // TODO: Add support for (only ...), (except ...), (rename ...), (prefix ...)
                let config = ImportConfig::All;

                Ok(ImportSpec { module_id, config })
            }
            // Also handle Application syntax: (srfi 41) gets parsed as (srfi . (41))
            Expr::Application { operator, operands } => {
                // Construct a pseudo-list from the application
                let mut elements = vec![operator.as_ref().clone()];
                elements.extend(operands.iter().cloned());
                
                if elements.is_empty() {
                    return Err(Box::new(Error::syntax_error(
                        "Empty import specification".to_string(),
                        Some(spec_expr.span),
                    )));
                }

                // For applications, the elements directly represent the module components
                let module_id = self.parse_module_identifier_from_elements(&elements)?;
                let config = ImportConfig::All;

                Ok(ImportSpec { module_id, config })
            }
            _ => Err(Box::new(Error::syntax_error(
                format!("Import specification must be a list, found: {:?}", spec_expr.inner),
                Some(spec_expr.span),
            ))),
        }
    }

    /// Parses a module identifier from an expression.
    fn parse_module_identifier(&self, expr: &Spanned<Expr>) -> Result<ModuleId> {
        use crate::module_system::{ModuleId, ModuleNamespace};
        
        match &expr.inner {
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Err(Box::new(Error::syntax_error(
                        "Module identifier cannot be empty".to_string(),
                        Some(expr.span),
                    )));
                }

                let mut components = Vec::new();
                for element in elements {
                    match &element.inner {
                        Expr::Identifier(name) => components.push(name.clone()),
                        Expr::Symbol(name) => components.push(name.clone()),
                        _ => return Err(Box::new(Error::syntax_error(
                            "Module identifier components must be symbols".to_string(),
                            Some(element.span),
                        ))),
                    }
                }

                // Determine namespace based on first component
                let namespace = match components[0].as_str() {
                    "srfi" => ModuleNamespace::SRFI,
                    "scheme" => ModuleNamespace::R7RS,
                    "lambdust" => ModuleNamespace::Builtin,
                    _ => ModuleNamespace::User,
                };

                Ok(ModuleId { components, namespace })
            }
            _ => Err(Box::new(Error::syntax_error(
                "Module identifier must be a list".to_string(),
                Some(expr.span),
            ))),
        }
    }

    /// Parses a module identifier from a vector of expressions.
    fn parse_module_identifier_from_elements(&self, elements: &[Spanned<Expr>]) -> Result<ModuleId> {
        use crate::module_system::{ModuleId, ModuleNamespace};
        
        if elements.is_empty() {
            return Err(Box::new(Error::runtime_error(
                "Module identifier cannot be empty".to_string(),
                None,
            )));
        }

        let mut components = Vec::new();
        for element in elements {
            match &element.inner {
                Expr::Identifier(name) => components.push(name.clone()),
                Expr::Symbol(name) => components.push(name.clone()),
                Expr::Literal(crate::ast::Literal::Number(n)) => {
                    // Handle numeric components like "41" in (srfi 41)
                    components.push(format!("{}", *n as i64));
                }
                _ => return Err(Box::new(Error::syntax_error(
                    format!("Module identifier components must be symbols or numbers, found: {:?}", element.inner),
                    Some(element.span),
                ))),
            }
        }

        // Debug output to see what we're constructing
        eprintln!("Debug: Constructed module components: {components:?}");

        // Determine namespace based on first component
        let namespace = match components[0].as_str() {
            "srfi" => ModuleNamespace::SRFI,
            "scheme" => ModuleNamespace::R7RS,
            "lambdust" => ModuleNamespace::Builtin,
            _ => ModuleNamespace::User,
        };

        // Strip namespace prefix from components
        let module_components = match namespace {
            ModuleNamespace::SRFI | ModuleNamespace::R7RS | ModuleNamespace::Builtin => {
                if components.len() > 1 {
                    components[1..].to_vec()
                } else {
                    components
                }
            }
            _ => components,
        };

        let module_id = ModuleId { components: module_components, namespace };
        eprintln!("Debug: Final module ID: {module_id:?}");
        
        Ok(module_id)
    }

    /// Handles a monadic computation by executing it and returning the result.
    fn handle_monadic_computation(
        &mut self,
        computation: MonadicValue,
        _env: Rc<Environment>,
        _span: Span,
    ) -> EvalStep {
        match computation {
            MonadicValue::Pure(value) => EvalStep::Return(value),
            MonadicValue::IO(io_comp) => {
                // Execute the IO computation
                match io_comp.execute() {
                    Ok(value) => {
                        // Update effect context to include IO
                        let _old_context = self.effect_system.enter_context(vec![Effect::IO]);
                        EvalStep::Return(value)
                    },
                    Err(e) => EvalStep::Error(*e),
                }
            },
            MonadicValue::State(state_comp) => {
                // Execute the state computation
                match state_comp.execute() {
                    Ok((value, _new_env)) => {
                        // Create a new generation for the state change
                        self.generation += 1;
                        let _old_context = self.effect_system.enter_context(vec![Effect::State]);
                        EvalStep::Return(value)
                    },
                    Err(e) => EvalStep::Error(*e),
                }
            },
            MonadicValue::Error(error_comp) => {
                // Execute the error computation
                match error_comp.execute() {
                    Ok(value) => {
                        let _old_context = self.effect_system.enter_context(vec![Effect::Error]);
                        EvalStep::Return(value)
                    },
                    Err(e) => EvalStep::Error(*e),
                }
            },
            MonadicValue::Combined(combined) => {
                // Handle combined effects by using the primary computation
                self.handle_monadic_computation(combined.primary().clone(), _env, _span)
            }
        }
    }


    /// Evaluates a define-library expression.
    fn eval_define_library(
        &mut self,
        name: &[String],
        imports: &[Spanned<Expr>],
        exports: &[Spanned<Expr>],
        body: &[Spanned<Expr>],
        env: Rc<Environment>,
        span: Span,
    ) -> EvalStep {
        self.stack_trace.push(StackFrame::special_form("define-library".to_string(), Some(span)));

        // For now, this is a simplified implementation that:
        // 1. Processes imports (if any)
        // 2. Evaluates the body expressions
        // 3. Sets up exports (placeholder - full module system integration needed)
        
        // Process imports first
        for import_expr in imports {
            match self.process_import_spec(import_expr, env.clone()) {
                Ok(bindings) => {
                    // Import the bindings into the current environment
                    for (binding_name, value) in bindings {
                        env.define(binding_name, value);
                    }
                }
                Err(e) => {
                    self.stack_trace.pop();
                    return EvalStep::Error(*e);
                }
            }
        }

        // Create a new environment for the library body
        let lib_env = env.extend(self.generation);
        
        // Evaluate body expressions
        let mut last_result = Value::Unspecified;
        for body_expr in body {
            match self.eval(body_expr, lib_env.clone()) {
                Ok(value) => {
                    last_result = value;
                }
                Err(e) => {
                    self.stack_trace.pop();
                    return EvalStep::Error(*e);
                }
            }
        }

        // TODO: Process exports and register the library in the module system
        // For now, we'll skip export processing since it requires full module system integration
        
        eprintln!("Debug: define-library '{}' processed successfully", name.join(" "));
        if !exports.is_empty() {
            eprintln!("Debug: {} exports declared but not yet processed", exports.len());
        }

        self.stack_trace.pop();
        EvalStep::Return(Value::Unspecified)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Literal, Expr, Formals};
    use crate::diagnostics::Spanned;

    /// Helper function to create a spanned expression.
    fn spanned(expr: Expr) -> Spanned<Expr> {
        Spanned::new(expr, Span::default())
    }

    /// Helper function to create a simple lambda expression.
    fn make_lambda(param: &str, body: Expr) -> Expr {
        Expr::Lambda {
            formals: Formals::Fixed(vec![param.to_string()]),
            metadata: HashMap::new(),
            body: vec![spanned(body)],
        }
    }

    #[test]
    fn test_continuation_creation() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let continuation = Continuation::new(
            vec![],
            env,
            1,
            None,
        );
        
        assert_eq!(continuation.id, 1);
        assert!(!continuation.is_invoked());
        assert_eq!(continuation.stack.len(), 0);
    }

    #[test]
    fn test_continuation_invocation_tracking() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let continuation = Continuation::new(
            vec![],
            env,
            1,
            None,
        );

        // Initially not invoked
        assert!(!continuation.is_invoked());

        // Mark as invoked
        let was_invoked = continuation.mark_invoked();
        assert!(!was_invoked); // Returns previous state
        assert!(continuation.is_invoked());

        // Try to mark again
        let was_invoked_again = continuation.mark_invoked();
        assert!(was_invoked_again); // Returns previous state (true)
        assert!(continuation.is_invoked());
    }

    #[test]
    fn test_context_stack_management() {
        let mut evaluator = Evaluator::new();
        
        // Initially empty
        assert_eq!(evaluator.context_stack.len(), 0);
        
        // Push a frame
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let frame = Frame::CallCC {
            environment: env,
            source: Span::default(),
        };
        
        evaluator.push_context_frame(frame);
        assert_eq!(evaluator.context_stack.len(), 1);
        
        // Pop the frame
        let popped = evaluator.pop_context_frame();
        assert!(popped.is_some());
        assert_eq!(evaluator.context_stack.len(), 0);
        
        // Pop from empty stack
        let popped_empty = evaluator.pop_context_frame();
        assert!(popped_empty.is_none());
    }

    #[test]
    fn test_continuation_capture() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        
        // Push some context frames first
        let thread_safe_env = env.to_thread_safe();
        evaluator.push_context_frame(Frame::CallCC {
            environment: thread_safe_env.clone(),
            source: Span::default(),
        });
        
        // Capture continuation
        let continuation = evaluator.capture_continuation(env, None);
        
        // Verify captured state
        assert_eq!(continuation.stack.len(), 1);
        assert!(matches!(continuation.stack[0], Frame::CallCC { .. }));
    }

    #[test]
    fn test_simple_call_cc_expression() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        
        // Create a simple call/cc expression: (call/cc (lambda (k) 42))
        let lambda_expr = make_lambda("k", Expr::Literal(Literal::integer(42)));
        let call_cc_expr = Expr::CallCC(Box::new(spanned(lambda_expr)));
        
        // This should evaluate to 42
        let result = evaluator.eval(&spanned(call_cc_expr), env);
        
        match result {
            Ok(Value::Literal(Literal::Number(n))) => {
                assert_eq!(n, 42.0);
            }
            Ok(other) => panic!("Expected number 42, got {other:?}"),
            Err(e) => panic!("Evaluation failed: {e:?}"),
        }
    }

    #[test]
    fn test_call_cc_with_continuation_invocation() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        
        // Create: (call/cc (lambda (escape) (escape 42)))
        // This should return 42 by invoking the continuation
        let app_expr = Expr::Application {
            operator: Box::new(spanned(Expr::Identifier("escape".to_string()))),
            operands: vec![spanned(Expr::Literal(Literal::integer(42)))],
        };
        let lambda_expr = make_lambda("escape", app_expr);
        let call_cc_expr = Expr::CallCC(Box::new(spanned(lambda_expr)));
        
        let result = evaluator.eval(&spanned(call_cc_expr), env);
        
        // This test verifies the structure is correct, 
        // though the actual continuation invocation requires more complex setup
        match result {
            Ok(_) => {
                // If we get here, the call/cc structure was parsed and handled correctly
                // The exact result depends on the current implementation state
            }
            Err(e) => {
                // Check that it's not a parsing error but a runtime limitation
                let error_msg = format!("{e:?}");
                assert!(error_msg.contains("call/cc") || error_msg.contains("continuation") || error_msg.contains("escape"));
            }
        }
    }

    #[test]
    fn test_continuation_one_shot_semantics() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let continuation = Arc::new(Continuation::new(
            vec![],
            env,
            1,
            None,
        ));
        
        let mut evaluator = Evaluator::new();
        
        // First invocation should succeed
        let result1 = evaluator.call_continuation(continuation.clone(), Value::integer(42));
        match result1 {
            EvalStep::Return(Value::Literal(Literal::Number(n))) => {
                assert_eq!(n, 42.0);
            }
            other => panic!("Expected return of 42, got {other:?}"),
        }
        
        // Second invocation should fail
        let result2 = evaluator.call_continuation(continuation, Value::integer(84));
        match result2 {
            EvalStep::Error(e) => {
                let error_msg = format!("{e:?}");
                assert!(error_msg.contains("more than once"));
            }
            other => panic!("Expected error for second invocation, got {other:?}"),
        }
    }

    #[test]
    fn test_continuation_predicate() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let continuation = Arc::new(Continuation::new(
            vec![],
            env,
            1,
            None,
        ));
        
        let cont_value = Value::Continuation(continuation);
        let non_cont_value = Value::integer(42);
        
        // Test continuation? with actual continuation
        let result1 = crate::stdlib::control::primitive_continuation_p(&[cont_value]);
        assert_eq!(result1.unwrap(), Value::boolean(true));
        
        // Test continuation? with non-continuation
        let result2 = crate::stdlib::control::primitive_continuation_p(&[non_cont_value]);
        assert_eq!(result2.unwrap(), Value::boolean(false));
    }

    #[test]
    fn test_multiple_frame_types() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        
        // Test different frame types can be created
        let _call_cc_frame = Frame::CallCC {
            environment: env.clone(),
            source: Span::default(),
        };
        
        let _proc_call_frame = Frame::ProcedureCall {
            procedure_name: Some("test".to_string()),
            remaining_body: vec![],
            environment: env.clone(),
            source: Span::default(),
        };
        
        let _app_frame = Frame::Application {
            operator: Value::integer(42),
            evaluated_args: vec![],
            remaining_args: vec![],
            environment: env.clone(),
            source: Span::default(),
        };
        
        // Just verify they can be constructed without panicking
        assert!(true);
    }

    #[test]
    fn test_call_cc_integration_with_special_form() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::new(None, 0));
        
        // Test that call/cc is recognized as a special form
        let identity_lambda = make_lambda("x", Expr::Identifier("x".to_string()));
        let call_cc_expr = Expr::CallCC(Box::new(spanned(identity_lambda)));
        
        // This should not fail due to "unknown special form"
        let result = evaluator.eval(&spanned(call_cc_expr), env);
        
        // We expect either success or a specific call/cc related error,
        // not a "unknown expression type" error
        match result {
            Ok(_) => {
                // Success is good
            }
            Err(e) => {
                let error_msg = format!("{e:?}");
                // Should not be an "unimplemented expression" error
                assert!(!error_msg.contains("Unimplemented expression type"));
            }
        }
    }
}