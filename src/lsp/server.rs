//! Language Server Protocol server implementation for Lambdust
//!
//! This module implements the main LSP server that handles client communication
//! and coordinates various language features like completion, diagnostics, and hover.

use crate::error::{LambdustError, Result};
use crate::interpreter::LambdustInterpreter;
use crate::lsp::{
    LspConfig, LspCapabilities, LspError,
    CompletionProvider, DiagnosticsEngine, HoverProvider, SymbolProvider,
    DocumentManager, PositionUtils,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;

/// Main Language Server Protocol server for Lambdust
pub struct LambdustLanguageServer {
    /// Server configuration
    config: LspConfig,
    
    /// Server capabilities
    capabilities: LspCapabilities,
    
    /// Document manager for tracking open files
    document_manager: Arc<AsyncRwLock<DocumentManager>>,
    
    /// Code completion provider
    completion_provider: Arc<RwLock<CompletionProvider>>,
    
    /// Diagnostics engine
    diagnostics_engine: Arc<RwLock<DiagnosticsEngine>>,
    
    /// Hover information provider
    hover_provider: Arc<RwLock<HoverProvider>>,
    
    /// Symbol provider for navigation
    symbol_provider: Arc<RwLock<SymbolProvider>>,
    
    /// Scheme interpreter for evaluation
    interpreter: Arc<RwLock<LambdustInterpreter>>,
    
    /// Workspace information
    workspace_folders: Vec<PathBuf>,
    
    /// Client capabilities
    client_capabilities: Option<lsp_types::ClientCapabilities>,
    
    /// Server state
    server_state: ServerState,
}

/// Server state tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerState {
    /// Server is starting up
    Starting,
    /// Server has been initialized
    Initialized,
    /// Server is running normally
    Running,
    /// Server is shutting down
    ShuttingDown,
    /// Server has shut down
    Shutdown,
}

/// LSP server initialization parameters
#[derive(Debug, Clone)]
pub struct InitializeParams {
    /// Process ID of the client
    pub process_id: Option<u32>,
    
    /// Root path of the workspace (deprecated, use workspace_folders)
    pub root_path: Option<String>,
    
    /// Root URI of the workspace (deprecated, use workspace_folders)
    pub root_uri: Option<String>,
    
    /// Workspace folders
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
    
    /// Client capabilities
    pub capabilities: lsp_types::ClientCapabilities,
    
    /// Initialization options
    pub initialization_options: Option<serde_json::Value>,
    
    /// Trace setting
    pub trace: Option<String>,
}

/// Workspace folder information
#[derive(Debug, Clone)]
pub struct WorkspaceFolder {
    /// The URI of the workspace folder
    pub uri: String,
    /// The name of the workspace folder
    pub name: String,
}

impl LambdustLanguageServer {
    /// Create a new language server instance
    pub fn new(config: LspConfig) -> Result<Self> {
        let interpreter = LambdustInterpreter::new();
        
        Ok(Self {
            capabilities: LspCapabilities::default(),
            document_manager: Arc::new(AsyncRwLock::new(DocumentManager::new())),
            completion_provider: Arc::new(RwLock::new(CompletionProvider::new()?)),
            diagnostics_engine: Arc::new(RwLock::new(DiagnosticsEngine::new(interpreter.clone())?)),
            hover_provider: Arc::new(RwLock::new(HoverProvider::new(interpreter.clone())?)),
            symbol_provider: Arc::new(RwLock::new(SymbolProvider::new(interpreter.clone())?)),
            interpreter: Arc::new(RwLock::new(interpreter)),
            workspace_folders: Vec::new(),
            client_capabilities: None,
            server_state: ServerState::Starting,
            config,
        })
    }
    
    /// Initialize the server with client capabilities
    pub async fn initialize(&mut self, params: InitializeParams) -> Result<lsp_types::InitializeResult> {
        self.client_capabilities = Some(params.capabilities.clone());
        
        // Set up workspace folders
        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                if let Ok(path) = url::Url::parse(&folder.uri) {
                    if let Ok(path) = path.to_file_path() {
                        self.workspace_folders.push(path);
                    }
                }
            }
        } else if let Some(root_uri) = params.root_uri {
            if let Ok(url) = url::Url::parse(&root_uri) {
                if let Ok(path) = url.to_file_path() {
                    self.workspace_folders.push(path);
                }
            }
        }
        
        // Update server state
        self.server_state = ServerState::Initialized;
        
        // Build server capabilities based on client capabilities and config
        let server_capabilities = self.build_server_capabilities(&params.capabilities);
        
        Ok(lsp_types::InitializeResult {
            capabilities: server_capabilities,
            server_info: Some(lsp_types::ServerInfo {
                name: "Lambdust Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }
    
    /// Mark server as initialized and ready
    pub fn initialized(&mut self) -> Result<()> {
        self.server_state = ServerState::Running;
        Ok(())
    }
    
    /// Handle text document synchronization
    pub async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) -> Result<()> {
        let mut doc_manager = self.document_manager.write().await;
        doc_manager.did_open(params.text_document.clone()).await?;
        
        // Trigger diagnostics
        self.update_diagnostics(&params.text_document.uri).await?;
        
        Ok(())
    }
    
    /// Handle document changes
    pub async fn did_change(&self, params: lsp_types::DidChangeTextDocumentParams) -> Result<()> {
        let mut doc_manager = self.document_manager.write().await;
        doc_manager.did_change(params.text_document.clone(), params.content_changes).await?;
        
        // Trigger diagnostics
        self.update_diagnostics(&params.text_document.uri).await?;
        
        Ok(())
    }
    
    /// Handle document close
    pub async fn did_close(&self, params: lsp_types::DidCloseTextDocumentParams) -> Result<()> {
        let mut doc_manager = self.document_manager.write().await;
        doc_manager.did_close(&params.text_document.uri).await?;
        
        Ok(())
    }
    
    /// Handle completion requests
    pub async fn completion(
        &self,
        params: lsp_types::CompletionParams,
    ) -> Result<Option<lsp_types::CompletionResponse>> {
        let doc_manager = self.document_manager.read().await;
        let document = doc_manager.get_document(&params.text_document_position.text_document.uri)
            .ok_or_else(|| LspError::DocumentNotFound(params.text_document_position.text_document.uri.to_string()))?;
        
        let position = PositionUtils::lsp_position_to_position(&params.text_document_position.position);
        
        // Build completion context
        let context = self.build_completion_context(&document, position, params.context.as_ref())?;
        
        // Get completions
        let completion_provider = self.completion_provider.read()
            .map_err(|_| LspError::Internal("Failed to acquire completion provider lock".to_string()))?;
        let completions = completion_provider.get_completions(&context)?;
        
        // Convert to LSP format
        let lsp_completions: Vec<lsp_types::CompletionItem> = completions
            .into_iter()
            .map(|item| self.convert_completion_item(item))
            .collect();
        
        Ok(Some(lsp_types::CompletionResponse::Array(lsp_completions)))
    }
    
    /// Handle hover requests
    pub async fn hover(
        &self,
        params: lsp_types::HoverParams,
    ) -> Result<Option<lsp_types::Hover>> {
        let doc_manager = self.document_manager.read().await;
        let document = doc_manager.get_document(&params.text_document_position_params.text_document.uri)
            .ok_or_else(|| LspError::DocumentNotFound(params.text_document_position_params.text_document.uri.to_string()))?;
        
        let position = PositionUtils::lsp_position_to_position(&params.text_document_position_params.position);
        
        let hover_provider = self.hover_provider.read()
            .map_err(|_| LspError::Internal("Failed to acquire hover provider lock".to_string()))?;
        
        if let Some(hover_info) = hover_provider.get_hover_info(&document, position)? {
            Ok(Some(lsp_types::Hover {
                contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: hover_info.content,
                }),
                range: hover_info.range.map(PositionUtils::range_to_lsp_range),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Handle shutdown request
    pub fn shutdown(&mut self) -> Result<()> {
        self.server_state = ServerState::ShuttingDown;
        Ok(())
    }
    
    /// Handle exit notification
    pub fn exit(&mut self) {
        self.server_state = ServerState::Shutdown;
    }
    
    /// Get current server state
    pub fn state(&self) -> ServerState {
        self.server_state.clone()
    }
    
    /// Build server capabilities based on client capabilities
    fn build_server_capabilities(
        &self,
        client_caps: &lsp_types::ClientCapabilities,
    ) -> lsp_types::ServerCapabilities {
        let mut capabilities = lsp_types::ServerCapabilities::default();
        
        // Text document sync
        if self.capabilities.text_document_sync {
            capabilities.text_document_sync = Some(lsp_types::TextDocumentSyncCapability::Kind(
                lsp_types::TextDocumentSyncKind::INCREMENTAL,
            ));
        }
        
        // Completion
        if self.capabilities.completion {
            capabilities.completion_provider = Some(lsp_types::CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(self.config.completion_triggers.clone()),
                all_commit_characters: None,
                work_done_progress_options: lsp_types::WorkDoneProgressOptions::default(),
                completion_item: None,
            });
        }
        
        // Hover
        if self.capabilities.hover {
            capabilities.hover_provider = Some(lsp_types::HoverProviderCapability::Simple(true));
        }
        
        // Definition provider
        if self.capabilities.definition {
            capabilities.definition_provider = Some(lsp_types::OneOf::Left(true));
        }
        
        // References provider
        if self.capabilities.references {
            capabilities.references_provider = Some(lsp_types::OneOf::Left(true));
        }
        
        // Document symbol provider
        if self.capabilities.symbol_navigation {
            capabilities.document_symbol_provider = Some(lsp_types::OneOf::Left(true));
        }
        
        capabilities
    }
    
    /// Update diagnostics for a document
    async fn update_diagnostics(&self, uri: &lsp_types::Url) -> Result<()> {
        let doc_manager = self.document_manager.read().await;
        let document = doc_manager.get_document(uri)
            .ok_or_else(|| LspError::DocumentNotFound(uri.to_string()))?;
        
        let diagnostics_engine = self.diagnostics_engine.read()
            .map_err(|_| LspError::Internal("Failed to acquire diagnostics engine lock".to_string()))?;
        
        let diagnostics = diagnostics_engine.analyze_document(&document)?;
        
        // TODO: Send diagnostics to client
        // This would typically use the LSP client connection to send diagnostics
        
        Ok(())
    }
    
    /// Build completion context from document and position
    fn build_completion_context(
        &self,
        document: &crate::lsp::document::Document,
        position: crate::lsp::position::Position,
        lsp_context: Option<&lsp_types::CompletionContext>,
    ) -> Result<crate::lsp::completion::CompletionContext> {
        use crate::lsp::completion::{CompletionContext, ExpressionContext};
        
        let line_content = document.get_line(position.line as usize)
            .unwrap_or_default();
        
        let prefix = if position.character as usize <= line_content.len() {
            line_content[..position.character as usize].to_string()
        } else {
            line_content.clone()
        };
        
        // Analyze expression context
        let expression_context = self.analyze_expression_context(&line_content, position.character);
        
        Ok(CompletionContext {
            position,
            trigger_character: lsp_context
                .and_then(|ctx| ctx.trigger_character)
                .and_then(|s| s.chars().next()),
            is_retrigger: lsp_context
                .map(|ctx| ctx.trigger_kind == lsp_types::CompletionTriggerKind::TRIGGER_FOR_INCOMPLETE_COMPLETIONS)
                .unwrap_or(false),
            line_content,
            prefix,
            expression_context,
            scope_bindings: Vec::new(), // TODO: Extract from document analysis
        })
    }
    
    /// Analyze expression context at position
    fn analyze_expression_context(&self, line: &str, character: u32) -> crate::lsp::completion::ExpressionContext {
        use crate::lsp::completion::ExpressionContext;
        
        // Simple context analysis - could be much more sophisticated
        let prefix = &line[..character.min(line.len() as u32) as usize];
        
        if prefix.trim_end().ends_with('(') {
            ExpressionContext::ExpressionStart
        } else if prefix.contains('(') && !prefix.contains(')') {
            ExpressionContext::FunctionPosition
        } else {
            ExpressionContext::TopLevel
        }
    }
    
    /// Convert internal completion item to LSP completion item
    fn convert_completion_item(&self, item: crate::lsp::completion::CompletionItem) -> lsp_types::CompletionItem {
        use crate::lsp::completion::CompletionItemKind;
        
        let kind = match item.kind {
            CompletionItemKind::Function => lsp_types::CompletionItemKind::FUNCTION,
            CompletionItemKind::Keyword => lsp_types::CompletionItemKind::KEYWORD,
            CompletionItemKind::Method => lsp_types::CompletionItemKind::METHOD,
            CompletionItemKind::Variable => lsp_types::CompletionItemKind::VARIABLE,
            CompletionItemKind::Constant => lsp_types::CompletionItemKind::CONSTANT,
            CompletionItemKind::Macro => lsp_types::CompletionItemKind::FUNCTION,
            CompletionItemKind::Module => lsp_types::CompletionItemKind::MODULE,
            CompletionItemKind::Class => lsp_types::CompletionItemKind::CLASS,
            CompletionItemKind::File => lsp_types::CompletionItemKind::FILE,
            CompletionItemKind::Snippet => lsp_types::CompletionItemKind::SNIPPET,
        };
        
        lsp_types::CompletionItem {
            label: item.label,
            kind: Some(kind),
            detail: item.detail,
            documentation: item.documentation.map(|doc| {
                lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: doc,
                })
            }),
            insert_text: item.insert_text,
            insert_text_format: if item.kind == CompletionItemKind::Snippet {
                Some(lsp_types::InsertTextFormat::SNIPPET)
            } else {
                Some(lsp_types::InsertTextFormat::PLAIN_TEXT)
            },
            sort_text: Some(format!("{:04}{}", item.sort_priority, item.label)),
            preselect: Some(item.preselect),
            ..Default::default()
        }
    }
}
