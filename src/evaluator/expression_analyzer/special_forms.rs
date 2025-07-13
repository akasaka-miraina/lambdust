//! Special Forms Analysis Module
//!
//! このモジュールは特殊形式（if, and, or, begin, lambda, define等）の解析を実装します。
//! 各特殊形式の特別な意味論的解析と最適化機会の検出を行います。

use super::core_types::{AnalysisResult, EvaluationComplexity, OptimizationHint, TypeHint};
use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};

/// Special forms analyzer
#[derive(Debug)]
pub struct SpecialFormsAnalyzer;

impl SpecialFormsAnalyzer {
    /// Check if a name is a special form
    pub fn is_special_form(name: &str) -> bool {
        matches!(
            name,
            "if"
                | "cond"
                | "case"
                | "and"
                | "or"
                | "when"
                | "unless"
                | "begin"
                | "lambda"
                | "define"
                | "define-syntax"
                | "let"
                | "let*"
                | "letrec"
                | "letrec*"
                | "let-values"
                | "let*-values"
                | "quote"
                | "quasiquote"
                | "unquote"
                | "unquote-splicing"
                | "set!"
                | "syntax-rules"
                | "syntax-case"
                | "delay"
                | "force"
                | "call/cc"
                | "call-with-current-continuation"
                | "eval"
                | "apply"
                | "values"
                | "call-with-values"
        )
    }

    /// Analyze a special form
    pub fn analyze_special_form(
        analyzer: &mut super::analyzer::ExpressionAnalyzer,
        name: &str,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        match name {
            "if" => Self::analyze_if_form(analyzer, args, env),
            "and" => Self::analyze_and_form(analyzer, args, env),
            "or" => Self::analyze_or_form(analyzer, args, env),
            "begin" => Self::analyze_begin_form(analyzer, args, env),
            "lambda" => Self::analyze_lambda_form(args),
            "define" => Self::analyze_define_form(analyzer, args, env),
            _ => Ok(AnalysisResult {
                is_constant: false,
                constant_value: None,
                type_hint: TypeHint::Unknown,
                complexity: EvaluationComplexity::Moderate,
                has_side_effects: Self::special_form_has_side_effects(name),
                dependencies: Vec::new(),
                optimizations: Vec::new(),
            }),
        }
    }

    /// Analyze if special form
    fn analyze_if_form(
        analyzer: &mut super::analyzer::ExpressionAnalyzer,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if args.len() < 2 || args.len() > 3 {
            return Err(LambdustError::syntax_error(
                "if requires 2 or 3 arguments".to_string(),
            ));
        }

        let test_analysis = analyzer.analyze(&args[0], env)?;
        let then_analysis = analyzer.analyze(&args[1], env)?;

        let else_analysis = if args.len() == 3 {
            Some(analyzer.analyze(&args[2], env)?)
        } else {
            None
        };

        let mut dependencies = test_analysis.dependencies.clone();
        dependencies.extend(then_analysis.dependencies.clone());
        if let Some(ref else_result) = else_analysis {
            dependencies.extend(else_result.dependencies.clone());
        }

        let mut optimizations = test_analysis.optimizations.clone();
        optimizations.extend(then_analysis.optimizations.clone());
        if let Some(ref else_result) = else_analysis {
            optimizations.extend(else_result.optimizations.clone());
        }

        // Check if test is constant
        if test_analysis.is_constant {
            if let Some(test_value) = &test_analysis.constant_value {
                // Determine which branch will be taken
                let test_is_true = match test_value {
                    crate::value::Value::Boolean(false) => false,
                    _ => true, // Everything else is truthy in Scheme
                };

                if test_is_true {
                    // Then branch will be taken
                    optimizations.push(OptimizationHint::DeadCode); // Else branch is dead
                    Ok(AnalysisResult {
                        is_constant: then_analysis.is_constant,
                        constant_value: then_analysis.constant_value,
                        type_hint: then_analysis.type_hint,
                        complexity: then_analysis.complexity,
                        has_side_effects: then_analysis.has_side_effects,
                        dependencies,
                        optimizations,
                    })
                } else {
                    // Else branch will be taken (or undefined if no else)
                    optimizations.push(OptimizationHint::DeadCode); // Then branch is dead
                    if let Some(else_result) = else_analysis {
                        Ok(AnalysisResult {
                            is_constant: else_result.is_constant,
                            constant_value: else_result.constant_value,
                            type_hint: else_result.type_hint,
                            complexity: else_result.complexity,
                            has_side_effects: else_result.has_side_effects,
                            dependencies,
                            optimizations,
                        })
                    } else {
                        // No else clause, result is undefined (implementation-specific)
                        Ok(AnalysisResult {
                            is_constant: true,
                            constant_value: Some(crate::value::Value::Undefined),
                            type_hint: TypeHint::Unknown,
                            complexity: EvaluationComplexity::Constant,
                            has_side_effects: false,
                            dependencies,
                            optimizations,
                        })
                    }
                }
            } else {
                // Fallback for constant test without value
                Self::create_conditional_result(then_analysis, else_analysis, dependencies, optimizations)
            }
        } else {
            // Test is not constant, both branches are possible
            Self::create_conditional_result(then_analysis, else_analysis, dependencies, optimizations)
        }
    }

    /// Helper to create result for conditional expressions
    fn create_conditional_result(
        then_analysis: AnalysisResult,
        else_analysis: Option<AnalysisResult>,
        dependencies: Vec<String>,
        optimizations: Vec<OptimizationHint>,
    ) -> Result<AnalysisResult> {
        let has_side_effects = then_analysis.has_side_effects
            || else_analysis
                .as_ref()
                .map(|e| e.has_side_effects)
                .unwrap_or(false);

        let complexity = std::cmp::max(
            then_analysis.complexity,
            else_analysis
                .as_ref()
                .map(|e| e.complexity.clone())
                .unwrap_or(EvaluationComplexity::Constant),
        );

        // Type hint is union of possible types
        let type_hint = if let Some(else_result) = &else_analysis {
            if then_analysis.type_hint == else_result.type_hint {
                then_analysis.type_hint
            } else {
                TypeHint::Union(vec![then_analysis.type_hint, else_result.type_hint.clone()])
            }
        } else {
            TypeHint::Union(vec![then_analysis.type_hint, TypeHint::Unknown])
        };

        Ok(AnalysisResult {
            is_constant: false,
            constant_value: None,
            type_hint,
            complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze and special form
    fn analyze_and_form(
        analyzer: &mut super::analyzer::ExpressionAnalyzer,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if args.is_empty() {
            // (and) evaluates to #t
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(crate::value::Value::Boolean(true)),
                type_hint: TypeHint::Boolean,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: Vec::new(),
            });
        }

        let mut all_constant = true;
        let mut dependencies = Vec::new();
        let mut optimizations = Vec::new();
        let mut has_side_effects = false;
        let mut max_complexity = EvaluationComplexity::Constant;

        // Analyze each argument
        for arg in args {
            let analysis = analyzer.analyze(arg, env)?;
            
            dependencies.extend(analysis.dependencies);
            optimizations.extend(analysis.optimizations);
            has_side_effects = has_side_effects || analysis.has_side_effects;
            max_complexity = std::cmp::max(max_complexity, analysis.complexity);

            if !analysis.is_constant {
                all_constant = false;
            }

            // Short-circuit optimization: if we find a constant false, rest is dead code
            if analysis.is_constant {
                if let Some(value) = &analysis.constant_value {
                    if matches!(value, crate::value::Value::Boolean(false)) {
                        optimizations.push(OptimizationHint::DeadCode);
                        return Ok(AnalysisResult {
                            is_constant: true,
                            constant_value: Some(crate::value::Value::Boolean(false)),
                            type_hint: TypeHint::Boolean,
                            complexity: max_complexity,
                            has_side_effects,
                            dependencies,
                            optimizations,
                        });
                    }
                }
            }
        }

        // If all arguments are constant and none is false, result is the last value
        if all_constant {
            let last_analysis = analyzer.analyze(&args[args.len() - 1], env)?;
            Ok(AnalysisResult {
                is_constant: true,
                constant_value: last_analysis.constant_value,
                type_hint: last_analysis.type_hint,
                complexity: max_complexity,
                has_side_effects,
                dependencies,
                optimizations,
            })
        } else {
            Ok(AnalysisResult {
                is_constant: false,
                constant_value: None,
                type_hint: TypeHint::Unknown, // Could be any of the argument types
                complexity: max_complexity,
                has_side_effects,
                dependencies,
                optimizations,
            })
        }
    }

    /// Analyze or special form
    fn analyze_or_form(
        analyzer: &mut super::analyzer::ExpressionAnalyzer,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if args.is_empty() {
            // (or) evaluates to #f
            return Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(crate::value::Value::Boolean(false)),
                type_hint: TypeHint::Boolean,
                complexity: EvaluationComplexity::Constant,
                has_side_effects: false,
                dependencies: Vec::new(),
                optimizations: Vec::new(),
            });
        }

        let mut all_constant = true;
        let mut dependencies = Vec::new();
        let mut optimizations = Vec::new();
        let mut has_side_effects = false;
        let mut max_complexity = EvaluationComplexity::Constant;

        // Analyze each argument
        for arg in args {
            let analysis = analyzer.analyze(arg, env)?;
            
            dependencies.extend(analysis.dependencies);
            optimizations.extend(analysis.optimizations);
            has_side_effects = has_side_effects || analysis.has_side_effects;
            max_complexity = std::cmp::max(max_complexity, analysis.complexity);

            if !analysis.is_constant {
                all_constant = false;
            }

            // Short-circuit optimization: if we find a constant truthy value, rest is dead code
            if analysis.is_constant {
                if let Some(value) = &analysis.constant_value {
                    if !matches!(value, crate::value::Value::Boolean(false)) {
                        optimizations.push(OptimizationHint::DeadCode);
                        return Ok(AnalysisResult {
                            is_constant: true,
                            constant_value: Some(value.clone()),
                            type_hint: analysis.type_hint,
                            complexity: max_complexity,
                            has_side_effects,
                            dependencies,
                            optimizations,
                        });
                    }
                }
            }
        }

        // If all arguments are constant and all are false, result is false
        if all_constant {
            Ok(AnalysisResult {
                is_constant: true,
                constant_value: Some(crate::value::Value::Boolean(false)),
                type_hint: TypeHint::Boolean,
                complexity: max_complexity,
                has_side_effects,
                dependencies,
                optimizations,
            })
        } else {
            Ok(AnalysisResult {
                is_constant: false,
                constant_value: None,
                type_hint: TypeHint::Unknown, // Could be any of the argument types
                complexity: max_complexity,
                has_side_effects,
                dependencies,
                optimizations,
            })
        }
    }

    /// Analyze begin special form
    fn analyze_begin_form(
        analyzer: &mut super::analyzer::ExpressionAnalyzer,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if args.is_empty() {
            return Err(LambdustError::syntax_error(
                "begin requires at least 1 argument".to_string(),
            ));
        }

        let mut dependencies = Vec::new();
        let mut optimizations = Vec::new();
        let mut has_side_effects = false;
        let mut max_complexity = EvaluationComplexity::Constant;

        // Analyze all expressions
        for (i, arg) in args.iter().enumerate() {
            let analysis = analyzer.analyze(arg, env)?;
            
            dependencies.extend(analysis.dependencies);
            optimizations.extend(analysis.optimizations);
            has_side_effects = has_side_effects || analysis.has_side_effects;
            max_complexity = std::cmp::max(max_complexity, analysis.complexity);

            // All expressions except the last are evaluated for side effects only
            if i < args.len() - 1 && !analysis.has_side_effects {
                optimizations.push(OptimizationHint::DeadCode);
            }
        }

        // Result is the analysis of the last expression
        let last_analysis = analyzer.analyze(&args[args.len() - 1], env)?;

        Ok(AnalysisResult {
            is_constant: last_analysis.is_constant,
            constant_value: last_analysis.constant_value,
            type_hint: last_analysis.type_hint,
            complexity: max_complexity,
            has_side_effects,
            dependencies,
            optimizations,
        })
    }

    /// Analyze lambda special form
    fn analyze_lambda_form(args: &[Expr]) -> Result<AnalysisResult> {
        if args.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda requires at least 2 arguments (parameters and body)".to_string(),
            ));
        }

        // Lambda creates a procedure
        Ok(AnalysisResult {
            is_constant: true, // Lambda expressions are constants
            constant_value: None, // But we can't represent the procedure as a simple value
            type_hint: TypeHint::Procedure,
            complexity: EvaluationComplexity::Simple,
            has_side_effects: false, // Creating a lambda has no side effects
            dependencies: Vec::new(), // Parameters will be bound in the new scope
            optimizations: Vec::new(),
        })
    }

    /// Analyze define special form
    fn analyze_define_form(
        analyzer: &mut super::analyzer::ExpressionAnalyzer,
        args: &[Expr],
        env: Option<&Environment>,
    ) -> Result<AnalysisResult> {
        if args.len() != 2 {
            return Err(LambdustError::syntax_error(
                "define requires exactly 2 arguments".to_string(),
            ));
        }

        // Analyze the value expression
        let value_analysis = analyzer.analyze(&args[1], env)?;

        Ok(AnalysisResult {
            is_constant: false, // define always has side effects
            constant_value: None,
            type_hint: TypeHint::Unknown, // define typically returns unspecified
            complexity: value_analysis.complexity,
            has_side_effects: true, // define modifies the environment
            dependencies: value_analysis.dependencies,
            optimizations: value_analysis.optimizations,
        })
    }

    /// Check if a special form has side effects
    fn special_form_has_side_effects(name: &str) -> bool {
        matches!(
            name,
            "define"
                | "define-syntax"
                | "set!"
                | "eval"
                | "force" // force can have side effects if the delayed computation does
        )
    }
}