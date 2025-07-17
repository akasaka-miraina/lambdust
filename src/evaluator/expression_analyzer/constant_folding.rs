//! Constant Folding Module
//!
//! このモジュールは式の定数畳み込み最適化を実装します。
//! 算術演算、比較演算、リスト操作などの定数畳み込みを提供します。

use super::core_types::TypeHint;
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::Value;

/// Constant folding operations
#[derive(Debug)]
pub struct ConstantFolder;

impl ConstantFolder {
    /// Attempt to constant fold a function call
    pub fn try_constant_fold(func_name: &str, args: &[Value]) -> Result<Value> {
        match func_name {
            "+" => Self::fold_arithmetic_operation(args, |a, b| a + b),
            "-" => Self::fold_subtraction(args),
            "*" => Self::fold_arithmetic_operation(args, |a, b| a * b),
            "/" => Self::fold_division(args),
            "=" => Self::fold_numeric_comparison(args, |a, b| a == b),
            "<" => Self::fold_numeric_comparison(args, |a, b| a < b),
            ">" => Self::fold_numeric_comparison(args, |a, b| a > b),
            "<=" => Self::fold_numeric_comparison(args, |a, b| a <= b),
            ">=" => Self::fold_numeric_comparison(args, |a, b| a >= b),
            "not" => Self::fold_not(args),
            "car" => Self::fold_car(args),
            "cdr" => Self::fold_cdr(args),
            "length" => Self::fold_length(args),
            _ => Err(LambdustError::runtime_error(format!(
                "Constant folding not implemented for function: {func_name}"
            ))),
        }
    }

    /// Fold arithmetic operations with binary operators
    fn fold_arithmetic_operation<F>(args: &[Value], op: F) -> Result<Value>
    where
        F: Fn(f64, f64) -> f64,
    {
        if args.len() < 2 {
            return Err(LambdustError::syntax_error(
                "Arithmetic operation requires at least 2 arguments".to_string(),
            ));
        }

        let mut result = match &args[0] {
            Value::Number(SchemeNumber::Integer(n)) => *n as f64,
            Value::Number(SchemeNumber::Real(n)) => *n,
            _ => {
                return Err(LambdustError::type_error(
                    "First argument must be a number".to_string(),
                ))
            }
        };

        for arg in &args[1..] {
            let n = match arg {
                Value::Number(SchemeNumber::Integer(n)) => *n as f64,
                Value::Number(SchemeNumber::Real(n)) => *n,
                _ => {
                    return Err(LambdustError::type_error(
                        "All arguments must be numbers".to_string(),
                    ))
                }
            };
            result = op(result, n);
        }

        // Convert back to appropriate number type
        if result.fract() == 0.0 && result.is_finite() {
            Ok(Value::Number(SchemeNumber::Integer(result as i64)))
        } else {
            Ok(Value::Number(SchemeNumber::Real(result)))
        }
    }

    /// Fold subtraction (handles unary negation)
    fn fold_subtraction(args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(LambdustError::syntax_error(
                "Subtraction requires at least 1 argument".to_string(),
            ));
        }

        if args.len() == 1 {
            // Unary negation
            let n = match &args[0] {
                Value::Number(SchemeNumber::Integer(n)) => -(*n as f64),
                Value::Number(SchemeNumber::Real(n)) => -n,
                _ => {
                    return Err(LambdustError::type_error(
                        "Argument must be a number".to_string(),
                    ))
                }
            };
            return Ok(Value::Number(SchemeNumber::Real(n)));
        }

        // Binary/n-ary subtraction
        Self::fold_arithmetic_operation(args, |a, b| a - b)
    }

    /// Fold division (handles division by zero)
    fn fold_division(args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::syntax_error(
                "Division requires at least 2 arguments".to_string(),
            ));
        }

        Self::fold_arithmetic_operation(args, |a, b| {
            if b == 0.0 {
                f64::INFINITY // Let the runtime handle this properly
            } else {
                a / b
            }
        })
    }

    /// Fold numeric comparison operations
    fn fold_numeric_comparison<F>(args: &[Value], op: F) -> Result<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        if args.len() != 2 {
            return Err(LambdustError::syntax_error(
                "Comparison requires exactly 2 arguments".to_string(),
            ));
        }

        let a = match &args[0] {
            Value::Number(SchemeNumber::Integer(n)) => *n as f64,
            Value::Number(SchemeNumber::Real(n)) => *n,
            _ => {
                return Err(LambdustError::type_error(
                    "First argument must be a number".to_string(),
                ))
            }
        };

        let b = match &args[1] {
            Value::Number(SchemeNumber::Integer(n)) => *n as f64,
            Value::Number(SchemeNumber::Real(n)) => *n,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be a number".to_string(),
                ))
            }
        };

        Ok(Value::Boolean(op(a, b)))
    }

    /// Fold not operation
    fn fold_not(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::syntax_error(
                "not requires exactly 1 argument".to_string(),
            ));
        }

        let result = match &args[0] {
            Value::Boolean(false) => true,
            _ => false, // Everything else is truthy in Scheme
        };

        Ok(Value::Boolean(result))
    }

    /// Fold car operation
    fn fold_car(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::syntax_error(
                "car requires exactly 1 argument".to_string(),
            ));
        }

        match &args[0] {
            Value::Pair(pair_data) => {
                let pair_data = pair_data.borrow();
                Ok(pair_data.car.clone())
            }
            _ => Err(LambdustError::type_error(
                "car requires a pair".to_string(),
            )),
        }
    }

    /// Fold cdr operation
    fn fold_cdr(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::syntax_error(
                "cdr requires exactly 1 argument".to_string(),
            ));
        }

        match &args[0] {
            Value::Pair(pair_data) => {
                let pair_data = pair_data.borrow();
                Ok(pair_data.cdr.clone())
            }
            _ => Err(LambdustError::type_error(
                "cdr requires a pair".to_string(),
            )),
        }
    }

    /// Fold length operation
    fn fold_length(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(LambdustError::syntax_error(
                "length requires exactly 1 argument".to_string(),
            ));
        }

        match &args[0] {
            Value::Nil => Ok(Value::Number(SchemeNumber::Integer(0))),
            Value::Vector(vec) => Ok(Value::Number(SchemeNumber::Integer(vec.len() as i64))),
            Value::String(s) => Ok(Value::Number(SchemeNumber::Integer(s.len() as i64))),
            _ => Err(LambdustError::type_error(
                "length requires a list, vector, or string".to_string(),
            )),
        }
    }

    /// Infer return type for function calls
    #[must_use] pub fn infer_function_return_type(func_name: &str, arg_types: &[TypeHint]) -> TypeHint {
        match func_name {
            "+" | "-" | "*" | "/" | "abs" | "floor" | "ceiling" | "sqrt" | "expt" => {
                TypeHint::Number
            }
            "=" | "<" | ">" | "<=" | ">=" | "eq?" | "eqv?" | "equal?" | "not" | "and" | "or"
            | "number?" | "string?" | "symbol?" | "pair?" | "null?" | "boolean?" | "char?"
            | "vector?" | "procedure?" => TypeHint::Boolean,
            "car" | "cdr" => {
                // For car/cdr, we can try to infer from the argument type
                if arg_types.is_empty() {
                    TypeHint::Unknown
                } else {
                    match &arg_types[0] {
                        TypeHint::List => TypeHint::Unknown, // Could be anything
                        _ => TypeHint::Unknown,
                    }
                }
            }
            "cons" => TypeHint::List,
            "list" => TypeHint::List,
            "length" => TypeHint::Number,
            "string-length" => TypeHint::Number,
            "string-ref" => TypeHint::Character,
            "substring" => TypeHint::String,
            "vector-length" => TypeHint::Number,
            "vector-ref" => TypeHint::Unknown,
            _ => TypeHint::Unknown,
        }
    }
}