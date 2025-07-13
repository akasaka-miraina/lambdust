//! Main Analyzer Module
//!
//! このモジュールはメイン式解析器の実装を提供します。
//! 各種式の解析、キャッシュ管理、統計収集を行います。

use super::constant_folding::ConstantFolder;
use super::core_types::{AnalysisResult, EvaluationComplexity, OptimizationHint, OptimizationStats, TypeHint};
use super::special_forms::SpecialFormsAnalyzer;
use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::{HashMap, HashSet};

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
    pure_functions: HashSet<String>,
}

impl Default for ExpressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionAnalyzer {
    /// Create a new expression analyzer
    #[must_use] 
    pub fn new() -> Self {
        let mut analyzer = ExpressionAnalyzer {
            constants: HashMap::new(),
            type_env: HashMap::new(),
            cache: HashMap::new(),
            pure_functions: HashSet::new(),
        };

        // Register known pure functions
        analyzer.register_pure_functions();
        analyzer
    }

    /// Register built-in pure functions (no side effects)
    fn register_pure_functions(&mut self) {
        let pure_funcs = [
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "abs", "floor", "ceiling", "sqrt", "expt",
            "car", "cdr", "cons", "list", "length", "string-length", "string-ref", "substring",
            "vector-length", "vector-ref", "not", "and", "or", "eq?", "eqv?", "equal?",
            "number?", "string?", "symbol?", "pair?", "null?", "boolean?", "char?", "vector?", "procedure?",
        ];

        for func in &pure_funcs {
            self.pure_functions.insert((*func).to_string());
        }
    }

    /// Analyze an expression for optimization opportunities
    pub fn analyze(&mut self, expr: &Expr, env: Option<&Environment>) -> Result<AnalysisResult> {
        // Check cache first
        let cache_key = format!("{expr:?}");
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let result = self.analyze_expression(expr, env)?;

        // Cache the result
        self.cache.insert(cache_key, result.clone());
        Ok(result)
    }

    /// Internal expression analysis implementation
    fn analyze_expression(
        &mut self,
        expr: &Expr,
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
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

            // Hygienic variables are analyzed like regular variables
            Expr::HygienicVariable(symbol) => self.analyze_variable(&symbol.unique_name(), env),

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
            constant_value: Some(value),
            type_hint,
            complexity: EvaluationComplexity::Constant,
            has_side_effects: false,
            dependencies: Vec::new(),
            optimizations: Vec::new(),
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

        // Check environment if available
        if let Some(env) = env {
            if let Some(value) = env.get(name) {
                if self.is_constant_value(&value) {
                    return Ok(AnalysisResult {
                        is_constant: true,
                        constant_value: Some(value.clone()),
                        type_hint: self.infer_type_from_value(&value),
                        complexity: EvaluationComplexity::Variable,
                        has_side_effects: false,
                        dependencies: vec![name.to_string()],
                        optimizations: vec![OptimizationHint::InlineVariable(name.to_string(), value.clone())],
                    });
                }
            }
        }

        // Get type hint if available
        let type_hint = self.type_env.get(name).cloned().unwrap_or(TypeHint::Unknown);

        Ok(AnalysisResult {
            is_constant: false,
            constant_value: None,
            type_hint,
            complexity: EvaluationComplexity::Variable,
            has_side_effects: false,
            dependencies: vec![name.to_string()],
            optimizations: Vec::new(),
        })
    }

    /// Analyze list expressions (function applications)
    fn analyze_list(
        &mut self,
        exprs: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if exprs.is_empty() {
            // Empty list
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(Value::Nil),
                type_hint: TypeHint::List,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: Vec::new(),
            });
        }

        // Check if it's a special form
        if let Expr::Variable(func_name) = &exprs[0] {
            if SpecialFormsAnalyzer::is_special_form(func_name) {
                return SpecialFormsAnalyzer::analyze_special_form(self, func_name, &exprs[1..], env);
            }
        }

        // Regular function application
        self.analyze_function_application(exprs, env)
    }

    /// Analyze quoted expressions
    fn analyze_quote(&self, expr: &Expr) -> Result<AnalysisResult> {
        // Quoted expressions are always constant
        let value = self.expr_to_value(expr)?;
        Ok(AnalysisResult {
            is_constant: true,
            constant_value: Some(value),
            type_hint: TypeHint::Unknown, // Could be any type
            complexity: EvaluationComplexity::Constant,
            has_side_effects: false,
            dependencies: Vec::new(),
            optimizations: Vec::new(),
        })
    }

    /// Analyze vector expressions
    fn analyze_vector(
        &mut self,
        exprs: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        let mut all_constant = true;
        let mut constant_values = Vec::new();
        let mut dependencies = Vec::new();
        let mut optimizations = Vec::new();
        let mut has_side_effects = false;
        let mut max_complexity = EvaluationComplexity::Constant;

        for expr in exprs {
            let analysis = self.analyze(expr, env)?;
            
            if analysis.is_constant {
                if let Some(value) = analysis.constant_value {
                    constant_values.push(value);
                } else {
                    all_constant = false;
                }
            } else {
                all_constant = false;
            }

            dependencies.extend(analysis.dependencies);
            optimizations.extend(analysis.optimizations);
            has_side_effects = has_side_effects || analysis.has_side_effects;
            max_complexity = std::cmp::max(max_complexity, analysis.complexity);
        }

        let (is_constant, constant_value) = if all_constant {
            (true, Some(Value::Vector(constant_values)))
        } else {
            (false, None)
        };

        Ok(AnalysisResult {
            is_constant,
            constant_value,
            type_hint: TypeHint::Vector,
            complexity: max_complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze function application
    fn analyze_function_application(
        &mut self,
        exprs: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if let Expr::Variable(func_name) = &exprs[0] {
            self.analyze_general_application(func_name, &exprs[1..], env)
        } else {
            // Complex function expression
            let func_analysis = self.analyze(&exprs[0], env)?;
            let mut dependencies = func_analysis.dependencies;
            let mut optimizations = func_analysis.optimizations;
            let mut has_side_effects = func_analysis.has_side_effects;
            let mut max_complexity = func_analysis.complexity;

            for arg in &exprs[1..] {
                let arg_analysis = self.analyze(arg, env)?;
                dependencies.extend(arg_analysis.dependencies);
                optimizations.extend(arg_analysis.optimizations);
                has_side_effects = has_side_effects || arg_analysis.has_side_effects;
                max_complexity = std::cmp::max(max_complexity, arg_analysis.complexity);
            }

            Ok(AnalysisResult {
                is_constant: false,
                constant_value: None,
                type_hint: TypeHint::Unknown,
                complexity: std::cmp::max(max_complexity, EvaluationComplexity::High),
                has_side_effects,
                dependencies,
                optimizations,
            })
        }
    }

    /// Analyze general function application
    fn analyze_general_application(
        &mut self,
        func_name: &str,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        let mut arg_analyses = Vec::new();
        let mut dependencies = Vec::new();
        let mut optimizations = Vec::new();
        let mut has_side_effects = false;
        let mut max_complexity = EvaluationComplexity::Simple;

        // Analyze arguments
        for arg in args {
            let analysis = self.analyze(arg, env)?;
            arg_analyses.push(analysis.clone());
            dependencies.extend(analysis.dependencies);
            optimizations.extend(analysis.optimizations);
            has_side_effects = has_side_effects || analysis.has_side_effects;
            max_complexity = std::cmp::max(max_complexity, analysis.complexity);
        }

        // Check if function is pure
        let func_has_side_effects = !self.pure_functions.contains(func_name);
        has_side_effects = has_side_effects || func_has_side_effects;

        // Attempt constant folding if all arguments are constant
        let all_args_constant = arg_analyses.iter().all(|a| a.is_constant);
        if all_args_constant && self.pure_functions.contains(func_name) {
            let arg_values: Vec<Value> = arg_analyses
                .iter()
                .filter_map(|a| a.constant_value.clone())
                .collect();
            
            if arg_values.len() == args.len() {
                if let Ok(folded_value) = ConstantFolder::try_constant_fold(func_name, &arg_values) {
                    optimizations.push(OptimizationHint::ConstantFold(folded_value.clone()));
                    return Ok(AnalysisResult {
                        is_constant: true,
                        constant_value: Some(folded_value),
                        type_hint: ConstantFolder::infer_function_return_type(func_name, &arg_analyses.iter().map(|a| a.type_hint.clone()).collect::<Vec<_>>()),
                        complexity: max_complexity,
                        has_side_effects,
                        dependencies,
                        optimizations,
                    });
                }
            }
        }

        // Infer return type
        let arg_types: Vec<TypeHint> = arg_analyses.iter().map(|a| a.type_hint.clone()).collect();
        let return_type = ConstantFolder::infer_function_return_type(func_name, &arg_types);

        Ok(AnalysisResult {
            is_constant: false,
            constant_value: None,
            type_hint: return_type,
            complexity: max_complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Infer type from value
    fn infer_type_from_value(&self, value: &Value) -> TypeHint {
        match value {
            Value::Boolean(_) => TypeHint::Boolean,
            Value::Number(_) => TypeHint::Number,
            Value::String(_) => TypeHint::String,
            Value::Character(_) => TypeHint::Character,
            Value::Symbol(_) => TypeHint::Symbol,
            Value::Nil => TypeHint::List,
            Value::Vector(_) => TypeHint::Vector,
            Value::Procedure(_) => TypeHint::Procedure,
            _ => TypeHint::Unknown,
        }
    }

    /// Check if a value is constant (immutable)
    fn is_constant_value(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(_) | Value::Number(_) | Value::String(_) | Value::Character(_) | Value::Symbol(_) => true,
            Value::Nil => true,
            Value::Vector(vec) => vec.iter().all(|v| self.is_constant_value(v)),
            _ => false,
        }
    }

    /// Convert expression to value (for quoted expressions)
    fn expr_to_value(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(Literal::Boolean(b)) => Ok(Value::Boolean(*b)),
            Expr::Literal(Literal::Number(n)) => Ok(Value::Number(n.clone())),
            Expr::Literal(Literal::String(s)) => Ok(Value::String(s.clone())),
            Expr::Literal(Literal::Character(c)) => Ok(Value::Character(*c)),
            Expr::Variable(name) => Ok(Value::Symbol(name.clone())),
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    Ok(Value::Nil)
                } else {
                    // Create a proper list structure with pairs
                    let values: Result<Vec<Value>> = exprs.iter().map(|e| self.expr_to_value(e)).collect();
                    let values = values?;
                    
                    // Convert to proper Scheme list structure
                    if values.is_empty() {
                        Ok(Value::Nil)
                    } else {
                        // For now, use vector representation since list construction is complex
                        Ok(Value::Vector(values))
                    }
                }
            }
            Expr::Vector(exprs) => {
                let values: Result<Vec<Value>> = exprs.iter().map(|e| self.expr_to_value(e)).collect();
                Ok(Value::Vector(values?))
            }
            _ => Err(LambdustError::runtime_error(
                "Cannot convert complex expression to value".to_string(),
            )),
        }
    }

    /// Add a constant to the analyzer's knowledge
    pub fn add_constant(&mut self, name: String, value: Value) {
        self.constants.insert(name, value);
    }

    /// Add a type hint for a variable
    pub fn add_type_hint(&mut self, name: String, type_hint: TypeHint) {
        self.type_env.insert(name, type_hint);
    }

    /// Clear the analysis cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get optimization statistics
    #[must_use] 
    pub fn optimization_stats(&self) -> OptimizationStats {
        let mut stats = OptimizationStats::default();
        stats.total_analyzed = self.cache.len();

        for result in self.cache.values() {
            if result.is_constant {
                stats.constants_found += 1;
            }

            for opt in &result.optimizations {
                match opt {
                    OptimizationHint::ConstantFold(_) => stats.constants_found += 1,
                    OptimizationHint::InlineVariable(_, _) => stats.inlinable_vars_found += 1,
                    OptimizationHint::TailCall => stats.tail_calls_found += 1,
                    OptimizationHint::DeadCode => stats.dead_code_found += 1,
                    _ => {}
                }
            }

            if !result.has_side_effects {
                stats.pure_calls_found += 1;
            }
        }

        // Simple cache hit ratio calculation
        stats.cache_hit_ratio = if stats.total_analyzed > 0 {
            0.5 // Placeholder
        } else {
            0.0
        };

        stats
    }

    /// Get optimization ratio (optimizable expressions / total expressions)
    #[must_use] 
    pub fn optimization_ratio(&self) -> f64 {
        if self.cache.is_empty() {
            return 0.0;
        }

        let optimizable = self.cache.values()
            .filter(|result| result.is_constant || !result.optimizations.is_empty())
            .count();

        optimizable as f64 / self.cache.len() as f64
    }
}