//! Distributed processing foundation with RPC and serialization.
//!
//! This module provides the building blocks for distributed computing
//! including remote procedure calls, serialization, and network communication.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use super::ConcurrencyError;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Node identifier in a distributed system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// Creates a new unique node ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a node ID from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Gets the underlying UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node-{}", self.0)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Remote procedure call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Unique request ID
    pub id: String,
    /// Target service name
    pub service: String,
    /// Method to call
    pub method: String,
    /// Arguments
    pub args: Vec<SerializableValue>,
    /// Sender node ID
    pub sender: NodeId,
    /// Request timestamp
    pub timestamp: u64,
    /// Timeout in milliseconds
    pub timeout: Option<u64>,
}

/// Remote procedure call response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Request ID this response corresponds to
    pub request_id: String,
    /// Response result
    pub result: std::result::Result<SerializableValue, String>,
    /// Response timestamp
    pub timestamp: u64,
    /// Processing time in microseconds
    pub processing_time: Option<u64>,
}

/// Serializable version of Value for network transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableValue {
    /// Nil/empty value
    Nil,
    /// Boolean true/false
    Boolean(bool),
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit floating point number
    Float(f64),
    /// UTF-8 string
    String(String),
    /// Lisp symbol
    Symbol(String),
    /// Ordered list of values
    List(Vec<SerializableValue>),
    /// Indexed vector of values
    Vector(Vec<SerializableValue>),
    /// Key-value mapping
    Map(HashMap<String, SerializableValue>),
    /// Raw byte data
    Bytes(Vec<u8>),
}

impl SerializableValue {
    /// Converts a Value to SerializableValue.
    pub fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Nil => Ok(SerializableValue::Nil),
            Value::Literal(lit) => match lit {
                crate::ast::Literal::Boolean(b) => Ok(SerializableValue::Boolean(*b)),
                crate::ast::Literal::Number(n) if n.fract() == 0.0 => Ok(SerializableValue::Integer(*n as i64)),
                crate::ast::Literal::Number(f) => Ok(SerializableValue::Float(*f)),
                crate::ast::Literal::String(s) => Ok(SerializableValue::String(s.clone())),
                crate::ast::Literal::Character(c) => Ok(SerializableValue::String(c.to_string())),
                // Handle other literal types
                crate::ast::Literal::Rational { numerator, denominator } => 
                    Ok(SerializableValue::Float(*numerator as f64 / *denominator as f64)),
                crate::ast::Literal::Complex { real, imaginary: _ } => 
                    Ok(SerializableValue::Float(*real)), // Only serialize real part
                crate::ast::Literal::Bytevector(bytes) => 
                    Ok(SerializableValue::String(format!("bytevector-{}", bytes.len()))),
                crate::ast::Literal::Nil => Ok(SerializableValue::Nil),
                crate::ast::Literal::Unspecified => Ok(SerializableValue::String("unspecified".to_string())),
            },
            Value::Symbol(sym) => Ok(SerializableValue::Symbol(format!("symbol-{}", sym.0))),
            Value::Pair(_car, _cdr) => {
                // Convert pair to list
                let mut list = Vec::new();
                let mut current = value;
                
                loop {
                    match current {
                        Value::Pair(car, cdr) => {
                            list.push(Self::from_value(car)?);
                            current = cdr;
                        }
                        Value::Nil => break,
                        _ => {
                            // Improper list - add the final element
                            list.push(Self::from_value(current)?);
                            break;
                        }
                    }
                }
                
                Ok(SerializableValue::List(list))
            }
            Value::Vector(vec) => {
                let guard = vec.read().unwrap();
                let mut serializable_vec = Vec::new();
                for item in guard.iter() {
                    serializable_vec.push(Self::from_value(item)?);
                }
                Ok(SerializableValue::Vector(serializable_vec))
            }
            _ => Err(Box::new(Error::runtime_error(
                format!("Cannot serialize value type: {:?}", value),
                None,
            )),
        }
    }

    /// Converts SerializableValue to Value.
    pub fn to_value(&self) -> Result<Value> {
        match self {
            SerializableValue::Nil => Ok(Value::Nil),
            SerializableValue::Boolean(b) => Ok(Value::Literal(crate::ast::Literal::Boolean(*b))),
            SerializableValue::Integer(i) => Ok(Value::Literal(crate::ast::Literal::Number(*i as f64))),
            SerializableValue::Float(f) => Ok(Value::Literal(crate::ast::Literal::Number(*f))),
            SerializableValue::String(s) => Ok(Value::Literal(crate::ast::Literal::String(s.clone()))),
            SerializableValue::Symbol(s) => {
                // Extract symbol ID from the string (simplified)
                Ok(Value::Symbol(crate::utils::SymbolId(s.len() as usize)))
            }
            SerializableValue::List(list) => {
                let mut result = Value::Nil;
                for item in list.iter().rev() {
                    let value = item.to_value()?;
                    result = Value::pair(value, result);
                }
                Ok(result)
            }
            SerializableValue::Vector(vec) => {
                let mut values = Vec::new();
                for item in vec {
                    values.push(item.to_value()?);
                }
                Ok(Value::Vector(Arc::new(std::sync::RwLock::new(values))))
            }
            SerializableValue::Map(_map) => {
                // Convert to hash table or similar structure
                Ok(Value::Nil) // Placeholder
            }
            SerializableValue::Bytes(bytes) => {
                // Convert to bytevector
                Ok(Value::Literal(crate::ast::Literal::String(
                    String::from_utf8_lossy(bytes).to_string()
                )))
            }
        }
    }
}

/// RPC service trait.
#[async_trait::async_trait]
pub trait RpcService: Send + Sync + std::fmt::Debug {
    /// Handles an RPC request.
    async fn handle_request(&self, request: RpcRequest) -> RpcResponse;
    
    /// Gets the service name.
    fn service_name(&self) -> &str;
}

/// RPC client for making remote calls.
#[derive(Debug, Clone)]
pub struct RpcClient {
    node_id: NodeId,
    connections: Arc<Mutex<HashMap<NodeId, Arc<Connection>>>>,
}

impl RpcClient {
    /// Creates a new RPC client.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Connects to a remote node.
    pub async fn connect(&self, node_id: NodeId, addr: SocketAddr) -> Result<()> {
        let stream = TcpStream::connect(addr).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;
        
        let connection = Arc::new(Connection::new(stream));
        
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(node_id, connection);
        }
        
        Ok(())
    }

    /// Makes a remote procedure call.
    pub async fn call(
        &self,
        target_node: NodeId,
        service: String,
        method: String,
        args: Vec<Value>,
        timeout: Option<Duration>,
    ) -> Result<Value> {
        let connection = {
            let connections = self.connections.lock().unwrap();
            connections.get(&target_node)
                .ok_or_else(|| ConcurrencyError::Network("Node not connected".to_string()).boxed())?
                .clone())
        };

        let request_id = Uuid::new_v4().to_string();
        let serializable_args: Result<Vec<_>> = args.iter()
            .map(SerializableValue::from_value)
            .collect();

        let request = RpcRequest {
            id: request_id.clone()),
            service,
            method,
            args: serializable_args?,
            sender: self.node_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            timeout: timeout.map(|t| t.as_millis() as u64),
        };

        let response = connection.send_request(request, timeout.unwrap_or(Duration::from_secs(30))).await?;
        
        match response.result {
            Ok(value) => value.to_value(),
            Err(error) => Err(Box::new(Error::runtime_error(error, None).boxed()),
        }
    }
}

/// RPC server for handling remote calls.
#[derive(Debug)]
pub struct RpcServer {
    node_id: NodeId,
    listener: Option<TcpListener>,
    services: Arc<Mutex<HashMap<String, Arc<dyn RpcService>>>>,
    connections: Arc<Mutex<HashMap<NodeId, Arc<Connection>>>>,
}

impl RpcServer {
    /// Creates a new RPC server.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            listener: None,
            services: Arc::new(Mutex::new(HashMap::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Binds the server to an address.
    pub async fn bind(&mut self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;
        
        self.listener = Some(listener);
        Ok(())
    }

    /// Registers an RPC service.
    pub fn register_service(&self, service: Arc<dyn RpcService>) {
        let mut services = self.services.lock().unwrap();
        services.insert(service.service_name().to_string(), service);
    }

    /// Starts the server.
    pub async fn serve(&self) -> Result<()> {
        let listener = self.listener.as_ref()
            .ok_or_else(|| Error::runtime_error("Server not bound to address".to_string(), None))?;

        loop {
            let (stream, _addr) = listener.accept().await
                .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;

            let connection = Arc::new(Connection::new(stream));
            let services = self.services.clone());
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(connection, services).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }

    /// Handles a client connection.
    async fn handle_connection(
        connection: Arc<Connection>,
        services: Arc<Mutex<HashMap<String, Arc<dyn RpcService>>>>,
    ) -> Result<()> {
        loop {
            match connection.receive_request().await {
                Ok(request) => {
                    let services = services.clone());
                    let connection = connection.clone());
                    
                    tokio::spawn(async move {
                        let response = Self::process_request(request, services).await;
                        if let Err(e) = connection.send_response(response).await {
                            eprintln!("Failed to send response: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to receive request: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }

    /// Processes an RPC request.
    async fn process_request(
        request: RpcRequest,
        services: Arc<Mutex<HashMap<String, Arc<dyn RpcService>>>>,
    ) -> RpcResponse {
        let start_time = Instant::now();
        
        let service = {
            let services = services.lock().unwrap();
            services.get(&request.service).clone())()
        };

        let result = if let Some(service) = service {
            service.handle_request(request.clone()).await
        } else {
            RpcResponse {
                request_id: request.id,
                result: Err(format!("Service '{}' not found", request.service)),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                processing_time: Some(start_time.elapsed().as_micros() as u64),
            }
        };

        result
    }
}

/// Network connection wrapper.
struct Connection {
    stream: Arc<tokio::sync::Mutex<TcpStream>>,
    pending_requests: Arc<tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<RpcResponse>>>>,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("stream", &"<TcpStream>")
            .field("pending_requests", &"<PendingRequests>")
            .finish()
    }
}

impl Connection {
    /// Creates a new connection.
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
            pending_requests: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Sends an RPC request.
    async fn send_request(&self, request: RpcRequest, timeout: Duration) -> Result<RpcResponse> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request.id.clone()), tx);
        }

        // Serialize and send request
        let data = serde_json::to_vec(&request)
            .map_err(|e| ConcurrencyError::Serialization(e.to_string()).boxed())?;
        
        let mut stream = self.stream.lock().await;
        stream.write_u32(data.len() as u32).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;
        stream.write_all(&data).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;

        // Wait for response
        tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| ConcurrencyError::Timeout.boxed())?
            .map_err(|_| Error::runtime_error("Request cancelled".to_string(), None).boxed())
    }

    /// Receives an RPC request.
    async fn receive_request(&self) -> Result<RpcRequest> {
        let mut stream = self.stream.lock().await;
        
        let len = stream.read_u32().await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;
        
        let mut buffer = vec![0u8; len as usize];
        stream.read_exact(&mut buffer).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;

        serde_json::from_slice(&buffer)
            .map_err(|e| ConcurrencyError::Serialization(e.to_string()).boxed())
    }

    /// Sends an RPC response.
    async fn send_response(&self, response: RpcResponse) -> Result<()> {
        let data = serde_json::to_vec(&response)
            .map_err(|e| ConcurrencyError::Serialization(e.to_string()).boxed())?;
        
        let mut stream = self.stream.lock().await;
        stream.write_u32(data.len() as u32).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;
        stream.write_all(&data).await
            .map_err(|e| ConcurrencyError::Network(e.to_string()).boxed())?;

        Ok(())
    }
}

/// Example RPC service implementation.
#[derive(Debug)]
pub struct CalculatorService;

#[async_trait::async_trait]
impl RpcService for CalculatorService {
    async fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        let start_time = Instant::now();
        
        let result = match request.method.as_str() {
            "add" => {
                if request.args.len() != 2 {
                    Err("add requires exactly 2 arguments".to_string())
                } else {
                    match (&request.args[0], &request.args[1]) {
                        (SerializableValue::Integer(a), SerializableValue::Integer(b)) => {
                            Ok(SerializableValue::Integer(a + b))
                        }
                        (SerializableValue::Float(a), SerializableValue::Float(b)) => {
                            Ok(SerializableValue::Float(a + b))
                        }
                        _ => Err("add requires numeric arguments".to_string()),
                    }
                }
            }
            "multiply" => {
                if request.args.len() != 2 {
                    Err("multiply requires exactly 2 arguments".to_string())
                } else {
                    match (&request.args[0], &request.args[1]) {
                        (SerializableValue::Integer(a), SerializableValue::Integer(b)) => {
                            Ok(SerializableValue::Integer(a * b))
                        }
                        (SerializableValue::Float(a), SerializableValue::Float(b)) => {
                            Ok(SerializableValue::Float(a * b))
                        }
                        _ => Err("multiply requires numeric arguments".to_string()),
                    }
                }
            }
            _ => Err(format!("Unknown method: {}", request.method)),
        };

        RpcResponse {
            request_id: request.id,
            result,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            processing_time: Some(start_time.elapsed().as_micros() as u64),
        }
    }

    fn service_name(&self) -> &str {
        "calculator"
    }
}

/// Distributed node in the system.
#[derive(Debug)]
pub struct DistributedNode {
    id: NodeId,
    rpc_client: RpcClient,
    rpc_server: RpcServer,
}

impl DistributedNode {
    /// Creates a new distributed node.
    pub fn new() -> Self {
        let id = NodeId::new();
        Self {
            id,
            rpc_client: RpcClient::new(id),
            rpc_server: RpcServer::new(id),
        }
    }

    /// Gets the node ID.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Gets the RPC client.
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Gets the RPC server.
    pub fn rpc_server(&mut self) -> &mut RpcServer {
        &mut self.rpc_server
    }

    /// Starts the node on the given address.
    pub async fn start(&mut self, addr: SocketAddr) -> Result<()> {
        self.rpc_server.bind(addr).await?;
        
        // Note: Server would be started in background in a real implementation
        // For now, we just bind and return
        println!("RPC server bound to {}", addr);
        
        Ok(())
    }
}

impl Default for DistributedNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for distributed computing.
pub struct DistributedOps;

impl DistributedOps {
    /// Creates a distributed map operation across multiple nodes.
    pub async fn distributed_map<F>(
        nodes: Vec<NodeId>,
        client: &RpcClient,
        data: Vec<Value>,
        _map_fn: F,
    ) -> Result<Vec<Value>>
    where
        F: Fn(&Value) -> Result<Value> + Send + Sync + 'static,
    {
        if nodes.is_empty() {
            return Err(Box::new(Error::runtime_error("No nodes available".to_string(), None).into())
        }

        let chunk_size = (data.len() + nodes.len() - 1) / nodes.len();
        let mut futures = Vec::new();

        for (i, chunk) in data.chunks(chunk_size).enumerate() {
            let node_id = nodes[i % nodes.len()];
            let chunk_data = chunk.to_vec();
            
            // In a real implementation, you'd serialize the function and send it
            // For now, we'll assume a predefined map service exists on remote nodes
            let future = client.call(
                node_id,
                "distributed".to_string(),
                "map".to_string(),
                chunk_data,
                Some(Duration::from_secs(30)),
            );
            
            futures.push(future);
        }

        let mut results = Vec::new();
        for future in futures {
            let result = future.await?;
            // Assuming result is a list of mapped values
            if let Value::Pair(_, _) = result {
                // Convert list back to Vec<Value>
                let mut current = &result;
                loop {
                    match current {
                        Value::Pair(car, cdr) => {
                            results.push((**car).clone());
                            current = cdr;
                        }
                        Value::Nil => break,
                        _ => {
                            results.push(current.clone());
                            break;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Creates a distributed reduce operation across multiple nodes.
    pub async fn distributed_reduce(
        nodes: Vec<NodeId>,
        client: &RpcClient,
        data: Vec<Value>,
        identity: Value,
    ) -> Result<Value> {
        // First, perform local reductions on each node
        let partial_results = Self::distributed_map(
            nodes.clone()),
            client,
            data,
            |_| Ok(Value::Nil), // Placeholder
        ).await?;

        // Then, reduce the partial results locally or on a coordinator node
        let mut result = identity;
        for partial in partial_results {
            // Combine partial results (implementation depends on the reduction operation)
            result = partial; // Placeholder
        }

        Ok(result)
    }
}