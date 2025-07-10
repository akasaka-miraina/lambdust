//! Continuation handling for pure semantic evaluator
//!
//! This module implements continuation application logic for the pure
//! R7RS semantic evaluator, maintaining strict adherence to formal semantics.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::Continuation;
use crate::value::Value;
use std::rc::Rc;

use super::semantic_core::SemanticEvaluator;

impl SemanticEvaluator {
    /// Apply continuation in pure semantic evaluation
    ///
    /// This implements the formal semantics continuation application
    /// as defined in R7RS Section 7.2.
    pub(super) fn apply_continuation_pure(
        &mut self,
        cont: Continuation,
        value: Value,
    ) -> Result<Value> {
        match cont {
            Continuation::Identity => Ok(value),
            
            // Special form continuations
            Continuation::IfTest {
                consequent,
                alternate,
                env,
                parent,
            } => self.apply_if_test_continuation(value, consequent, alternate, env, *parent),
            
            Continuation::Define {
                variable,
                env,
                parent,
            } => self.apply_define_continuation(value, variable, env, *parent),
            
            Continuation::Assignment {
                variable,
                env,
                parent,
            } => self.apply_assignment_continuation(value, variable, env, *parent),
            
            Continuation::Begin {
                remaining,
                env,
                parent,
            } => self.apply_begin_continuation(value, remaining, env, *parent),
            
            Continuation::And {
                remaining,
                env,
                parent,
            } => self.apply_and_continuation(value, remaining, env, *parent),
            
            Continuation::Or {
                remaining,
                env,
                parent,
            } => self.apply_or_continuation(value, remaining, env, *parent),
            
            Continuation::CondTest {
                consequent,
                remaining_clauses,
                env,
                parent,
            } => self.apply_cond_test_continuation(
                value,
                consequent,
                remaining_clauses,
                env,
                *parent,
            ),
            
            Continuation::LetBinding {
                variable,
                remaining_bindings,
                mut evaluated_bindings,
                body,
                env,
                parent,
            } => {
                evaluated_bindings.push((variable, value));
                self.eval_let_bindings_pure(
                    remaining_bindings,
                    evaluated_bindings,
                    env,
                    body,
                    *parent,
                )
            }
            
            // Function application continuations
            Continuation::Operator { args, env, parent } => {
                self.eval_args_pure(args, env, vec![], value, *parent)
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
            
            // Other continuations from control flow module
            _ => {
                // Delegate to control flow module for complex continuations
                // This maintains separation of concerns
                self.apply_other_continuation_pure(cont, value)
            }
        }
    }
    
    /// Apply if test continuation
    fn apply_if_test_continuation(
        &mut self,
        value: Value,
        consequent: Expr,
        alternate: Option<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            self.eval_pure(consequent, env, parent)
        } else if let Some(alt) = alternate {
            self.eval_pure(alt, env, parent)
        } else {
            self.apply_continuation_pure(parent, Value::Undefined)
        }
    }
    
    /// Apply define continuation
    fn apply_define_continuation(
        &mut self,
        value: Value,
        variable: String,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        env.define(variable, value);
        self.apply_continuation_pure(parent, Value::Undefined)
    }
    
    /// Apply assignment continuation
    fn apply_assignment_continuation(
        &mut self,
        value: Value,
        variable: String,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        env.set(&variable, value)?;
        self.apply_continuation_pure(parent, Value::Undefined)
    }
    
    /// Apply begin continuation
    fn apply_begin_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if remaining.is_empty() {
            self.apply_continuation_pure(parent, value)
        } else {
            self.eval_sequence_pure(remaining, env, parent)
        }
    }
    
    /// Apply and continuation
    fn apply_and_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if !value.is_truthy() {
            self.apply_continuation_pure(parent, Value::Boolean(false))
        } else if remaining.is_empty() {
            self.apply_continuation_pure(parent, value)
        } else {
            self.eval_and_pure(&remaining, env, parent)
        }
    }
    
    /// Apply or continuation
    fn apply_or_continuation(
        &mut self,
        value: Value,
        remaining: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            self.apply_continuation_pure(parent, value)
        } else if remaining.is_empty() {
            self.apply_continuation_pure(parent, Value::Boolean(false))
        } else {
            self.eval_or_pure(&remaining, env, parent)
        }
    }
    
    /// Apply cond test continuation
    fn apply_cond_test_continuation(
        &mut self,
        value: Value,
        consequent: Vec<Expr>,
        remaining_clauses: Vec<(Expr, Vec<Expr>)>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        if value.is_truthy() {
            if consequent.is_empty() {
                self.apply_continuation_pure(parent, value)
            } else {
                self.eval_sequence_pure(consequent, env, parent)
            }
        } else {
            self.eval_cond_clauses_pure(remaining_clauses, env, parent)
        }
    }
    
    /// Apply application continuation
    fn apply_application_continuation(
        &mut self,
        value: Value,
        operator: Value,
        evaluated_args: Vec<Value>,
        remaining_args: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        let mut new_evaluated = evaluated_args;
        new_evaluated.push(value);
        
        if remaining_args.is_empty() {
            self.apply_procedure_pure(operator, new_evaluated, parent)
        } else {
            let (next_arg, rest) = remaining_args.split_first().unwrap();
            let next_cont = Continuation::Application {
                operator,
                evaluated_args: new_evaluated,
                remaining_args: rest.to_vec(),
                env: env.clone(),
                parent: Box::new(parent),
            };
            self.eval_pure(next_arg.clone(), env, next_cont)
        }
    }
}