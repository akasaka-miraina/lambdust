//! I/O operations for Scheme

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::LambdustError;
use crate::value::Value;
use std::collections::HashMap;

/// Register all I/O functions
pub fn register_io_functions(builtins: &mut HashMap<String, Value>) {
    // Basic I/O functions
    builtins.insert("display".to_string(), io_display());
    builtins.insert("newline".to_string(), io_newline());

    // Additional I/O functions
    builtins.insert("read".to_string(), io_read());
    builtins.insert("write".to_string(), io_write());
    builtins.insert("read-char".to_string(), io_read_char());
    builtins.insert("peek-char".to_string(), io_peek_char());
    builtins.insert("write-char".to_string(), io_write_char());
}

fn io_display() -> Value {
    make_builtin_procedure("display", Some(1), |args| {
        check_arity(args, 1)?;
        match &args[0] {
            Value::String(s) => print!("{}", s),
            other => print!("{}", other),
        }
        std::io::Write::flush(&mut std::io::stdout()).ok();
        Ok(Value::Undefined)
    })
}

fn io_newline() -> Value {
    make_builtin_procedure("newline", Some(0), |args| {
        check_arity(args, 0)?;
        println!();
        Ok(Value::Undefined)
    })
}

fn io_read() -> Value {
    make_builtin_procedure("read", Some(0), |_args| {
        // For now, this is a placeholder implementation
        // A complete implementation would parse Scheme expressions from input
        Err(LambdustError::runtime_error(
            "read: not yet implemented".to_string(),
        ))
    })
}

fn io_write() -> Value {
    make_builtin_procedure("write", Some(1), |args| {
        check_arity(args, 1)?;
        print!("{}", args[0]);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        Ok(Value::Undefined)
    })
}

fn io_read_char() -> Value {
    make_builtin_procedure("read-char", Some(0), |_args| {
        // For now, this is a placeholder implementation
        // A complete implementation would read a character from input
        Err(LambdustError::runtime_error(
            "read-char: not yet implemented".to_string(),
        ))
    })
}

fn io_peek_char() -> Value {
    make_builtin_procedure("peek-char", Some(0), |_args| {
        // For now, this is a placeholder implementation
        // A complete implementation would peek at the next character without consuming it
        Err(LambdustError::runtime_error(
            "peek-char: not yet implemented".to_string(),
        ))
    })
}

fn io_write_char() -> Value {
    make_builtin_procedure("write-char", Some(1), |args| {
        check_arity(args, 1)?;
        
        match &args[0] {
            Value::Character(c) => {
                print!("{}", c);
                std::io::Write::flush(&mut std::io::stdout()).ok();
                Ok(Value::Undefined)
            }
            _ => Err(LambdustError::type_error(format!(
                "write-char: expected character, got {}", args[0]
            ))),
        }
    })
}
