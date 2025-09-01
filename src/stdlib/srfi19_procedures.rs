//! SRFI-19 procedures implementation
//!
//! This module provides the core procedures for SRFI-19 time and date handling:
//! - Time and date constructors
//! - Type predicates
//! - Field accessors
//! - Current time functions
//! - Arithmetic operations
//! - Comparison operations
//! - Conversion functions
//! - String formatting and parsing

use crate::eval::Value;
use crate::stdlib::srfi19_time::{Time, Date, TimeType, time_to_date, date_to_time};
use crate::ast::literal::Literal;
use crate::diagnostics::{Result, Error as DiagnosticError, Span};
use crate::utils::{SymbolId, intern_symbol};
use std::rc::Rc;
use num_traits::cast::ToPrimitive;

// ============= CONSTRUCTORS =============

/// Creates a new time object.
/// 
/// ## Syntax
/// `(make-time type nanosecond second) -> time`
///
/// ## Parameters
/// - `type`: Symbol indicating the time type (time-utc, time-tai, etc.)
/// - `nanosecond`: Nanosecond component (0-999999999)
/// - `second`: Second component (integer)
///
/// ## Returns
/// A new time object
///
/// ## Example
/// ```scheme
/// (make-time time-utc 0 1234567890)
/// ```
pub fn make_time(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-time: expected 3 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    // Extract time type
    let time_type = match &args[0] {
        Value::Symbol(symbol) => TimeType::from_symbol(*symbol),
        _ => return Err(Box::new(DiagnosticError::type_error(
            "make-time: first argument must be a time type symbol".to_string(),
            Span::new(0, 0),
        ))),
    };

    // Extract nanosecond
    let nanosecond = match &args[1] {
        Value::Literal(Literal::Number(n)) => n.to_i32().ok_or_else(|| {
            Box::new(DiagnosticError::type_error(
                "make-time: nanosecond must be an integer".to_string(),
                Span::new(0, 0),
            ))
        })?,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "make-time: nanosecond must be an integer".to_string(),
            Span::new(0, 0),
        ))),
    };

    // Extract second
    let second = match &args[2] {
        Value::Literal(Literal::Number(n)) => n.to_i64().ok_or_else(|| {
            Box::new(DiagnosticError::type_error(
                "make-time: second must be an integer".to_string(),
                Span::new(0, 0),
            ))
        })?,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "make-time: second must be an integer".to_string(),
            Span::new(0, 0),
        ))),
    };

    let time = Time::new(time_type, second, nanosecond)?;
    Ok(Value::Time(Rc::new(time)))
}

/// Creates a new date object.
///
/// ## Syntax  
/// `(make-date nanosecond second minute hour day month year zone-offset) -> date`
///
/// ## Parameters
/// - `nanosecond`: Nanosecond component (0-999999999)
/// - `second`: Second component (0-60)
/// - `minute`: Minute component (0-59)
/// - `hour`: Hour component (0-23)
/// - `day`: Day component (1-31)
/// - `month`: Month component (1-12)
/// - `year`: Year component (integer)
/// - `zone-offset`: Time zone offset in seconds from UTC
///
/// ## Returns
/// A new date object
pub fn make_date(args: &[Value]) -> Result<Value> {
    if args.len() != 8 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-date: expected 8 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let mut components = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        let value = match arg {
            Value::Literal(Literal::Number(n)) => n.to_i32().ok_or_else(|| {
                Box::new(DiagnosticError::type_error(
                    format!("make-date: argument {} must be an integer", i),
                    Span::new(0, 0),
                ))
            })?,
            _ => return Err(Box::new(DiagnosticError::type_error(
                format!("make-date: argument {} must be an integer", i),
                Span::new(0, 0),
            ))),
        };
        components.push(value);
    }

    let date = Date::new(
        components[0], // nanosecond
        components[1], // second
        components[2], // minute
        components[3], // hour
        components[4], // day
        components[5], // month
        components[6], // year
        components[7], // zone-offset
    )?;

    Ok(Value::Date(Rc::new(date)))
}

// ============= TYPE PREDICATES =============

/// Checks if a value is a time object.
///
/// ## Syntax
/// `(time? obj) -> boolean`
pub fn time_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time?: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let result = matches!(&args[0], Value::Time(_));
    Ok(Value::Literal(Literal::Boolean(result)))
}

/// Checks if a value is a date object.  
///
/// ## Syntax
/// `(date? obj) -> boolean`
pub fn date_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("date?: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let result = matches!(&args[0], Value::Date(_));
    Ok(Value::Literal(Literal::Boolean(result)))
}

// ============= CURRENT TIME =============

/// Gets the current time of the specified type.
///
/// ## Syntax
/// `(current-time [type]) -> time`
///
/// ## Parameters  
/// - `type`: Optional time type symbol (defaults to time-utc)
pub fn current_time(args: &[Value]) -> Result<Value> {
    let time_type = if args.is_empty() {
        TimeType::Utc
    } else if args.len() == 1 {
        match &args[0] {
            Value::Symbol(symbol) => TimeType::from_symbol(*symbol),
            _ => return Err(Box::new(DiagnosticError::type_error(
                "current-time: argument must be a time type symbol".to_string(),
                Span::new(0, 0),
            ))),
        }
    } else {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("current-time: expected 0 or 1 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    };

    let time = match time_type {
        TimeType::Utc => Time::current_utc(),
        TimeType::Monotonic => Time::current_monotonic(),
        TimeType::Tai => {
            // TAI is UTC + leap seconds (approximately 37 seconds as of 2023)
            let mut utc = Time::current_utc()?;
            utc.time_type = TimeType::Tai;
            utc.second += 37; // Simplified leap second handling
            Ok(utc)
        },
        _ => return Err(Box::new(DiagnosticError::runtime_error(
            format!("current-time: unsupported time type: {:?}", time_type),
            Some(Span::new(0, 0)),
        ))),
    }?;

    Ok(Value::Time(Rc::new(time)))
}

// ============= ACCESSORS =============

/// Gets the type of a time object.
pub fn time_type(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time-type: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    match &args[0] {
        Value::Time(time) => {
            let symbol = time.time_type.to_symbol();
            Ok(Value::Symbol(symbol))
        },
        _ => Err(Box::new(DiagnosticError::type_error(
            "time-type: argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    }
}

/// Gets the second component of a time object.
pub fn time_second(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time-second: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    match &args[0] {
        Value::Time(time) => {
            Ok(Value::Literal(Literal::Number((time.second as f64).into())))
        },
        _ => Err(Box::new(DiagnosticError::type_error(
            "time-second: argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    }
}

/// Gets the nanosecond component of a time object.
pub fn time_nanosecond(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time-nanosecond: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    match &args[0] {
        Value::Time(time) => {
            Ok(Value::Literal(Literal::Number((time.nanosecond as f64).into())))
        },
        _ => Err(Box::new(DiagnosticError::type_error(
            "time-nanosecond: argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    }
}

// Date accessors
macro_rules! date_accessor {
    ($name:ident, $field:ident) => {
        pub fn $name(args: &[Value]) -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("{}: expected 1 argument, got {}", stringify!($name), args.len()),
                    Some(Span::new(0, 0)),
                )));
            }

            match &args[0] {
                Value::Date(date) => {
                    Ok(Value::Literal(Literal::Number((date.$field as f64).into())))
                },
                _ => Err(Box::new(DiagnosticError::type_error(
                    format!("{}: argument must be a date object", stringify!($name)),
                    Span::new(0, 0),
                ))),
            }
        }
    };
}

date_accessor!(date_nanosecond, nanosecond);
date_accessor!(date_second, second);
date_accessor!(date_minute, minute);
date_accessor!(date_hour, hour);
date_accessor!(date_day, day);
date_accessor!(date_month, month);
date_accessor!(date_year, year);
date_accessor!(date_zone_offset, zone_offset);

// ============= CONVERSIONS =============

/// Convert a time object to a date object.
pub fn time_to_date_proc(args: &[Value]) -> Result<Value> {
    if args.len() < 1 || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time->date: expected 1 or 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let time = match &args[0] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "time->date: first argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let zone_offset = if args.len() == 2 {
        match &args[1] {
            Value::Literal(Literal::Number(n)) => n.to_i32().ok_or_else(|| {
                Box::new(DiagnosticError::type_error(
                    "time->date: zone offset must be an integer".to_string(),
                    Span::new(0, 0),
                ))
            })?,
            _ => return Err(Box::new(DiagnosticError::type_error(
                "time->date: zone offset must be an integer".to_string(),
                Span::new(0, 0),
            ))),
        }
    } else {
        0 // UTC default
    };

    let mut date = time_to_date(time)?;
    date.zone_offset = zone_offset;

    Ok(Value::Date(Rc::new(date)))
}

/// Convert a date object to a time object.
pub fn date_to_time_proc(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("date->time: expected 1 argument, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let date = match &args[0] {
        Value::Date(date) => date,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "date->time: argument must be a date object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let time = date_to_time(date)?;
    Ok(Value::Time(Rc::new(time)))
}

// ============= COMPARISONS =============

/// Compare two time objects for equality.
pub fn time_equal(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time=?: expected 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let time1 = match &args[0] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "time=?: arguments must be time objects".to_string(),
            Span::new(0, 0),
        ))),
    };

    let time2 = match &args[1] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "time=?: arguments must be time objects".to_string(),
            Span::new(0, 0),
        ))),
    };

    let result = time1.compare(time2) == std::cmp::Ordering::Equal;
    Ok(Value::Literal(Literal::Boolean(result)))
}

/// Compare two time objects for less-than.
pub fn time_less(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("time<?: expected 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let time1 = match &args[0] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "time<?: arguments must be time objects".to_string(),
            Span::new(0, 0),
        ))),
    };

    let time2 = match &args[1] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "time<?: arguments must be time objects".to_string(),
            Span::new(0, 0),
        ))),
    };

    let result = time1.compare(time2) == std::cmp::Ordering::Less;
    Ok(Value::Literal(Literal::Boolean(result)))
}

/// Add duration to a time.
pub fn add_duration(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("add-duration: expected 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let time1 = match &args[0] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "add-duration: first argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let time2 = match &args[1] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "add-duration: second argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let result = time1.add(time2)?;
    Ok(Value::Time(Rc::new(result)))
}

/// Subtract duration from a time.
pub fn subtract_duration(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("subtract-duration: expected 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let time1 = match &args[0] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "subtract-duration: first argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let time2 = match &args[1] {
        Value::Time(time) => time,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "subtract-duration: second argument must be a time object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let result = time1.subtract(time2)?;
    Ok(Value::Time(Rc::new(result)))
}

// ============= STRING CONVERSION =============

/// Convert a date to a string representation.
pub fn date_to_string(args: &[Value]) -> Result<Value> {
    if args.len() < 1 || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("date->string: expected 1 or 2 arguments, got {}", args.len()),
            Some(Span::new(0, 0)),
        )));
    }

    let date = match &args[0] {
        Value::Date(date) => date,
        _ => return Err(Box::new(DiagnosticError::type_error(
            "date->string: first argument must be a date object".to_string(),
            Span::new(0, 0),
        ))),
    };

    let format = if args.len() == 2 {
        match &args[1] {
            Value::Literal(Literal::String(s)) => s.as_str(),
            _ => return Err(Box::new(DiagnosticError::type_error(
                "date->string: format must be a string".to_string(),
                Span::new(0, 0),
            ))),
        }
    } else {
        "~Y-~m-~d ~H:~M:~S" // Default ISO-like format
    };

    // Simple format implementation (subset of SRFI-19 formatting)
    let result = format
        .replace("~Y", &format!("{:04}", date.year))
        .replace("~m", &format!("{:02}", date.month))
        .replace("~d", &format!("{:02}", date.day))
        .replace("~H", &format!("{:02}", date.hour))
        .replace("~M", &format!("{:02}", date.minute))
        .replace("~S", &format!("{:02}", date.second));

    Ok(Value::Literal(Literal::String(Box::new(result))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_time() {
        let args = vec![
            Value::Symbol(intern_symbol("time-utc".to_string())),
            Value::Literal(Literal::Number(123456789.into())),
            Value::Literal(Literal::Number(1234567890.into())),
        ];

        let result = make_time(&args).unwrap();
        if let Value::Time(time) = result {
            assert_eq!(time.time_type, TimeType::Utc);
            assert_eq!(time.second, 1234567890);
            assert_eq!(time.nanosecond, 123456789);
        } else {
            panic!("Expected time object");
        }
    }

    #[test]
    fn test_time_predicate() {
        let time_val = Value::Time(Rc::new(Time::new(TimeType::Utc, 0, 0).unwrap()));
        let result = time_p(&[time_val]).unwrap();
        assert_eq!(result, Value::Literal(Literal::Boolean(true)));

        let non_time = Value::Literal(Literal::Number(42.into()));
        let result = time_p(&[non_time]).unwrap();
        assert_eq!(result, Value::Literal(Literal::Boolean(false)));
    }

    #[test]
    fn test_current_time() {
        let result = current_time(&[]).unwrap();
        assert!(matches!(result, Value::Time(_)));
    }

    #[test]
    fn test_time_accessors() {
        let time = Time::new(TimeType::Utc, 1234567890, 123456789).unwrap();
        let time_val = Value::Time(Rc::new(time));

        let second_result = time_second(&[time_val.clone()]).unwrap();
        assert_eq!(second_result, Value::Literal(Literal::Number(1234567890.into())));

        let nano_result = time_nanosecond(&[time_val]).unwrap();
        assert_eq!(nano_result, Value::Literal(Literal::Number(123456789.into())));
    }
}