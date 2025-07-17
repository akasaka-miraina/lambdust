//! Macro expansion built-in functions
//!
//! Provides `macro-expand-1`, `macro-expand`, and `macro-expand-all` functions
//! for interactive macro debugging and development, inspired by Common Lisp's
//! macroexpansion facilities.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::{Value, PairData};
use crate::ast::{Expr, Literal};
use crate::macros::hygiene::{HygienicEnvironment, HygienicSymbol};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};

/// Register macro expansion functions as builtins
pub fn register_macro_expansion_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("macro-expand-1".to_string(), macro_expand_1_builtin());
    builtins.insert("macro-expand".to_string(), macro_expand_builtin());
    builtins.insert("macro-expand-all".to_string(), macro_expand_all_builtin());
}

/// Result of macro expansion attempt
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    /// The expanded form (or original if no expansion)
    pub form: Expr,
    /// Whether expansion occurred
    pub expanded: bool,
    /// Information about the expansion (for debugging)
    pub expansion_info: Option<ExpansionInfo>,
}

/// Detailed information about macro expansion
#[derive(Debug, Clone)]
pub struct ExpansionInfo {
    /// Name of the macro that was expanded
    pub macro_name: String,
    /// Type of macro (syntax-rules, hygienic, etc.)
    pub macro_type: String,
    /// Expansion depth (for nested expansions)
    pub depth: usize,
    /// Original symbols → hygienic symbols mapping
    pub symbol_mapping: HashMap<String, HygienicSymbol>,
    /// Time taken for expansion (for performance debugging)
    pub expansion_time: Duration,
}

/// Macro expansion engine
pub struct MacroExpander {
    /// Environment for macro lookup
    environment: Rc<HygienicEnvironment>,
    /// Maximum expansion depth (prevents infinite recursion)
    max_depth: usize,
    /// Whether to preserve expansion metadata
    preserve_metadata: bool,
}

impl MacroExpander {
    /// Create new macro expander
    pub fn new(environment: Rc<HygienicEnvironment>) -> Self {
        Self {
            environment,
            max_depth: 256,
            preserve_metadata: true,
        }
    }
    
    /// Simplified hygienic macro expansion for demonstration
    fn try_expand_hygienic_macro(&self, macro_name: &str, _args: &[Expr]) -> Result<Expr> {
        // This is a placeholder implementation for demonstration
        // In a real implementation, this would integrate with the full macro system
        match macro_name {
            "when" => {
                // Return a simple if-expression with hygienic symbols
                Ok(Expr::List(vec![
                    Expr::Variable("if#1".to_string()),
                    Expr::Variable("test".to_string()),
                    Expr::List(vec![
                        Expr::Variable("begin#2".to_string()),
                        Expr::Variable("expr".to_string()),
                    ]),
                ]))
            }
            _ => Err(LambdustError::runtime_error(format!(
                "Unknown macro: {macro_name}"
            )))
        }
    }
    
    /// Create macro expander with custom settings
    pub fn with_settings(
        environment: Rc<HygienicEnvironment>, 
        max_depth: usize,
        preserve_metadata: bool,
    ) -> Self {
        Self {
            environment,
            max_depth,
            preserve_metadata,
        }
    }
    
    /// Expand macro exactly one level
    pub fn expand_once(&self, form: &Expr) -> Result<ExpansionResult> {
        let start_time = Instant::now();
        
        // Check if this is a macro call
        if let Expr::List(exprs) = form {
            if let Some(Expr::Variable(macro_name)) = exprs.first() {
                // Check if it's a hygienic macro in the environment
                if self.environment.get_hygienic_macro(macro_name).is_some() {
                    // For now, we'll implement a simplified expansion
                    // TODO: Integrate with actual macro expansion system
                    let args: Vec<Expr> = exprs.iter().skip(1).cloned().collect();
                    
                    // Simplified expansion - just return a placeholder result
                    if let Ok(expanded) = self.try_expand_hygienic_macro(macro_name, &args) {
                        let expansion_time = start_time.elapsed();
                        
                        let expansion_info = if self.preserve_metadata {
                            Some(ExpansionInfo {
                                macro_name: macro_name.clone(),
                                macro_type: "syntax-rules".to_string(), // TODO: Get actual type
                                depth: 1,
                                symbol_mapping: HashMap::new(), // TODO: Extract from expansion
                                expansion_time,
                            })
                        } else {
                            None
                        };
                        
                        return Ok(ExpansionResult {
                            form: expanded,
                            expanded: true,
                            expansion_info,
                        });
                    }
                    // Macro expansion failed, treat as non-macro
                }
            }
        }
        
        // Not a macro call or expansion failed
        Ok(ExpansionResult {
            form: form.clone(),
            expanded: false,
            expansion_info: None,
        })
    }
    
    /// Expand macro completely until no more expansions possible
    pub fn expand_completely(&self, form: &Expr) -> Result<ExpansionResult> {
        let mut current_form = form.clone();
        let mut total_expanded = false;
        let mut total_depth = 0;
        let start_time = Instant::now();
        
        loop {
            if total_depth >= self.max_depth {
                return Err(LambdustError::runtime_error(format!(
                    "Macro expansion exceeded maximum depth ({}). Possible infinite recursion.",
                    self.max_depth
                )));
            }
            
            let result = self.expand_once(&current_form)?;
            
            if !result.expanded {
                // No more expansions possible
                break;
            }
            
            current_form = result.form;
            total_expanded = true;
            total_depth += 1;
        }
        
        let expansion_time = start_time.elapsed();
        
        let expansion_info = if self.preserve_metadata && total_expanded {
            Some(ExpansionInfo {
                macro_name: "<multiple>".to_string(),
                macro_type: "complete-expansion".to_string(),
                depth: total_depth,
                symbol_mapping: HashMap::new(),
                expansion_time,
            })
        } else {
            None
        };
        
        Ok(ExpansionResult {
            form: current_form,
            expanded: total_expanded,
            expansion_info,
        })
    }
    
    /// Expand all macros recursively throughout the form
    pub fn expand_all(&self, form: &Expr) -> Result<Expr> {
        match form {
            Expr::List(_exprs) => {
                // First try to expand this level
                let expansion_result = self.expand_completely(form)?;
                let mut current_form = expansion_result.form;
                
                // Then recursively expand subforms
                if let Expr::List(expanded_exprs) = current_form {
                    let recursively_expanded: Result<Vec<Expr>> = expanded_exprs
                        .iter()
                        .map(|expr| self.expand_all(expr))
                        .collect();
                    
                    current_form = Expr::List(recursively_expanded?);
                }
                
                Ok(current_form)
            }
            Expr::Vector(exprs) => {
                let expanded_exprs: Result<Vec<Expr>> = exprs
                    .iter()
                    .map(|expr| self.expand_all(expr))
                    .collect();
                Ok(Expr::Vector(expanded_exprs?))
            }
            Expr::Quote(_inner) => {
                // Don't expand inside quotes
                Ok(form.clone())
            }
            Expr::Quasiquote(_inner) => {
                // TODO: Handle quasiquote expansion properly
                Ok(form.clone())
            }
            _ => {
                // Literal values, variables, etc. - no expansion needed
                Ok(form.clone())
            }
        }
    }
}

/// Convert Value to Expr for macro expansion
fn value_to_expr(value: &Value) -> Result<Expr> {
    match value {
        Value::Boolean(b) => Ok(Expr::Literal(Literal::Boolean(*b))),
        Value::Number(n) => Ok(Expr::Literal(Literal::Number(n.clone()))),
        Value::String(s) => Ok(Expr::Literal(Literal::String(s.clone()))),
        Value::ShortString(s) => Ok(Expr::Literal(Literal::String(s.as_str().to_string()))),
        Value::Character(c) => Ok(Expr::Literal(Literal::Character(*c))),
        Value::Symbol(s) => Ok(Expr::Variable(s.clone())),
        Value::ShortSymbol(s) => Ok(Expr::Variable(s.as_str().to_string())),
        Value::Nil => Ok(Expr::Literal(Literal::Nil)),
        Value::Pair(pair_ref) => {
            // Convert pair to list by traversing the chain
            let mut exprs = Vec::new();
            let mut current_pair = pair_ref.clone();
            
            loop {
                let (car_expr, next_value) = {
                    let pair = current_pair.borrow();
                    (value_to_expr(&pair.car)?, pair.cdr.clone())
                };
                
                exprs.push(car_expr);
                
                match next_value {
                    Value::Nil => break,
                    Value::Pair(next_pair) => {
                        current_pair = next_pair;
                    }
                    _ => {
                        // Improper list - TODO: Handle dotted pairs
                        return Err(LambdustError::type_error(
                            "Cannot convert improper list to expression".to_string()
                        ));
                    }
                }
            }
            
            Ok(Expr::List(exprs))
        }
        Value::Vector(values) => {
            let exprs: Result<Vec<Expr>> = values
                .iter()
                .map(value_to_expr)
                .collect();
            Ok(Expr::Vector(exprs?))
        }
        _ => Err(LambdustError::type_error(format!(
            "Cannot convert {value} to expression for macro expansion"
        ))),
    }
}

/// Convert Expr back to Value
fn expr_to_value(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(Literal::Boolean(b)) => Ok(Value::Boolean(*b)),
        Expr::Literal(Literal::Number(n)) => Ok(Value::Number(n.clone())),
        Expr::Literal(Literal::String(s)) => Ok(Value::from(s.as_str())),
        Expr::Literal(Literal::Character(c)) => Ok(Value::Character(*c)),
        Expr::Literal(Literal::Nil) => Ok(Value::Nil),
        Expr::Variable(name) => Ok(Value::from(name.as_str())),
        Expr::HygienicVariable(symbol) => Ok(Value::from(symbol.name.as_str())),
        Expr::List(exprs) => {
            if exprs.is_empty() {
                Ok(Value::Nil)
            } else {
                // Convert to proper list structure
                let mut result = Value::Nil;
                for expr in exprs.iter().rev() {
                    let value = expr_to_value(expr)?;
                    result = Value::Pair(Rc::new(RefCell::new(PairData {
                        car: value,
                        cdr: result,
                    })));
                }
                Ok(result)
            }
        }
        Expr::Vector(exprs) => {
            let values: Result<Vec<Value>> = exprs
                .iter()
                .map(expr_to_value)
                .collect();
            Ok(Value::Vector(values?))
        }
        _ => Err(LambdustError::type_error(format!(
            "Cannot convert expression {expr:?} back to value"
        ))),
    }
}

/// Built-in macro-expand-1 function
fn macro_expand_1_builtin() -> Value {
    make_builtin_procedure("macro-expand-1", Some(1), |args| {
        check_arity(args, 1)?;
        let form = &args[0];
        
        // Convert Value to Expr for expansion
        let expr = value_to_expr(form)?;
        
        // Create default hygienic environment for now
        // TODO: Get actual current environment
        let env = Rc::new(HygienicEnvironment::new());
        
        // Create expander
        let expander = MacroExpander::new(env);
        
        // Expand once
        let result = expander.expand_once(&expr)?;
        
        // Convert back to Value and return as pair
        let expanded_value = expr_to_value(&result.form)?;
        let expanded_flag = Value::Boolean(result.expanded);
        
        Ok(Value::Pair(Rc::new(RefCell::new(PairData {
            car: expanded_value,
            cdr: expanded_flag,
        }))))
    })
}

/// Built-in macro-expand function
fn macro_expand_builtin() -> Value {
    make_builtin_procedure("macro-expand", Some(1), |args| {
        check_arity(args, 1)?;
        let form = &args[0];
        
        // Convert Value to Expr for expansion
        let expr = value_to_expr(form)?;
        
        // Create default hygienic environment for now
        // TODO: Get actual current environment
        let env = Rc::new(HygienicEnvironment::new());
        
        // Create expander
        let expander = MacroExpander::new(env);
        
        // Expand completely
        let result = expander.expand_completely(&expr)?;
        
        // Convert back to Value and return as pair
        let expanded_value = expr_to_value(&result.form)?;
        let expanded_flag = Value::Boolean(result.expanded);
        
        Ok(Value::Pair(Rc::new(RefCell::new(PairData {
            car: expanded_value,
            cdr: expanded_flag,
        }))))
    })
}

/// Built-in macro-expand-all function
fn macro_expand_all_builtin() -> Value {
    make_builtin_procedure("macro-expand-all", Some(1), |args| {
        check_arity(args, 1)?;
        let form = &args[0];
        
        // Convert Value to Expr for expansion
        let expr = value_to_expr(form)?;
        
        // Create default hygienic environment for now
        // TODO: Get actual current environment
        let env = Rc::new(HygienicEnvironment::new());
        
        // Create expander
        let expander = MacroExpander::new(env);
        
        // Expand all recursively
        let expanded_expr = expander.expand_all(&expr)?;
        
        // Convert back to Value
        expr_to_value(&expanded_expr)
    })
}