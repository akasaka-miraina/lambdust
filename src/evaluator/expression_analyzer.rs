//! Expression analyzer for compile-time optimization (Phase 5-Step1)
//!
//! This module provides static analysis capabilities for Scheme expressions,
//! enabling constant folding, type hints, and other compile-time optimizations
//! to improve runtime evaluation performance.

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::Value;
use std::collections::HashMap;

/// Expression analysis results containing optimization hints
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Whether the expression is a constant that can be folded
    pub is_constant: bool,
    /// Pre-computed constant value (if is_constant is true)
    pub constant_value: Option<Value>,
    /// Inferred type information
    pub type_hint: TypeHint,
    /// Estimated evaluation complexity
    pub complexity: EvaluationComplexity,
    /// Whether the expression has side effects
    pub has_side_effects: bool,
    /// Variable dependencies
    pub dependencies: Vec<String>,
    /// Optimization suggestions
    pub optimizations: Vec<OptimizationHint>,
}

/// Type hint information for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum TypeHint {
    /// Unknown type
    Unknown,
    /// Boolean type
    Boolean,
    /// Number type
    Number,
    /// String type
    String,
    /// Character type
    Character,
    /// Symbol type
    Symbol,
    /// List type
    List,
    /// Vector type
    Vector,
    /// Procedure type
    Procedure,
    /// Multiple possible types
    Union(Vec<TypeHint>),
}

/// Evaluation complexity estimation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvaluationComplexity {
    /// Constant time (literals, constants)
    Constant,
    /// Simple variable lookup
    Variable,
    /// Simple function call
    Simple,
    /// Moderate complexity (loops, conditions)
    Moderate,
    /// High complexity (recursion, complex computations)
    High,
}

/// Optimization hints for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationHint {
    /// Expression can be constant-folded
    ConstantFold(Value),
    /// Variable can be inlined
    InlineVariable(String, Value),
    /// Function call can be specialized
    SpecializeCall(String, Vec<TypeHint>),
    /// Tail call optimization available
    TailCall,
    /// Dead code elimination possible
    DeadCode,
    /// Loop unrolling opportunity
    UnrollLoop(usize),
}

/// Expression analyzer for static analysis and optimization
#[derive(Debug)]
pub struct ExpressionAnalyzer {
    /// Known constant values in the environment
    constants: HashMap<String, Value>,
    /// Type environment for variables
    type_env: HashMap<String, TypeHint>,
    /// Analysis cache for memoization
    cache: HashMap<String, AnalysisResult>,
    /// Pure function registry (functions without side effects)
    pure_functions: std::collections::HashSet<String>,
}

impl Default for ExpressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionAnalyzer {
    /// Create a new expression analyzer
    pub fn new() -> Self {
        let mut analyzer = ExpressionAnalyzer {
            constants: HashMap::new(),
            type_env: HashMap::new(),
            cache: HashMap::new(),
            pure_functions: std::collections::HashSet::new(),
        };

        // Register known pure functions
        analyzer.register_pure_functions();
        analyzer
    }

    /// Register built-in pure functions (no side effects)
    fn register_pure_functions(&mut self) {
        let pure_funcs = [
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
            "abs", "floor", "ceiling", "sqrt", "expt",
            "car", "cdr", "cons", "list", "length",
            "string-length", "string-ref", "substring",
            "vector-length", "vector-ref",
            "not", "and", "or",
            "eq?", "eqv?", "equal?",
            "number?", "string?", "symbol?", "pair?", "null?",
            "boolean?", "char?", "vector?", "procedure?",
        ];

        for func in &pure_funcs {
            self.pure_functions.insert(func.to_string());
        }
    }

    /// Analyze an expression for optimization opportunities
    pub fn analyze(&mut self, expr: &Expr, env: Option<&Environment>) -> Result<AnalysisResult> {
        // Check cache first
        let cache_key = format!("{:?}", expr);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let result = self.analyze_expression(expr, env)?;
        
        // Cache the result
        self.cache.insert(cache_key, result.clone());
        Ok(result)
    }

    /// Internal expression analysis implementation
    fn analyze_expression(&mut self, expr: &Expr, env: Option<&Environment>) -> Result<AnalysisResult> {
        match expr {
            // Literals are always constant
            Expr::Literal(lit) => self.analyze_literal(lit),
            
            // Variables depend on environment
            Expr::Variable(name) => self.analyze_variable(name, env),
            
            // Lists (function applications)
            Expr::List(exprs) => self.analyze_list(exprs, env),
            
            // Quoted expressions are constant
            Expr::Quote(inner) => self.analyze_quote(inner),
            
            // Quasiquote (simplified as quote for now)
            Expr::Quasiquote(inner) => self.analyze_quote(inner),
            
            // Vectors
            Expr::Vector(exprs) => self.analyze_vector(exprs, env),
            
            // Dotted lists
            Expr::DottedList(_, _) => Ok(AnalysisResult {
                is_constant: false,
                constant_value: None,
                type_hint: TypeHint::List,
                complexity: EvaluationComplexity::Simple,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: Vec::new(),
            }),
            
            // Unquote and UnquoteSplicing (not yet implemented)
            Expr::Unquote(_) | Expr::UnquoteSplicing(_) => Ok(AnalysisResult {
                is_constant: false,
                constant_value: None,
                type_hint: TypeHint::Unknown,
                complexity: EvaluationComplexity::Simple,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: Vec::new(),
            }),
        }
    }

    /// Analyze literal expressions
    fn analyze_literal(&self, lit: &Literal) -> Result<AnalysisResult> {
        let (value, type_hint) = match lit {
            Literal::Boolean(b) => (Value::Boolean(*b), TypeHint::Boolean),
            Literal::Number(n) => (Value::Number(n.clone()), TypeHint::Number),
            Literal::String(s) => (Value::String(s.clone()), TypeHint::String),
            Literal::Character(c) => (Value::Character(*c), TypeHint::Character),
            Literal::Nil => (Value::Nil, TypeHint::List),
        };

        Ok(AnalysisResult {
            is_constant: true,
            constant_value: Some(value.clone()),
            type_hint,
            complexity: EvaluationComplexity::Constant,
            has_side_effects: false,
            dependencies: Vec::new(),
            optimizations: vec![OptimizationHint::ConstantFold(value)],
        })
    }

    /// Analyze variable expressions
    fn analyze_variable(&self, name: &str, env: Option<&Environment>) -> Result<AnalysisResult> {
        // Check if it's a known constant
        if let Some(value) = self.constants.get(name) {
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(value.clone()),
                type_hint: self.infer_type_from_value(value),
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: vec![OptimizationHint::InlineVariable(name.to_string(), value.clone())],
            });
        }

        // Try to get type hint
        let type_hint = self.type_env.get(name).cloned().unwrap_or(TypeHint::Unknown);

        // Check environment for constant values
        let (is_constant, constant_value, optimizations) = if let Some(env) = env {
            if let Some(value) = env.get(name) {
                if self.is_constant_value(&value) {
                    (true, Some(value.clone()), vec![OptimizationHint::InlineVariable(name.to_string(), value)])
                } else {
                    (false, None, Vec::new())
                }
            } else {
                (false, None, Vec::new())
            }
        } else {
            (false, None, Vec::new())
        };

        Ok(AnalysisResult {
            is_constant,
            constant_value,
            type_hint,
            complexity: EvaluationComplexity::Variable,
            has_side_effects: false,
            dependencies: vec![name.to_string()],
            optimizations,
        })
    }

    /// Analyze list expressions (function applications)
    fn analyze_list(&mut self, exprs: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        if exprs.is_empty() {
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(Value::Nil),
                type_hint: TypeHint::List,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: vec![OptimizationHint::ConstantFold(Value::Nil)],
            });
        }

        // Check for special forms
        if let Expr::Variable(name) = &exprs[0] {
            if self.is_special_form(name) {
                return self.analyze_special_form(name, &exprs[1..], env);
            }
            
            // Check for function applications
            return self.analyze_function_application(name, &exprs[1..], env);
        }

        // General application case
        self.analyze_general_application(exprs, env)
    }

    /// Analyze quoted expressions
    fn analyze_quote(&self, expr: &Expr) -> Result<AnalysisResult> {
        // Quoted expressions are always constant
        let value = self.expr_to_value(expr)?;
        Ok(AnalysisResult {
            is_constant: true,
            constant_value: Some(value.clone()),
            type_hint: self.infer_type_from_value(&value),
            complexity: EvaluationComplexity::Constant,
            has_side_effects: false,
            dependencies: Vec::new(),
            optimizations: vec![OptimizationHint::ConstantFold(value)],
        })
    }

    /// Analyze vector expressions
    fn analyze_vector(&mut self, exprs: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        let mut all_constant = true;
        let mut constant_values = Vec::new();
        let mut complexity = EvaluationComplexity::Constant;
        let mut has_side_effects = false;
        let mut dependencies = Vec::new();
        let mut optimizations = Vec::new();

        for expr in exprs {
            let result = self.analyze_expression(expr, env)?;
            
            if result.is_constant {
                if let Some(value) = result.constant_value {
                    constant_values.push(value);
                } else {
                    all_constant = false;
                }
            } else {
                all_constant = false;
            }

            complexity = complexity.max(result.complexity);
            has_side_effects |= result.has_side_effects;
            dependencies.extend(result.dependencies);
        }

        let (constant_value, opt_hint) = if all_constant {
            let vector_value = Value::Vector(constant_values);
            (Some(vector_value.clone()), vec![OptimizationHint::ConstantFold(vector_value)])
        } else {
            (None, Vec::new())
        };

        optimizations.extend(opt_hint);

        Ok(AnalysisResult {
            is_constant: all_constant,
            constant_value,
            type_hint: TypeHint::Vector,
            complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze special forms
    fn analyze_special_form(&mut self, name: &str, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        match name {
            "if" => self.analyze_if_form(args, env),
            "and" => self.analyze_and_form(args, env),
            "or" => self.analyze_or_form(args, env),
            "begin" => self.analyze_begin_form(args, env),
            "quote" => {
                if args.len() == 1 {
                    self.analyze_quote(&args[0])
                } else {
                    Err(LambdustError::syntax_error("quote requires exactly one argument".to_string()))
                }
            }
            "lambda" => self.analyze_lambda_form(args, env),
            "define" => self.analyze_define_form(args, env),
            _ => {
                // Default special form analysis
                Ok(AnalysisResult {
                    is_constant: false,
                    constant_value: None,
                    type_hint: TypeHint::Unknown,
                    complexity: EvaluationComplexity::Moderate,
                    has_side_effects: true,
                    dependencies: Vec::new(),
                    optimizations: Vec::new(),
                })
            }
        }
    }

    /// Analyze function applications
    fn analyze_function_application(&mut self, func_name: &str, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        // Analyze arguments
        let mut arg_results = Vec::new();
        let mut all_args_constant = true;
        let mut arg_values = Vec::new();
        let mut complexity = EvaluationComplexity::Simple;
        let mut has_side_effects = false;
        let mut dependencies = Vec::new();
        let mut arg_types = Vec::new();

        for arg in args {
            let result = self.analyze_expression(arg, env)?;
            
            if result.is_constant {
                if let Some(value) = &result.constant_value {
                    arg_values.push(value.clone());
                } else {
                    all_args_constant = false;
                }
            } else {
                all_args_constant = false;
            }

            complexity = complexity.max(result.complexity.clone());
            has_side_effects |= result.has_side_effects;
            dependencies.extend(result.dependencies.clone());
            arg_types.push(result.type_hint.clone());
            arg_results.push(result);
        }

        // Check if function is pure and arguments are constant
        let is_pure = self.pure_functions.contains(func_name);
        has_side_effects |= !is_pure;

        let (is_constant, constant_value, optimizations) = if is_pure && all_args_constant {
            // Try constant folding for pure functions with constant arguments
            match self.try_constant_fold(func_name, &arg_values) {
                Ok(value) => (true, Some(value.clone()), vec![OptimizationHint::ConstantFold(value)]),
                Err(_) => (false, None, Vec::new()),
            }
        } else {
            let mut opts = Vec::new();
            
            // Suggest specialization if we have type hints
            if !arg_types.iter().any(|t| matches!(t, TypeHint::Unknown)) {
                opts.push(OptimizationHint::SpecializeCall(func_name.to_string(), arg_types.clone()));
            }
            
            (false, None, opts)
        };

        Ok(AnalysisResult {
            is_constant,
            constant_value,
            type_hint: self.infer_function_return_type(func_name, &arg_types),
            complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze general applications (non-function-name first element)
    fn analyze_general_application(&mut self, exprs: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        let mut complexity = EvaluationComplexity::Simple;
        let mut has_side_effects = true; // Conservative assumption
        let mut dependencies = Vec::new();

        for expr in exprs {
            let result = self.analyze_expression(expr, env)?;
            complexity = complexity.max(result.complexity);
            has_side_effects |= result.has_side_effects;
            dependencies.extend(result.dependencies);
        }

        Ok(AnalysisResult {
            is_constant: false,
            constant_value: None,
            type_hint: TypeHint::Unknown,
            complexity,
            has_side_effects,
            dependencies,
            optimizations: Vec::new(),
        })
    }

    /// Analyze if expressions
    fn analyze_if_form(&mut self, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        if args.len() < 2 || args.len() > 3 {
            return Err(LambdustError::syntax_error("if requires 2 or 3 arguments".to_string()));
        }

        let test_result = self.analyze_expression(&args[0], env)?;
        let then_result = self.analyze_expression(&args[1], env)?;
        let else_result = if args.len() == 3 {
            Some(self.analyze_expression(&args[2], env)?)
        } else {
            None
        };

        // Check for constant condition
        let (is_constant, constant_value, optimizations) = if test_result.is_constant {
            if let Some(test_value) = &test_result.constant_value {
                if test_value.is_truthy() {
                    // Condition is always true, use then branch
                    (then_result.is_constant, then_result.constant_value.clone(), 
                     if then_result.is_constant && then_result.constant_value.is_some() {
                         vec![OptimizationHint::ConstantFold(then_result.constant_value.unwrap())]
                     } else {
                         vec![OptimizationHint::DeadCode] // else branch is dead
                     })
                } else {
                    // Condition is always false, use else branch
                    if let Some(else_res) = &else_result {
                        (else_res.is_constant, else_res.constant_value.clone(),
                         if else_res.is_constant && else_res.constant_value.is_some() {
                             vec![OptimizationHint::ConstantFold(else_res.constant_value.clone().unwrap())]
                         } else {
                             vec![OptimizationHint::DeadCode] // then branch is dead
                         })
                    } else {
                        (true, Some(Value::Undefined), vec![OptimizationHint::ConstantFold(Value::Undefined)])
                    }
                }
            } else {
                (false, None, Vec::new())
            }
        } else {
            (false, None, Vec::new())
        };

        let mut complexity = test_result.complexity.max(then_result.complexity);
        let mut has_side_effects = test_result.has_side_effects || then_result.has_side_effects;
        let mut dependencies = test_result.dependencies;
        dependencies.extend(then_result.dependencies);

        if let Some(else_res) = &else_result {
            complexity = complexity.max(else_res.complexity.clone());
            has_side_effects |= else_res.has_side_effects;
            dependencies.extend(else_res.dependencies.clone());
        }

        let type_hint = if let Some(else_res) = else_result {
            if then_result.type_hint == else_res.type_hint {
                then_result.type_hint
            } else {
                TypeHint::Union(vec![then_result.type_hint, else_res.type_hint])
            }
        } else {
            TypeHint::Union(vec![then_result.type_hint, TypeHint::Unknown])
        };

        Ok(AnalysisResult {
            is_constant,
            constant_value,
            type_hint,
            complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze and expressions
    fn analyze_and_form(&mut self, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        if args.is_empty() {
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(Value::Boolean(true)),
                type_hint: TypeHint::Boolean,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: vec![OptimizationHint::ConstantFold(Value::Boolean(true))],
            });
        }

        let mut complexity = EvaluationComplexity::Constant;
        let mut has_side_effects = false;
        let mut dependencies = Vec::new();
        let mut all_constant = true;

        for arg in args {
            let result = self.analyze_expression(arg, env)?;
            complexity = complexity.max(result.complexity);
            has_side_effects |= result.has_side_effects;
            dependencies.extend(result.dependencies);

            if !result.is_constant {
                all_constant = false;
            } else if let Some(value) = &result.constant_value {
                if !value.is_truthy() {
                    // Short circuit: and returns false
                    return Ok(AnalysisResult {
                        is_constant: true,
                        constant_value: Some(Value::Boolean(false)),
                        type_hint: TypeHint::Boolean,
                        complexity,
                        has_side_effects,
                        dependencies,
                        optimizations: vec![OptimizationHint::ConstantFold(Value::Boolean(false))],
                    });
                }
            }
        }

        let (is_constant, constant_value, optimizations) = if all_constant {
            // All arguments are truthy constants, return the last one
            let last_result = self.analyze_expression(&args[args.len() - 1], env)?;
            (true, last_result.constant_value.clone(), 
             if let Some(value) = last_result.constant_value {
                 vec![OptimizationHint::ConstantFold(value)]
             } else {
                 Vec::new()
             })
        } else {
            (false, None, Vec::new())
        };

        Ok(AnalysisResult {
            is_constant,
            constant_value,
            type_hint: TypeHint::Unknown, // Could be any type
            complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze or expressions
    fn analyze_or_form(&mut self, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        if args.is_empty() {
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(Value::Boolean(false)),
                type_hint: TypeHint::Boolean,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: vec![OptimizationHint::ConstantFold(Value::Boolean(false))],
            });
        }

        let mut complexity = EvaluationComplexity::Constant;
        let mut has_side_effects = false;
        let mut dependencies = Vec::new();

        for arg in args {
            let result = self.analyze_expression(arg, env)?;
            complexity = complexity.max(result.complexity);
            has_side_effects |= result.has_side_effects;
            dependencies.extend(result.dependencies);

            if result.is_constant {
                if let Some(value) = &result.constant_value {
                    if value.is_truthy() {
                        // Short circuit: or returns this truthy value
                        return Ok(AnalysisResult {
                            is_constant: true,
                            constant_value: Some(value.clone()),
                            type_hint: self.infer_type_from_value(value),
                            complexity,
                            has_side_effects,
                            dependencies,
                            optimizations: vec![OptimizationHint::ConstantFold(value.clone())],
                        });
                    }
                }
            } else {
                // Can't determine constant folding with non-constant expressions
                return Ok(AnalysisResult {
                    is_constant: false,
                    constant_value: None,
                    type_hint: TypeHint::Unknown,
                    complexity,
                    has_side_effects,
                    dependencies,
                    optimizations: Vec::new(),
                });
            }
        }

        // All arguments are falsy constants
        Ok(AnalysisResult {
            is_constant: true,
            constant_value: Some(Value::Boolean(false)),
            type_hint: TypeHint::Boolean,
            complexity,
            has_side_effects,
            dependencies,
            optimizations: vec![OptimizationHint::ConstantFold(Value::Boolean(false))],
        })
    }

    /// Analyze begin expressions
    fn analyze_begin_form(&mut self, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        if args.is_empty() {
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(Value::Undefined),
                type_hint: TypeHint::Unknown,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: vec![OptimizationHint::ConstantFold(Value::Undefined)],
            });
        }

        let mut complexity = EvaluationComplexity::Constant;
        let mut has_side_effects = false;
        let mut dependencies = Vec::new();

        // Analyze all expressions except the last for side effects
        for arg in &args[..args.len() - 1] {
            let result = self.analyze_expression(arg, env)?;
            complexity = complexity.max(result.complexity);
            has_side_effects |= result.has_side_effects;
            dependencies.extend(result.dependencies);
        }

        // The result is determined by the last expression
        let last_result = self.analyze_expression(&args[args.len() - 1], env)?;
        complexity = complexity.max(last_result.complexity);
        has_side_effects |= last_result.has_side_effects;
        dependencies.extend(last_result.dependencies);

        Ok(AnalysisResult {
            is_constant: !has_side_effects && last_result.is_constant,
            constant_value: if !has_side_effects { last_result.constant_value.clone() } else { None },
            type_hint: last_result.type_hint,
            complexity,
            has_side_effects,
            dependencies,
            optimizations: if !has_side_effects {
                if let Some(value) = &last_result.constant_value {
                    vec![OptimizationHint::ConstantFold(value.clone())]
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            },
        })
    }

    /// Analyze lambda expressions
    fn analyze_lambda_form(&mut self, args: &[Expr], _env: Option<&Environment>) -> Result<AnalysisResult> {
        if args.len() < 2 {
            return Err(LambdustError::syntax_error("lambda requires at least 2 arguments".to_string()));
        }

        // Lambda expressions are never constant but have no immediate side effects
        Ok(AnalysisResult {
            is_constant: false,
            constant_value: None,
            type_hint: TypeHint::Procedure,
            complexity: EvaluationComplexity::Simple,
            has_side_effects: false,
            dependencies: Vec::new(),
            optimizations: Vec::new(),
        })
    }

    /// Analyze define expressions
    fn analyze_define_form(&mut self, args: &[Expr], env: Option<&Environment>) -> Result<AnalysisResult> {
        if args.len() != 2 {
            return Err(LambdustError::syntax_error("define requires exactly 2 arguments".to_string()));
        }

        // Analyze the value expression
        let value_result = self.analyze_expression(&args[1], env)?;

        // Define has side effects (environment modification)
        Ok(AnalysisResult {
            is_constant: false,
            constant_value: None,
            type_hint: TypeHint::Unknown,
            complexity: value_result.complexity,
            has_side_effects: true,
            dependencies: value_result.dependencies,
            optimizations: Vec::new(),
        })
    }

    /// Try to perform constant folding for pure functions
    fn try_constant_fold(&self, func_name: &str, args: &[Value]) -> Result<Value> {
        match func_name {
            "+" => self.fold_arithmetic_operation(args, |a, b| a + b),
            "-" => self.fold_subtraction(args),
            "*" => self.fold_arithmetic_operation(args, |a, b| a * b),
            "/" => self.fold_division(args),
            "=" => self.fold_numeric_comparison(args, |a, b| a == b),
            "<" => self.fold_numeric_comparison(args, |a, b| a < b),
            ">" => self.fold_numeric_comparison(args, |a, b| a > b),
            "<=" => self.fold_numeric_comparison(args, |a, b| a <= b),
            ">=" => self.fold_numeric_comparison(args, |a, b| a >= b),
            "not" => self.fold_not(args),
            "car" => self.fold_car(args),
            "cdr" => self.fold_cdr(args),
            "length" => self.fold_length(args),
            _ => Err(LambdustError::runtime_error(format!("Cannot constant fold {}", func_name))),
        }
    }

    /// Fold arithmetic operations (+, *)
    fn fold_arithmetic_operation<F>(&self, args: &[Value], op: F) -> Result<Value>
    where
        F: Fn(f64, f64) -> f64,
    {
        if args.is_empty() {
            return Err(LambdustError::arity_error(1, 0));
        }

        let mut has_real = false;
        let mut result = 0.0;
        
        for (i, arg) in args.iter().enumerate() {
            match arg {
                Value::Number(SchemeNumber::Integer(n)) => {
                    let val = *n as f64;
                    if i == 0 {
                        result = val;
                    } else {
                        result = op(result, val);
                    }
                }
                Value::Number(SchemeNumber::Real(f)) => {
                    has_real = true;
                    if i == 0 {
                        result = *f;
                    } else {
                        result = op(result, *f);
                    }
                }
                _ => {
                    return Err(LambdustError::type_error("Expected number".to_string()));
                }
            }
        }

        // Return integer if all inputs were integers and result is a whole number
        if !has_real && result.fract() == 0.0 && result.is_finite() {
            Ok(Value::Number(SchemeNumber::Integer(result as i64)))
        } else {
            Ok(Value::Number(SchemeNumber::Real(result)))
        }
    }

    /// Fold subtraction
    fn fold_subtraction(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(LambdustError::arity_error(1, 0));
        }

        if args.len() == 1 {
            // Negation
            match &args[0] {
                Value::Number(SchemeNumber::Integer(n)) => Ok(Value::Number(SchemeNumber::Integer(-n))),
                Value::Number(SchemeNumber::Real(f)) => Ok(Value::Number(SchemeNumber::Real(-f))),
                _ => Err(LambdustError::type_error("Expected number".to_string())),
            }
        } else {
            self.fold_arithmetic_operation(args, |a, b| a - b)
        }
    }

    /// Fold division
    fn fold_division(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }

        self.fold_arithmetic_operation(args, |a, b| {
            if b == 0.0 {
                f64::NAN // This will be caught by the evaluator
            } else {
                a / b
            }
        })
    }

    /// Fold numeric comparisons
    fn fold_numeric_comparison<F>(&self, args: &[Value], op: F) -> Result<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        if args.len() != 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }

        let a = match &args[0] {
            Value::Number(SchemeNumber::Integer(n)) => *n as f64,
            Value::Number(SchemeNumber::Real(f)) => *f,
            _ => return Err(LambdustError::type_error("Expected number".to_string())),
        };

        let b = match &args[1] {
            Value::Number(SchemeNumber::Integer(n)) => *n as f64,
            Value::Number(SchemeNumber::Real(f)) => *f,
            _ => return Err(LambdustError::type_error("Expected number".to_string())),
        };

        Ok(Value::Boolean(op(a, b)))
    }

    /// Fold not operation
    fn fold_not(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        Ok(Value::Boolean(!args[0].is_truthy()))
    }

    /// Fold car operation
    fn fold_car(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        match &args[0] {
            Value::Pair(pair) => {
                let pair_data = pair.borrow();
                Ok(pair_data.car.clone())
            }
            Value::Nil => Err(LambdustError::runtime_error("Cannot take car of empty list".to_string())),
            _ => Err(LambdustError::type_error("Expected pair".to_string())),
        }
    }

    /// Fold cdr operation
    fn fold_cdr(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        match &args[0] {
            Value::Pair(pair) => {
                let pair_data = pair.borrow();
                Ok(pair_data.cdr.clone())
            }
            Value::Nil => Err(LambdustError::runtime_error("Cannot take cdr of empty list".to_string())),
            _ => Err(LambdustError::type_error("Expected pair".to_string())),
        }
    }

    /// Fold length operation
    fn fold_length(&self, args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        match &args[0] {
            Value::Nil => Ok(Value::Number(SchemeNumber::Integer(0))),
            Value::Vector(vec) => Ok(Value::Number(SchemeNumber::Integer(vec.len() as i64))),
            Value::String(s) => Ok(Value::Number(SchemeNumber::Integer(s.len() as i64))),
            Value::Pair(_) => {
                // For now, we can't easily constant fold list length without more complex logic
                // This would require walking the list structure at analysis time
                Err(LambdustError::runtime_error("Cannot constant fold list length".to_string()))
            }
            _ => Err(LambdustError::type_error("Expected list, vector, or string".to_string())),
        }
    }

    /// Check if a name represents a special form
    fn is_special_form(&self, name: &str) -> bool {
        matches!(
            name,
            "lambda" | "if" | "set!" | "quote" | "define" | "begin" | "and" | "or" | "cond" | "case" | "do"
            | "delay" | "lazy" | "force" | "promise?" | "call/cc" | "call-with-current-continuation"
            | "values" | "call-with-values" | "dynamic-wind" | "raise" | "with-exception-handler" | "guard"
        )
    }

    /// Infer return type of function
    fn infer_function_return_type(&self, func_name: &str, arg_types: &[TypeHint]) -> TypeHint {
        match func_name {
            "+" | "-" | "*" | "/" | "abs" | "floor" | "ceiling" | "sqrt" | "expt" | "length" => TypeHint::Number,
            "=" | "<" | ">" | "<=" | ">=" | "not" | "eq?" | "eqv?" | "equal?" 
            | "number?" | "string?" | "symbol?" | "pair?" | "null?" | "boolean?" 
            | "char?" | "vector?" | "procedure?" => TypeHint::Boolean,
            "string-ref" | "char-upcase" | "char-downcase" => TypeHint::Character,
            "cons" => TypeHint::List,
            "car" | "cdr" => {
                if let Some(TypeHint::List) = arg_types.first() {
                    TypeHint::Unknown // Could be any type
                } else {
                    TypeHint::Unknown
                }
            }
            "list" => TypeHint::List,
            _ => TypeHint::Unknown,
        }
    }

    /// Infer type from a value
    fn infer_type_from_value(&self, value: &Value) -> TypeHint {
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

    /// Check if a value is considered constant
    fn is_constant_value(&self, value: &Value) -> bool {
        matches!(
            value,
            Value::Boolean(_) | Value::Number(_) | Value::String(_) 
            | Value::Character(_) | Value::Symbol(_) | Value::Nil
        )
    }

    /// Convert expression to value (for quoted expressions)
    #[allow(clippy::only_used_in_recursion)]
    fn expr_to_value(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                    Literal::Number(n) => Ok(Value::Number(n.clone())),
                    Literal::String(s) => Ok(Value::String(s.clone())),
                    Literal::Character(c) => Ok(Value::Character(*c)),
                    Literal::Nil => Ok(Value::Nil),
                }
            }
            Expr::Variable(name) => Ok(Value::Symbol(name.clone())),
            Expr::List(exprs) => {
                // Convert to list structure
                let mut result = Value::Nil;
                for expr in exprs.iter().rev() {
                    let value = self.expr_to_value(expr)?;
                    result = Value::cons(value, result);
                }
                Ok(result)
            }
            Expr::Vector(exprs) => {
                let mut values = Vec::new();
                for expr in exprs {
                    values.push(self.expr_to_value(expr)?);
                }
                Ok(Value::Vector(values))
            }
            _ => Err(LambdustError::runtime_error("Cannot convert expression to value".to_string())),
        }
    }

    /// Add a constant to the analyzer
    pub fn add_constant(&mut self, name: String, value: Value) {
        self.constants.insert(name, value);
    }

    /// Add type hint for a variable
    pub fn add_type_hint(&mut self, name: String, type_hint: TypeHint) {
        self.type_env.insert(name, type_hint);
    }

    /// Clear the analysis cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get optimization statistics
    pub fn optimization_stats(&self) -> OptimizationStats {
        let mut constant_folds = 0;
        let mut dead_code_eliminations = 0;
        let mut specializations = 0;

        for result in self.cache.values() {
            for optimization in &result.optimizations {
                match optimization {
                    OptimizationHint::ConstantFold(_) => constant_folds += 1,
                    OptimizationHint::DeadCode => dead_code_eliminations += 1,
                    OptimizationHint::SpecializeCall(_, _) => specializations += 1,
                    _ => {}
                }
            }
        }

        OptimizationStats {
            constant_folds,
            dead_code_eliminations,
            specializations,
            total_analyses: self.cache.len(),
        }
    }
}

/// Statistics for optimization analysis
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Number of constant folding opportunities
    pub constant_folds: usize,
    /// Number of dead code elimination opportunities
    pub dead_code_eliminations: usize,
    /// Number of function specialization opportunities
    pub specializations: usize,
    /// Total number of analyses performed
    pub total_analyses: usize,
}

impl OptimizationStats {
    /// Calculate optimization ratio
    pub fn optimization_ratio(&self) -> f64 {
        if self.total_analyses == 0 {
            0.0
        } else {
            (self.constant_folds + self.dead_code_eliminations + self.specializations) as f64 
            / self.total_analyses as f64
        }
    }
}