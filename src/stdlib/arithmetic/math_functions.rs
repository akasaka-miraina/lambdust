//! Mathematical and transcendental functions for Lambdust
//!
//! This module implements R7RS-compliant mathematical functions including
//! exponential, logarithmic, and trigonometric operations.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements expt (exponentiation) primitive
pub fn primitive_expt(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "expt expects exactly two arguments",
            None,
        )
        .into());
    }

    match (&args[0], &args[1]) {
        (
            Value::Literal(crate::ast::Literal::ExactInteger(base)),
            Value::Literal(crate::ast::Literal::ExactInteger(exp)),
        ) => {
            if *exp >= 0 {
                Ok(Value::Literal(crate::ast::Literal::ExactInteger(
                    base.pow(*exp as u32),
                )))
            } else {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(
                    (*base as f64).powf(*exp as f64),
                )))
            }
        }
        (
            Value::Literal(crate::ast::Literal::InexactReal(base)),
            Value::Literal(crate::ast::Literal::InexactReal(exp)),
        ) => Ok(Value::Literal(crate::ast::Literal::InexactReal(
            base.powf(*exp),
        ))),
        (
            Value::Literal(crate::ast::Literal::ExactInteger(base)),
            Value::Literal(crate::ast::Literal::InexactReal(exp)),
        ) => Ok(Value::Literal(crate::ast::Literal::InexactReal(
            (*base as f64).powf(*exp),
        ))),
        (
            Value::Literal(crate::ast::Literal::InexactReal(base)),
            Value::Literal(crate::ast::Literal::ExactInteger(exp)),
        ) => Ok(Value::Literal(crate::ast::Literal::InexactReal(
            base.powf(*exp as f64),
        ))),
        _ => Err(crate::diagnostics::Error::type_error(
            "expt requires two numbers",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements sqrt (square root) primitive
pub fn primitive_sqrt(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "sqrt expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            if *x >= 0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(
                    (*x as f64).sqrt(),
                )))
            } else {
                Err(
                    crate::diagnostics::Error::runtime_error("sqrt of negative number", None)
                        .into(),
                )
            }
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            if *x >= 0.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(x.sqrt())))
            } else {
                Err(
                    crate::diagnostics::Error::runtime_error("sqrt of negative number", None)
                        .into(),
                )
            }
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "sqrt requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements exp (exponential) primitive
pub fn primitive_exp(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "exp expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::Literal(
            crate::ast::Literal::InexactReal((*x as f64).exp()),
        )),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x.exp())))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "exp requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements log (logarithm) primitive
pub fn primitive_log(args: &[Value]) -> Result<Value> {
    match args.len() {
        1 => {
            // Natural logarithm
            match &args[0] {
                Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
                    if *x > 0 {
                        Ok(Value::Literal(crate::ast::Literal::InexactReal(
                            (*x as f64).ln(),
                        )))
                    } else {
                        Err(crate::diagnostics::Error::runtime_error(
                            "log of non-positive number",
                            None,
                        )
                        .into())
                    }
                }
                Value::Literal(crate::ast::Literal::InexactReal(x)) => {
                    if *x > 0.0 {
                        Ok(Value::Literal(crate::ast::Literal::InexactReal(x.ln())))
                    } else {
                        Err(crate::diagnostics::Error::runtime_error(
                            "log of non-positive number",
                            None,
                        )
                        .into())
                    }
                }
                _ => Err(crate::diagnostics::Error::type_error(
                    "log requires a number",
                    crate::diagnostics::Span::default(),
                )
                .into()),
            }
        }
        2 => {
            // Logarithm with specified base
            let base = match &args[1] {
                Value::Literal(crate::ast::Literal::ExactInteger(x)) => *x as f64,
                Value::Literal(crate::ast::Literal::InexactReal(x)) => *x,
                _ => {
                    return Err(crate::diagnostics::Error::type_error(
                        "log base must be a number",
                        crate::diagnostics::Span::default(),
                    )
                    .into());
                }
            };

            let value = match &args[0] {
                Value::Literal(crate::ast::Literal::ExactInteger(x)) => *x as f64,
                Value::Literal(crate::ast::Literal::InexactReal(x)) => *x,
                _ => {
                    return Err(crate::diagnostics::Error::type_error(
                        "log argument must be a number",
                        crate::diagnostics::Span::default(),
                    )
                    .into());
                }
            };

            if value > 0.0 && base > 0.0 && base != 1.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(
                    value.log(base),
                )))
            } else {
                Err(
                    crate::diagnostics::Error::runtime_error("Invalid arguments to log", None)
                        .into(),
                )
            }
        }
        _ => Err(crate::diagnostics::Error::runtime_error(
            "log expects one or two arguments",
            None,
        )
        .into()),
    }
}

/// Implements sin (sine) primitive
pub fn primitive_sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "sin expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::Literal(
            crate::ast::Literal::InexactReal((*x as f64).sin()),
        )),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x.sin())))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "sin requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements cos (cosine) primitive
pub fn primitive_cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "cos expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::Literal(
            crate::ast::Literal::InexactReal((*x as f64).cos()),
        )),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x.cos())))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "cos requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements tan (tangent) primitive
pub fn primitive_tan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "tan expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::Literal(
            crate::ast::Literal::InexactReal((*x as f64).tan()),
        )),
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x.tan())))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "tan requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements asin (arc sine) primitive
pub fn primitive_asin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "asin expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            let val = *x as f64;
            if val >= -1.0 && val <= 1.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(val.asin())))
            } else {
                Err(crate::diagnostics::Error::runtime_error("asin domain error", None).into())
            }
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            if *x >= -1.0 && *x <= 1.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(x.asin())))
            } else {
                Err(crate::diagnostics::Error::runtime_error("asin domain error", None).into())
            }
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "asin requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements acos (arc cosine) primitive
pub fn primitive_acos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "acos expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            let val = *x as f64;
            if val >= -1.0 && val <= 1.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(val.acos())))
            } else {
                Err(crate::diagnostics::Error::runtime_error("acos domain error", None).into())
            }
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            if *x >= -1.0 && *x <= 1.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(x.acos())))
            } else {
                Err(crate::diagnostics::Error::runtime_error("acos domain error", None).into())
            }
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "acos requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}

/// Implements atan (arc tangent) primitive
pub fn primitive_atan(args: &[Value]) -> Result<Value> {
    match args.len() {
        1 => match &args[0] {
            Value::Literal(crate::ast::Literal::ExactInteger(x)) => Ok(Value::Literal(
                crate::ast::Literal::InexactReal((*x as f64).atan()),
            )),
            Value::Literal(crate::ast::Literal::InexactReal(x)) => {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(x.atan())))
            }
            _ => Err(crate::diagnostics::Error::type_error(
                "atan requires a number",
                crate::diagnostics::Span::default(),
            )
            .into()),
        },
        2 => {
            // Two-argument atan (atan2)
            let y = match &args[0] {
                Value::Literal(crate::ast::Literal::ExactInteger(x)) => *x as f64,
                Value::Literal(crate::ast::Literal::InexactReal(x)) => *x,
                _ => {
                    return Err(crate::diagnostics::Error::type_error(
                        "atan y-coordinate must be a number",
                        crate::diagnostics::Span::default(),
                    )
                    .into());
                }
            };

            let x = match &args[1] {
                Value::Literal(crate::ast::Literal::ExactInteger(x)) => *x as f64,
                Value::Literal(crate::ast::Literal::InexactReal(x)) => *x,
                _ => {
                    return Err(crate::diagnostics::Error::type_error(
                        "atan x-coordinate must be a number",
                        crate::diagnostics::Span::default(),
                    )
                    .into());
                }
            };

            Ok(Value::Literal(crate::ast::Literal::InexactReal(y.atan2(x))))
        }
        _ => Err(crate::diagnostics::Error::runtime_error(
            "atan expects one or two arguments",
            None,
        )
        .into()),
    }
}

/// Implements square primitive
pub fn primitive_square(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "square expects exactly one argument",
            None,
        )
        .into());
    }

    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x * x)))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x * x)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "square requires a number",
            crate::diagnostics::Span::default(),
        )
        .into()),
    }
}
