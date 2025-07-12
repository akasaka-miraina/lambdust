//! Special forms evaluation for pure semantic evaluator
//!
//! This module implements R7RS special forms evaluation within the
//! pure semantic evaluator framework.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::Continuation;
use crate::value::{Procedure, Value};
use std::rc::Rc;

use super::semantic_core::SemanticEvaluator;

impl SemanticEvaluator {
    /// Evaluate special form in pure semantic evaluation
    pub(super) fn eval_special_form_pure(
        &mut self,
        name: &str,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match name {
            "if" => self.eval_if_pure(operands, env, cont),
            "define" => self.eval_define_pure(operands, env, cont),
            "begin" => self.eval_begin_pure(operands, env, cont),
            "lambda" => self.eval_lambda_pure(operands, env, cont),
            "let" => self.eval_let_pure(operands, env, cont),
            "and" => self.eval_and_pure(operands, env, cont),
            "or" => self.eval_or_pure(operands, env, cont),
            "cond" => self.eval_cond_pure(operands, env, cont),
            _ => Err(LambdustError::syntax_error(format!(
                "Special form '{name}' not implemented in pure evaluator"
            ))),
        }
    }
    
    /// Evaluate if special form
    fn eval_if_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 || operands.len() > 3 {
            return Err(LambdustError::syntax_error(
                "if: requires test and consequent, optionally alternate".to_string(),
            ));
        }
        
        let test = operands[0].clone();
        let consequent = operands[1].clone();
        let alternate = operands.get(2).cloned();
        
        let if_cont = Continuation::IfTest {
            consequent,
            alternate,
            env: env.clone(),
            parent: Box::new(cont),
        };
        
        self.eval_pure(test, env, if_cont)
    }
    
    /// Evaluate define special form
    fn eval_define_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }
        
        let variable = match &operands[0] {
            Expr::Variable(name) => name.clone(),
            _ => return Err(LambdustError::syntax_error(
                "define: first argument must be a variable".to_string(),
            )),
        };
        
        let value_expr = operands[1].clone();
        let define_cont = Continuation::Define {
            variable,
            env: env.clone(),
            parent: Box::new(cont),
        };
        
        self.eval_pure(value_expr, env, define_cont)
    }
    
    /// Evaluate begin special form
    fn eval_begin_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        self.eval_sequence_pure(operands.to_vec(), env, cont)
    }
    
    /// Evaluate lambda special form
    fn eval_lambda_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda: requires parameter list and body".to_string(),
            ));
        }
        
        let (params, is_variadic) = self.parse_lambda_params(&operands[0])?;
        let body = operands[1..].to_vec();
        
        let procedure = Procedure::Lambda {
            params,
            body,
            closure: env.clone(),
            variadic: is_variadic,
        };
        
        let value = Value::Procedure(procedure);
        self.apply_continuation_pure(cont, value)
    }
    
    /// Parse lambda parameters
    fn parse_lambda_params(&self, params_expr: &Expr) -> Result<(Vec<String>, bool)> {
        match params_expr {
            Expr::List(params) => {
                let mut param_names = Vec::new();
                for param in params {
                    match param {
                        Expr::Variable(name) => param_names.push(name.clone()),
                        _ => return Err(LambdustError::syntax_error(
                            "lambda: parameter must be a symbol".to_string(),
                        )),
                    }
                }
                Ok((param_names, false))
            }
            Expr::Variable(name) => {
                // Variadic lambda: (lambda x ...)
                Ok((vec![name.clone()], true))
            }
            _ => Err(LambdustError::syntax_error(
                "lambda: parameter list must be a list or symbol".to_string(),
            )),
        }
    }
    
    /// Evaluate let special form
    fn eval_let_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "let: requires bindings and body".to_string(),
            ));
        }
        
        let Expr::List(bindings) = &operands[0] else { return Err(LambdustError::syntax_error(
                "let: bindings must be a list".to_string(),
            )) };
        
        let body = operands[1..].to_vec();
        
        // Extract binding pairs
        let mut binding_pairs = Vec::new();
        for binding in bindings {
            match binding {
                Expr::List(pair) if pair.len() == 2 => {
                    let var = match &pair[0] {
                        Expr::Variable(name) => name.clone(),
                        _ => return Err(LambdustError::syntax_error(
                            "let: binding variable must be a symbol".to_string(),
                        )),
                    };
                    binding_pairs.push((var, pair[1].clone()));
                }
                _ => return Err(LambdustError::syntax_error(
                    "let: each binding must be a two-element list".to_string(),
                )),
            }
        }
        
        self.eval_let_bindings_pure(binding_pairs, vec![], env, body, cont)
    }
    
    /// Evaluate let bindings sequentially
    pub(super) fn eval_let_bindings_pure(
        &mut self,
        remaining_bindings: Vec<(String, Expr)>,
        evaluated_bindings: Vec<(String, Value)>,
        env: Rc<Environment>,
        body: Vec<Expr>,
        cont: Continuation,
    ) -> Result<Value> {
        if remaining_bindings.is_empty() {
            // All bindings evaluated, create new environment and evaluate body
            let new_env = Environment::with_parent(env);
            for (var, val) in evaluated_bindings {
                new_env.define(var, val);
            }
            return self.eval_sequence_pure(body, Rc::new(new_env), cont);
        }
        
        let (var, expr) = remaining_bindings[0].clone();
        let remaining = remaining_bindings[1..].to_vec();
        
        let binding_cont = Continuation::LetBinding {
            variable: var,
            remaining_bindings: remaining,
            evaluated_bindings,
            body,
            env: env.clone(),
            parent: Box::new(cont),
        };
        
        self.eval_pure(expr, env, binding_cont)
    }
    
    /// Evaluate and special form
    pub(super) fn eval_and_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation_pure(cont, Value::Boolean(true));
        }
        
        if operands.len() == 1 {
            return self.eval_pure(operands[0].clone(), env, cont);
        }
        
        let first = operands[0].clone();
        let rest = operands[1..].to_vec();
        
        let and_cont = Continuation::And {
            remaining: rest,
            env: env.clone(),
            parent: Box::new(cont),
        };
        
        self.eval_pure(first, env, and_cont)
    }
    
    /// Evaluate or special form
    pub(super) fn eval_or_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation_pure(cont, Value::Boolean(false));
        }
        
        if operands.len() == 1 {
            return self.eval_pure(operands[0].clone(), env, cont);
        }
        
        let first = operands[0].clone();
        let rest = operands[1..].to_vec();
        
        let or_cont = Continuation::Or {
            remaining: rest,
            env: env.clone(),
            parent: Box::new(cont),
        };
        
        self.eval_pure(first, env, or_cont)
    }
    
    /// Evaluate cond special form
    fn eval_cond_pure(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        let clauses = operands.to_vec();
        self.eval_cond_clauses_pure(
            clauses.into_iter().map(|clause| {
                match clause {
                    Expr::List(parts) if !parts.is_empty() => {
                        let test = parts[0].clone();
                        let consequent = parts[1..].to_vec();
                        Ok((test, consequent))
                    }
                    _ => Err(LambdustError::syntax_error(
                        "cond: each clause must be a non-empty list".to_string(),
                    ))
                }
            }).collect::<Result<Vec<_>>>()?,
            env,
            cont,
        )
    }
    
    /// Evaluate cond clauses
    pub(super) fn eval_cond_clauses_pure(
        &mut self,
        clauses: Vec<(Expr, Vec<Expr>)>,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if clauses.is_empty() {
            return self.apply_continuation_pure(cont, Value::Undefined);
        }
        
        let (test, consequent) = clauses[0].clone();
        let remaining = clauses[1..].to_vec();
        
        // Check for else clause
        if let Expr::Variable(name) = &test {
            if name == "else" {
                return self.eval_sequence_pure(consequent, env, cont);
            }
        }
        
        let cond_cont = Continuation::CondTest {
            consequent,
            remaining_clauses: remaining,
            env: env.clone(),
            parent: Box::new(cont),
        };
        
        self.eval_pure(test, env, cond_cont)
    }
}