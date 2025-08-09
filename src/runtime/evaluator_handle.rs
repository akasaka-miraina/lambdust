use super::EvaluatorMessage;
use crate::ast::Expr;
use crate::diagnostics::{Result, Span};
use crate::eval::Value;
use crate::module_system::ImportSpec;
use crossbeam::channel::{self, Sender};
use std::collections::HashMap;
use std::thread::ThreadId;

/// Type alias for large error type
type SendError = Box<crossbeam::channel::SendError<EvaluatorMessage>>;

/// Handle to a spawned evaluator.
///
/// This provides a communication channel to send evaluation requests
/// to a specific evaluator thread.
#[derive(Debug)]
pub struct EvaluatorHandle {
    /// Thread ID of the evaluator
    pub thread_id: ThreadId,
    /// Channel for sending evaluation messages
    pub sender: Sender<EvaluatorMessage>,
    /// Unique handle ID
    pub id: u64,
}

impl EvaluatorHandle {
    /// Creates a new evaluator handle.
    pub fn new(thread_id: ThreadId, sender: Sender<EvaluatorMessage>, id: u64) -> Self {
        Self {
            thread_id,
            sender,
            id,
        }
    }
    
    /// Gets the thread ID.
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }
    
    /// Gets the handle ID.
    pub fn id(&self) -> u64 {
        self.id
    }
    
    /// Sends a message to the evaluator.
    pub fn send(&self, message: EvaluatorMessage) -> std::result::Result<(), SendError> {
        self.sender.send(message).map_err(Box::new)
    }

    /// Sends an evaluation request to this evaluator.
    pub async fn eval(&self, expr: Expr, span: Option<Span>) -> Result<Value> {
        let (sender, receiver) = channel::bounded(1);
        
        let message = EvaluatorMessage::Evaluate {
            expr,
            span,
            sender,
        };
        
        self.sender.send(message).map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to send evaluation message: {e}"),
                span,
            )
        })?;
        
        receiver.recv().map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to receive evaluation result: {e}"),
                span,
            )
        })?
    }

    /// Defines a global variable on this evaluator.
    pub fn define_global(&self, name: String, value: Value) -> Result<()> {
        let message = EvaluatorMessage::DefineGlobal { name, value };
        
        self.sender.send(message).map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to send define message: {e}"),
                None,
            ).boxed()
        })
    }

    /// Imports a module on this evaluator.
    pub async fn import_module(&self, import_spec: ImportSpec) -> Result<HashMap<String, Value>> {
        let (sender, receiver) = channel::bounded(1);
        
        let message = EvaluatorMessage::ImportModule {
            import_spec,
            sender,
        };
        
        self.sender.send(message).map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to send import message: {e}"),
                None,
            )
        })?;
        
        receiver.recv().map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to receive import result: {e}"),
                None,
            )
        })?
    }

    /// Shuts down this evaluator.
    pub fn shutdown(&self) -> Result<()> {
        self.sender.send(EvaluatorMessage::Shutdown).map_err(|e| {
            crate::diagnostics::Error::runtime_error(
                format!("Failed to send shutdown message: {e}"),
                None,
            ).boxed()
        })
    }
}