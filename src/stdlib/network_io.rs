#![allow(unused_variables)]
//! Network I/O operations for R7RS-large compliance.
//!
//! This module provides:
//! - TCP/UDP socket operations
//! - Unix domain sockets
//! - SSL/TLS support
//! - HTTP/WebSocket client and server
//! - DNS resolution
//! - Network utilities and diagnostics

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{
    Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment
};
use crate::effects::Effect;
use std::sync::Arc;
use std::collections::HashMap;
// Network address types not currently used
// use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

#[cfg(feature = "async")]
use tokio::net::{TcpListener, TcpStream, UdpSocket};
#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(feature = "async")]
use tokio::time::timeout;

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

#[cfg(feature = "tls")]
use rustls::{ClientConfig, ServerConfig};
#[cfg(feature = "tls")]
use tokio_rustls::{TlsConnector, TlsAcceptor};

use url::Url;
use hickory_resolver::TokioAsyncResolver;

/// Network socket wrapper
#[derive(Debug, Clone)]
pub enum NetworkSocket {
    Tcp(Arc<std::sync::Mutex<Option<TcpStream>>>),
    Udp(Arc<std::sync::Mutex<Option<UdpSocket>>>),
    #[cfg(unix)]
    Unix(Arc<std::sync::Mutex<Option<UnixStream>>>),
    #[cfg(feature = "tls")]
    Tls(Arc<std::sync::Mutex<Option<tokio_rustls::client::TlsStream<TcpStream>>>>),
}

/// Network listener wrapper
#[derive(Debug, Clone)]
pub enum NetworkListener {
    Tcp(Arc<std::sync::Mutex<Option<TcpListener>>>),
    #[cfg(unix)]
    Unix(Arc<std::sync::Mutex<Option<UnixListener>>>),
    #[cfg(feature = "tls")]
    Tls(Arc<std::sync::Mutex<Option<TcpListener>>>, Arc<TlsAcceptor>),
}

/// HTTP request/response structures
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub reason: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// Creates network I/O operation bindings.
pub fn create_network_io_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // TCP operations
    bind_tcp_operations(env);
    
    // UDP operations
    bind_udp_operations(env);
    
    // Unix domain socket operations
    bind_unix_socket_operations(env);
    
    // DNS operations
    bind_dns_operations(env);
    
    // HTTP operations
    bind_http_operations(env);
    
    // TLS operations
    bind_tls_operations(env);
    
    // Network utilities
    bind_network_utilities(env);
}

// ============= TCP OPERATIONS =============

fn bind_tcp_operations(env: &Arc<ThreadSafeEnvironment>) {
    // tcp-connect
    env.define("tcp-connect".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-connect".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_connect),
        effects: vec![Effect::IO],
    })));
    
    // tcp-listen
    env.define("tcp-listen".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-listen".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_listen),
        effects: vec![Effect::IO],
    })));
    
    // tcp-accept
    env.define("tcp-accept".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-accept".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_accept),
        effects: vec![Effect::IO],
    })));
    
    // tcp-read
    env.define("tcp-read".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-read".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_read),
        effects: vec![Effect::IO],
    })));
    
    // tcp-write
    env.define("tcp-write".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-write".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_write),
        effects: vec![Effect::IO],
    })));
    
    // tcp-close
    env.define("tcp-close".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tcp-close".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_tcp_close),
        effects: vec![Effect::IO],
    })));
}

fn bind_udp_operations(env: &Arc<ThreadSafeEnvironment>) {
    // udp-socket
    env.define("udp-socket".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "udp-socket".to_string(),
        arity_min: 0,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_udp_socket),
        effects: vec![Effect::IO],
    })));
    
    // udp-bind
    env.define("udp-bind".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "udp-bind".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_udp_bind),
        effects: vec![Effect::IO],
    })));
    
    // udp-send-to
    env.define("udp-send-to".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "udp-send-to".to_string(),
        arity_min: 4,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_udp_send_to),
        effects: vec![Effect::IO],
    })));
    
    // udp-recv-from
    env.define("udp-recv-from".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "udp-recv-from".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_udp_recv_from),
        effects: vec![Effect::IO],
    })));
}

fn bind_unix_socket_operations(env: &Arc<ThreadSafeEnvironment>) {
    // unix-connect
    env.define("unix-connect".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unix-connect".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_unix_connect),
        effects: vec![Effect::IO],
    })));
    
    // unix-listen
    env.define("unix-listen".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "unix-listen".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_unix_listen),
        effects: vec![Effect::IO],
    })));
}

fn bind_dns_operations(env: &Arc<ThreadSafeEnvironment>) {
    // resolve-hostname
    env.define("resolve-hostname".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "resolve-hostname".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_resolve_hostname),
        effects: vec![Effect::IO],
    })));
    
    // reverse-dns-lookup
    env.define("reverse-dns-lookup".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "reverse-dns-lookup".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_reverse_dns_lookup),
        effects: vec![Effect::IO],
    })));
}

fn bind_http_operations(env: &Arc<ThreadSafeEnvironment>) {
    // http-get
    env.define("http-get".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "http-get".to_string(),
        arity_min: 1,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_http_get),
        effects: vec![Effect::IO],
    })));
    
    // http-post
    env.define("http-post".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "http-post".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_http_post),
        effects: vec![Effect::IO],
    })));
    
    // http-server
    env.define("http-server".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "http-server".to_string(),
        arity_min: 2,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_http_server),
        effects: vec![Effect::IO],
    })));
}

fn bind_tls_operations(env: &Arc<ThreadSafeEnvironment>) {
    // tls-connect
    env.define("tls-connect".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tls-connect".to_string(),
        arity_min: 2,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_tls_connect),
        effects: vec![Effect::IO],
    })));
    
    // tls-listen
    env.define("tls-listen".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tls-listen".to_string(),
        arity_min: 3,
        arity_max: Some(4),
        implementation: PrimitiveImpl::RustFn(primitive_tls_listen),
        effects: vec![Effect::IO],
    })));
}

fn bind_network_utilities(env: &Arc<ThreadSafeEnvironment>) {
    // parse-url
    env.define("parse-url".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "parse-url".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_parse_url),
        effects: vec![Effect::Pure],
    })));
    
    // format-url
    env.define("format-url".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "format-url".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_format_url),
        effects: vec![Effect::Pure],
    })));
    
    // network-interface-list
    env.define("network-interface-list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "network-interface-list".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_network_interface_list),
        effects: vec![Effect::IO],
    })));
}

// ============= IMPLEMENTATION FUNCTIONS =============

// === TCP Operations ===

#[cfg(feature = "async")]
pub fn primitive_tcp_connect(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("tcp-connect expects 2 to 4 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let host = extract_string(&args[0], "tcp-connect")?;
    let port = extract_integer(&args[1], "tcp-connect")? as u16;
    let timeout_ms = if args.len() > 2 {
        Some(extract_integer(&args[2], "tcp-connect")? as u64)
    } else {
        None
    };
    let nodelay = if args.len() > 3 {
        extract_boolean(&args[3], "tcp-connect")?
    } else {
        false
    };
    
    let runtime = crate::stdlib::async_io::get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let addr = format!("{host}:{port}");
        
        let connect_future = TcpStream::connect(&addr);
        
        let stream = if let Some(timeout_val) = timeout_ms {
            match timeout(Duration::from_millis(timeout_val), connect_future).await {
                Ok(Ok(stream)) => stream,
                Ok(Err(e)) => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("Cannot connect to {host}:{port}: {e}"),
                        None,
                    )));
                }
                Err(_) => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("Connection timeout to {host}:{port}"),
                        None,
                    )));
                }
            }
        } else {
            match connect_future.await {
                Ok(stream) => stream,
                Err(e) => {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("Cannot connect to {host}:{port}: {e}"),
                        None,
                    )));
                }
            }
        };
        
        if nodelay {
            if let Err(e) = stream.set_nodelay(true) {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot set TCP_NODELAY: {e}"),
                    None,
                )));
            }
        }
        
        let socket = NetworkSocket::Tcp(Arc::new(std::sync::Mutex::new(Some(stream))));
        Ok(Value::opaque(Box::new(socket)))
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_tcp_connect(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-connect requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_tcp_listen(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("tcp-listen expects 1 to 3 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let port = extract_integer(&args[0], "tcp-listen")? as u16;
    let host = if args.len() > 1 {
        extract_string(&args[1], "tcp-listen")?
    } else {
        "127.0.0.1".to_string()
    };
    let backlog = if args.len() > 2 {
        extract_integer(&args[2], "tcp-listen")? as u32
    } else {
        128
    };
    
    let runtime = crate::stdlib::async_io::get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let addr = format!("{host}:{port}");
        
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                let network_listener = NetworkListener::Tcp(Arc::new(std::sync::Mutex::new(Some(listener))));
                Ok(Value::opaque(Box::new(network_listener)))
            }
            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                format!("Cannot bind to {host}:{port}: {e}"),
                None,
            ))),
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_tcp_listen(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-listen requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_tcp_accept(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("tcp-accept expects 1 or 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let timeout_ms = if args.len() > 1 {
        Some(extract_integer(&args[1], "tcp-accept")? as u64)
    } else {
        None
    };
    
    // Extract listener from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(listener) = opaque_data.downcast_ref::<NetworkListener>() {
                match listener {
                    NetworkListener::Tcp(listener_mutex) => {
                        let runtime = crate::stdlib::async_io::get_async_runtime();
                        
                        
                        
                        #[allow(clippy::await_holding_lock)]
                        runtime.block_on(async move {
                            let listener_guard = listener_mutex.lock().unwrap();
                            if let Some(listener) = listener_guard.as_ref() {
                                let accept_future = listener.accept();
                                
                                let (stream, addr) = if let Some(timeout_val) = timeout_ms {
                                    match timeout(Duration::from_millis(timeout_val), accept_future).await {
                                        Ok(Ok((stream, addr))) => (stream, addr),
                                        Ok(Err(e)) => {
                                            return Err(Box::new(DiagnosticError::runtime_error(
                                                format!("Accept error: {e}"),
                                                None,
                                            )));
                                        }
                                        Err(_) => {
                                            return Err(Box::new(DiagnosticError::runtime_error(
                                                "Accept timeout".to_string(),
                                                None,
                                            )));
                                        }
                                    }
                                } else {
                                    match accept_future.await {
                                        Ok((stream, addr)) => (stream, addr),
                                        Err(e) => {
                                            return Err(Box::new(DiagnosticError::runtime_error(
                                                format!("Accept error: {e}"),
                                                None,
                                            )));
                                        }
                                    }
                                };
                                
                                let socket = NetworkSocket::Tcp(Arc::new(std::sync::Mutex::new(Some(stream))));
                                let addr_str = addr.to_string();
                                
                                Ok(Value::Pair(
                                    Arc::new(Value::opaque(Box::new(socket))),
                                    Arc::new(Value::string(addr_str))
                                ))
                            } else {
                                Err(Box::new(DiagnosticError::runtime_error(
                                    "TCP listener is closed".to_string(),
                                    None,
                                )))
                            }
                        })
                    }
                    _ => Err(Box::new(DiagnosticError::runtime_error(
                        "tcp-accept requires TCP listener".to_string(),
                        None,
                    ))),
                }
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "tcp-accept requires network listener".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "tcp-accept requires network listener".to_string(),
            None,
        ))),
    }
}

#[cfg(not(feature = "async"))]
pub fn primitive_tcp_accept(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-accept requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_tcp_read(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("tcp-read expects 1 to 3 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let buffer_size = if args.len() > 1 {
        extract_integer(&args[1], "tcp-read")? as usize
    } else {
        4096
    };
    
    let timeout_ms = if args.len() > 2 {
        Some(extract_integer(&args[2], "tcp-read")? as u64)
    } else {
        None
    };
    
    // Extract socket from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(socket) = opaque_data.downcast_ref::<NetworkSocket>() {
                match socket {
                    NetworkSocket::Tcp(stream_mutex) => {
                        let runtime = crate::stdlib::async_io::get_async_runtime();
                        
                        
                        
                        #[allow(clippy::await_holding_lock)]
                        runtime.block_on(async move {
                            let mut stream_guard = stream_mutex.lock().unwrap();
                            if let Some(stream) = stream_guard.as_mut() {
                                let mut buffer = vec![0u8; buffer_size];
                                
                                let read_future = stream.read(&mut buffer);
                                
                                let bytes_read = if let Some(timeout_val) = timeout_ms {
                                    match timeout(Duration::from_millis(timeout_val), read_future).await {
                                        Ok(Ok(n)) => n,
                                        Ok(Err(e)) => {
                                            return Err(Box::new(DiagnosticError::runtime_error(
                                                format!("TCP read error: {e}"),
                                                None,
                                            )));
                                        }
                                        Err(_) => {
                                            return Err(Box::new(DiagnosticError::runtime_error(
                                                "TCP read timeout".to_string(),
                                                None,
                                            )));
                                        }
                                    }
                                } else {
                                    match read_future.await {
                                        Ok(n) => n,
                                        Err(e) => {
                                            return Err(Box::new(DiagnosticError::runtime_error(
                                                format!("TCP read error: {e}"),
                                                None,
                                            )));
                                        }
                                    }
                                };
                                
                                if bytes_read == 0 {
                                    Ok(Value::Nil) // EOF
                                } else {
                                    buffer.truncate(bytes_read);
                                    Ok(Value::bytevector(buffer))
                                }
                            } else {
                                Err(Box::new(DiagnosticError::runtime_error(
                                    "TCP socket is closed".to_string(),
                                    None,
                                )))
                            }
                        })
                    }
                    _ => Err(Box::new(DiagnosticError::runtime_error(
                        "tcp-read requires TCP socket".to_string(),
                        None,
                    ))),
                }
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "tcp-read requires network socket".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "tcp-read requires network socket".to_string(),
            None,
        ))),
    }
}

#[cfg(not(feature = "async"))]
pub fn primitive_tcp_read(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-read requires async feature".to_string(),
        None,
    )))
}

#[cfg(feature = "async")]
pub fn primitive_tcp_write(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("tcp-write expects 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let data = match &args[1] {
        Value::Literal(crate::ast::Literal::String(s)) => s.as_bytes().to_vec(),
        Value::Literal(crate::ast::Literal::Bytevector(bv)) => bv.clone(),
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "tcp-write requires string or bytevector data".to_string(),
                None,
            )));
        }
    };
    
    // Extract socket from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(socket) = opaque_data.downcast_ref::<NetworkSocket>() {
                match socket {
                    NetworkSocket::Tcp(stream_mutex) => {
                        let runtime = crate::stdlib::async_io::get_async_runtime();
                        
                        #[allow(clippy::await_holding_lock)]
                        runtime.block_on(async move {
                            let mut stream_guard = stream_mutex.lock().unwrap();
                            if let Some(stream) = stream_guard.as_mut() {
                                match stream.write_all(&data).await {
                                    Ok(()) => {
                                        match stream.flush().await {
                                            Ok(()) => Ok(Value::integer(data.len() as i64)),
                                            Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                                                format!("TCP flush error: {e}"),
                                                None,
                                            ))),
                                        }
                                    }
                                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                                        format!("TCP write error: {e}"),
                                        None,
                                    ))),
                                }
                            } else {
                                Err(Box::new(DiagnosticError::runtime_error(
                                    "TCP socket is closed".to_string(),
                                    None,
                                )))
                            }
                        })
                    }
                    _ => Err(Box::new(DiagnosticError::runtime_error(
                        "tcp-write requires TCP socket".to_string(),
                        None,
                    ))),
                }
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "tcp-write requires network socket".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "tcp-write requires network socket".to_string(),
            None,
        ))),
    }
}

#[cfg(not(feature = "async"))]
pub fn primitive_tcp_write(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "tcp-write requires async feature".to_string(),
        None,
    )))
}

pub fn primitive_tcp_close(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("tcp-close expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // Extract socket from opaque value
    match &args[0] {
        Value::Opaque(opaque_data) => {
            if let Some(socket) = opaque_data.downcast_ref::<NetworkSocket>() {
                match socket {
                    NetworkSocket::Tcp(stream_mutex) => {
                        let mut stream_guard = stream_mutex.lock().unwrap();
                        *stream_guard = None; // Drop the stream to close it
                        Ok(Value::Unspecified)
                    }
                    _ => Err(Box::new(DiagnosticError::runtime_error(
                        "tcp-close requires TCP socket".to_string(),
                        None,
                    ))),
                }
            } else {
                Err(Box::new(DiagnosticError::runtime_error(
                    "tcp-close requires network socket".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "tcp-close requires network socket".to_string(),
            None,
        ))),
    }
}

// === Stub implementations for remaining functions ===

pub fn primitive_udp_socket(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "udp-socket not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_udp_bind(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "udp-bind not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_udp_send_to(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "udp-send-to not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_udp_recv_from(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "udp-recv-from not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_unix_connect(_args: &[Value]) -> Result<Value> {
    #[cfg(unix)]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "unix-connect not yet implemented".to_string(),
            None,
        )))
    }
    
    #[cfg(not(unix))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "Unix domain sockets not available on this platform".to_string(),
            None,
        )))
    }
}

pub fn primitive_unix_listen(_args: &[Value]) -> Result<Value> {
    #[cfg(unix)]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "unix-listen not yet implemented".to_string(),
            None,
        )))
    }
    
    #[cfg(not(unix))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "Unix domain sockets not available on this platform".to_string(),
            None,
        )))
    }
}

#[cfg(feature = "async")]
pub fn primitive_resolve_hostname(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("resolve-hostname expects 1 or 2 arguments, got {args_len}", args_len = args.len()),
            None,
        )));
    }
    
    let hostname = extract_string(&args[0], "resolve-hostname")?;
    let record_type = if args.len() > 1 {
        extract_string(&args[1], "resolve-hostname")?
    } else {
        "A".to_string()
    };
    
    let runtime = crate::stdlib::async_io::get_async_runtime();
    
    
    
    runtime.block_on(async move {
        let resolver = match TokioAsyncResolver::tokio_from_system_conf() {
            Ok(resolver) => resolver,
            Err(e) => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("Cannot create DNS resolver: {e}"),
                    None,
                )));
            }
        };
        
        match record_type.as_str() {
            "A" => {
                match resolver.lookup_ip(&hostname).await {
                    Ok(lookup) => {
                        let ips: Vec<Value> = lookup
                            .iter()
                            .map(|ip| Value::string(ip.to_string()))
                            .collect();
                        Ok(list_to_value(ips))
                    }
                    Err(e) => Err(Box::new(DiagnosticError::runtime_error(
                        format!("DNS lookup failed for '{hostname}': {e}"),
                        None,
                    ))),
                }
            }
            _ => Err(Box::new(DiagnosticError::runtime_error(
                format!("Unsupported DNS record type: {record_type}"),
                None,
            ))),
        }
    })
}

#[cfg(not(feature = "async"))]
pub fn primitive_resolve_hostname(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "resolve-hostname requires async feature".to_string(),
        None,
    )))
}

pub fn primitive_reverse_dns_lookup(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "reverse-dns-lookup not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_http_get(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "http-get not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_http_post(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "http-post not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_http_server(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "http-server not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_tls_connect(_args: &[Value]) -> Result<Value> {
    #[cfg(feature = "tls")]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "tls-connect not yet implemented".to_string(),
            None,
        )))
    }
    
    #[cfg(not(feature = "tls"))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "tls-connect requires TLS feature".to_string(),
            None,
        )))
    }
}

pub fn primitive_tls_listen(_args: &[Value]) -> Result<Value> {
    #[cfg(feature = "tls")]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "tls-listen not yet implemented".to_string(),
            None,
        )))
    }
    
    #[cfg(not(feature = "tls"))]
    {
        Err(Box::new(DiagnosticError::runtime_error(
            "tls-listen requires TLS feature".to_string(),
            None,
        )))
    }
}

pub fn primitive_parse_url(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("parse-url expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let url_str = extract_string(&args[0], "parse-url")?;
    
    match Url::parse(&url_str) {
        Ok(url) => {
            #[allow(clippy::mutable_key_type)]
            let mut result = HashMap::new();
            
            result.insert(
                Value::Symbol(crate::utils::intern_symbol("scheme")),
                Value::string(url.scheme().to_string())
            );
            
            if let Some(host) = url.host_str() {
                result.insert(
                    Value::Symbol(crate::utils::intern_symbol("host")),
                    Value::string(host.to_string())
                );
            }
            
            if let Some(port) = url.port() {
                result.insert(
                    Value::Symbol(crate::utils::intern_symbol("port")),
                    Value::integer(port as i64)
                );
            }
            
            result.insert(
                Value::Symbol(crate::utils::intern_symbol("path")),
                Value::string(url.path().to_string())
            );
            
            if let Some(query) = url.query() {
                result.insert(
                    Value::Symbol(crate::utils::intern_symbol("query")),
                    Value::string(query.to_string())
                );
            }
            
            if let Some(fragment) = url.fragment() {
                result.insert(
                    Value::Symbol(crate::utils::intern_symbol("fragment")),
                    Value::string(fragment.to_string())
                );
            }
            
            Ok(Value::Hashtable(Arc::new(std::sync::RwLock::new(result))))
        }
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("Invalid URL '{url_str}': {e}"),
            None,
        ))),
    }
}

pub fn primitive_format_url(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "format-url not yet implemented".to_string(),
        None,
    )))
}

pub fn primitive_network_interface_list(_args: &[Value]) -> Result<Value> {
    Err(Box::new(DiagnosticError::runtime_error(
        "network-interface-list not yet implemented".to_string(),
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
    use std::time::Duration;
    
    #[test]
    fn test_url_parsing() {
        let url_str = "https://example.com:8080/path?query=value#fragment";
        let args = vec![Value::string(url_str.to_string())];
        
        let result = primitive_parse_url(&args);
        assert!(result.is_ok());
        
        if let Ok(Value::Hashtable(hashtable)) = result {
            let table = hashtable.read().unwrap();
            assert!(table.contains_key(&Value::Symbol(crate::utils::intern_symbol("scheme"))));
            assert!(table.contains_key(&Value::Symbol(crate::utils::intern_symbol("host"))));
            assert!(table.contains_key(&Value::Symbol(crate::utils::intern_symbol("port"))));
        } else {
            panic!("Expected hashtable result");
        }
    }
    
    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_tcp_operations() {
        // Test TCP connect to a known service (this might fail in CI)
        let args = vec![
            Value::string("127.0.0.1".to_string()),
            Value::integer(22), // SSH port
            Value::integer(1000), // 1 second timeout
        ];
        
        // This test might fail if no SSH server is running
        // It's mainly to verify the function signature works
        let _result = primitive_tcp_connect(&args);
    }
}