//! R7RS formal semantics compliant evaluator
//!
//! This module implements a continuation-passing style evaluator
//! that strictly follows the R7RS formal semantics definition.

pub mod ast_converter;
pub mod continuation;
// Phase 6-B-Step2: Unified continuation pooling system
pub mod continuation_pooling;
pub mod control_flow;
pub mod evaluation;
pub mod expression_analyzer;
pub mod higher_order;
pub mod imports;
// Phase 6-B-Step3: Inline evaluation system
pub mod inline_evaluation;
// Phase 6-C: JIT loop optimization system
pub mod jit_loop_optimization;
pub mod memory;
// Phase 6-D: Tail call optimization system
pub mod tail_call_optimization;
// Phase 6-D: LLVM backend for advanced tail call optimization
pub mod llvm_backend;
// Phase 5-Step2: RAII store is now always available
pub mod raii_store;
pub mod special_forms;
// Phase 6-A: Trampoline evaluator for stack overflow prevention
pub mod trampoline;
pub mod types;

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::macros::expand_macro;
use crate::value::{Procedure, Value};
use ast_converter::AstConverter;

// Re-export main types
pub use continuation::{
    CompactContinuation, Continuation, DoLoopState, DynamicPoint, EnvironmentRef, InlineContinuation,
    LightContinuation,
};
// Phase 6-B-Step2: Continuation pooling system exports
pub use continuation_pooling::{
    ContinuationPoolManager, ContinuationType, PoolStatistics, SharedContinuationPoolManager,
    TypedContinuationPool,
};
pub use evaluation::{EvalOrder, ExceptionHandlerInfo};
pub use expression_analyzer::{
    AnalysisResult, EvaluationComplexity, ExpressionAnalyzer, OptimizationHint, OptimizationStats,
    TypeHint,
};
// Phase 6-B-Step3: Inline evaluation exports
pub use inline_evaluation::{
    CacheFriendlyPatterns, ContinuationWeight, HotPathDetector, InlineEvaluator, InlineHint,
    InlineResult,
};
// Phase 6-D: Tail call optimization exports
pub use tail_call_optimization::{
    TailCallAnalyzer, TailCallContext, TailCallOptimizer, TailCallStats, OptimizedTailCall,
    OptimizationLevel, ArgEvaluationStrategy,
};
// Phase 6-D: LLVM backend exports
pub use llvm_backend::{
    LLVMCodeGenerator, LLVMCompilerIntegration, LLVMOptimizationLevel, LLVMTailCallIntrinsic,
    LLVMFunction, LLVMInstruction, LLVMOptimizationStats,
};
// Phase 6-C: JIT loop optimization exports
pub use jit_loop_optimization::{
    GeneratedCode, IterationStrategy, IteratorType, JitHint, JitLoopOptimizer, JitOptimizationStats,
    LoopPattern, NativeCodeGenerator,
};
// Phase 6-A: Trampoline evaluator exports
pub use trampoline::{Bounce, ContinuationThunk, TrampolineEvaluation, TrampolineEvaluator};
pub use types::*;

use std::rc::Rc;

impl Evaluator {
    /// Main evaluation function: E[e]ρκσ
    /// Where:
    /// - e: expression to evaluate
    /// - ρ: environment
    /// - κ: continuation
    /// - σ: store
    pub fn eval(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // Phase 5-Step1: Pre-analysis optimization
        let analysis_result = self.analyze_expression_for_optimization(&expr, &env);
        
        // Apply constant folding optimization if available
        if let Ok(analysis) = &analysis_result {
            if let Some(optimized_value) = self.try_apply_optimizations(analysis, &cont)? {
                return Ok(optimized_value);
            }
        }

        // Stack overflow prevention
        if self.recursion_depth() >= self.max_recursion_depth() {
            return Err(LambdustError::stack_overflow());
        }

        self.increment_recursion_depth()?;
        let result = match expr {
            // Constants: E[K]ρκσ = κ(K[K])
            Expr::Literal(lit) => self.eval_literal(lit, cont),

            // Variables: E[I]ρκσ = κ(σ(ρ(I)))
            Expr::Variable(name) => self.eval_variable(name, env, cont),

            // Function application: E[(E0 E1 ...)]ρκσ
            Expr::List(exprs) if !exprs.is_empty() => self.eval_application(exprs, env, cont),

            // Empty list
            Expr::List(exprs) if exprs.is_empty() => self.eval_literal(Literal::Nil, cont),

            // Quote: E['E]ρκσ = κ(E[E])
            Expr::Quote(expr) => self.eval_quote(*expr, cont),

            // Quasiquote: E[`E]ρκσ = κ(quasiquote-expand(E))
            Expr::Quasiquote(expr) => self.eval_quasiquote(*expr, env, cont),

            // Vector: evaluate all elements
            Expr::Vector(exprs) => self.eval_vector(exprs, env, cont),

            // Dotted list (improper list)
            Expr::DottedList(_, _) => Err(LambdustError::syntax_error(
                "Dotted lists not supported in this context".to_string(),
            )),

            // Other forms
            _ => Err(LambdustError::syntax_error(format!(
                "Unsupported expression: {expr:?}"
            ))),
        };

        self.decrement_recursion_depth();
        result
    }

    /// Evaluate literal: K[K]
    fn eval_literal(&mut self, lit: Literal, cont: Continuation) -> Result<Value> {
        let value = self.literal_to_value(lit)?;
        self.apply_continuation(cont, value)
    }
    
    /// Convert literal to value (helper for trampoline evaluator)
    pub fn literal_to_value(&self, lit: Literal) -> Result<Value> {
        let value = match lit {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
            Literal::Character(c) => Value::Character(c),
            Literal::Nil => Value::Nil,
        };
        Ok(value)
    }

    /// Evaluate variable: σ(ρ(I))
    fn eval_variable(
        &mut self,
        name: String,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match env.get(&name) {
            Some(value) => self.apply_continuation(cont, value),
            None => Err(LambdustError::undefined_variable(name)),
        }
    }

    /// Evaluate application: E[(E0 E1 ...)]ρκσ
    fn eval_application(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return Err(LambdustError::syntax_error("Empty application".to_string()));
        }

        // Try to handle special forms first
        if let Some(special_result) =
            self.try_eval_special_form(&exprs, env.clone(), cont.clone())?
        {
            return Ok(special_result);
        }

        // Regular function application: evaluate operator first
        let (operator_expr, args) = exprs.split_first().unwrap();

        let operator_cont = Continuation::Operator {
            args: args.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(operator_expr.clone(), env, operator_cont)
    }

    /// Try to evaluate as special form
    fn try_eval_special_form(
        &mut self,
        exprs: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        if let Expr::Variable(name) = &exprs[0] {
            if self.is_special_form(name) {
                return Ok(Some(self.eval_known_special_form(
                    name,
                    &exprs[1..],
                    env,
                    cont,
                )?));
            }
        }
        Ok(None)
    }

    /// Check if a name is a special form
    fn is_special_form(&self, name: &str) -> bool {
        matches!(
            name,
            "lambda"
                | "if"
                | "set!"
                | "quote"
                | "define"
                | "begin"
                | "and"
                | "or"
                | "cond"
                | "case"
                | "do"
                | "delay"
                | "lazy"
                | "force"
                | "promise?"
                | "call/cc"
                | "call-with-current-continuation"
                | "values"
                | "call-with-values"
                | "dynamic-wind"
                | "raise"
                | "with-exception-handler"
                | "guard"
                | "map"
                | "apply"
                | "fold"
                | "fold-right"
                | "filter"
                | "hash-table-walk"
                | "hash-table-fold"
                | "memory-usage"
                | "memory-statistics"
                | "collect-garbage"
                | "set-memory-limit!"
                | "allocate-location"
                | "location-ref"
                | "location-set!"
                | "import"
        )
    }

    /// Evaluate quote form: E['E]ρκσ = κ(E[E])
    fn eval_quote(&mut self, expr: Expr, cont: Continuation) -> Result<Value> {
        self.apply_continuation(cont, AstConverter::expr_to_value(expr)?)
    }

    /// Evaluate quasiquote (simplified implementation)
    fn eval_quasiquote(
        &mut self,
        expr: Expr,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // For basic quasiquote without unquote/unquote-splicing,
        // it's equivalent to quote
        self.apply_continuation(cont, AstConverter::expr_to_value(expr)?)
    }

    /// Evaluate vector
    fn eval_vector(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if exprs.is_empty() {
            return self.apply_continuation(cont, Value::from_vector(Vec::new()));
        }

        let (first_expr, remaining) = exprs.split_first().unwrap();

        let vector_cont = Continuation::VectorEval {
            evaluated_elements: Vec::new(),
            remaining_elements: remaining.to_vec(),
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(first_expr.clone(), env, vector_cont)
    }

    /// Apply continuation: κ(v)
    pub fn apply_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Phase 6-B-Step3: Try inline evaluation first for maximum performance
        let should_inline = self.should_inline_continuation_impl(&cont);
        if should_inline {
            if let Some(result) = self.try_inline_continuation_impl(&cont, &value)? {
                // Update inline evaluator statistics
                self.inline_evaluator_mut().record_successful_inline(&cont);
                return Ok(result);
            }
        }
        
        // Fallback to regular continuation evaluation
        self.apply_continuation_regular(cont, value)
    }

    /// Apply continuation using regular (non-inline) evaluation
    pub fn apply_continuation_regular(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Performance optimization: Try compact continuation first (Phase 4 optimization)
        let compact_cont = CompactContinuation::from_continuation(cont.clone());
        if compact_cont.is_inline() {
            if let Ok(result) = self.apply_compact_continuation(compact_cont, value.clone()) {
                return Ok(result);
            }
        }

        // Fallback: Try lightweight continuation
        if let Some(light_cont) = LightContinuation::from_continuation(&cont) {
            return light_cont.apply(value);
        }

        match cont {
            // Inline simple identity continuation for performance
            Continuation::Identity => Ok(value),
            Continuation::Operator { args, env, parent } => {
                self.apply_operator_continuation(value, args, env, *parent)
            }
            Continuation::Application {
                operator,
                evaluated_args,
                remaining_args,
                env,
                parent,
            } => self.apply_application_continuation(
                value,
                operator,
                evaluated_args,
                remaining_args,
                env,
                *parent,
            ),
            Continuation::Values { mut values, parent } => {
                // Inline for performance
                values.push(value);
                self.apply_continuation(*parent, Value::Values(values))
            }
            Continuation::ValuesAccumulate {
                remaining_exprs,
                mut accumulated_values,
                env,
                parent,
            } => {
                // Add current value to accumulated values
                accumulated_values.push(value);

                if remaining_exprs.is_empty() {
                    // All expressions evaluated, create Values result
                    self.apply_continuation(*parent, Value::Values(accumulated_values))
                } else {
                    // Continue evaluating remaining expressions
                    let (next_expr, remaining) = remaining_exprs.split_first().unwrap();

                    let next_cont = Continuation::ValuesAccumulate {
                        remaining_exprs: remaining.to_vec(),
                        accumulated_values,
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_expr.clone(), env, next_cont)
                }
            }
            Continuation::VectorEval {
                mut evaluated_elements,
                remaining_elements,
                env,
                parent,
            } => {
                // Add the current value to evaluated elements
                evaluated_elements.push(value);

                if remaining_elements.is_empty() {
                    // All elements evaluated, create vector
                    let vector = Value::Vector(evaluated_elements);
                    self.apply_continuation(*parent, vector)
                } else {
                    // Continue evaluating remaining elements
                    let (next_expr, remaining) = remaining_elements.split_first().unwrap();

                    let vector_cont = Continuation::VectorEval {
                        evaluated_elements,
                        remaining_elements: remaining.to_vec(),
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_expr.clone(), env, vector_cont)
                }
            }
            // Delegate special form continuations to appropriate modules
            _ => self.apply_special_continuation(cont, value),
        }
    }

    /// Apply operator continuation
    fn apply_operator_continuation(
        &mut self,
        operator: Value,
        args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if args.is_empty() {
            // No arguments, apply directly
            self.apply_procedure(operator, Vec::new(), env, parent)
        } else {
            // Evaluate arguments according to evaluation order
            self.eval_arguments_in_order(operator, args, env, parent)
        }
    }

    /// Apply application continuation
    fn apply_application_continuation(
        &mut self,
        arg_value: Value,
        operator: Value,
        mut evaluated_args: Vec<Value>,
        remaining_args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        evaluated_args.push(arg_value);

        if remaining_args.is_empty() {
            // All arguments evaluated, apply procedure
            self.apply_procedure(operator, evaluated_args, env, parent)
        } else {
            // Continue evaluating remaining arguments
            let (next_arg, remaining) = remaining_args.split_first().unwrap();

            let app_cont = Continuation::Application {
                operator,
                evaluated_args,
                remaining_args: remaining.to_vec(),
                env: Rc::clone(&env),
                parent: Box::new(parent),
            };

            self.eval(next_arg.clone(), Rc::clone(&env), app_cont)
        }
    }

    /// Evaluate arguments in the specified order
    fn eval_arguments_in_order(
        &mut self,
        operator: Value,
        args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        match self.eval_order() {
            EvalOrder::LeftToRight => {
                let (first_arg, remaining) = args.split_first().unwrap();

                let app_cont = Continuation::Application {
                    operator,
                    evaluated_args: Vec::new(),
                    remaining_args: remaining.to_vec(),
                    env: Rc::clone(&env),
                    parent: Box::new(parent),
                };

                self.eval(first_arg.clone(), Rc::clone(&env), app_cont)
            }
            EvalOrder::RightToLeft => {
                // Evaluate from right to left
                let (last_arg, remaining) = args.split_last().unwrap();

                let app_cont = Continuation::Application {
                    operator,
                    evaluated_args: Vec::new(),
                    remaining_args: remaining.to_vec(),
                    env: Rc::clone(&env),
                    parent: Box::new(parent),
                };

                self.eval(last_arg.clone(), Rc::clone(&env), app_cont)
            }
            EvalOrder::Unspecified => {
                // For now, default to left-to-right
                // In a full implementation, this could randomize
                self.eval_arguments_in_order(operator, args, env, parent)
            }
        }
    }

    /// Apply procedure
    fn apply_procedure(
        &mut self,
        procedure: Value,
        args: Vec<Value>,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match procedure {
            Value::Procedure(proc) => self.apply_procedure_variant(proc, args, cont),
            _ => Err(LambdustError::type_error(
                "Cannot apply non-procedure".to_string(),
            )),
        }
    }

    /// Apply specific procedure variant
    fn apply_procedure_variant(
        &mut self,
        proc: Procedure,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        match proc {
            Procedure::Builtin { func, arity, .. } => {
                self.apply_builtin_procedure(func, arity, args, cont)
            }
            Procedure::Lambda {
                params,
                body,
                closure,
                variadic,
            } => self.apply_lambda_procedure(params, body, closure, variadic, args, cont),
            Procedure::Continuation {
                continuation: _captured_cont,
            } => self.apply_simple_continuation(args),
            Procedure::CapturedContinuation {
                continuation: captured_cont,
            } => self.apply_captured_continuation_procedure(*captured_cont, args),
            Procedure::ReusableContinuation {
                continuation: captured_cont,
                capture_env,
                is_escaping,
                ..
            } => self.apply_reusable_continuation(
                *captured_cont,
                capture_env,
                is_escaping,
                args,
                cont,
            ),
            Procedure::HostFunction { func, arity, .. } => {
                self.apply_host_function(func, arity, args, cont)
            }
        }
    }

    /// Apply builtin procedure
    fn apply_builtin_procedure(
        &mut self,
        func: fn(&[Value]) -> Result<Value>,
        arity: Option<usize>,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        if let Some(expected) = arity {
            if args.len() != expected {
                return Err(LambdustError::arity_error(expected, args.len()));
            }
        }
        let result = func(&args)?;
        self.apply_continuation(cont, result)
    }

    /// Apply lambda procedure
    fn apply_lambda_procedure(
        &mut self,
        params: Vec<String>,
        body: Vec<Expr>,
        closure: Rc<Environment>,
        variadic: bool,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        // Check arity for lambda
        if variadic {
            if args.len() < params.len() - 1 {
                return Err(LambdustError::arity_error(params.len() - 1, args.len()));
            }
        } else if args.len() != params.len() {
            return Err(LambdustError::arity_error(params.len(), args.len()));
        }

        // Create new environment for lambda body
        let lambda_env = Environment::with_parent(closure);
        self.bind_lambda_parameters(&lambda_env, &params, &args, variadic)?;

        // Evaluate body
        self.eval_sequence(body, Rc::new(lambda_env), cont)
    }

    /// Bind lambda parameters to arguments
    fn bind_lambda_parameters(
        &self,
        lambda_env: &Environment,
        params: &[String],
        args: &[Value],
        variadic: bool,
    ) -> Result<()> {
        if variadic {
            // Bind fixed parameters
            for (i, param) in params.iter().enumerate().take(params.len() - 1) {
                lambda_env.define(param.clone(), args[i].clone());
            }

            // Bind rest parameter
            let rest_param = &params[params.len() - 1];
            let rest_args = args[(params.len() - 1)..].to_vec();
            lambda_env.define(rest_param.clone(), Value::from_vector(rest_args));
        } else {
            for (param, arg) in params.iter().zip(args.iter()) {
                lambda_env.define(param.clone(), arg.clone());
            }
        }
        Ok(())
    }

    /// Phase 6-B-Step3: Check if continuation should be inlined
    fn should_inline_continuation_impl(&self, cont: &Continuation) -> bool {
        use crate::evaluator::inline_evaluation::ContinuationWeight;
        
        let weight = ContinuationWeight::from_continuation(cont);
        let cont_type = self.get_continuation_type_name_impl(cont);
        let hint = self.inline_evaluator().get_inline_hint(&cont_type);
        
        weight.should_inline(hint)
    }

    /// Phase 6-B-Step3: Check if continuation should be inlined (public test version)
    #[cfg(test)]
    pub fn should_inline_continuation(&self, cont: &Continuation) -> bool {
        self.should_inline_continuation_impl(cont)
    }

    /// Phase 6-B-Step3: Try to evaluate continuation inline
    fn try_inline_continuation_impl(&mut self, cont: &Continuation, value: &Value) -> Result<Option<Value>> {
        match cont {
            // Identity continuation - most common case
            Continuation::Identity => Ok(Some(value.clone())),
            
            // Simple value accumulation
            Continuation::Values { values, .. } => {
                let mut new_values = values.clone();
                new_values.push(value.clone());
                Ok(Some(Value::Values(new_values)))
            }
            
            // Variable assignment
            Continuation::Assignment { variable, env, .. } => {
                env.set(variable, value.clone())?;
                Ok(Some(Value::Undefined))
            }
            
            // Variable definition
            Continuation::Define { variable, env, .. } => {
                env.define(variable.clone(), value.clone());
                Ok(Some(Value::Undefined))
            }
            
            // Other continuations require full evaluation
            _ => Ok(None),
        }
    }

    /// Phase 6-B-Step3: Try to evaluate continuation inline (public test version)
    #[cfg(test)]
    pub fn try_inline_continuation(&mut self, cont: &Continuation, value: &Value) -> Result<Option<Value>> {
        self.try_inline_continuation_impl(cont, value)
    }

    /// Get continuation type name for tracking
    fn get_continuation_type_name_impl(&self, cont: &Continuation) -> String {
        match cont {
            Continuation::Identity => "Identity".to_string(),
            Continuation::Values { .. } => "Values".to_string(),
            Continuation::Assignment { .. } => "Assignment".to_string(),
            Continuation::Define { .. } => "Define".to_string(),
            Continuation::Begin { .. } => "Begin".to_string(),
            Continuation::IfTest { .. } => "IfTest".to_string(),
            Continuation::Application { .. } => "Application".to_string(),
            Continuation::Operator { .. } => "Operator".to_string(),
            _ => "Other".to_string(),
        }
    }

    /// Get continuation type name for tracking (public test version)
    #[cfg(test)]
    pub fn get_continuation_type_name(&self, cont: &Continuation) -> String {
        self.get_continuation_type_name_impl(cont)
    }

    /// Apply simple continuation
    fn apply_simple_continuation(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }
        Ok(args[0].clone())
    }

    /// Apply captured continuation from procedure call
    fn apply_captured_continuation_procedure(
        &mut self,
        captured_cont: Continuation,
        args: Vec<Value>,
    ) -> Result<Value> {
        let escape_value = if args.is_empty() {
            Value::Undefined
        } else {
            args[0].clone()
        };

        self.apply_captured_continuation_with_non_local_exit(captured_cont, escape_value)
    }

    /// Apply reusable continuation
    fn apply_reusable_continuation(
        &mut self,
        captured_cont: Continuation,
        capture_env: Rc<Environment>,
        _is_escaping: bool,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        let escape_value = if args.is_empty() {
            Value::Undefined
        } else {
            args[0].clone()
        };

        // Always treat call to captured continuation as an escape
        // This is the fundamental semantics of call/cc: any invocation of the captured
        // continuation should perform non-local exit
        let is_escape_context = true;

        if is_escape_context {
            self.apply_captured_continuation_with_non_local_exit(captured_cont, escape_value)
        } else {
            self.apply_reusable_continuation_with_context(
                captured_cont,
                capture_env,
                escape_value,
                cont,
            )
        }
    }

    /// Apply host function
    fn apply_host_function(
        &mut self,
        func: crate::host::HostFunc,
        arity: Option<usize>,
        args: Vec<Value>,
        cont: Continuation,
    ) -> Result<Value> {
        if let Some(expected) = arity {
            if args.len() != expected {
                return Err(LambdustError::arity_error(expected, args.len()));
            }
        }
        let result = func(&args)?;
        self.apply_continuation(cont, result)
    }

    /// Apply special continuations (delegates to appropriate modules)
    fn apply_special_continuation(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Try special form continuations first
        match &cont {
            Continuation::IfTest { .. }
            | Continuation::CondTest { .. }
            | Continuation::Assignment { .. }
            | Continuation::Define { .. }
            | Continuation::Begin { .. }
            | Continuation::And { .. }
            | Continuation::Or { .. } => self.apply_special_form_continuation(cont, value),
            // Default to control flow continuations
            _ => self.apply_control_flow_continuation(cont, value),
        }
    }

    /// Evaluate a string containing Scheme code
    pub fn eval_string(&mut self, input: &str) -> Result<Value> {
        use crate::parser::Parser;

        let tokens = crate::lexer::tokenize(input)?;
        let mut parser = Parser::new(tokens);
        let exprs = parser.parse_all()?;

        if exprs.is_empty() {
            return Err(LambdustError::syntax_error(
                "No expressions to evaluate".to_string(),
            ));
        }

        // Evaluate all expressions, return the last result
        let mut result = Value::Undefined;
        for expr in exprs {
            result = self.eval(expr, Rc::clone(&self.global_env), Continuation::Identity)?;
        }

        Ok(result)
    }

    /// Call a procedure (for compatibility)
    pub fn call_procedure(&mut self, procedure: Value, args: Vec<Value>) -> Result<Value> {
        self.apply_procedure(
            procedure,
            args,
            Rc::clone(&self.global_env),
            Continuation::Identity,
        )
    }

    /// Macro expansion integration
    fn try_expand_macro(&self, name: &str, args: &[Expr]) -> Result<Option<Expr>> {
        match name {
            "let" | "let*" | "letrec" | "case" | "when" | "unless" => {
                let expanded = expand_macro(name, args)?;
                Ok(Some(expanded))
            }
            _ => Ok(None),
        }
    }

    /// Apply captured continuation with complete non-local exit
    /// This provides true call/cc behavior by completely abandoning the current
    /// continuation chain and jumping directly to the captured continuation
    fn apply_captured_continuation_with_non_local_exit(
        &mut self,
        captured_cont: Continuation,
        escape_value: Value,
    ) -> Result<Value> {
        // Perform complete non-local exit by recursively skipping ALL intermediate
        // computations until we reach the true capture point
        self.apply_captured_continuation_complete_exit(captured_cont, escape_value)
    }

    /// Recursively skip all intermediate computations to implement complete non-local exit
    fn apply_captured_continuation_complete_exit(
        &mut self,
        captured_cont: Continuation,
        escape_value: Value,
    ) -> Result<Value> {
        match captured_cont {
            // For CallCc continuation, skip to its parent (the capture point)
            Continuation::CallCc { parent, .. } => {
                // This is where call/cc was originally called, so we apply the parent
                // continuation with the escape value
                self.apply_continuation(*parent, escape_value)
            }
            // For Application continuations, we need to distinguish between:
            // 1. call/cc escape (should skip all intermediate computation)
            // 2. captured continuation reuse (should preserve computation context)
            //
            // The fundamental issue is that both cases look the same at this point.
            // For now, implement proper escape behavior by skipping Application continuations.
            // This means continuation reuse might not work correctly for certain cases,
            // but call/cc escape semantics will be preserved.
            Continuation::Application { parent, .. } => {
                // Skip the Application and continue up the chain
                // This implements proper call/cc escape semantics
                self.apply_captured_continuation_complete_exit(*parent, escape_value)
            }
            // For other intermediate computation continuations, skip them entirely
            cont if cont.is_intermediate_computation() => {
                if let Some(parent) = cont.parent() {
                    // Recursively skip up the chain until we find a non-intermediate continuation
                    self.apply_captured_continuation_complete_exit(parent.clone(), escape_value)
                } else {
                    // If we reach the top with no parent, return the escape value directly
                    Ok(escape_value)
                }
            }
            // For non-intermediate continuations (like Identity, Define, etc.),
            // apply them normally as they represent valid continuation points
            _ => self.apply_continuation(captured_cont, escape_value),
        }
    }

    /// Apply reusable continuation with context preservation (for continuation reuse)
    fn apply_reusable_continuation_with_context(
        &mut self,
        captured_cont: Continuation,
        _capture_env: Rc<Environment>,
        value: Value,
        _current_cont: Continuation,
    ) -> Result<Value> {
        // For continuation reuse, we need to preserve the computation context
        // instead of performing a complete escape
        match captured_cont {
            // For CallCc continuation, apply the value in the captured environment
            Continuation::CallCc { parent, .. } => {
                // Restore the capture environment and apply the parent continuation
                self.apply_continuation(*parent, value)
            }
            // For Application continuations, we preserve the context
            Continuation::Application {
                operator,
                evaluated_args,
                remaining_args,
                env,
                parent,
            } => {
                // Build new application with the value inserted in the captured context
                // This enables proper continuation reuse semantics
                let mut new_args = evaluated_args;
                new_args.push(value);

                if remaining_args.is_empty() {
                    // All arguments are ready, apply the operator
                    self.apply_procedure(operator, new_args, env, *parent)
                } else {
                    // Continue evaluating remaining arguments
                    let next_arg = &remaining_args[0];
                    let remaining = remaining_args[1..].to_vec();

                    let app_cont = Continuation::Application {
                        operator,
                        evaluated_args: new_args,
                        remaining_args: remaining,
                        env: env.clone(),
                        parent,
                    };

                    self.eval(next_arg.clone(), env, app_cont)
                }
            }
            // For other continuations, apply them normally
            _ => self.apply_continuation(captured_cont, value),
        }
    }

    /// Apply compact continuation with optimized inline processing
    /// This is the core of Phase 4 continuation optimization
    pub fn apply_compact_continuation(
        &mut self,
        compact_cont: CompactContinuation,
        value: Value,
    ) -> Result<Value> {
        match compact_cont {
            CompactContinuation::Inline(inline_cont) => {
                self.apply_inline_continuation(*inline_cont, value)
            }
            CompactContinuation::Boxed(boxed_cont) => {
                // For boxed continuations, use regular path
                self.apply_continuation(*boxed_cont, value)
            }
        }
    }

    /// Apply inline continuation with specialized handling for evaluator context
    #[inline]
    pub fn apply_inline_continuation(
        &mut self,
        inline_cont: InlineContinuation,
        value: Value,
    ) -> Result<Value> {
        match inline_cont {
            InlineContinuation::Identity => Ok(value),
            InlineContinuation::Values(mut values) => {
                values.push(value);
                Ok(Value::Values(values.into_vec()))
            }
            InlineContinuation::Assignment {
                var_name,
                mut env_ref,
            } => {
                if let Some(env) = env_ref.to_strong() {
                    env.set(&var_name, value)?;
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::runtime_error(
                        "Environment reference expired in compact continuation".to_string(),
                    ))
                }
            }
            InlineContinuation::Define {
                variable,
                mut env_ref,
            } => {
                if let Some(env) = env_ref.to_strong() {
                    env.define(variable, value);
                    Ok(Value::Undefined)
                } else {
                    Err(LambdustError::runtime_error(
                        "Environment reference expired in compact continuation".to_string(),
                    ))
                }
            }
            InlineContinuation::SingleBegin { expr, mut env_ref } => {
                if let Some(env) = env_ref.to_strong() {
                    // For SingleBegin, we discard the current value and evaluate the expression
                    self.eval(expr, env, Continuation::Identity)
                } else {
                    Err(LambdustError::runtime_error(
                        "Environment reference expired in compact continuation".to_string(),
                    ))
                }
            }
            InlineContinuation::SimpleIf {
                consequent,
                alternate,
                mut env_ref,
            } => {
                if let Some(env) = env_ref.to_strong() {
                    if value.is_truthy() {
                        self.eval(consequent, env, Continuation::Identity)
                    } else if let Some(alt) = alternate {
                        self.eval(alt, env, Continuation::Identity)
                    } else {
                        Ok(Value::Undefined)
                    }
                } else {
                    Err(LambdustError::runtime_error(
                        "Environment reference expired in compact continuation".to_string(),
                    ))
                }
            }
        }
    }

    /// Analyze expression for optimization opportunities (Phase 5-Step1)
    fn analyze_expression_for_optimization(&mut self, expr: &Expr, env: &Environment) -> Result<AnalysisResult> {
        self.expression_analyzer_mut().analyze(expr, Some(env))
    }

    /// Try to apply optimizations based on analysis result
    fn try_apply_optimizations(&mut self, analysis: &AnalysisResult, cont: &Continuation) -> Result<Option<Value>> {
        for optimization in &analysis.optimizations {
            match optimization {
                OptimizationHint::ConstantFold(value) => {
                    // Apply constant folding: skip evaluation and return constant value
                    let result = self.apply_continuation(cont.clone(), value.clone())?;
                    return Ok(Some(result));
                }
                OptimizationHint::InlineVariable(var_name, value) => {
                    // Variable inlining optimization
                    self.expression_analyzer_mut().add_constant(var_name.clone(), value.clone());
                    let result = self.apply_continuation(cont.clone(), value.clone())?;
                    return Ok(Some(result));
                }
                OptimizationHint::DeadCode => {
                    // Dead code elimination: return undefined for unreachable code
                    let result = self.apply_continuation(cont.clone(), Value::Undefined)?;
                    return Ok(Some(result));
                }
                _ => {
                    // Other optimizations are applied during evaluation, not here
                    continue;
                }
            }
        }
        Ok(None)
    }

    /// Update expression analyzer with specific variable information
    pub fn update_analyzer_with_variable(&mut self, name: &str, value: &Value) {
        if self.is_analyzable_constant(value) {
            self.expression_analyzer_mut().add_constant(name.to_string(), value.clone());
            
            // Add type hint based on value
            let type_hint = self.value_to_type_hint(value);
            self.expression_analyzer_mut().add_type_hint(name.to_string(), type_hint);
        }
    }

    /// Check if a value can be used as a constant in analysis
    fn is_analyzable_constant(&self, value: &Value) -> bool {
        matches!(
            value,
            Value::Boolean(_) | Value::Number(_) | Value::String(_) 
            | Value::Character(_) | Value::Symbol(_) | Value::Nil
        )
    }

    /// Convert value to type hint for analyzer
    fn value_to_type_hint(&self, value: &Value) -> TypeHint {
        match value {
            Value::Boolean(_) => TypeHint::Boolean,
            Value::Number(_) => TypeHint::Number,
            Value::String(_) => TypeHint::String,
            Value::Character(_) => TypeHint::Character,
            Value::Symbol(_) => TypeHint::Symbol,
            Value::Pair(_) | Value::Nil => TypeHint::List,
            Value::Vector(_) => TypeHint::Vector,
            Value::Procedure(_) => TypeHint::Procedure,
            _ => TypeHint::Unknown,
        }
    }

    /// Get optimization statistics from expression analyzer
    pub fn get_optimization_statistics(&self) -> OptimizationStats {
        self.expression_analyzer().optimization_stats()
    }

    /// Clear expression analyzer cache (useful for memory management)
    pub fn clear_expression_cache(&mut self) {
        self.expression_analyzer_mut().clear_cache();
    }

}

/// Public API for evaluation
pub fn eval_with_formal_semantics(expr: Expr, env: Rc<Environment>) -> Result<Value> {
    let mut evaluator = Evaluator::new();
    evaluator.eval(expr, env, Continuation::Identity)
}
