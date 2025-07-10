//! Debug tracing system for lambdust interpreter
//!
//! This module provides comprehensive tracing of evaluator steps,
//! including class/method/line information, Rust values, and S-expressions.
//! Only active in debug builds.

use crate::ast::Expr;
use crate::value::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Global trace counter for unique step IDs
static TRACE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Global trace log storage (protected by mutex)
static TRACE_LOG: Mutex<Vec<TraceEntry>> = Mutex::new(Vec::new());

/// Single trace entry with complete context information
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Unique step ID
    pub step_id: usize,
    /// Module/class name (e.g., "evaluator::mod", "special_forms")
    pub module: &'static str,
    /// Method/function name
    pub method: &'static str,
    /// Source file line number
    pub line: u32,
    /// Trace level (ENTRY, EXIT, INFO, ERROR)
    pub level: TraceLevel,
    /// Rust value being processed (if any)
    pub rust_value: Option<String>,
    /// S-expression being processed (if any)
    pub s_expr: Option<String>,
    /// Additional context message
    pub message: String,
    /// Continuation type (if applicable)
    pub continuation_type: Option<String>,
    /// Environment depth
    pub env_depth: Option<usize>,
}

/// Trace entry severity levels
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TraceLevel {
    /// Function/method entry
    ENTRY,
    /// Function/method exit with result
    EXIT,
    /// General information
    INFO,
    /// Warning condition
    WARN,
    /// Error condition
    ERROR,
}

impl TraceLevel {
    /// Convert trace level to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            TraceLevel::ENTRY => "ENTRY",
            TraceLevel::EXIT => "EXIT",
            TraceLevel::INFO => "INFO",
            TraceLevel::WARN => "WARN",
            TraceLevel::ERROR => "ERROR",
        }
    }
}

/// Main tracing interface
pub struct DebugTracer;

impl DebugTracer {
    /// Add a trace entry (only in debug builds)
    #[cfg(debug_assertions)]
    pub fn trace(
        module: &'static str,
        method: &'static str,
        line: u32,
        level: TraceLevel,
        message: String,
    ) {
        let step_id = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = TraceEntry {
            step_id,
            module,
            method,
            line,
            level,
            rust_value: None,
            s_expr: None,
            message: message.clone(),
            continuation_type: None,
            env_depth: None,
        };

        if let Ok(mut log) = TRACE_LOG.lock() {
            log.push(entry.clone());

            // Print to stderr for immediate visibility
            eprintln!(
                "[TRACE:{}] {}::{}:{} [{}] {}",
                step_id,
                module,
                method,
                line,
                level.as_str(),
                message
            );
        }
    }

    /// Trace with Rust value
    #[cfg(debug_assertions)]
    pub fn trace_value(
        module: &'static str,
        method: &'static str,
        line: u32,
        level: TraceLevel,
        message: String,
        value: &Value,
    ) {
        let step_id = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = TraceEntry {
            step_id,
            module,
            method,
            line,
            level,
            rust_value: Some(format!("{:?}", value)),
            s_expr: Some(Self::value_to_sexpr(value)),
            message: message.clone(),
            continuation_type: None,
            env_depth: None,
        };

        if let Ok(mut log) = TRACE_LOG.lock() {
            log.push(entry.clone());

            eprintln!(
                "[TRACE:{}] {}::{}:{} [{}] {} | Rust: {:?} | S-expr: {}",
                step_id,
                module,
                method,
                line,
                level.as_str(),
                message,
                value,
                Self::value_to_sexpr(value)
            );
        }
    }

    /// Trace with expression
    #[cfg(debug_assertions)]
    pub fn trace_expr(
        module: &'static str,
        method: &'static str,
        line: u32,
        level: TraceLevel,
        message: String,
        expr: &Expr,
    ) {
        let step_id = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = TraceEntry {
            step_id,
            module,
            method,
            line,
            level,
            rust_value: Some(format!("{:?}", expr)),
            s_expr: Some(Self::expr_to_sexpr(expr)),
            message: message.clone(),
            continuation_type: None,
            env_depth: None,
        };

        if let Ok(mut log) = TRACE_LOG.lock() {
            log.push(entry.clone());

            eprintln!(
                "[TRACE:{}] {}::{}:{} [{}] {} | Expr: {}",
                step_id,
                module,
                method,
                line,
                level.as_str(),
                message,
                Self::expr_to_sexpr(expr)
            );
        }
    }

    /// Trace with continuation information
    #[cfg(debug_assertions)]
    pub fn trace_continuation(
        module: &'static str,
        method: &'static str,
        line: u32,
        level: TraceLevel,
        message: String,
        cont_type: &str,
        env_depth: Option<usize>,
    ) {
        let step_id = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let entry = TraceEntry {
            step_id,
            module,
            method,
            line,
            level,
            rust_value: None,
            s_expr: None,
            message: message.clone(),
            continuation_type: Some(cont_type.to_string()),
            env_depth,
        };

        if let Ok(mut log) = TRACE_LOG.lock() {
            log.push(entry.clone());

            eprintln!(
                "[TRACE:{}] {}::{}:{} [{}] {} | Cont: {} | Depth: {:?}",
                step_id,
                module,
                method,
                line,
                level.as_str(),
                message,
                cont_type,
                env_depth
            );
        }
    }

    /// No-op for release builds
    #[cfg(not(debug_assertions))]
    pub fn trace(
        _module: &'static str,
        _method: &'static str,
        _line: u32,
        _level: TraceLevel,
        _message: String,
    ) {
    }

    #[cfg(not(debug_assertions))]
    pub fn trace_value(
        _module: &'static str,
        _method: &'static str,
        _line: u32,
        _level: TraceLevel,
        _message: String,
        _value: &Value,
    ) {
    }

    #[cfg(not(debug_assertions))]
    pub fn trace_expr(
        _module: &'static str,
        _method: &'static str,
        _line: u32,
        _level: TraceLevel,
        _message: String,
        _expr: &Expr,
    ) {
    }

    #[cfg(not(debug_assertions))]
    pub fn trace_continuation(
        _module: &'static str,
        _method: &'static str,
        _line: u32,
        _level: TraceLevel,
        _message: String,
        _cont_type: &str,
        _env_depth: Option<usize>,
    ) {
    }

    /// Convert Value to readable S-expression
    fn value_to_sexpr(value: &Value) -> String {
        match value {
            Value::Undefined => "#<undefined>".to_string(),
            Value::Boolean(b) => {
                if *b {
                    "#t".to_string()
                } else {
                    "#f".to_string()
                }
            }
            Value::Number(n) => format!("{}", n),
            Value::String(s) => format!("\"{}\"", s),
            Value::Character(c) => format!("#\\{}", c),
            Value::Symbol(s) => s.clone(),
            Value::Nil => "()".to_string(),
            Value::Vector(v) => {
                let items: Vec<String> = v.iter().map(Self::value_to_sexpr).collect();
                format!("#({})", items.join(" "))
            }
            Value::Procedure(_) => "#<procedure>".to_string(),
            Value::Continuation(_) => "#<continuation>".to_string(),
            Value::Values(vals) => {
                let items: Vec<String> = vals.iter().map(Self::value_to_sexpr).collect();
                format!("#<values {}>", items.join(" "))
            }
            _ => format!("#<{}>", std::any::type_name::<Value>()),
        }
    }

    /// Convert Expr to readable S-expression
    fn expr_to_sexpr(expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match lit {
                crate::ast::Literal::Number(n) => format!("{}", n),
                crate::ast::Literal::String(s) => format!("\"{}\"", s),
                crate::ast::Literal::Character(c) => format!("#\\{}", c),
                crate::ast::Literal::Boolean(b) => {
                    if *b {
                        "#t".to_string()
                    } else {
                        "#f".to_string()
                    }
                }
                crate::ast::Literal::Nil => "()".to_string(),
            },
            Expr::Variable(v) => v.clone(),
            Expr::HygienicVariable(symbol) => symbol.unique_name(),
            Expr::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(Self::expr_to_sexpr).collect();
                format!("({})", items.join(" "))
            }
            Expr::Vector(exprs) => {
                let items: Vec<String> = exprs.iter().map(Self::expr_to_sexpr).collect();
                format!("#({})", items.join(" "))
            }
            Expr::Quote(expr) => format!("'{}", Self::expr_to_sexpr(expr)),
            Expr::Quasiquote(expr) => format!("`{}", Self::expr_to_sexpr(expr)),
            Expr::Unquote(expr) => format!(",{}", Self::expr_to_sexpr(expr)),
            Expr::UnquoteSplicing(expr) => format!(",@{}", Self::expr_to_sexpr(expr)),
            Expr::DottedList(exprs, tail) => {
                let items: Vec<String> = exprs.iter().map(Self::expr_to_sexpr).collect();
                format!("({} . {})", items.join(" "), Self::expr_to_sexpr(tail))
            }
        }
    }

    /// Get complete trace log (for analysis)
    #[cfg(debug_assertions)]
    pub fn get_trace_log() -> Vec<TraceEntry> {
        TRACE_LOG.lock().unwrap().clone()
    }

    /// Clear trace log
    #[cfg(debug_assertions)]
    pub fn clear_trace_log() {
        TRACE_LOG.lock().unwrap().clear();
        TRACE_COUNTER.store(0, Ordering::SeqCst);
    }

    /// Print formatted trace log to file
    #[cfg(debug_assertions)]
    pub fn dump_trace_to_file(filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let log = TRACE_LOG.lock().unwrap();
        let mut file = File::create(filename)?;

        writeln!(file, "=== Lambdust Debug Trace Log ===")?;
        writeln!(file, "Total steps: {}", log.len())?;
        writeln!(file, "")?;

        for entry in log.iter() {
            writeln!(
                file,
                "[{}] {}::{}:{} [{}] {}",
                entry.step_id,
                entry.module,
                entry.method,
                entry.line,
                entry.level.as_str(),
                entry.message
            )?;

            if let Some(ref rust_val) = entry.rust_value {
                writeln!(file, "    Rust: {}", rust_val)?;
            }

            if let Some(ref sexpr) = entry.s_expr {
                writeln!(file, "    S-expr: {}", sexpr)?;
            }

            if let Some(ref cont_type) = entry.continuation_type {
                writeln!(file, "    Continuation: {}", cont_type)?;
            }

            if let Some(depth) = entry.env_depth {
                writeln!(file, "    Env depth: {}", depth)?;
            }

            writeln!(file, "")?;
        }

        Ok(())
    }
}

/// Convenience macro for tracing
#[macro_export]
macro_rules! debug_trace {
    ($level:expr, $msg:expr) => {
        #[cfg(debug_assertions)]
        crate::debug::DebugTracer::trace(
            module_path!(),
            function_name!(),
            line!(),
            $level,
            $msg.to_string(),
        );
    };

    ($level:expr, $msg:expr, $value:expr) => {
        #[cfg(debug_assertions)]
        crate::debug::DebugTracer::trace_value(
            module_path!(),
            function_name!(),
            line!(),
            $level,
            $msg.to_string(),
            $value,
        );
    };
}

/// Convenience macro for tracing expressions
#[macro_export]
macro_rules! debug_trace_expr {
    ($level:expr, $msg:expr, $expr:expr) => {
        #[cfg(debug_assertions)]
        crate::debug::DebugTracer::trace_expr(
            module_path!(),
            function_name!(),
            line!(),
            $level,
            $msg.to_string(),
            $expr,
        );
    };
}

/// Convenience macro for tracing continuations
#[macro_export]
macro_rules! debug_trace_continuation {
    ($level:expr, $msg:expr, $cont_type:expr) => {
        #[cfg(debug_assertions)]
        crate::debug::DebugTracer::trace_continuation(
            module_path!(),
            function_name!(),
            line!(),
            $level,
            $msg.to_string(),
            $cont_type,
            None,
        );
    };

    ($level:expr, $msg:expr, $cont_type:expr, $env_depth:expr) => {
        #[cfg(debug_assertions)]
        crate::debug::DebugTracer::trace_continuation(
            module_path!(),
            function_name!(),
            line!(),
            $level,
            $msg.to_string(),
            $cont_type,
            Some($env_depth),
        );
    };
}

/// Function name extraction (requires nightly or external crate, simplified here)
#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3] // Remove "::f"
    }};
}
