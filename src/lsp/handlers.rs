//! LSP request handlers and utilities
//!
//! This module contains handlers for various LSP requests and provides
//! utilities for processing and routing LSP messages.

use crate::error::{LambdustError, Result};
use crate::lsp::{
    LambdustLanguageServer, DiagnosticsEngine, CompletionProvider,
    HoverProvider, SymbolProvider, DocumentManager, LspError,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler context for LSP requests
pub struct HandlerContext {
    /// Language server instance
    pub server: Arc<RwLock<LambdustLanguageServer>>,
}

/// Request handler trait
pub trait RequestHandler<Params, Response> {
    /// Handle the request
    fn handle(&self, params: Params, context: &HandlerContext) -> Result<Response>;
}

/// Notification handler trait
pub trait NotificationHandler<Params> {
    /// Handle the notification
    fn handle(&self, params: Params, context: &HandlerContext) -> Result<()>;
}

/// Initialize request handler
pub struct InitializeHandler;

impl RequestHandler<lsp_types::InitializeParams, lsp_types::InitializeResult> for InitializeHandler {
    fn handle(&self, params: lsp_types::InitializeParams, context: &HandlerContext) -> Result<lsp_types::InitializeResult> {
        // Convert LSP params to internal params
        let internal_params = crate::lsp::server::InitializeParams {
            process_id: params.process_id,
            root_path: params.root_path,
            root_uri: params.root_uri.map(|uri| uri.to_string()),
            workspace_folders: params.workspace_folders.map(|folders| {
                folders.into_iter().map(|folder| {
                    crate::lsp::server::WorkspaceFolder {
                        uri: folder.uri.to_string(),
                        name: folder.name,
                    }
                }).collect()
            }),
            capabilities: params.capabilities,
            initialization_options: params.initialization_options,
            trace: params.trace.map(|t| t.to_string()),
        };

        // This would need to be async in a real implementation
        // For now, we'll simulate the async call
        tokio::runtime::Handle::current().block_on(async {
            let mut server = context.server.write().await;
            server.initialize(internal_params).await
        })
    }
}

/// Initialized notification handler
pub struct InitializedHandler;

impl NotificationHandler<lsp_types::InitializedParams> for InitializedHandler {
    fn handle(&self, _params: lsp_types::InitializedParams, context: &HandlerContext) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let mut server = context.server.write().await;
            server.initialized()
        })
    }
}

/// Text document did open notification handler
pub struct DidOpenHandler;

impl NotificationHandler<lsp_types::DidOpenTextDocumentParams> for DidOpenHandler {
    fn handle(&self, params: lsp_types::DidOpenTextDocumentParams, context: &HandlerContext) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let server = context.server.read().await;
            server.did_open(params).await
        })
    }
}

/// Text document did change notification handler
pub struct DidChangeHandler;

impl NotificationHandler<lsp_types::DidChangeTextDocumentParams> for DidChangeHandler {
    fn handle(&self, params: lsp_types::DidChangeTextDocumentParams, context: &HandlerContext) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let server = context.server.read().await;
            server.did_change(params).await
        })
    }
}

/// Text document did close notification handler
pub struct DidCloseHandler;

impl NotificationHandler<lsp_types::DidCloseTextDocumentParams> for DidCloseHandler {
    fn handle(&self, params: lsp_types::DidCloseTextDocumentParams, context: &HandlerContext) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let server = context.server.read().await;
            server.did_close(params).await
        })
    }
}

/// Completion request handler
pub struct CompletionHandler;

impl RequestHandler<lsp_types::CompletionParams, Option<lsp_types::CompletionResponse>> for CompletionHandler {
    fn handle(&self, params: lsp_types::CompletionParams, context: &HandlerContext) -> Result<Option<lsp_types::CompletionResponse>> {
        tokio::runtime::Handle::current().block_on(async {
            let server = context.server.read().await;
            server.completion(params).await
        })
    }
}

/// Hover request handler
pub struct HoverHandler;

impl RequestHandler<lsp_types::HoverParams, Option<lsp_types::Hover>> for HoverHandler {
    fn handle(&self, params: lsp_types::HoverParams, context: &HandlerContext) -> Result<Option<lsp_types::Hover>> {
        tokio::runtime::Handle::current().block_on(async {
            let server = context.server.read().await;
            server.hover(params).await
        })
    }
}

/// Document symbols request handler
pub struct DocumentSymbolHandler;

impl RequestHandler<lsp_types::DocumentSymbolParams, Option<lsp_types::DocumentSymbolResponse>> for DocumentSymbolHandler {
    fn handle(&self, params: lsp_types::DocumentSymbolParams, context: &HandlerContext) -> Result<Option<lsp_types::DocumentSymbolResponse>> {
        // TODO: Implement document symbol handling
        Ok(None)
    }
}

/// Workspace symbols request handler
pub struct WorkspaceSymbolHandler;

impl RequestHandler<lsp_types::WorkspaceSymbolParams, Option<Vec<lsp_types::SymbolInformation>>> for WorkspaceSymbolHandler {
    fn handle(&self, params: lsp_types::WorkspaceSymbolParams, context: &HandlerContext) -> Result<Option<Vec<lsp_types::SymbolInformation>>> {
        // TODO: Implement workspace symbol handling
        Ok(None)
    }
}

/// Definition request handler
pub struct DefinitionHandler;

impl RequestHandler<lsp_types::GotoDefinitionParams, Option<lsp_types::GotoDefinitionResponse>> for DefinitionHandler {
    fn handle(&self, params: lsp_types::GotoDefinitionParams, context: &HandlerContext) -> Result<Option<lsp_types::GotoDefinitionResponse>> {
        // TODO: Implement definition handling
        Ok(None)
    }
}

/// References request handler
pub struct ReferencesHandler;

impl RequestHandler<lsp_types::ReferenceParams, Option<Vec<lsp_types::Location>>> for ReferencesHandler {
    fn handle(&self, params: lsp_types::ReferenceParams, context: &HandlerContext) -> Result<Option<Vec<lsp_types::Location>>> {
        // TODO: Implement references handling
        Ok(None)
    }
}

/// Shutdown request handler
pub struct ShutdownHandler;

impl RequestHandler<(), ()> for ShutdownHandler {
    fn handle(&self, _params: (), context: &HandlerContext) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let mut server = context.server.write().await;
            server.shutdown()
        })
    }
}

/// Exit notification handler
pub struct ExitHandler;

impl NotificationHandler<()> for ExitHandler {
    fn handle(&self, _params: (), context: &HandlerContext) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let mut server = context.server.write().await;
            server.exit();
        });
        Ok(())
    }
}

/// LSP message router
pub struct MessageRouter {
    /// Handler context
    context: HandlerContext,
    
    /// Request handlers
    request_handlers: RequestHandlers,
    
    /// Notification handlers
    notification_handlers: NotificationHandlers,
}

/// Collection of request handlers
pub struct RequestHandlers {
    pub initialize: InitializeHandler,
    pub completion: CompletionHandler,
    pub hover: HoverHandler,
    pub document_symbol: DocumentSymbolHandler,
    pub workspace_symbol: WorkspaceSymbolHandler,
    pub definition: DefinitionHandler,
    pub references: ReferencesHandler,
    pub shutdown: ShutdownHandler,
}

/// Collection of notification handlers
pub struct NotificationHandlers {
    pub initialized: InitializedHandler,
    pub did_open: DidOpenHandler,
    pub did_change: DidChangeHandler,
    pub did_close: DidCloseHandler,
    pub exit: ExitHandler,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(server: Arc<RwLock<LambdustLanguageServer>>) -> Self {
        Self {
            context: HandlerContext { server },
            request_handlers: RequestHandlers {
                initialize: InitializeHandler,
                completion: CompletionHandler,
                hover: HoverHandler,
                document_symbol: DocumentSymbolHandler,
                workspace_symbol: WorkspaceSymbolHandler,
                definition: DefinitionHandler,
                references: ReferencesHandler,
                shutdown: ShutdownHandler,
            },
            notification_handlers: NotificationHandlers {
                initialized: InitializedHandler,
                did_open: DidOpenHandler,
                did_change: DidChangeHandler,
                did_close: DidCloseHandler,
                exit: ExitHandler,
            },
        }
    }
    
    /// Route an LSP request
    pub fn route_request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        match method {
            "initialize" => {
                let params: lsp_types::InitializeParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid initialize params: {}", e)))?;
                let result = self.request_handlers.initialize.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "textDocument/completion" => {
                let params: lsp_types::CompletionParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid completion params: {}", e)))?;
                let result = self.request_handlers.completion.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "textDocument/hover" => {
                let params: lsp_types::HoverParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid hover params: {}", e)))?;
                let result = self.request_handlers.hover.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "textDocument/documentSymbol" => {
                let params: lsp_types::DocumentSymbolParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid document symbol params: {}", e)))?;
                let result = self.request_handlers.document_symbol.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "workspace/symbol" => {
                let params: lsp_types::WorkspaceSymbolParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid workspace symbol params: {}", e)))?;
                let result = self.request_handlers.workspace_symbol.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "textDocument/definition" => {
                let params: lsp_types::GotoDefinitionParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid definition params: {}", e)))?;
                let result = self.request_handlers.definition.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "textDocument/references" => {
                let params: lsp_types::ReferenceParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid references params: {}", e)))?;
                let result = self.request_handlers.references.handle(params, &self.context)?;
                Ok(serde_json::to_value(result)
                    .map_err(|e| LspError::Protocol(format!("Failed to serialize response: {}", e)))?)
            },
            
            "shutdown" => {
                self.request_handlers.shutdown.handle((), &self.context)?;
                Ok(serde_json::Value::Null)
            },
            
            _ => Err(LspError::Protocol(format!("Unknown request method: {}", method)).into()),
        }
    }
    
    /// Route an LSP notification
    pub fn route_notification(&self, method: &str, params: Option<serde_json::Value>) -> Result<()> {
        let params = params.unwrap_or(serde_json::Value::Null);
        
        match method {
            "initialized" => {
                let params: lsp_types::InitializedParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid initialized params: {}", e)))?;
                self.notification_handlers.initialized.handle(params, &self.context)
            },
            
            "textDocument/didOpen" => {
                let params: lsp_types::DidOpenTextDocumentParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid didOpen params: {}", e)))?;
                self.notification_handlers.did_open.handle(params, &self.context)
            },
            
            "textDocument/didChange" => {
                let params: lsp_types::DidChangeTextDocumentParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid didChange params: {}", e)))?;
                self.notification_handlers.did_change.handle(params, &self.context)
            },
            
            "textDocument/didClose" => {
                let params: lsp_types::DidCloseTextDocumentParams = serde_json::from_value(params)
                    .map_err(|e| LspError::Protocol(format!("Invalid didClose params: {}", e)))?;
                self.notification_handlers.did_close.handle(params, &self.context)
            },
            
            "exit" => {
                self.notification_handlers.exit.handle((), &self.context)
            },
            
            _ => {
                // Unknown notifications are ignored according to LSP spec
                Ok(())
            }
        }
    }
    
    /// Get handler context
    pub fn context(&self) -> &HandlerContext {
        &self.context
    }
}

/// Utility functions for handlers
pub mod utils {
    use super::*;
    use crate::lsp::position::{Position, Range, PositionUtils};
    
    /// Convert LSP position to internal position
    pub fn convert_position(lsp_pos: &lsp_types::Position) -> Position {
        PositionUtils::lsp_position_to_position(lsp_pos)
    }
    
    /// Convert internal position to LSP position
    pub fn convert_position_to_lsp(pos: Position) -> lsp_types::Position {
        PositionUtils::position_to_lsp_position(pos)
    }
    
    /// Convert LSP range to internal range
    pub fn convert_range(lsp_range: &lsp_types::Range) -> Range {
        PositionUtils::lsp_range_to_range(lsp_range)
    }
    
    /// Convert internal range to LSP range
    pub fn convert_range_to_lsp(range: Range) -> lsp_types::Range {
        PositionUtils::range_to_lsp_range(range)
    }
    
    /// Extract text from document at range
    pub fn extract_text_at_range(
        document: &crate::lsp::document::Document,
        range: Range,
    ) -> Result<String> {
        document.get_text_range(range)
    }
    
    /// Get word at position
    pub fn get_word_at_position(
        document: &crate::lsp::document::Document,
        position: Position,
    ) -> Result<Option<(String, Range)>> {
        let line_content = document.get_line(position.line as usize)
            .ok_or_else(|| LambdustError::runtime_error("Invalid line"))?;
        
        let char_pos = position.character as usize;
        if char_pos > line_content.len() {
            return Ok(None);
        }
        
        let chars: Vec<char> = line_content.chars().collect();
        
        // Find word boundaries
        let mut start = char_pos;
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }
        
        let mut end = char_pos;
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }
        
        if start < end {
            let word: String = chars[start..end].iter().collect();
            let range = Range::new(
                Position::new(position.line, start as u32),
                Position::new(position.line, end as u32),
            );
            Ok(Some((word, range)))
        } else {
            Ok(None)
        }
    }
    
    /// Check if character is part of a word
    fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '?' || ch == '!'
    }
    
    /// Create error response
    pub fn create_error_response(code: i32, message: String) -> lsp_types::ResponseError {
        lsp_types::ResponseError {
            code,
            message,
            data: None,
        }
    }
    
    /// Validate document URI
    pub fn validate_uri(uri: &lsp_types::Url) -> Result<()> {
        if uri.scheme() != "file" {
            return Err(LspError::Protocol("Only file URIs are supported".to_string()).into());
        }
        Ok(())
    }
}
