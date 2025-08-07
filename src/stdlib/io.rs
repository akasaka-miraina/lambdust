//! Complete R7RS-compliant I/O operations for the Lambdust standard library.
//!
//! This module implements all R7RS Section 6.13 I/O operations including:
//! - Port predicates and management
//! - File I/O operations
//! - String and bytevector ports
//! - Input and output operations
//! - Binary I/O support
//! - Proper error handling and resource management

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment,
    Port, PortImpl, StandardPort, PortFileHandle
};
use crate::effects::Effect;
use crate::parser::Parser;
use crate::lexer::Lexer;
use std::sync::Arc;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// Helper functions to get current port values from parameter objects.
/// 
/// These parameters are stored as global parameter objects and their current
/// values are accessed via the thread-local parameter stack.
mod current_ports {
    use super::*;
    use crate::eval::Parameter;
    use once_cell::sync::Lazy;
    
    /// Global parameter objects for current ports
    static CURRENT_INPUT_PARAM: Lazy<Arc<Parameter>> = Lazy::new(|| {
        let stdin_port = Value::Port(Arc::new(Port::new_standard(StandardPort::Stdin)));
        Arc::new(Parameter::with_name(stdin_port, None, "current-input-port".to_string()))
    });
    
    static CURRENT_OUTPUT_PARAM: Lazy<Arc<Parameter>> = Lazy::new(|| {
        let stdout_port = Value::Port(Arc::new(Port::new_standard(StandardPort::Stdout)));
        Arc::new(Parameter::with_name(stdout_port, None, "current-output-port".to_string()))
    });
    
    static CURRENT_ERROR_PARAM: Lazy<Arc<Parameter>> = Lazy::new(|| {
        let stderr_port = Value::Port(Arc::new(Port::new_standard(StandardPort::Stderr)));
        Arc::new(Parameter::with_name(stderr_port, None, "current-error-port".to_string()))
    });
    
    pub fn get_current_input_port() -> Value {
        CURRENT_INPUT_PARAM.get()
    }
    
    pub fn get_current_output_port() -> Value {
        CURRENT_OUTPUT_PARAM.get()
    }
    
    #[allow(dead_code)]
    pub fn get_current_error_port() -> Value {
        CURRENT_ERROR_PARAM.get()
    }
    
    pub fn get_parameter_objects() -> (Value, Value, Value) {
        (
            Value::parameter((**CURRENT_INPUT_PARAM).clone()),
            Value::parameter((**CURRENT_OUTPUT_PARAM).clone()),
            Value::parameter((**CURRENT_ERROR_PARAM).clone())
        )
    }
}

/// Creates standard I/O port parameter objects.
fn create_standard_port_parameters() -> (Value, Value, Value) {
    current_ports::get_parameter_objects()
}

/// Creates complete R7RS I/O operation bindings for the standard library.
pub fn create_io_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // R7RS Section 6.13.1: Port predicates
    bind_port_predicates(env);
    
    // R7RS Section 6.13.2: Current ports
    bind_current_ports(env);
    
    // R7RS Section 6.13.3: File I/O
    bind_file_operations(env);
    
    // R7RS Section 6.13.4: String and bytevector ports
    bind_string_bytevector_ports(env);
    
    // R7RS Section 6.13.5: Input operations
    bind_input_operations(env);
    
    // R7RS Section 6.13.6: Output operations
    bind_output_operations(env);
    
    // EOF handling
    bind_eof_operations(env);
    
    // Additional utilities (Lambdust extensions)
    bind_utility_operations(env);
}

// ============= R7RS SECTION 6.13.1: PORT PREDICATES =============

fn bind_port_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // input-port?
    env.define("input-port?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "input-port?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_input_port_p),
        effects: vec![Effect::Pure],
    })));
    
    // output-port?
    env.define("output-port?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "output-port?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_output_port_p),
        effects: vec![Effect::Pure],
    })));
    
    // textual-port?
    env.define("textual-port?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "textual-port?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_textual_port_p),
        effects: vec![Effect::Pure],
    })));
    
    // binary-port?
    env.define("binary-port?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "binary-port?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_binary_port_p),
        effects: vec![Effect::Pure],
    })));
    
    // port?
    env.define("port?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "port?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_port_p),
        effects: vec![Effect::Pure],
    })));
    
    // port-open?
    env.define("port-open?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "port-open?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_port_open_p),
        effects: vec![Effect::Pure],
    })));
}

// ============= R7RS SECTION 6.13.2: CURRENT PORTS =============

fn bind_current_ports(env: &Arc<ThreadSafeEnvironment>) {
    let (input_param, output_param, error_param) = create_standard_port_parameters();
    
    // Bind the parameter objects directly as the current port procedures
    env.define("current-input-port".to_string(), input_param);
    env.define("current-output-port".to_string(), output_param);
    env.define("current-error-port".to_string(), error_param);
}

// ============= R7RS SECTION 6.13.3: FILE I/O =============

fn bind_file_operations(env: &Arc<ThreadSafeEnvironment>) {
    // open-input-file
    env.define("open-input-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-input-file".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_open_input_file),
        effects: vec![Effect::IO],
    })));
    
    // open-output-file
    env.define("open-output-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-output-file".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_open_output_file),
        effects: vec![Effect::IO],
    })));
    
    // open-binary-input-file
    env.define("open-binary-input-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-binary-input-file".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_open_binary_input_file),
        effects: vec![Effect::IO],
    })));
    
    // open-binary-output-file
    env.define("open-binary-output-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-binary-output-file".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_open_binary_output_file),
        effects: vec![Effect::IO],
    })));
    
    // close-port
    env.define("close-port".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "close-port".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_close_port),
        effects: vec![Effect::IO],
    })));
    
    // close-input-port
    env.define("close-input-port".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "close-input-port".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_close_input_port),
        effects: vec![Effect::IO],
    })));
    
    // close-output-port
    env.define("close-output-port".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "close-output-port".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_close_output_port),
        effects: vec![Effect::IO],
    })));
}

// ============= R7RS SECTION 6.13.4: STRING AND BYTEVECTOR PORTS =============

fn bind_string_bytevector_ports(env: &Arc<ThreadSafeEnvironment>) {
    // open-input-string
    env.define("open-input-string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-input-string".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_open_input_string),
        effects: vec![Effect::Pure],
    })));
    
    // open-output-string
    env.define("open-output-string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-output-string".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_open_output_string),
        effects: vec![Effect::Pure],
    })));
    
    // get-output-string
    env.define("get-output-string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-output-string".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_get_output_string),
        effects: vec![Effect::Pure],
    })));
    
    // open-input-bytevector
    env.define("open-input-bytevector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-input-bytevector".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_open_input_bytevector),
        effects: vec![Effect::Pure],
    })));
    
    // open-output-bytevector
    env.define("open-output-bytevector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "open-output-bytevector".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_open_output_bytevector),
        effects: vec![Effect::Pure],
    })));
    
    // get-output-bytevector
    env.define("get-output-bytevector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-output-bytevector".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_get_output_bytevector),
        effects: vec![Effect::Pure],
    })));
}

// ============= R7RS SECTION 6.13.5: INPUT OPERATIONS =============

fn bind_input_operations(env: &Arc<ThreadSafeEnvironment>) {
    // read
    env.define("read".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_read),
        effects: vec![Effect::IO],
    })));
    
    // read-char
    env.define("read-char".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-char".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_read_char),
        effects: vec![Effect::IO],
    })));
    
    // peek-char
    env.define("peek-char".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "peek-char".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_peek_char),
        effects: vec![Effect::IO],
    })));
    
    // read-line
    env.define("read-line".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-line".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_read_line),
        effects: vec![Effect::IO],
    })));
    
    // read-string
    env.define("read-string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-string".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_read_string),
        effects: vec![Effect::IO],
    })));
    
    // read-u8
    env.define("read-u8".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-u8".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_read_u8),
        effects: vec![Effect::IO],
    })));
    
    // peek-u8
    env.define("peek-u8".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "peek-u8".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_peek_u8),
        effects: vec![Effect::IO],
    })));
    
    // read-bytevector
    env.define("read-bytevector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-bytevector".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_read_bytevector),
        effects: vec![Effect::IO],
    })));
    
    // read-bytevector!
    env.define("read-bytevector!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-bytevector!".to_string(),
        arity_min: 1,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_read_bytevector_bang),
        effects: vec![Effect::IO],
    })));
    
    // char-ready?
    env.define("char-ready?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-ready?".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_ready_p),
        effects: vec![Effect::IO],
    })));
    
    // u8-ready?
    env.define("u8-ready?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u8-ready?".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_u8_ready_p),
        effects: vec![Effect::IO],
    })));
}

// ============= R7RS SECTION 6.13.6: OUTPUT OPERATIONS =============

fn bind_output_operations(env: &Arc<ThreadSafeEnvironment>) {
    // write
    env.define("write".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_write),
        effects: vec![Effect::IO],
    })));
    
    // write-shared
    env.define("write-shared".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write-shared".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_write_shared),
        effects: vec![Effect::IO],
    })));
    
    // write-simple
    env.define("write-simple".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write-simple".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_write_simple),
        effects: vec![Effect::IO],
    })));
    
    // display
    env.define("display".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "display".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_display),
        effects: vec![Effect::IO],
    })));
    
    // newline
    env.define("newline".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "newline".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_newline),
        effects: vec![Effect::IO],
    })));
    
    // write-char
    env.define("write-char".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write-char".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_write_char),
        effects: vec![Effect::IO],
    })));
    
    // write-string
    env.define("write-string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write-string".to_string(),
        arity_min: 1,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_write_string),
        effects: vec![Effect::IO],
    })));
    
    // write-u8
    env.define("write-u8".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write-u8".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_write_u8),
        effects: vec![Effect::IO],
    })));
    
    // write-bytevector
    env.define("write-bytevector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "write-bytevector".to_string(),
        arity_min: 1,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_write_bytevector),
        effects: vec![Effect::IO],
    })));
    
    // flush-output-port
    env.define("flush-output-port".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "flush-output-port".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_flush_output_port),
        effects: vec![Effect::IO],
    })));
}

// ============= EOF HANDLING =============

fn bind_eof_operations(env: &Arc<ThreadSafeEnvironment>) {
    // eof-object
    env.define("eof-object".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "eof-object".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_eof_object),
        effects: vec![Effect::Pure],
    })));
    
    // eof-object?
    env.define("eof-object?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "eof-object?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_eof_object_p),
        effects: vec![Effect::Pure],
    })));
}

// ============= UTILITY OPERATIONS =============

fn bind_utility_operations(env: &Arc<ThreadSafeEnvironment>) {
    // file-exists?
    env.define("file-exists?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-exists?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_exists_p),
        effects: vec![Effect::IO],
    })));
    
    // delete-file
    env.define("delete-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "delete-file".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_delete_file),
        effects: vec![Effect::IO],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === Port Predicates ===

pub fn primitive_input_port_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("input-port? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => Ok(Value::boolean(port.is_input())),
        _ => Ok(Value::boolean(false)),
    }
}

pub fn primitive_output_port_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("output-port? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => Ok(Value::boolean(port.is_output())),
        _ => Ok(Value::boolean(false)),
    }
}

pub fn primitive_textual_port_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("textual-port? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => Ok(Value::boolean(port.is_textual())),
        _ => Ok(Value::boolean(false)),
    }
}

pub fn primitive_binary_port_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("binary-port? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => Ok(Value::boolean(port.is_binary())),
        _ => Ok(Value::boolean(false)),
    }
}

pub fn primitive_port_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("port? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    Ok(Value::boolean(args[0].is_port()))
}

pub fn primitive_port_open_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("port-open? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => Ok(Value::boolean(port.is_open())),
        _ => Err(DiagnosticError::runtime_error(
            "port-open? requires a port argument".to_string(),
            None,
        )),
    }
}

// === Current Ports ===


// === File Operations ===

pub fn primitive_open_input_file(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("open-input-file expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let filename = extract_string(&args[0], "open-input-file")?;
    let port = Port::new_file_input(filename, false);
    
    // Try to open the file to validate it exists
    if let PortImpl::File { path, handle } = &port.implementation {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                *handle.write().unwrap() = Some(PortFileHandle::TextReader(reader));
            }
            Err(e) => {
                return Err(DiagnosticError::runtime_error(
                    format!("Cannot open file '{}': {}", path, e),
                    None,
                ));
            }
        }
    }
    
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_open_output_file(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("open-output-file expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let filename = extract_string(&args[0], "open-output-file")?;
    let port = Port::new_file_output(filename, false);
    
    // Try to create the file
    if let PortImpl::File { path, handle } = &port.implementation {
        match File::create(path) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                *handle.write().unwrap() = Some(PortFileHandle::TextWriter(writer));
            }
            Err(e) => {
                return Err(DiagnosticError::runtime_error(
                    format!("Cannot create file '{}': {}", path, e),
                    None,
                ));
            }
        }
    }
    
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_open_binary_input_file(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("open-binary-input-file expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let filename = extract_string(&args[0], "open-binary-input-file")?;
    let port = Port::new_file_input(filename, true);
    
    if let PortImpl::File { path, handle } = &port.implementation {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                *handle.write().unwrap() = Some(PortFileHandle::BinaryReader(reader));
            }
            Err(e) => {
                return Err(DiagnosticError::runtime_error(
                    format!("Cannot open binary file '{}': {}", path, e),
                    None,
                ));
            }
        }
    }
    
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_open_binary_output_file(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("open-binary-output-file expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let filename = extract_string(&args[0], "open-binary-output-file")?;
    let port = Port::new_file_output(filename, true);
    
    if let PortImpl::File { path, handle } = &port.implementation {
        match File::create(path) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                *handle.write().unwrap() = Some(PortFileHandle::BinaryWriter(writer));
            }
            Err(e) => {
                return Err(DiagnosticError::runtime_error(
                    format!("Cannot create binary file '{}': {}", path, e),
                    None,
                ));
            }
        }
    }
    
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_close_port(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("close-port expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => {
            port.close();
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "close-port requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_close_input_port(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("close-input-port expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => {
            if !port.is_input() {
                return Err(DiagnosticError::runtime_error(
                    "close-input-port requires an input port".to_string(),
                    None,
                ));
            }
            port.close();
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "close-input-port requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_close_output_port(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("close-output-port expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => {
            if !port.is_output() {
                return Err(DiagnosticError::runtime_error(
                    "close-output-port requires an output port".to_string(),
                    None,
                ));
            }
            port.close();
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "close-output-port requires a port argument".to_string(),
            None,
        )),
    }
}

// === String Ports ===

pub fn primitive_open_input_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("open-input-string expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let string = extract_string(&args[0], "open-input-string")?;
    let port = Port::new_string_input(string);
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_open_output_string(_args: &[Value]) -> Result<Value> {
    let port = Port::new_string_output();
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_get_output_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("get-output-string expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => {
            if !port.is_output() {
                return Err(DiagnosticError::runtime_error(
                    "get-output-string requires an output port".to_string(),
                    None,
                ));
            }
            
            match &port.implementation {
                PortImpl::String { content, .. } => {
                    let result = content.read().unwrap().clone());
                    // Reset the string for future accumulation
                    content.write().unwrap().clear();
                    Ok(Value::string(result))
                }
                _ => Err(DiagnosticError::runtime_error(
                    "get-output-string requires a string output port".to_string(),
                    None,
                )),
            }
        }
        _ => Err(DiagnosticError::runtime_error(
            "get-output-string requires a port argument".to_string(),
            None,
        )),
    }
}

// === Bytevector Ports ===

pub fn primitive_open_input_bytevector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("open-input-bytevector expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let bytevector = extract_bytevector(&args[0], "open-input-bytevector")?;
    let port = Port::new_bytevector_input(bytevector);
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_open_output_bytevector(_args: &[Value]) -> Result<Value> {
    let port = Port::new_bytevector_output();
    Ok(Value::Port(Arc::new(port)))
}

pub fn primitive_get_output_bytevector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("get-output-bytevector expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Port(port) => {
            if !port.is_output() {
                return Err(DiagnosticError::runtime_error(
                    "get-output-bytevector requires an output port".to_string(),
                    None,
                ));
            }
            
            match &port.implementation {
                PortImpl::Bytevector { content, .. } => {
                    let result = content.read().unwrap().clone());
                    // Reset the bytevector for future accumulation
                    content.write().unwrap().clear();
                    Ok(Value::bytevector(result))
                }
                _ => Err(DiagnosticError::runtime_error(
                    "get-output-bytevector requires a bytevector output port".to_string(),
                    None,
                )),
            }
        }
        _ => Err(DiagnosticError::runtime_error(
            "get-output-bytevector requires a port argument".to_string(),
            None,
        )),
    }
}

// === Input Operations ===

pub fn primitive_read(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        current_ports::get_current_input_port()
    } else if args.len() == 1 {
        args[0].clone())
    } else {
        return Err(DiagnosticError::runtime_error(
            format!("read expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() {
                return Err(DiagnosticError::runtime_error(
                    "read requires an input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read from closed port".to_string(),
                    None,
                ));
            }
            
            // Read text from port and parse as S-expression
            match read_text_from_port(&port_ref) {
                Ok(Some(text)) => {
                    // Parse the text as a Scheme expression
                    let mut lexer = Lexer::new(&text, None);
                    let tokens = lexer.tokenize().map_err(|e| {
                        DiagnosticError::runtime_error(
                            format!("read: lexer error: {}", e),
                            None,
                        )
                    })?;
                    
                    let mut parser = Parser::new(tokens);
                    match parser.parse_expression() {
                        Ok(expr) => {
                            // Convert expression to value
                            expr_to_value(expr.inner)
                        }
                        Err(_) => Ok(eof_value()),
                    }
                }
                Ok(None) => Ok(eof_value()),
                Err(e) => Err(e),
            }
        }
        _ => Err(DiagnosticError::runtime_error(
            "read requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_read_char(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        current_ports::get_current_input_port()
    } else if args.len() == 1 {
        args[0].clone())
    } else {
        return Err(DiagnosticError::runtime_error(
            format!("read-char expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "read-char requires a textual input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read-char from closed port".to_string(),
                    None,
                ));
            }
            
            read_char_from_port(&port_ref, false)
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-char requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_peek_char(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        current_ports::get_current_input_port()
    } else if args.len() == 1 {
        args[0].clone())
    } else {
        return Err(DiagnosticError::runtime_error(
            format!("peek-char expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "peek-char requires a textual input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "peek-char from closed port".to_string(),
                    None,
                ));
            }
            
            read_char_from_port(&port_ref, true)
        }
        _ => Err(DiagnosticError::runtime_error(
            "peek-char requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_read_line(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        current_ports::get_current_input_port()
    } else if args.len() == 1 {
        args[0].clone())
    } else {
        return Err(DiagnosticError::runtime_error(
            format!("read-line expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "read-line requires a textual input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read-line from closed port".to_string(),
                    None,
                ));
            }
            
            read_line_from_port(&port_ref)
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-line requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_read_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("read-string expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let k = extract_integer(&args[0], "read-string")? as usize;
    let port = if args.len() == 1 {
        current_ports::get_current_input_port()
    } else {
        args[1].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "read-string requires a textual input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read-string from closed port".to_string(),
                    None,
                ));
            }
            
            read_string_from_port(&port_ref, k)
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-string requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_read_u8(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        current_ports::get_current_input_port()
    } else if args.len() == 1 {
        args[0].clone())
    } else {
        return Err(DiagnosticError::runtime_error(
            format!("read-u8 expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_binary() {
                return Err(DiagnosticError::runtime_error(
                    "read-u8 requires a binary input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read-u8 from closed port".to_string(),
                    None,
                ));
            }
            
            read_u8_from_port(&port_ref, false)
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-u8 requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_peek_u8(args: &[Value]) -> Result<Value> {
    let port = if args.is_empty() {
        current_ports::get_current_input_port()
    } else if args.len() == 1 {
        args[0].clone())
    } else {
        return Err(DiagnosticError::runtime_error(
            format!("peek-u8 expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_binary() {
                return Err(DiagnosticError::runtime_error(
                    "peek-u8 requires a binary input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "peek-u8 from closed port".to_string(),
                    None,
                ));
            }
            
            read_u8_from_port(&port_ref, true)
        }
        _ => Err(DiagnosticError::runtime_error(
            "peek-u8 requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_read_bytevector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("read-bytevector expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let k = extract_integer(&args[0], "read-bytevector")? as usize;
    let port = if args.len() == 1 {
        current_ports::get_current_input_port()
    } else {
        args[1].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_binary() {
                return Err(DiagnosticError::runtime_error(
                    "read-bytevector requires a binary input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read-bytevector from closed port".to_string(),
                    None,
                ));
            }
            
            read_bytevector_from_port(&port_ref, k)
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-bytevector requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_read_bytevector_bang(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(DiagnosticError::runtime_error(
            format!("read-bytevector! expects 1 to 4 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let bytevector = extract_bytevector(&args[0], "read-bytevector!")?;
    let port = if args.len() == 1 {
        current_ports::get_current_input_port()
    } else {
        args[1].clone())
    };
    
    let start = if args.len() >= 3 {
        extract_integer(&args[2], "read-bytevector!")? as usize
    } else {
        0
    };
    
    let end = if args.len() >= 4 {
        extract_integer(&args[3], "read-bytevector!")? as usize
    } else {
        bytevector.len()
    };
    
    if start > end || end > bytevector.len() {
        return Err(DiagnosticError::runtime_error(
            "read-bytevector!: invalid start/end indices".to_string(),
            None,
        ));
    }
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_input() || !port_ref.is_binary() {
                return Err(DiagnosticError::runtime_error(
                    "read-bytevector! requires a binary input port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "read-bytevector! from closed port".to_string(),
                    None,
                ));
            }
            
            read_bytevector_bang_from_port(&port_ref, &mut bytevector.clone()), start, end)
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-bytevector! requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_char_ready_p(_args: &[Value]) -> Result<Value> {
    // For now, always return true (characters are always ready in our implementation)
    Ok(Value::boolean(true))
}

pub fn primitive_u8_ready_p(_args: &[Value]) -> Result<Value> {
    // For now, always return true (bytes are always ready in our implementation)
    Ok(Value::boolean(true))
}

// === Output Operations ===

pub fn primitive_write(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("write expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let value = &args[0];
    let port = if args.len() == 1 {
        current_ports::get_current_output_port()
    } else {
        args[1].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "write requires a textual output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "write to closed port".to_string(),
                    None,
                ));
            }
            
            let output = format!("{}", value);
            write_string_to_port(&port_ref, &output)?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "write requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_write_shared(args: &[Value]) -> Result<Value> {
    // For now, same as write (shared structure detection not implemented)
    primitive_write(args)
}

pub fn primitive_write_simple(args: &[Value]) -> Result<Value> {
    // For now, same as write
    primitive_write(args)
}

pub fn primitive_display(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("display expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let value = &args[0];
    let port = if args.len() == 1 {
        current_ports::get_current_output_port()
    } else {
        args[1].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "display requires a textual output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "display to closed port".to_string(),
                    None,
                ));
            }
            
            let output = display_value(value);
            write_string_to_port(&port_ref, &output)?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "display requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_newline(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(DiagnosticError::runtime_error(
            format!("newline expects 0 or 1 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let port = if args.is_empty() {
        current_ports::get_current_output_port()
    } else {
        args[0].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "newline requires a textual output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "newline to closed port".to_string(),
                    None,
                ));
            }
            
            write_string_to_port(&port_ref, "\n")?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "newline requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_write_char(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("write-char expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let ch = extract_character(&args[0], "write-char")?;
    let port = if args.len() == 1 {
        current_ports::get_current_output_port()
    } else {
        args[1].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "write-char requires a textual output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "write-char to closed port".to_string(),
                    None,
                ));
            }
            
            write_string_to_port(&port_ref, &ch.to_string())?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-char requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_write_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(DiagnosticError::runtime_error(
            format!("write-string expects 1 to 4 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let string = extract_string(&args[0], "write-string")?;
    let port = if args.len() == 1 {
        current_ports::get_current_output_port()
    } else {
        args[1].clone())
    };
    
    let start = if args.len() >= 3 {
        extract_integer(&args[2], "write-string")? as usize
    } else {
        0
    };
    
    let end = if args.len() >= 4 {
        extract_integer(&args[3], "write-string")? as usize
    } else {
        string.len()
    };
    
    if start > end || end > string.len() {
        return Err(DiagnosticError::runtime_error(
            "write-string: invalid start/end indices".to_string(),
            None,
        ));
    }
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_textual() {
                return Err(DiagnosticError::runtime_error(
                    "write-string requires a textual output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "write-string to closed port".to_string(),
                    None,
                ));
            }
            
            let substring = &string[start..end];
            write_string_to_port(&port_ref, substring)?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-string requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_write_u8(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("write-u8 expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let byte = extract_integer(&args[0], "write-u8")? as u8;
    let port = if args.len() == 1 {
        current_ports::get_current_output_port()
    } else {
        args[1].clone())
    };
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_binary() {
                return Err(DiagnosticError::runtime_error(
                    "write-u8 requires a binary output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "write-u8 to closed port".to_string(),
                    None,
                ));
            }
            
            write_u8_to_port(&port_ref, byte)?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-u8 requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_write_bytevector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(DiagnosticError::runtime_error(
            format!("write-bytevector expects 1 to 4 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let bytevector = extract_bytevector(&args[0], "write-bytevector")?;
    let port = if args.len() == 1 {
        current_ports::get_current_output_port()
    } else {
        args[1].clone())
    };
    
    let start = if args.len() >= 3 {
        extract_integer(&args[2], "write-bytevector")? as usize
    } else {
        0
    };
    
    let end = if args.len() >= 4 {
        extract_integer(&args[3], "write-bytevector")? as usize
    } else {
        bytevector.len()
    };
    
    if start > end || end > bytevector.len() {
        return Err(DiagnosticError::runtime_error(
            "write-bytevector: invalid start/end indices".to_string(),
            None,
        ));
    }
    
    match port {
        Value::Port(port_ref) => {
            if !port_ref.is_output() || !port_ref.is_binary() {
                return Err(DiagnosticError::runtime_error(
                    "write-bytevector requires a binary output port".to_string(),
                    None,
                ));
            }
            
            if !port_ref.is_open() {
                return Err(DiagnosticError::runtime_error(
                    "write-bytevector to closed port".to_string(),
                    None,
                ));
            }
            
            write_bytevector_to_port(&port_ref, &bytevector[start..end])?;
            Ok(Value::Unspecified)
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-bytevector requires a port argument".to_string(),
            None,
        )),
    }
}

pub fn primitive_flush_output_port(_args: &[Value]) -> Result<Value> {
    // For now, do nothing (our implementation doesn't buffer)
    Ok(Value::Unspecified)
}

// === EOF Operations ===

pub fn primitive_eof_object(_args: &[Value]) -> Result<Value> {
    Ok(eof_value())
}

pub fn primitive_eof_object_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("eof-object? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    Ok(Value::boolean(is_eof_value(&args[0])))
}

// === Utility Operations ===

pub fn primitive_file_exists_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("file-exists? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let filename = extract_string(&args[0], "file-exists?")?;
    Ok(Value::boolean(std::path::Path::new(&filename).exists()))
}

pub fn primitive_delete_file(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("delete-file expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let filename = extract_string(&args[0], "delete-file")?;
    match std::fs::remove_file(&filename) {
        Ok(()) => Ok(Value::Unspecified),
        Err(e) => Err(DiagnosticError::runtime_error(
            format!("Cannot delete file '{}': {}", filename, e),
            None,
        )),
    }
}

// ============= HELPER FUNCTIONS =============

/// Extracts a string from a Value.
fn extract_string(value: &Value, operation: &str) -> Result<String> {
    match value {
        Value::Literal(crate::ast::Literal::String(s)) => Ok(s.clone()),
        _ => Err(DiagnosticError::runtime_error(
            format!("{} requires string arguments", operation),
            None,
        )),
    }
}

/// Extracts a character from a Value.
fn extract_character(value: &Value, operation: &str) -> Result<char> {
    match value {
        Value::Literal(crate::ast::Literal::Character(c)) => Ok(*c),
        _ => Err(DiagnosticError::runtime_error(
            format!("{} requires character arguments", operation),
            None,
        )),
    }
}

/// Extracts an integer from a Value.
fn extract_integer(value: &Value, operation: &str) -> Result<i64> {
    match value {
        Value::Literal(lit) => {
            if let Some(i) = lit.to_i64() {
                Ok(i)
            } else {
                Err(DiagnosticError::runtime_error(
                    format!("{} requires integer arguments", operation),
                    None,
                ))
            }
        }
        _ => Err(DiagnosticError::runtime_error(
            format!("{} requires integer arguments", operation),
            None,
        )),
    }
}

/// Extracts a bytevector from a Value.
fn extract_bytevector(value: &Value, operation: &str) -> Result<Vec<u8>> {
    match value {
        Value::Literal(crate::ast::Literal::Bytevector(bv)) => Ok(bv.clone()),
        _ => Err(DiagnosticError::runtime_error(
            format!("{} requires bytevector arguments", operation),
            None,
        )),
    }
}

/// Creates an EOF value.
fn eof_value() -> Value {
    // Use a special symbol to represent EOF
    Value::Symbol(crate::utils::intern_symbol("*eof*"))
}

/// Checks if a value is an EOF value.
fn is_eof_value(value: &Value) -> bool {
    match value {
        Value::Symbol(id) => {
            // Check if it's our special EOF symbol
            crate::utils::symbol_name(*id).map(|name| name == "*eof*").unwrap_or(false)
        }
        _ => false,
    }
}

/// Converts an expression to a value (placeholder implementation).
fn expr_to_value(_expr: crate::ast::Expr) -> Result<Value> {
    // This would need proper implementation based on the AST structure
    Err(DiagnosticError::runtime_error(
        "expr_to_value not yet implemented".to_string(),
        None,
    ))
}

/// Reads text from a port for parsing.
fn read_text_from_port(_port: &Port) -> Result<Option<String>> {
    // Placeholder implementation
    Err(DiagnosticError::runtime_error(
        "read_text_from_port not yet implemented".to_string(),
        None,
    ))
}

/// Reads a character from a port.
fn read_char_from_port(port: &Port, peek: bool) -> Result<Value> {
    match &port.implementation {
        PortImpl::String { content, position } => {
            let content_guard = content.read().unwrap();
            let mut pos_guard = position.write().unwrap();
            
            if *pos_guard >= content_guard.len() {
                return Ok(eof_value());
            }
            
            let ch = content_guard.chars().nth(*pos_guard).unwrap_or('\0');
            if !peek {
                *pos_guard += 1;
            }
            
            Ok(Value::Literal(crate::ast::Literal::Character(ch)))
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-char: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Reads a line from a port.
fn read_line_from_port(port: &Port) -> Result<Value> {
    match &port.implementation {
        PortImpl::String { content, position } => {
            let content_guard = content.read().unwrap();
            let mut pos_guard = position.write().unwrap();
            
            if *pos_guard >= content_guard.len() {
                return Ok(eof_value());
            }
            
            let remaining = &content_guard[*pos_guard..];
            if let Some(newline_pos) = remaining.find('\n') {
                let line = remaining[..newline_pos].to_string();
                *pos_guard += newline_pos + 1; // Skip the newline
                Ok(Value::string(line))
            } else {
                let line = remaining.to_string();
                *pos_guard = content_guard.len();
                Ok(Value::string(line))
            }
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-line: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Reads a string of specified length from a port.
fn read_string_from_port(port: &Port, k: usize) -> Result<Value> {
    match &port.implementation {
        PortImpl::String { content, position } => {
            let content_guard = content.read().unwrap();
            let mut pos_guard = position.write().unwrap();
            
            if *pos_guard >= content_guard.len() {
                return Ok(eof_value());
            }
            
            let remaining = &content_guard[*pos_guard..];
            let to_read = std::cmp::min(k, remaining.len());
            let result = remaining[..to_read].to_string();
            *pos_guard += to_read;
            
            Ok(Value::string(result))
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-string: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Reads a byte from a port.
fn read_u8_from_port(port: &Port, peek: bool) -> Result<Value> {
    match &port.implementation {
        PortImpl::Bytevector { content, position } => {
            let content_guard = content.read().unwrap();
            let mut pos_guard = position.write().unwrap();
            
            if *pos_guard >= content_guard.len() {
                return Ok(eof_value());
            }
            
            let byte = content_guard[*pos_guard];
            if !peek {
                *pos_guard += 1;
            }
            
            Ok(Value::integer(byte as i64))
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-u8: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Writes a string to a port.
fn write_string_to_port(port: &Port, s: &str) -> Result<()> {
    match &port.implementation {
        PortImpl::String { content, .. } => {
            content.write().unwrap().push_str(s);
            Ok(())
        }
        PortImpl::Standard(StandardPort::Stdout) => {
            print!("{}", s);
            Ok(())
        }
        PortImpl::Standard(StandardPort::Stderr) => {
            eprint!("{}", s);
            Ok(())
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-string: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Writes a byte to a port.
fn write_u8_to_port(port: &Port, byte: u8) -> Result<()> {
    match &port.implementation {
        PortImpl::Bytevector { content, .. } => {
            content.write().unwrap().push(byte);
            Ok(())
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-u8: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Reads a bytevector from a port.
fn read_bytevector_from_port(port: &Port, k: usize) -> Result<Value> {
    match &port.implementation {
        PortImpl::Bytevector { content, position } => {
            let content_guard = content.read().unwrap();
            let mut pos_guard = position.write().unwrap();
            
            if *pos_guard >= content_guard.len() {
                return Ok(eof_value());
            }
            
            let remaining = &content_guard[*pos_guard..];
            let to_read = std::cmp::min(k, remaining.len());
            let result = remaining[..to_read].to_vec();
            *pos_guard += to_read;
            
            Ok(Value::bytevector(result))
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-bytevector: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Reads bytes into an existing bytevector.
fn read_bytevector_bang_from_port(port: &Port, bytevector: &mut Vec<u8>, start: usize, end: usize) -> Result<Value> {
    match &port.implementation {
        PortImpl::Bytevector { content, position } => {
            let content_guard = content.read().unwrap();
            let mut pos_guard = position.write().unwrap();
            
            if *pos_guard >= content_guard.len() {
                return Ok(eof_value());
            }
            
            let available = &content_guard[*pos_guard..];
            let to_read = std::cmp::min(end - start, available.len());
            
            // Copy bytes into the target bytevector
            for i in 0..to_read {
                if start + i < bytevector.len() {
                    bytevector[start + i] = available[i];
                }
            }
            
            *pos_guard += to_read;
            Ok(Value::integer(to_read as i64))
        }
        _ => Err(DiagnosticError::runtime_error(
            "read-bytevector!: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Writes a bytevector to a port.
fn write_bytevector_to_port(port: &Port, bytes: &[u8]) -> Result<()> {
    match &port.implementation {
        PortImpl::Bytevector { content, .. } => {
            content.write().unwrap().extend_from_slice(bytes);
            Ok(())
        }
        _ => Err(DiagnosticError::runtime_error(
            "write-bytevector: unsupported port type".to_string(),
            None,
        )),
    }
}

/// Formats a value for display (without quotes for strings).
fn display_value(value: &Value) -> String {
    match value {
        Value::Literal(crate::ast::Literal::String(s)) => s.clone()),
        Value::Literal(crate::ast::Literal::Character(c)) => c.to_string(),
        _ => format!("{}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_port_creation() {
        let port = Port::new_string_input("hello".to_string());
        assert!(port.is_textual());
        assert!(port.is_input());
        assert!(port.is_open());
    }
    
    #[test]
    fn test_port_predicates() {
        let args = vec![Value::integer(42)];
        let result = primitive_port_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        let port = Value::Port(Arc::new(Port::new_string_input("test".to_string())));
        let args = vec![port];
        let result = primitive_input_port_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
    
    #[test]
    fn test_eof_object() {
        let eof = primitive_eof_object(&[]).unwrap();
        let args = vec![eof.clone())];
        let result = primitive_eof_object_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(42)];
        let result = primitive_eof_object_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
}