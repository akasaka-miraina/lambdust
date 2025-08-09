//! Advanced I/O operations for R7RS-large compliance.
//!
//! This module provides:
//! - Advanced filesystem operations (directory management, metadata, permissions)
//! - Asynchronous I/O with high-performance backends
//! - Network I/O (TCP/UDP sockets, Unix domain sockets)
//! - SSL/TLS support
//! - Streaming I/O with compression and encryption
//! - Platform-specific optimizations
//! - Security and sandboxing features

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment
};
use crate::effects::Effect;
use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;

// Async imports disabled - not currently used
// #[cfg(feature = "async")]
// use tokio::fs as async_fs;
// #[cfg(feature = "async")]
// use tokio::net::{TcpListener, TcpStream, UdpSocket};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Creates advanced I/O operation bindings for R7RS-large.
pub fn create_advanced_io_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Filesystem operations
    bind_filesystem_operations(env);
    
    // Directory operations
    bind_directory_operations(env);
    
    // File metadata and attributes
    bind_metadata_operations(env);
    
    // Permission management
    bind_permission_operations(env);
    
    // File monitoring
    bind_monitoring_operations(env);
    
    // Network I/O
    bind_network_operations(env);
    
    // Compression and encryption
    bind_compression_operations(env);
    
    // Platform-specific operations
    bind_platform_operations(env);
}

// ============= FILESYSTEM OPERATIONS =============

fn bind_filesystem_operations(env: &Arc<ThreadSafeEnvironment>) {
    // create-directory
    env.define("create-directory".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "create-directory".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_create_directory),
        effects: vec![Effect::IO],
    })));
    
    // delete-directory
    env.define("delete-directory".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "delete-directory".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_delete_directory),
        effects: vec![Effect::IO],
    })));
    
    // copy-file
    env.define("copy-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "copy-file".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_copy_file),
        effects: vec![Effect::IO],
    })));
    
    // move-file
    env.define("move-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "move-file".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_move_file),
        effects: vec![Effect::IO],
    })));
    
    // create-symbolic-link
    env.define("create-symbolic-link".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "create-symbolic-link".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_create_symbolic_link),
        effects: vec![Effect::IO],
    })));
    
    // read-symbolic-link
    env.define("read-symbolic-link".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "read-symbolic-link".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_read_symbolic_link),
        effects: vec![Effect::IO],
    })));
}

fn bind_directory_operations(env: &Arc<ThreadSafeEnvironment>) {
    // list-directory
    env.define("list-directory".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list-directory".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_list_directory),
        effects: vec![Effect::IO],
    })));
    
    // directory?
    env.define("directory?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "directory?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_directory_p),
        effects: vec![Effect::IO],
    })));
    
    // current-directory
    env.define("current-directory".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "current-directory".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_current_directory),
        effects: vec![Effect::IO],
    })));
    
    // walk-directory
    env.define("walk-directory".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "walk-directory".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_walk_directory),
        effects: vec![Effect::IO],
    })));
}

fn bind_metadata_operations(env: &Arc<ThreadSafeEnvironment>) {
    // file-metadata
    env.define("file-metadata".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-metadata".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_metadata),
        effects: vec![Effect::IO],
    })));
    
    // file-size
    env.define("file-size".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-size".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_size),
        effects: vec![Effect::IO],
    })));
    
    // file-modification-time
    env.define("file-modification-time".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-modification-time".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_modification_time),
        effects: vec![Effect::IO],
    })));
    
    // file-access-time
    env.define("file-access-time".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-access-time".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_access_time),
        effects: vec![Effect::IO],
    })));
}

fn bind_permission_operations(env: &Arc<ThreadSafeEnvironment>) {
    // file-readable?
    env.define("file-readable?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-readable?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_readable_p),
        effects: vec![Effect::IO],
    })));
    
    // file-writable?
    env.define("file-writable?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-writable?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_writable_p),
        effects: vec![Effect::IO],
    })));
    
    // file-executable?
    env.define("file-executable?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "file-executable?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_file_executable_p),
        effects: vec![Effect::IO],
    })));
    
    // set-file-permissions
    env.define("set-file-permissions".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "set-file-permissions".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_set_file_permissions),
        effects: vec![Effect::IO],
    })));
}

fn bind_monitoring_operations(env: &Arc<ThreadSafeEnvironment>) {
    // watch-file
    env.define("watch-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "watch-file".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_watch_file),
        effects: vec![Effect::IO],
    })));
    
    // unwatch-file
    env.define("unwatch-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unwatch-file".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_unwatch_file),
        effects: vec![Effect::IO],
    })));
}

fn bind_network_operations(env: &Arc<ThreadSafeEnvironment>) {
    // tcp-connect
    env.define("tcp-connect".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-connect".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_connect),
        effects: vec![Effect::IO],
    })));
    
    // tcp-listen
    env.define("tcp-listen".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-listen".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_listen),
        effects: vec![Effect::IO],
    })));
    
    // udp-socket
    env.define("udp-socket".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "udp-socket".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_udp_socket),
        effects: vec![Effect::IO],
    })));
    
    // resolve-hostname
    env.define("resolve-hostname".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "resolve-hostname".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_resolve_hostname),
        effects: vec![Effect::IO],
    })));
}

fn bind_compression_operations(env: &Arc<ThreadSafeEnvironment>) {
    // compress-data
    env.define("compress-data".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "compress-data".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_compress_data),
        effects: vec![Effect::Pure],
    })));
    
    // decompress-data
    env.define("decompress-data".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "decompress-data".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_decompress_data),
        effects: vec![Effect::Pure],
    })));
}

fn bind_platform_operations(env: &Arc<ThreadSafeEnvironment>) {
    // platform-specific operations will be added here
    
    // memory-map-file
    env.define("memory-map-file".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "memory-map-file".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_memory_map_file),
        effects: vec![Effect::IO],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === Filesystem Operations ===

pub fn primitive_create_directory(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("create-directory expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "create-directory")?;
    let create_parents = if args.len() > 1 {
        extract_boolean(&args[1], "create-directory")?
    } else {
        false
    };
    
    let result = if create_parents {
        std::fs::create_dir_all(&path)
    } else {
        std::fs::create_dir(&path)
    };
    
    match result {
        Ok(()) => Ok(Value::Unspecified),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot create directory '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_delete_directory(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("delete-directory expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "delete-directory")?;
    let recursive = if args.len() > 1 {
        extract_boolean(&args[1], "delete-directory")?
    } else {
        false
    };
    
    let result = if recursive {
        std::fs::remove_dir_all(&path)
    } else {
        std::fs::remove_dir(&path)
    };
    
    match result {
        Ok(()) => Ok(Value::Unspecified),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot delete directory '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_copy_file(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("copy-file expects 2 or 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let src = extract_string(&args[0], "copy-file")?;
    let dst = extract_string(&args[1], "copy-file")?;
    let overwrite = if args.len() > 2 {
        extract_boolean(&args[2], "copy-file")?
    } else {
        false
    };
    
    if !overwrite && Path::new(&dst).exists() {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("Destination file '{dst}' already exists"),
            None,
        )));
    }
    
    match std::fs::copy(&src, &dst) {
        Ok(_) => Ok(Value::Unspecified),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot copy file '{src}' to '{dst}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_move_file(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("move-file expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let src = extract_string(&args[0], "move-file")?;
    let dst = extract_string(&args[1], "move-file")?;
    
    match std::fs::rename(&src, &dst) {
        Ok(()) => Ok(Value::Unspecified),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot move file '{src}' to '{dst}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_create_symbolic_link(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("create-symbolic-link expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let target = extract_string(&args[0], "create-symbolic-link")?;
    let link = extract_string(&args[1], "create-symbolic-link")?;
    
    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(&target, &link) {
            Ok(()) => Ok(Value::Unspecified),
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot create symbolic link '{link}' -> '{target}': {e}"),
                None,
            ))),
        }
    }
    
    #[cfg(windows)]
    {
        let target_path = Path::new(&target);
        let result = if target_path.is_dir() {
            std::os::windows::fs::symlink_dir(&target, &link)
        } else {
            std::os::windows::fs::symlink_file(&target, &link)
        };
        
        match result {
            Ok(()) => Ok(Value::Unspecified),
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot create symbolic link '{link}' -> '{target}': {e}"),
                None,
            ))),
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "Symbolic links not supported on this platform".to_string(),
            None,
        )))
    }
}

pub fn primitive_read_symbolic_link(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("read-symbolic-link expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let link = extract_string(&args[0], "read-symbolic-link")?;
    
    match std::fs::read_link(&link) {
        Ok(target) => Ok(Value::string(target.to_string_lossy().to_string())),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot read symbolic link '{link}': {e}"),
            None,
        ))),
    }
}

// === Directory Operations ===

pub fn primitive_list_directory(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list-directory expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "list-directory")?;
    let include_hidden = if args.len() > 1 {
        extract_boolean(&args[1], "list-directory")?
    } else {
        false
    };
    
    match std::fs::read_dir(&path) {
        Ok(entries) => {
            let mut result = Vec::new();
            
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if include_hidden || !name.starts_with('.') {
                            result.push(Value::string(name));
                        }
                    }
                    Err(e) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            format!("Error reading directory entry: {e}"),
                            None,
                        )));
                    }
                }
            }
            
            Ok(list_to_value(result))
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot read directory '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_directory_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("directory? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "directory?")?;
    Ok(Value::boolean(Path::new(&path).is_dir()))
}

pub fn primitive_current_directory(args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => {
            // Get current directory
            match std::env::current_dir() {
                Ok(path) => Ok(Value::string(path.to_string_lossy().to_string())),
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot get current directory: {e}"),
                    None,
                ))),
            }
        }
        1 => {
            // Set current directory
            let path = extract_string(&args[0], "current-directory")?;
            match std::env::set_current_dir(&path) {
                Ok(()) => Ok(Value::Unspecified),
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot set current directory to '{path}': {e}"),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("current-directory expects 0 or 1 arguments, got {}", args.len()),
            None,
        ))),
    }
}

pub fn primitive_walk_directory(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("walk-directory expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "walk-directory")?;
    let recursive = if args.len() > 1 {
        extract_boolean(&args[1], "walk-directory")?
    } else {
        true
    };
    let follow_links = if args.len() > 2 {
        extract_boolean(&args[2], "walk-directory")?
    } else {
        false
    };
    
    let walker = walkdir::WalkDir::new(&path)
        .follow_links(follow_links);
    
    let walker = if !recursive {
        walker.max_depth(1)
    } else {
        walker
    };
    
    let mut result = Vec::new();
    
    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path().to_string_lossy().to_string();
                result.push(Value::string(path));
            }
            Err(e) => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Error walking directory: {e}"),
                    None,
                )));
            }
        }
    }
    
    Ok(list_to_value(result))
}

// === Metadata Operations ===

pub fn primitive_file_metadata(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-metadata expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-metadata")?;
    
    match std::fs::metadata(&path) {
        Ok(metadata) => {
            #[allow(clippy::mutable_key_type)]
            let mut result = HashMap::new();
            
            // Basic properties
            result.insert(Value::Symbol(crate::utils::intern_symbol("size")), 
                         Value::integer(metadata.len() as i64));
            result.insert(Value::Symbol(crate::utils::intern_symbol("is-file")), 
                         Value::boolean(metadata.is_file()));
            result.insert(Value::Symbol(crate::utils::intern_symbol("is-dir")), 
                         Value::boolean(metadata.is_dir()));
            result.insert(Value::Symbol(crate::utils::intern_symbol("is-symlink")), 
                         Value::boolean(metadata.is_symlink()));
            
            // Timestamps
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    result.insert(Value::Symbol(crate::utils::intern_symbol("modified")), 
                                 Value::integer(duration.as_secs() as i64));
                }
            }
            
            if let Ok(accessed) = metadata.accessed() {
                if let Ok(duration) = accessed.duration_since(UNIX_EPOCH) {
                    result.insert(Value::Symbol(crate::utils::intern_symbol("accessed")), 
                                 Value::integer(duration.as_secs() as i64));
                }
            }
            
            if let Ok(created) = metadata.created() {
                if let Ok(duration) = created.duration_since(UNIX_EPOCH) {
                    result.insert(Value::Symbol(crate::utils::intern_symbol("created")), 
                                 Value::integer(duration.as_secs() as i64));
                }
            }
            
            // Platform-specific metadata
            #[cfg(unix)]
            {
                result.insert(Value::Symbol(crate::utils::intern_symbol("mode")), 
                             Value::integer(metadata.mode() as i64));
                result.insert(Value::Symbol(crate::utils::intern_symbol("uid")), 
                             Value::integer(metadata.uid() as i64));
                result.insert(Value::Symbol(crate::utils::intern_symbol("gid")), 
                             Value::integer(metadata.gid() as i64));
                result.insert(Value::Symbol(crate::utils::intern_symbol("nlink")), 
                             Value::integer(metadata.nlink() as i64));
            }
            
            #[cfg(windows)]
            {
                result.insert(Value::Symbol(crate::utils::intern_symbol("file-attributes")), 
                             Value::integer(metadata.file_attributes() as i64));
            }
            
            Ok(Value::Hashtable(Arc::new(std::sync::RwLock::new(result))))
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot get metadata for '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_file_size(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-size expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-size")?;
    
    match std::fs::metadata(&path) {
        Ok(metadata) => Ok(Value::integer(metadata.len() as i64)),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot get size of '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_file_modification_time(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-modification-time expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-modification-time")?;
    
    match std::fs::metadata(&path) {
        Ok(metadata) => {
            match metadata.modified() {
                Ok(time) => {
                    match time.duration_since(UNIX_EPOCH) {
                        Ok(duration) => Ok(Value::integer(duration.as_secs() as i64)),
                        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                            format!("Invalid modification time: {e}"),
                            None,
                        ))),
                    }
                }
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot get modification time for '{path}': {e}"),
                    None,
                ))),
            }
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot get metadata for '{path}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_file_access_time(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-access-time expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-access-time")?;
    
    match std::fs::metadata(&path) {
        Ok(metadata) => {
            match metadata.accessed() {
                Ok(time) => {
                    match time.duration_since(UNIX_EPOCH) {
                        Ok(duration) => Ok(Value::integer(duration.as_secs() as i64)),
                        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                            format!("Invalid access time: {e}"),
                            None,
                        ))),
                    }
                }
                Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot get access time for '{path}': {e}"),
                    None,
                ))),
            }
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Cannot get metadata for '{path}': {e}"),
            None,
        ))),
    }
}

// === Permission Operations ===

pub fn primitive_file_readable_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-readable? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-readable?")?;
    
    // Try to open the file for reading
    match std::fs::File::open(&path) {
        Ok(_) => Ok(Value::boolean(true)),
        Err(_) => Ok(Value::boolean(false)),
    }
}

pub fn primitive_file_writable_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-writable? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-writable?")?;
    
    // Check if we can open the file for writing
    match std::fs::OpenOptions::new().write(true).open(&path) {
        Ok(_) => Ok(Value::boolean(true)),
        Err(_) => Ok(Value::boolean(false)),
    }
}

pub fn primitive_file_executable_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("file-executable? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "file-executable?")?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                let permissions = metadata.permissions();
                Ok(Value::boolean(permissions.mode() & 0o111 != 0))
            }
            Err(_) => Ok(Value::boolean(false)),
        }
    }
    
    #[cfg(windows)]
    {
        // On Windows, check if it's an executable file by extension
        let path_obj = Path::new(&path);
        if let Some(ext) = path_obj.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            Ok(Value::boolean(matches!(ext_str.as_str(), "exe" | "bat" | "cmd" | "com")))
        } else {
            Ok(Value::boolean(false))
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        // Default to false on unknown platforms
        Ok(Value::boolean(false))
    }
}

pub fn primitive_set_file_permissions(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("set-file-permissions expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let path = extract_string(&args[0], "set-file-permissions")?;
    let mode = extract_integer(&args[1], "set-file-permissions")? as u32;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        match std::fs::set_permissions(&path, permissions) {
            Ok(()) => Ok(Value::Unspecified),
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot set permissions for '{path}': {e}"),
                None,
            ))),
        }
    }
    
    #[cfg(not(unix))]
    {
        // On non-Unix systems, we can only set readonly
        let readonly = mode & 0o200 == 0;
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(readonly);
                match std::fs::set_permissions(&path, permissions) {
                    Ok(()) => Ok(Value::Unspecified),
                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                        format!("Cannot set permissions for '{path}': {e}"),
                        None,
                    ))),
                }
            }
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot get metadata for '{path}': {e}"),
                None,
            ))),
        }
    }
}

// === Stub implementations for remaining functions ===

pub fn primitive_watch_file(_args: &[Value]) -> Result<Value> {
    // TODO: Implement file watching using notify crate
    Err(Box::new(DiagnosticError::runtime_error(
        "watch-file not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_unwatch_file(_args: &[Value]) -> Result<Value> {
    // TODO: Implement file unwatching
    Err(Box::new(DiagnosticError::runtime_error(
        "unwatch-file not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_tcp_connect(_args: &[Value]) -> Result<Value> {
    // TODO: Implement TCP connection
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-connect not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_tcp_listen(_args: &[Value]) -> Result<Value> {
    // TODO: Implement TCP listening
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-listen not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_udp_socket(_args: &[Value]) -> Result<Value> {
    // TODO: Implement UDP socket creation
    Err(Box::new(DiagnosticError::runtime_error(
        "udp-socket not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_resolve_hostname(_args: &[Value]) -> Result<Value> {
    // TODO: Implement hostname resolution
    Err(Box::new(DiagnosticError::runtime_error(
        "resolve-hostname not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_compress_data(_args: &[Value]) -> Result<Value> {
    // TODO: Implement data compression
    Err(Box::new(DiagnosticError::runtime_error(
        "compress-data not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_decompress_data(_args: &[Value]) -> Result<Value> {
    // TODO: Implement data decompression
    Err(Box::new(DiagnosticError::runtime_error(
        "decompress-data not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_memory_map_file(_args: &[Value]) -> Result<Value> {
    // TODO: Implement memory-mapped files
    Err(Box::new(DiagnosticError::runtime_error(
        "memory-map-file not yet implemented".to_string(),
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

    #[test]
    fn test_create_delete_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        let test_path = test_dir.to_string_lossy().to_string();
        
        // Create directory
        let args = vec![Value::string(test_path.clone())];
        let result = primitive_create_directory(&args);
        assert!(result.is_ok());
        assert!(test_dir.exists());
        
        // Delete directory
        let result = primitive_delete_directory(&args);
        assert!(result.is_ok());
        assert!(!test_dir.exists());
    }
    
    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create a test file
        std::fs::write(&test_file, "hello world").unwrap();
        
        let file_path = test_file.to_string_lossy().to_string();
        let args = vec![Value::string(file_path.clone())];
        
        // Test file size
        let result = primitive_file_size(&args).unwrap();
        if let Value::Literal(crate::ast::Literal::Number(size)) = result {
            assert_eq!(size, 11.0); // "hello world" is 11 bytes
        } else {
            panic!("Expected integer result");
        }
        
        // Test file readable
        let result = primitive_file_readable_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
}