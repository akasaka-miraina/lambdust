//! Multithreaded evaluator for the Lambdust language.
//!
//! This module implements the actor-based evaluator architecture that provides
//! thread-safe evaluation while maintaining proper Scheme semantics.

use super::{EvaluatorMessage, GlobalEnvironmentManager, EffectCoordinator};
use crate::ast::Expr;
use crate::diagnostics::{Result, Span, Spanned};
use crate::eval::{Value, ThreadSafeEnvironment, Generation, StackTrace, StackFrame};
use crate::effects::{EffectSystem, EffectLifter};
use crate::macro_system::MacroExpander;
use crate::module_system::ImportSpec;
use crate::ffi::FfiBridge;
use crossbeam::channel::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::{self, ThreadId};

/// A multithreaded evaluator that runs in its own thread.
///
/// This evaluator maintains thread-local state while coordinating with
/// other evaluators through shared global state. It processes evaluation
/// requests through a message queue.
#[derive(Debug)]
pub struct MultithreadedEvaluator {
    /// Unique ID for this evaluator
    pub id: u64,
    /// Thread ID where this evaluator runs
    pub thread_id: ThreadId,
    /// Thread-local generation counter
    local_generation: Generation,
    /// Thread-local stack trace
    local_stack: StackTrace,
    /// Shared global environment manager
    global_env: Arc<GlobalEnvironmentManager>,
    /// Local environment extensions (thread-specific bindings)
    local_env: Arc<ThreadSafeEnvironment>,
    /// Shared effect coordinator
    effect_coordinator: Arc<EffectCoordinator>,
    /// Thread-local effect system
    #[allow(dead_code)]
    effect_system: EffectSystem,
    /// Thread-local effect lifter
    #[allow(dead_code)]
    effect_lifter: EffectLifter,
    /// Thread-local macro expander
    macro_expander: MacroExpander,
    /// Thread-local FFI bridge
    #[allow(dead_code)]
    ffi_bridge: FfiBridge,
    /// Message queue for this evaluator
    message_queue: Receiver<EvaluatorMessage>,
    /// Flag indicating if the evaluator should shutdown
    should_shutdown: bool,
}

/// Worker thread state for running a MultithreadedEvaluator.
#[derive(Debug)]
pub struct EvaluatorWorker {
    #[allow(dead_code)]
    evaluator: MultithreadedEvaluator,
    #[allow(dead_code)]
    sender: Sender<EvaluatorMessage>,
}

impl MultithreadedEvaluator {
    /// Creates a new multithreaded evaluator.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this evaluator
    /// * `global_env` - Shared global environment manager
    /// * `effect_coordinator` - Shared effect coordinator
    /// * `message_queue` - Channel for receiving evaluation messages
    pub fn new(
        id: u64,
        global_env: Arc<GlobalEnvironmentManager>,
        effect_coordinator: Arc<EffectCoordinator>,
        message_queue: Receiver<EvaluatorMessage>,
    ) -> Self {
        let thread_id = thread::current().id();
        
        // Create thread-local environment that extends the global environment
        let local_env = global_env.create_thread_local_env(thread_id);
        
        Self {
            id,
            thread_id,
            local_generation: 0,
            local_stack: StackTrace::new(),
            global_env,
            local_env,
            effect_coordinator,
            effect_system: EffectSystem::new(),
            effect_lifter: EffectLifter::new(),
            macro_expander: MacroExpander::with_builtins(),
            ffi_bridge: FfiBridge::with_builtins(),
            message_queue,
            should_shutdown: false,
        }
    }

    /// Runs the evaluator's main message processing loop.
    ///
    /// This method processes messages until a shutdown signal is received.
    pub fn run(&mut self) -> Result<()> {
        while !self.should_shutdown {
            match self.message_queue.recv() {
                Ok(message) => {
                    if let Err(e) = self.handle_message(message) {
                        eprintln!("Error handling message in evaluator {}: {}", self.id, e);
                    }
                }
                Err(_) => {
                    // Channel closed, shutdown
                    break;
                }
            }
        }
        
        Ok(())
    }

    /// Handles a single evaluation message.
    fn handle_message(&mut self, message: EvaluatorMessage) -> Result<()> {
        match message {
            EvaluatorMessage::Evaluate { expr, span, sender } => {
                let spanned_expr = Spanned {
                    inner: expr,
                    span: span.unwrap_or(crate::diagnostics::Span { start: 0, len: 0, file_id: None, line: 1, column: 1 }),
                };
                
                // Push evaluation frame to stack
                self.local_stack.push(StackFrame::special_form("eval".to_string(), span));
                
                // Evaluate the expression using our adapted evaluator logic
                let result = self.eval_expression(&spanned_expr);
                
                // Pop evaluation frame
                self.local_stack.pop();
                
                // Send result back (ignore send errors as requestor may have disconnected)
                let _ = sender.send(result);
            }
            EvaluatorMessage::DefineGlobal { name, value } => {
                // Define in global environment
                self.global_env.define_global(name, value)?;
            }
            EvaluatorMessage::ImportModule { import_spec, sender } => {
                // Handle module import
                let result = self.handle_import(import_spec);
                let _ = sender.send(result);
            }
            EvaluatorMessage::Shutdown => {
                self.should_shutdown = true;
            }
        }
        
        Ok(())
    }

    /// Evaluates an expression using thread-safe evaluation logic.
    ///
    /// This is adapted from the main Evaluator but uses thread-safe environments
    /// and coordinates with the global state appropriately.
    fn eval_expression(&mut self, expr: &Spanned<Expr>) -> Result<Value> {
        // First, expand macros in the expression
        let expanded_expr = self.macro_expander.expand(expr)?;
        
        // Create a thread-safe evaluator context
        let context = ThreadSafeEvalContext {
            local_env: self.local_env.clone(),
            generation: self.local_generation,
            global_env: self.global_env.clone(),
            effect_coordinator: self.effect_coordinator.clone(),
        };
        
        // Use a simplified evaluation approach that's thread-safe
        self.eval_with_context(&expanded_expr, context)
    }

    /// Evaluates an expression with the given thread-safe context.
    fn eval_with_context(
        &mut self, 
        expr: &Spanned<Expr>, 
        context: ThreadSafeEvalContext
    ) -> Result<Value> {
        match &expr.inner {
            // Self-evaluating expressions
            Expr::Literal(lit) => Ok(Value::Literal(lit.clone())),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),

            // Variable lookup
            Expr::Identifier(name) => {
                // First check local environment, then global
                if let Some(value) = context.local_env.lookup(name) {
                    Ok(value)
                } else if let Some(value) = context.global_env.lookup_global(name) {
                    Ok(value)
                } else {
                    Err(crate::diagnostics::Error::runtime_error(
                        format!("Unbound variable: {name}"),
                        Some(expr.span),
                    ).boxed())
                }
            }

            // Quote
            Expr::Quote(quoted) => {
                Self::ast_to_value(&quoted.inner)
            }

            // Lambda (creates closure with thread-safe environment)
            Expr::Lambda { formals, metadata: _, body } => {
                if body.is_empty() {
                    return Err(crate::diagnostics::Error::runtime_error(
                        "Lambda body cannot be empty".to_string(),
                        Some(expr.span),
                    ).boxed());
                }

                let procedure = crate::eval::value::Procedure {
                    formals: formals.clone(),
                    body: body.to_vec(),
                    environment: context.local_env.clone(),
                    name: None,
                    metadata: std::collections::HashMap::new(),
                    source: Some(expr.span),
                };

                Ok(Value::Procedure(Arc::new(procedure)))
            }

            // If expression
            Expr::If { test, consequent, alternative } => {
                let test_value = self.eval_with_context(test, context.clone())?;
                
                if test_value.is_truthy() {
                    self.eval_with_context(consequent, context)
                } else if let Some(alt) = alternative {
                    self.eval_with_context(alt, context)
                } else {
                    Ok(Value::Unspecified)
                }
            }

            // Define (affects global environment)
            Expr::Define { name, value, metadata: _ } => {
                let val = self.eval_with_context(value, context.clone())?;
                context.global_env.define_global(name.clone(), val)?;
                Ok(Value::Unspecified)
            }

            // Application
            Expr::Application { operator, operands } => {
                let procedure = self.eval_with_context(operator, context.clone())?;
                
                let mut args = Vec::new();
                for operand in operands {
                    args.push(self.eval_with_context(operand, context.clone())?);
                }
                
                self.apply_procedure_thread_safe(procedure, args, context, expr.span)
            }

            // Begin
            Expr::Begin(exprs) => {
                if exprs.is_empty() {
                    return Err(crate::diagnostics::Error::runtime_error(
                        "Begin form cannot be empty".to_string(),
                        Some(expr.span),
                    ).boxed());
                }

                let mut result = Value::Unspecified;
                for expr in exprs {
                    result = self.eval_with_context(expr, context.clone())?;
                }
                Ok(result)
            }

            // Other forms - simplified for now
            _ => Err(crate::diagnostics::Error::runtime_error(
                format!("Unimplemented expression type in multithreaded evaluator: {:?}", expr.inner),
                Some(expr.span),
            ).boxed()),
        }
    }

    /// Applies a procedure in a thread-safe manner.
    fn apply_procedure_thread_safe(
        &mut self,
        procedure: Value,
        args: Vec<Value>,
        context: ThreadSafeEvalContext,
        span: Span,
    ) -> Result<Value> {
        match procedure {
            Value::Procedure(proc) => {
                // Create new environment for procedure body
                let new_env = context.local_env.extend(context.generation);
                
                // Bind parameters
                let bound_env = self.bind_parameters_thread_safe(&proc.formals, &args, new_env)?;
                
                // Create new context with bound environment
                let new_context = ThreadSafeEvalContext {
                    local_env: bound_env,
                    generation: context.generation + 1,
                    global_env: context.global_env.clone(),
                    effect_coordinator: context.effect_coordinator.clone(),
                };
                
                // Evaluate body
                let mut result = Value::Unspecified;
                for expr in &proc.body {
                    result = self.eval_with_context(expr, new_context.clone())?;
                }
                Ok(result)
            }
            Value::Primitive(prim) => {
                // Apply primitive procedure
                match &prim.implementation {
                    crate::eval::value::PrimitiveImpl::RustFn(f) => {
                        f(&args)
                    }
                    crate::eval::value::PrimitiveImpl::Native(f) => {
                        f(&args)
                    }
                    crate::eval::value::PrimitiveImpl::EvaluatorIntegrated(_) => {
                        Err(crate::diagnostics::Error::runtime_error(
                            "EvaluatorIntegrated primitives not supported in multithreaded evaluator".to_string(),
                            Some(span),
                        ).boxed())
                    }
                    crate::eval::value::PrimitiveImpl::ForeignFn { .. } => {
                        Err(crate::diagnostics::Error::runtime_error(
                            "FFI not yet implemented in multithreaded evaluator".to_string(),
                            Some(span),
                        ).boxed())
                    }
                }
            }
            _ => Err(crate::diagnostics::Error::runtime_error(
                format!("Cannot apply non-procedure: {procedure}"),
                Some(span),
            ).boxed()),
        }
    }

    /// Binds parameters for procedure application (thread-safe version).
    fn bind_parameters_thread_safe(
        &self,
        formals: &crate::ast::Formals,
        args: &[Value],
        env: Arc<ThreadSafeEnvironment>,
    ) -> Result<Arc<ThreadSafeEnvironment>> {
        use crate::ast::Formals;
        
        let mut current_env = env;
        
        match formals {
            Formals::Fixed(params) => {
                if args.len() != params.len() {
                    return Err(crate::diagnostics::Error::runtime_error(
                        format!("Expected {} arguments, got {}", params.len(), args.len()),
                        None,
                    ).boxed());
                }
                
                for (param, arg) in params.iter().zip(args.iter()) {
                    current_env = current_env.define_cow(param.clone(), arg.clone());
                }
            }
            Formals::Variable(param) => {
                let args_list = Value::list(args.to_vec());
                current_env = current_env.define_cow(param.clone(), args_list);
            }
            Formals::Mixed { fixed, rest } => {
                if args.len() < fixed.len() {
                    return Err(crate::diagnostics::Error::runtime_error(
                        format!("Expected at least {} arguments, got {}", fixed.len(), args.len()),
                        None,
                    ).boxed());
                }
                
                // Bind fixed parameters
                for (param, arg) in fixed.iter().zip(args.iter()) {
                    current_env = current_env.define_cow(param.clone(), arg.clone());
                }
                
                // Bind rest parameters
                let rest_args = if args.len() > fixed.len() {
                    Value::list(args[fixed.len()..].to_vec())
                } else {
                    Value::Nil
                };
                current_env = current_env.define_cow(rest.clone(), rest_args);
            }
            Formals::Keyword { .. } => {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Keyword arguments not yet implemented in multithreaded evaluator".to_string(),
                    None,
                ).boxed());
            }
        }
        
        Ok(current_env)
    }

    /// Converts an AST expression to a runtime value (for quote).
    fn ast_to_value(expr: &Expr) -> Result<Value> {
        use crate::utils::intern_symbol;
        
        match expr {
            Expr::Literal(lit) => Ok(Value::Literal(lit.clone())),
            Expr::Identifier(name) => Ok(Value::Symbol(intern_symbol(name))),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),
            Expr::Pair { car, cdr } => {
                let car_val = Self::ast_to_value(&car.inner)?;
                let cdr_val = Self::ast_to_value(&cdr.inner)?;
                Ok(Value::pair(car_val, cdr_val))
            }
            Expr::Application { operator, operands } => {
                let mut values = vec![Self::ast_to_value(&operator.inner)?];
                for operand in operands {
                    values.push(Self::ast_to_value(&operand.inner)?);
                }
                Ok(Value::list(values))
            }
            _ => Ok(Value::list(vec![])), // Other forms become empty lists
        }
    }

    /// Gets the evaluator's ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Gets the evaluator's thread ID.
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    /// Gets the current generation.
    pub fn generation(&self) -> Generation {
        self.local_generation
    }

    /// Gets a reference to the stack trace.
    pub fn stack_trace(&self) -> &StackTrace {
        &self.local_stack
    }

    /// Handles module import requests.
    fn handle_import(&self, _import_spec: ImportSpec) -> Result<HashMap<String, Value>> {
        // For now, return an empty HashMap as a placeholder
        // In a full implementation, this would:
        // 1. Load the module from the module system
        // 2. Apply import configuration (only, except, rename, prefix)
        // 3. Return the resulting bindings
        
        // Placeholder implementation
        Ok(HashMap::new())
    }
}

/// Thread-safe evaluation context passed during expression evaluation.
#[derive(Clone)]
struct ThreadSafeEvalContext {
    local_env: Arc<ThreadSafeEnvironment>,
    generation: Generation,
    global_env: Arc<GlobalEnvironmentManager>,
    effect_coordinator: Arc<EffectCoordinator>,
}

impl EvaluatorWorker {
    /// Creates a new evaluator worker.
    pub fn new(
        id: u64,
        global_env: Arc<GlobalEnvironmentManager>,
        effect_coordinator: Arc<EffectCoordinator>,
    ) -> (Self, Sender<EvaluatorMessage>) {
        let (sender, receiver) = crossbeam::channel::unbounded();
        
        let evaluator = MultithreadedEvaluator::new(
            id,
            global_env,
            effect_coordinator,
            receiver,
        );
        
        let worker = Self {
            evaluator,
            sender: sender.clone(),
        };
        
        (worker, sender)
    }

    /// Runs the evaluator worker until shutdown.
    #[allow(dead_code)]
    pub fn run(mut self) -> Result<()> {
        self.evaluator.run()
    }

    /// Gets the sender for this worker.
    #[allow(dead_code)]
    pub fn sender(&self) -> &Sender<EvaluatorMessage> {
        &self.sender
    }
}