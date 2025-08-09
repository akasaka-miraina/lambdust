#![allow(unused_variables)]
//! Asynchronous I/O operations with high-performance backends.
//!
//! This module provides:
//! - Async file operations with tokio
//! - High-performance I/O with epoll/kqueue/io_uring
//! - Async buffering strategies
//! - Non-blocking I/O primitives
//! - Future-based I/O operations

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment
};
use crate::effects::Effect;
use std::sync::Arc;
use std::sync::OnceLock;

#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt};
#[cfg(feature = "async")]
use tokio::fs::File as AsyncFile;
#[cfg(feature = "async")]
use tokio::runtime::{Runtime, Handle};
#[cfg(feature = "async")]
use std::future::Future;
// Unused async utilities - commented out
// #[cfg(feature = "async")]
// use std::pin::Pin;
// #[cfg(feature = "async")]
// use std::task::{Context, Poll};

/// Async I/O runtime manager
#[cfg(feature = "async")]
pub struct AsyncIoRuntime {
    runtime: Option<Runtime>,
}

#[cfg(feature = "async")]
impl AsyncIoRuntime {
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;
        Ok(AsyncIoRuntime {
            runtime: Some(runtime),
        })
    }
    
    pub fn get_handle(&self) -> Option<Handle> {
        self.runtime.as_ref().map(|rt| rt.handle().clone())
    }
    
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        if let Some(rt) = &self.runtime {
            rt.block_on(future)
        } else {
            // Fallback to current runtime if available
            Handle::current().block_on(future)
        }
    }
}

#[cfg(not(feature = "async"))]
pub struct AsyncIoRuntime;

#[cfg(not(feature = "async"))]
impl AsyncIoRuntime {
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(AsyncIoRuntime)
    }
}

/// Global async runtime instance
static ASYNC_RUNTIME: OnceLock<AsyncIoRuntime> = OnceLock::new();

pub fn get_async_runtime() -> &'static AsyncIoRuntime {
    ASYNC_RUNTIME.get_or_init(|| {
        AsyncIoRuntime::new().expect("Failed to create async runtime")
    })
}

/// Creates async I/O operation bindings.
pub fn create_async_io_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Async file operations
    bind_async_file_operations(env);
    
    // Async buffered I/O
    bind_async_buffered_operations(env);
    
    // High-performance I/O operations
    bind_high_performance_operations(env);
    
    // Future combinators
    bind_future_operations(env);
}

// ============= ASYNC FILE OPERATIONS =============

fn bind_async_file_operations(env: &Arc<ThreadSafeEnvironment>) {
    // async-read-file
    env.define("async-read-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-read-file".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_async_read_file),
        effects: vec![Effect::IO],
    })));
    
    // async-write-file
    env.define("async-write-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-write-file".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_async_write_file),
        effects: vec![Effect::IO],
    })));
    
    // async-append-file
    env.define("async-append-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-append-file".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_async_append_file),
        effects: vec![Effect::IO],
    })));
    
    // async-copy-file
    env.define("async-copy-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-copy-file".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_async_copy_file),
        effects: vec![Effect::IO],
    })));
}

fn bind_async_buffered_operations(env: &Arc<ThreadSafeEnvironment>) {
    // async-read-lines
    env.define("async-read-lines".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-read-lines".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_async_read_lines),
        effects: vec![Effect::IO],
    })));
    
    // async-read-chunks
    env.define("async-read-chunks".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-read-chunks".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_async_read_chunks),
        effects: vec![Effect::IO],
    })));
    
    // async-write-chunks
    env.define("async-write-chunks".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "async-write-chunks".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_async_write_chunks),
        effects: vec![Effect::IO],
    })));
}

fn bind_high_performance_operations(env: &Arc<ThreadSafeEnvironment>) {
    // io-uring-read (Linux-specific)
    env.define("io-uring-read".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "io-uring-read".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_io_uring_read),
        effects: vec![Effect::IO],
    })));
    
    // io-uring-write (Linux-specific)
    env.define("io-uring-write".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "io-uring-write".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_io_uring_write),
        effects: vec![Effect::IO],
    })));
    
    // batch-io-operations
    env.define("batch-io-operations".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "batch-io-operations".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_batch_io_operations),
        effects: vec![Effect::IO],
    })));
}

fn bind_future_operations(env: &Arc<ThreadSafeEnvironment>) {
    // await-future
    env.define("await-future".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "await-future".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_await_future),
        effects: vec![Effect::IO],
    })));
    
    // spawn-task
    env.define("spawn-task".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "spawn-task".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_spawn_task),
        effects: vec![Effect::IO],
    })));
    
    // join-tasks
    env.define("join-tasks".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "join-tasks".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_join_tasks),
        effects: vec![Effect::IO],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === Async File Operations ===

#[cfg(feature = "async")]
pub fn primitive_async_read_file(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("async-read-file expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "async-read-file")?;
    let as_binary = if args.len() > 1 {
        extract_boolean(&args[1], "async-read-file")?
    } else {
        false
    };
    
    let runtime = get_async_runtime();
    
    runtime.block_on(async move {
        if as_binary {
            match AsyncFile::open(&path).await {
                Ok(mut file) => {
                    let mut contents = Vec::new();
                    match file.read_to_end(&mut contents).await {
                        Ok(_) => Ok(Value::bytevector(contents)),
                        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                            format!("Error reading file '{path}': {e}"),
                            None,
                        ))),
                    }
                }
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot open file '{path}': {e}"),
                    None,
                ))),
            }
        } else {
            match tokio::fs::read_to_string(&path).await {
                Ok(contents) => Ok(Value::string(contents)),
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Error reading file '{path}': {e}"),
                    None,
                ))),
            }
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_read_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-read-file requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_async_write_file(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            {
                let arg_count = args.len();
                format!("async-write-file expects 2 or 3 arguments, got {arg_count}")
            },
            None,
        )));
    }
    
    let path = extract_string(&args[0], "async-write-file")?;
    let create_dirs = if args.len() > 2 {
        extract_boolean(&args[2], "async-write-file")?
    } else {
        false
    };
    
    let runtime = get_async_runtime();
    
    runtime.block_on(async move {
        if create_dirs {
            if let Some(parent) = std::path::Path::new(&path).parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("Cannot create parent directories for '{path}': {e}"),
                        None,
                    )));
                }
            }
        }
        
        match &args[1] {
            Value::Literal(crate::ast::Literal::String(content)) => {
                match tokio::fs::write(&path, content).await {
                    Ok(()) => Ok(Value::Unspecified),
                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                        format!("Error writing to file '{path}': {e}"),
                        None,
                    ))),
                }
            }
            Value::Literal(crate::ast::Literal::Bytevector(content)) => {
                match tokio::fs::write(&path, content).await {
                    Ok(()) => Ok(Value::Unspecified),
                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                        format!("Error writing to file '{path}': {e}"),
                        None,
                    ))),
                }
            }
            _ => Err(Box::new(DiagnosticError::runtime_error(
                "async-write-file requires string or bytevector content".to_string(),
                None,
            ))),
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_write_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-write-file requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_async_append_file(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            {
                let arg_count = args.len();
                format!("async-append-file expects 2 arguments, got {arg_count}")
            },
            None,
        )));
    }
    
    let path = extract_string(&args[0], "async-append-file")?;
    
    let runtime = get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let mut file = match tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
        {
            Ok(file) => file,
            Err(e) => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot open file '{path}' for appending: {e}"),
                    None,
                )));
            }
        };
        
        match &args[1] {
            Value::Literal(crate::ast::Literal::String(content)) => {
                match file.write_all(content.as_bytes()).await {
                    Ok(()) => {
                        match file.flush().await {
                            Ok(()) => Ok(Value::Unspecified),
                            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                                format!("Error flushing file '{path}': {e}"),
                                None,
                            ))),
                        }
                    }
                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                        format!("Error appending to file '{path}': {e}"),
                        None,
                    ))),
                }
            }
            Value::Literal(crate::ast::Literal::Bytevector(content)) => {
                match file.write_all(content).await {
                    Ok(()) => {
                        match file.flush().await {
                            Ok(()) => Ok(Value::Unspecified),
                            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                                format!("Error flushing file '{path}': {e}"),
                                None,
                            ))),
                        }
                    }
                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                        format!("Error appending to file '{path}': {e}"),
                        None,
                    ))),
                }
            }
            _ => Err(Box::new(DiagnosticError::runtime_error(
                "async-append-file requires string or bytevector content".to_string(),
                None,
            ))),
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_append_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-append-file requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_async_copy_file(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            {
                let arg_count = args.len();
                format!("async-copy-file expects 2 or 3 arguments, got {arg_count}")
            },
            None,
        )));
    }
    
    let src = extract_string(&args[0], "async-copy-file")?;
    let dst = extract_string(&args[1], "async-copy-file")?;
    let overwrite = if args.len() > 2 {
        extract_boolean(&args[2], "async-copy-file")?
    } else {
        false
    };
    
    let runtime = get_async_runtime();
    
    
    
    runtime.block_on(async move {
        if !overwrite && tokio::fs::try_exists(&dst).await.unwrap_or(true) {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("Destination file '{dst}' already exists"),
                None,
            )));
        }
        
        match tokio::fs::copy(&src, &dst).await {
            Ok(bytes_copied) => Ok(Value::integer(bytes_copied as i64)),
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot copy file '{src}' to '{dst}': {e}"),
                None,
            ))),
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_copy_file(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-copy-file requires async feature".to_string(),
        None,
    )))
}

// === Async Buffered Operations ===

#[cfg(feature = "async")]
pub fn primitive_async_read_lines(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            {
                let arg_count = args.len();
                format!("async-read-lines expects 1 or 2 arguments, got {arg_count}")
            },
            None,
        )));
    }
    
    let path = extract_string(&args[0], "async-read-lines")?;
    let max_lines = if args.len() > 1 {
        Some(extract_integer(&args[1], "async-read-lines")? as usize)
    } else {
        None
    };
    
    let runtime = get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let file = match AsyncFile::open(&path).await {
            Ok(file) => file,
            Err(e) => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot open file '{path}': {e}"),
                    None,
                )));
            }
        };
        
        let reader = tokio::io::BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = Vec::new();
        let mut count = 0;
        
        while let Some(line) = lines.next_line().await.map_err(|e| {
            Box::new(DiagnosticError::runtime_error(
                format!("Error reading line from file '{path}': {e}"),
                None,
            ))
        })? {
            result.push(Value::string(line));
            count += 1;
            
            if let Some(max) = max_lines {
                if count >= max {
                    break;
                }
            }
        }
        
        Ok(list_to_value(result))
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_read_lines(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-read-lines requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_async_read_chunks(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            {
                let arg_count = args.len();
                format!("async-read-chunks expects 2 or 3 arguments, got {arg_count}")
            },
            None,
        )));
    }
    
    let path = extract_string(&args[0], "async-read-chunks")?;
    let chunk_size = extract_integer(&args[1], "async-read-chunks")? as usize;
    let max_chunks = if args.len() > 2 {
        Some(extract_integer(&args[2], "async-read-chunks")? as usize)
    } else {
        None
    };
    
    let runtime = get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let mut file = match AsyncFile::open(&path).await {
            Ok(file) => file,
            Err(e) => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot open file '{path}': {e}"),
                    None,
                )));
            }
        };
        
        let mut result = Vec::new();
        let mut count = 0;
        
        loop {
            let mut buffer = vec![0u8; chunk_size];
            match file.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    buffer.truncate(n);
                    result.push(Value::bytevector(buffer));
                    count += 1;
                    
                    if let Some(max) = max_chunks {
                        if count >= max {
                            break;
                        }
                    }
                }
                Err(e) => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("Error reading chunk from file '{path}': {e}"),
                        None,
                    )));
                }
            }
        }
        
        Ok(list_to_value(result))
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_read_chunks(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-read-chunks requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_async_write_chunks(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            {
                let arg_count = args.len();
                format!("async-write-chunks expects 2 or 3 arguments, got {arg_count}")
            },
            None,
        )));
    }
    
    let path = extract_string(&args[0], "async-write-chunks")?;
    let chunks = extract_list(&args[1], "async-write-chunks")?;
    let append = if args.len() > 2 {
        extract_boolean(&args[2], "async-write-chunks")?
    } else {
        false
    };
    
    let runtime = get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let file = if append {
            tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
        } else {
            tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
                .await
        };
        
        let mut file = match file {
            Ok(file) => file,
            Err(e) => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot open file '{path}': {e}"),
                    None,
                )));
            }
        };
        
        let mut total_written = 0u64;
        
        for chunk in chunks {
            match chunk {
                Value::Literal(crate::ast::Literal::Bytevector(bytes)) => {
                    match file.write_all(&bytes).await {
                        Ok(()) => total_written += bytes.len() as u64,
                        Err(e) => {
                            return Err(Box::new(DiagnosticError::runtime_error(
                                format!("Error writing chunk to file '{path}': {e}"),
                                None,
                            )));
                        }
                    }
                }
                Value::Literal(crate::ast::Literal::String(text)) => {
                    match file.write_all(text.as_bytes()).await {
                        Ok(()) => total_written += text.len() as u64,
                        Err(e) => {
                            return Err(Box::new(DiagnosticError::runtime_error(
                                format!("Error writing chunk to file '{path}': {e}"),
                                None,
                            )));
                        }
                    }
                }
                _ => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        "async-write-chunks requires bytevector or string chunks".to_string(),
                        None,
                    )));
                }
            }
        }
        
        match file.flush().await {
            Ok(()) => Ok(Value::integer(total_written as i64)),
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Error flushing file '{path}': {e}"),
                None,
            ))),
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_async_write_chunks(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "async-write-chunks requires async feature".to_string(),
        None,
    )))
}

// === High-performance Operations ===

pub fn primitive_io_uring_read(_args: &[Value]) -> Result<Value> {
    // TODO: Implement io_uring-based reading for Linux
    #[cfg(target_os = "linux")]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "io-uring-read not yet implemented".to_string(),
            None,
        )))
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "io-uring-read is only available on Linux".to_string(),
            None,
        )))
    }
}

pub fn primitive_io_uring_write(_args: &[Value]) -> Result<Value> {
    // TODO: Implement io_uring-based writing for Linux
    #[cfg(target_os = "linux")]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "io-uring-write not yet implemented".to_string(),
            None,
        )))
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "io-uring-write is only available on Linux".to_string(),
            None,
        )))
    }
}

pub fn primitive_batch_io_operations(_args: &[Value]) -> Result<Value> {
    // TODO: Implement batched I/O operations
    Err(Box::new(DiagnosticError::runtime_error(
        "batch-io-operations not yet implemented".to_string(),
        None,
    )))
}

// === Future Operations ===

pub fn primitive_await_future(_args: &[Value]) -> Result<Value> {
    // TODO: Implement future awaiting
    Err(Box::new(DiagnosticError::runtime_error(
        "await-future not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_spawn_task(_args: &[Value]) -> Result<Value> {
    // TODO: Implement task spawning
    Err(Box::new(DiagnosticError::runtime_error(
        "spawn-task not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_join_tasks(_args: &[Value]) -> Result<Value> {
    // TODO: Implement task joining
    Err(Box::new(DiagnosticError::runtime_error(
        "join-tasks not yet implemented".to_string(),
        None,
    )))
}

// ============= HELPER FUNCTIONS =============

/// Extracts a string from a Value.
fn extract_string(value: &Value, operation: &str) -> Result<String> {
    match value {
        Value::Literal(crate::ast::Literal::String(s)) => Ok(s.clone()),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))),
    }
}

/// Extracts a boolean from a Value.
fn extract_boolean(value: &Value, operation: &str) -> Result<bool> {
    match value {
        Value::Literal(crate::ast::Literal::Boolean(b)) => Ok(*b),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires boolean arguments"),
            None,
        ))),
    }
}

/// Extracts an integer from a Value.
fn extract_integer(value: &Value, operation: &str) -> Result<i64> {
    match value {
        Value::Literal(lit) => {
            if let Some(i) = lit.to_i64() {
                Ok(i)
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    format!("{operation} requires integer arguments"),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires integer arguments"),
            None,
        ))),
    }
}

/// Extracts a list from a Value.
fn extract_list(value: &Value, operation: &str) -> Result<Vec<Value>> {
    fn list_to_vec(value: &Value, acc: &mut Vec<Value>) -> Result<()> {
        match value {
            Value::Nil => Ok(()),
            Value::Pair(car, cdr) => {
                acc.push((**car).clone());
                list_to_vec(cdr, acc)
            }
            _ => Err(Box::new(DiagnosticError::runtime_error(
                "Invalid list structure".to_string(),
                None,
            ))),
        }
    }
    
    let mut result = Vec::new();
    list_to_vec(value, &mut result)?;
    Ok(result)
}

/// Converts a vector of values to a Scheme list.
fn list_to_value(values: Vec<Value>) -> Value {
    values.into_iter().rev().fold(Value::Nil, |acc, val| {
        Value::Pair(Arc::new(val), Arc::new(acc))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("async_test.txt");
        let file_path = test_file.to_string_lossy().to_string();
        
        // Test async write
        let args = vec![
            Value::string(file_path.clone()),
            Value::string("Hello, async world!".to_string()),
        ];
        let result = primitive_async_write_file(&args);
        assert!(result.is_ok());
        
        // Test async read
        let args = vec![Value::string(file_path)];
        let result = primitive_async_read_file(&args);
        assert!(result.is_ok());
        
        if let Ok(Value::Literal(crate::ast::Literal::String(content))) = result {
            assert_eq!(content, "Hello, async world!");
        } else {
            panic!("Expected string result");
        }
    }
    
    #[test]
    fn test_runtime_creation() {
        let runtime = get_async_runtime();
        // Just verify we can create the runtime without panicking
        assert!(true);
    }
}