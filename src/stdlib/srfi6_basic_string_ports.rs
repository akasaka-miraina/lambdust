//! SRFI-6: Basic String Ports
//!
//! This library provides the essential string port operations as defined by SRFI-6.
//! These are fundamental I/O abstractions that allow treating strings as input or output ports.
//!
//! ## Specification
//!
//! SRFI-6 defines three essential procedures:
//! - `(open-input-string string)` - Creates an input port that reads from the given string
//! - `(open-output-string)` - Creates an output port that collects written output as a string  
//! - `(get-output-string port)` - Retrieves the string written to an output string port
//!
//! ## R7RS Integration
//!
//! These procedures became part of R7RS-small, making SRFI-6 essential for full R7RS compliance.
//! They enable string-based I/O which is fundamental for testing, serialization, and text processing.
//!
//! ## Usage Examples
//!
//! ```scheme
//! ;; Read from a string
//! (define in-port (open-input-string "hello world"))
//! (read-char in-port)  ; => #\h
//! (read in-port)       ; => hello
//!
//! ;; Write to a string
//! (define out-port (open-output-string))
//! (write "hello" out-port)
//! (write-char #\space out-port)
//! (write "world" out-port)
//! (get-output-string out-port)  ; => "hello world"
//! ```

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{Environment, Generation};
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, Value};
use std::sync::Arc;

/// Install SRFI-6 basic string port procedures into the environment.
///
/// This adds the three core procedures defined by SRFI-6:
/// - `open-input-string` - Create input port from string
/// - `open-output-string` - Create output string port  
/// - `get-output-string` - Extract string from output port
pub fn install_srfi6_procedures(env: &mut Environment) {
    // open-input-string
    env.define(
        "open-input-string".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "open-input-string".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(open_input_string),
            effects: vec![Effect::Pure],
        })),
    );

    // open-output-string
    env.define(
        "open-output-string".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "open-output-string".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(open_output_string),
            effects: vec![Effect::Pure],
        })),
    );

    // get-output-string
    env.define(
        "get-output-string".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "get-output-string".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(get_output_string),
            effects: vec![Effect::Pure],
        })),
    );
}

/// `(open-input-string string)` - Create an input port that reads from the given string.
///
/// Returns a new input port that will supply the characters from `string` in order.
/// The port will return EOF when all characters have been read.
///
/// ## Arguments
/// - `string` - The string to read from
///
/// ## Returns
/// An input port that reads from the string
///
/// ## Examples
/// ```scheme
/// (define port (open-input-string "hello"))
/// (read-char port)  ; => #\h
/// (read port)       ; => ello
/// ```
pub fn open_input_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("open-input-string expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let string =
        crate::stdlib::strings::common::extract_string(&args[0], "open-input-string")?.to_string();
    let port = crate::eval::value::Port::new_string_input(string);
    Ok(Value::Port(Arc::new(port)))
}

/// `(open-output-string)` - Create an output port that collects written data as a string.
///
/// Returns a new output port that will collect all written characters into a string
/// that can be retrieved using `get-output-string`.
///
/// ## Arguments
/// None
///
/// ## Returns
/// An output port that collects written data
///
/// ## Examples
/// ```scheme
/// (define port (open-output-string))
/// (write "hello" port)
/// (write-char #\space port)
/// (write "world" port)
/// (get-output-string port)  ; => "hello world"
/// ```
pub fn open_output_string(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("open-output-string expects 0 arguments, got {}", args.len()),
            None,
        )));
    }

    let port = crate::eval::value::Port::new_string_output();
    Ok(Value::Port(Arc::new(port)))
}

/// `(get-output-string port)` - Retrieve the string written to an output string port.
///
/// Returns the string that has been written to the output string port `port`.
/// The port's output buffer is cleared after this operation.
///
/// ## Arguments  
/// - `port` - An output string port created by `open-output-string`
///
/// ## Returns
/// The string containing all data written to the port
///
/// ## Examples
/// ```scheme
/// (define port (open-output-string))
/// (display "hello world" port)
/// (get-output-string port)  ; => "hello world"
/// (get-output-string port)  ; => "" (buffer was cleared)
/// ```
pub fn get_output_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("get-output-string expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    // Delegate to the existing implementation in io.rs
    crate::stdlib::io::primitive_get_output_string(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::{Environment, Generation};
    use std::collections::HashMap;

    /// Test basic string port operations
    #[test]
    fn test_string_ports_basic() {
        let mut env = Environment::new(None, 0 as Generation);
        install_srfi6_procedures(&mut env);

        // Test input string port
        let input_port = open_input_string(&[Value::string("hello world")]).unwrap();
        assert!(matches!(input_port, Value::Port(_)));

        // Test output string port
        let output_port = open_output_string(&[]).unwrap();
        assert!(matches!(output_port, Value::Port(_)));

        // Test get-output-string with empty port
        let result = get_output_string(&[output_port.clone()]).unwrap();
        assert_eq!(result.as_string().unwrap(), "");
    }

    /// Test error conditions
    #[test]
    fn test_string_ports_errors() {
        // Wrong number of arguments for open-input-string
        assert!(open_input_string(&[]).is_err());
        assert!(open_input_string(&[Value::string("test"), Value::string("extra")]).is_err());

        // Wrong number of arguments for open-output-string
        assert!(open_output_string(&[Value::string("unexpected")]).is_err());

        // Wrong number of arguments for get-output-string
        assert!(get_output_string(&[]).is_err());
        assert!(get_output_string(&[Value::string("test"), Value::string("extra")]).is_err());

        // Wrong argument type for open-input-string
        assert!(open_input_string(&[Value::integer(42)]).is_err());
    }

    /// Test SRFI-6 environment integration
    #[test]
    fn test_srfi6_environment_integration() {
        let mut env = Environment::new(None, 0 as Generation);
        install_srfi6_procedures(&mut env);

        // Verify all procedures are defined
        assert!(env.lookup("open-input-string").is_some());
        assert!(env.lookup("open-output-string").is_some());
        assert!(env.lookup("get-output-string").is_some());

        // Verify they are callable primitives
        let open_input = env.lookup("open-input-string").unwrap();
        assert!(matches!(open_input, Value::Primitive(_)));

        let open_output = env.lookup("open-output-string").unwrap();
        assert!(matches!(open_output, Value::Primitive(_)));

        let get_output = env.lookup("get-output-string").unwrap();
        assert!(matches!(get_output, Value::Primitive(_)));
    }
}
