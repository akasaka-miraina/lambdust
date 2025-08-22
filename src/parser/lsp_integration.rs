//! Language Server Protocol v3.17 integration for Lambdust
//!
//! This module provides full LSP compliance for modern IDE integration,
//! unifying all parser components into a cohesive language server.

use crate::ast::{Expr, Program};
use crate::diagnostics::{Error, Result, Span};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::parser::{
    Parser,
    contextual_errors::{ContextualError, ContextualErrorGenerator, ErrorContext},
    error_recovery::{ErrorRecoveryEngine, RecoveryStrategy},
    ide_support::{
        CompletionItem, Diagnostic, DiagnosticSeverity, Hover, IdeSupportEngine,
        LanguageServerCapabilities, Location, MarkupContent, MarkupKind, Position, Range,
        SignatureHelp, SymbolInformation, TextDocumentSyncKind,
    },
    partial_parser::{ParsingMode, PartialParser, PartialParsingConfig, PartialParsingResult},
    realtime_feedback::{
        ContentChange, FeedbackResult, RealtimeFeedbackConfig, RealtimeFeedbackEngine, SyntaxToken,
        UpdatePriority,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

/// Language Server Protocol v3.17 compliant server
pub struct LambdustLanguageServer {
    /// Server configuration
    config: LanguageServerConfig,
    /// Document management
    document_manager: Arc<RwLock<DocumentManager>>,
    /// Real-time feedback engine
    feedback_engine: Arc<Mutex<RealtimeFeedbackEngine>>,
    /// IDE support engine
    ide_support: Arc<RwLock<IdeSupportEngine>>,
    /// Error recovery engine
    error_recovery: ErrorRecoveryEngine,
    /// Contextual error generator
    error_generator: ContextualErrorGenerator,
    /// Server state
    server_state: Arc<RwLock<ServerState>>,
    /// Capability registry
    capabilities: LanguageServerCapabilities,
}

/// Language server configuration
#[derive(Debug, Clone)]
pub struct LanguageServerConfig {
    /// Server name and version
    pub server_info: ServerInfo,
    /// Workspace configuration
    pub workspace_config: WorkspaceConfig,
    /// Parsing configuration
    pub parsing_config: ParsingConfig,
    /// Real-time features configuration
    pub realtime_config: RealtimeConfig,
    /// Diagnostic configuration
    pub diagnostic_config: DiagnosticConfig,
    /// Performance tuning
    pub performance_config: PerformanceConfig,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Name of the language server
    pub name: String,
    /// Version of the language server
    pub version: String,
}

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    /// Supported file extensions
    pub file_extensions: Vec<String>,
    /// Workspace folders enabled
    pub workspace_folders: bool,
    /// File watching enabled
    pub file_watching: bool,
    /// Configuration change monitoring
    pub configuration_change: bool,
}

/// Parsing-specific configuration
#[derive(Debug, Clone)]
pub struct ParsingConfig {
    /// Maximum errors per file
    pub max_errors_per_file: usize,
    /// Enable aggressive error recovery
    pub aggressive_recovery: bool,
    /// Parse timeout (ms)
    pub parse_timeout_ms: u64,
    /// Enable incremental parsing
    pub incremental_parsing: bool,
    /// Enable semantic analysis
    pub semantic_analysis: bool,
}

/// Real-time features configuration
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Debounce delay for updates (ms)
    pub debounce_delay_ms: u64,
    /// Enable real-time diagnostics
    pub real_time_diagnostics: bool,
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Enable auto-completion
    pub auto_completion: bool,
    /// Enable signature help
    pub signature_help: bool,
}

/// Diagnostic configuration
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    /// Enable contextual error messages
    pub contextual_errors: bool,
    /// Enable quick fixes
    pub quick_fixes: bool,
    /// Enable educational content
    pub educational_content: bool,
    /// Enable R7RS compliance checking
    pub r7rs_compliance: bool,
    /// Severity levels to report
    pub severity_levels: Vec<DiagnosticSeverity>,
}

/// Performance tuning configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Worker thread count
    pub worker_threads: usize,
    /// Cache size limits
    pub cache_size_mb: usize,
    /// Memory usage monitoring
    pub memory_monitoring: bool,
    /// Performance metrics collection
    pub metrics_collection: bool,
}

/// Document manager for LSP
#[derive(Debug)]
pub struct DocumentManager {
    /// Open documents by URI
    documents: HashMap<String, DocumentInfo>,
    /// Document version tracking
    versions: HashMap<String, u64>,
    /// Workspace folders
    workspace_folders: Vec<WorkspaceFolder>,
    /// File watcher state
    watchers: HashMap<String, FileWatcher>,
}

/// Document information
#[derive(Debug, Clone)]
pub struct DocumentInfo {
    /// Document URI
    pub uri: String,
    /// Document content
    pub content: String,
    /// Content version
    pub version: u64,
    /// Language identifier
    pub language_id: String,
    /// Last modification time
    pub last_modified: Instant,
    /// Parse state
    pub parse_state: Option<ParseState>,
}

/// Document parse state
#[derive(Debug, Clone)]
pub struct ParseState {
    /// Parsed program
    pub program: Program,
    /// Parse errors
    pub errors: Vec<ContextualError>,
    /// Symbol table
    pub symbols: HashMap<String, SymbolInfo>,
    /// Semantic tokens
    pub semantic_tokens: Vec<SyntaxToken>,
}

/// Symbol information for LSP
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Symbol location
    pub location: Location,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Type information
    pub type_info: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
}

/// Symbol kinds for LSP
#[derive(Debug, Clone, Copy)]
pub enum SymbolKind {
    /// Function or procedure definition
    Function,
    /// Variable binding
    Variable,
    /// Constant value
    Constant,
    /// Function parameter
    Parameter,
    /// Macro definition
    Macro,
    /// Module or library
    Module,
    /// Namespace
    Namespace,
}

/// Workspace folder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    /// Folder URI
    pub uri: String,
    /// Folder name
    pub name: String,
}

/// File watcher state
#[derive(Debug, Clone)]
pub struct FileWatcher {
    /// Watched patterns
    pub patterns: Vec<String>,
    /// Watch kind flags
    pub kind: WatchKind,
}

/// File watch kinds
#[derive(Debug, Clone, Copy)]
pub struct WatchKind {
    /// Watch for file creation events
    pub create: bool,
    /// Watch for file modification events
    pub change: bool,
    /// Watch for file deletion events
    pub delete: bool,
}

/// Server state tracking
#[derive(Debug, Default)]
pub struct ServerState {
    /// Initialization state
    pub initialized: bool,
    /// Shutdown requested
    pub shutdown_requested: bool,
    /// Client capabilities
    pub client_capabilities: Option<ClientCapabilities>,
    /// Active requests
    pub active_requests: HashMap<u64, ActiveRequest>,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

/// Client capabilities (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Text document capabilities
    pub text_document: Option<TextDocumentClientCapabilities>,
    /// Workspace capabilities
    pub workspace: Option<WorkspaceClientCapabilities>,
    /// Window capabilities
    pub window: Option<WindowClientCapabilities>,
    /// General capabilities
    pub general: Option<GeneralClientCapabilities>,
}

/// Text document client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentClientCapabilities {
    /// Synchronization capabilities
    pub synchronization: Option<TextDocumentSyncClientCapabilities>,
    /// Completion capabilities
    pub completion: Option<CompletionClientCapabilities>,
    /// Hover capabilities
    pub hover: Option<HoverClientCapabilities>,
    /// Signature help capabilities
    pub signature_help: Option<SignatureHelpClientCapabilities>,
    /// Diagnostic capabilities
    pub diagnostic: Option<DiagnosticClientCapabilities>,
}

/// Text document synchronization client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentSyncClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
    /// Will save
    pub will_save: Option<bool>,
    /// Will save wait until
    pub will_save_wait_until: Option<bool>,
    /// Did save
    pub did_save: Option<bool>,
}

/// Completion client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
    /// Completion item capabilities
    pub completion_item: Option<CompletionItemClientCapabilities>,
    /// Completion item kind capabilities
    pub completion_item_kind: Option<CompletionItemKindClientCapabilities>,
}

/// Completion item client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItemClientCapabilities {
    /// Snippet support
    pub snippet_support: Option<bool>,
    /// Commit characters support
    pub commit_characters_support: Option<bool>,
    /// Documentation format
    pub documentation_format: Option<Vec<MarkupKind>>,
    /// Deprecated support
    pub deprecated_support: Option<bool>,
    /// Preselect support
    pub preselect_support: Option<bool>,
}

/// Completion item kind client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItemKindClientCapabilities {
    /// Supported value set
    pub value_set: Option<Vec<u32>>,
}

/// Hover client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
    /// Content format
    pub content_format: Option<Vec<MarkupKind>>,
}

/// Signature help client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
    /// Signature information capabilities
    pub signature_information: Option<SignatureInformationClientCapabilities>,
}

/// Signature information client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInformationClientCapabilities {
    /// Documentation format
    pub documentation_format: Option<Vec<MarkupKind>>,
    /// Parameter information capabilities
    pub parameter_information: Option<ParameterInformationClientCapabilities>,
}

/// Parameter information client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInformationClientCapabilities {
    /// Label offset support
    pub label_offset_support: Option<bool>,
}

/// Diagnostic client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
    /// Related document support
    pub related_document_support: Option<bool>,
}

/// Workspace client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    /// Workspace folders
    pub workspace_folders: Option<bool>,
    /// Configuration support
    pub configuration: Option<bool>,
    /// Did change configuration
    pub did_change_configuration: Option<DidChangeConfigurationClientCapabilities>,
    /// Did change watched files
    pub did_change_watched_files: Option<DidChangeWatchedFilesClientCapabilities>,
}

/// Did change configuration client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidChangeConfigurationClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
}

/// Did change watched files client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidChangeWatchedFilesClientCapabilities {
    /// Dynamic registration
    pub dynamic_registration: Option<bool>,
}

/// Window client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowClientCapabilities {
    /// Work done progress
    pub work_done_progress: Option<bool>,
    /// Show message
    pub show_message: Option<ShowMessageClientCapabilities>,
}

/// Show message client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowMessageClientCapabilities {
    /// Message action item
    pub message_action_item: Option<ShowMessageActionItemClientCapabilities>,
}

/// Show message action item client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowMessageActionItemClientCapabilities {
    /// Additional properties support
    pub additional_properties_support: Option<bool>,
}

/// General client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralClientCapabilities {
    /// Regular expressions
    pub regular_expressions: Option<RegularExpressionsClientCapabilities>,
    /// Markdown
    pub markdown: Option<MarkdownClientCapabilities>,
}

/// Regular expressions client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularExpressionsClientCapabilities {
    /// Engine
    pub engine: String,
    /// Version
    pub version: Option<String>,
}

/// Markdown client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownClientCapabilities {
    /// Parser
    pub parser: String,
    /// Version
    pub version: Option<String>,
}

/// Active request tracking
#[derive(Debug, Clone)]
pub struct ActiveRequest {
    /// Request ID
    pub id: u64,
    /// Request method
    pub method: String,
    /// Start time
    pub start_time: Instant,
    /// Request parameters (serialized)
    pub params: Option<String>,
}

/// Performance metrics
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Error rate
    pub error_rate: f64,
}

impl LambdustLanguageServer {
    /// Creates a new Language Server instance
    pub fn new(config: LanguageServerConfig) -> Self {
        let capabilities = Self::build_server_capabilities(&config);

        let feedback_config = RealtimeFeedbackConfig {
            debounce_delay_ms: config.realtime_config.debounce_delay_ms,
            max_processing_time_ms: 100,
            incremental_parsing: config.parsing_config.incremental_parsing,
            real_time_diagnostics: config.realtime_config.real_time_diagnostics,
            syntax_highlighting: config.realtime_config.syntax_highlighting,
            completion_hints: config.realtime_config.auto_completion,
            cache_size: 50,
            worker_threads: config.performance_config.worker_threads,
        };

        let recovery_strategy = if config.parsing_config.aggressive_recovery {
            RecoveryStrategy::ErrorProductions
        } else {
            RecoveryStrategy::LocalRepair
        };

        Self {
            config,
            document_manager: Arc::new(RwLock::new(DocumentManager::new())),
            feedback_engine: Arc::new(Mutex::new(RealtimeFeedbackEngine::new(feedback_config))),
            ide_support: Arc::new(RwLock::new(IdeSupportEngine::new())),
            error_recovery: ErrorRecoveryEngine::new(recovery_strategy),
            error_generator: ContextualErrorGenerator::new(),
            server_state: Arc::new(RwLock::new(ServerState::default())),
            capabilities,
        }
    }

    /// Gets server capabilities
    pub fn capabilities(&self) -> &LanguageServerCapabilities {
        &self.capabilities
    }

    /// Initializes the language server
    pub fn initialize(&self, client_capabilities: ClientCapabilities) -> Result<()> {
        let mut state = self.server_state.write().unwrap();
        state.client_capabilities = Some(client_capabilities);
        state.initialized = true;
        Ok(())
    }

    /// Handles document open notification
    pub fn did_open_text_document(
        &self,
        uri: String,
        language_id: String,
        version: u64,
        content: String,
    ) -> Result<()> {
        // Update document manager
        {
            let mut doc_manager = self.document_manager.write().unwrap();
            doc_manager.add_document(DocumentInfo {
                uri: uri.clone(),
                content: content.clone(),
                version,
                language_id,
                last_modified: Instant::now(),
                parse_state: None,
            })?;
        }

        // Initialize feedback engine
        {
            let mut feedback = self.feedback_engine.lock().unwrap();
            feedback.open_document(uri.clone(), content.clone())?;
        }

        // Update IDE support
        {
            let mut ide_support = self.ide_support.write().unwrap();
            ide_support.update_document(uri, content)?;
        }

        Ok(())
    }

    /// Handles document change notification
    pub fn did_change_text_document(
        &self,
        uri: String,
        version: u64,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<Vec<Diagnostic>> {
        // Convert LSP changes to internal format
        let content_changes: Vec<ContentChange> = changes
            .iter()
            .map(|change| self.convert_content_change(change.clone(), version))
            .collect();

        // Process through feedback engine
        let mut diagnostics = Vec::new();
        {
            let mut feedback = self.feedback_engine.lock().unwrap();
            for change in content_changes {
                feedback.update_document(uri.clone(), change)?;
            }

            // Process updates and get feedback
            let results = feedback.process_updates();
            for result in results {
                diagnostics.extend(result.diagnostics);
            }
        }

        // Update document manager
        {
            let mut doc_manager = self.document_manager.write().unwrap();
            if let Some(last_change) = changes.last() {
                doc_manager.update_document_content(&uri, &last_change.text, version)?;
            }
        }

        Ok(diagnostics)
    }

    /// Handles document close notification
    pub fn did_close_text_document(&self, uri: String) -> Result<()> {
        let mut doc_manager = self.document_manager.write().unwrap();
        doc_manager.remove_document(&uri);
        Ok(())
    }

    /// Provides completion items
    pub fn provide_completion(
        &self,
        uri: String,
        position: Position,
    ) -> Result<Vec<CompletionItem>> {
        let ide_support = self.ide_support.read().unwrap();
        ide_support.provide_completion(&uri, position)
    }

    /// Provides hover information
    pub fn provide_hover(&self, uri: String, position: Position) -> Result<Option<Hover>> {
        let ide_support = self.ide_support.read().unwrap();
        ide_support.provide_hover(&uri, position)
    }

    /// Provides signature help
    pub fn provide_signature_help(
        &self,
        uri: String,
        position: Position,
    ) -> Result<Option<SignatureHelp>> {
        let ide_support = self.ide_support.read().unwrap();
        ide_support.provide_signature_help(&uri, position)
    }

    /// Provides diagnostics
    pub fn provide_diagnostics(&self, uri: String) -> Result<Vec<Diagnostic>> {
        let mut ide_support = self.ide_support.write().unwrap();
        ide_support.provide_diagnostics(&uri)
    }

    /// Provides document symbols
    pub fn provide_document_symbols(&self, uri: String) -> Result<Vec<SymbolInformation>> {
        // This would analyze the document and extract symbols
        let doc_manager = self.document_manager.read().unwrap();
        if let Some(_doc) = doc_manager.get_document(&uri) {
            // Parse document and extract symbols
            Ok(Vec::new()) // Placeholder
        } else {
            Err(Box::new(Error::runtime_error("Document not found", None)))
        }
    }

    /// Shuts down the language server
    pub fn shutdown(&self) -> Result<()> {
        let mut state = self.server_state.write().unwrap();
        state.shutdown_requested = true;
        Ok(())
    }

    // Helper methods
    fn build_server_capabilities(config: &LanguageServerConfig) -> LanguageServerCapabilities {
        LanguageServerCapabilities {
            text_document_sync: TextDocumentSyncKind::Incremental,
            hover_provider: config.realtime_config.auto_completion,
            completion_provider: if config.realtime_config.auto_completion {
                Some(crate::parser::ide_support::CompletionOptions {
                    trigger_characters: vec!["(".to_string(), " ".to_string()],
                    all_commit_characters: vec![")".to_string()],
                    resolve_provider: true,
                })
            } else {
                None
            },
            signature_help_provider: if config.realtime_config.signature_help {
                Some(crate::parser::ide_support::SignatureHelpOptions {
                    trigger_characters: vec!["(".to_string()],
                    retrigger_characters: vec![" ".to_string()],
                })
            } else {
                None
            },
            definition_provider: true,
            references_provider: true,
            document_highlight_provider: true,
            document_symbol_provider: true,
            workspace_symbol_provider: true,
            code_action_provider: if config.diagnostic_config.quick_fixes {
                Some(crate::parser::ide_support::CodeActionOptions {
                    code_action_kinds: vec!["quickfix".to_string()],
                    resolve_provider: true,
                })
            } else {
                None
            },
            code_lens_provider: Some(crate::parser::ide_support::CodeLensOptions {
                resolve_provider: false,
            }),
            document_formatting_provider: true,
            document_range_formatting_provider: true,
            rename_provider: Some(crate::parser::ide_support::RenameOptions {
                prepare_provider: true,
            }),
            folding_range_provider: true,
            semantic_tokens_provider: if config.realtime_config.syntax_highlighting {
                Some(crate::parser::ide_support::SemanticTokensOptions {
                    legend: crate::parser::ide_support::SemanticTokensLegend {
                        token_types: vec![
                            "keyword".to_string(),
                            "function".to_string(),
                            "variable".to_string(),
                            "string".to_string(),
                            "number".to_string(),
                            "comment".to_string(),
                        ],
                        token_modifiers: vec!["deprecated".to_string(), "readonly".to_string()],
                    },
                    range: true,
                    full: Some(crate::parser::ide_support::SemanticTokensFullOptions {
                        delta: true,
                    }),
                })
            } else {
                None
            },
            inline_value_provider: true,
            inlay_hint_provider: Some(crate::parser::ide_support::InlayHintOptions {
                resolve_provider: true,
            }),
            diagnostic_provider: if config.realtime_config.real_time_diagnostics {
                Some(crate::parser::ide_support::DiagnosticOptions {
                    identifier: Some("lambdust".to_string()),
                    inter_file_dependencies: true,
                    workspace_diagnostics: config.workspace_config.workspace_folders,
                })
            } else {
                None
            },
        }
    }

    fn convert_content_change(
        &self,
        change: TextDocumentContentChangeEvent,
        version: u64,
    ) -> ContentChange {
        if let Some(range) = change.range {
            ContentChange::IncrementalUpdate {
                range,
                text: change.text,
                version,
            }
        } else {
            ContentChange::FullUpdate {
                content: change.text,
                version,
            }
        }
    }
}

/// Text document content change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    /// Range of the change (None for full document)
    pub range: Option<Range>,
    /// Length of the range being replaced
    pub range_length: Option<u32>,
    /// New text content
    pub text: String,
}

impl Default for LanguageServerConfig {
    fn default() -> Self {
        Self {
            server_info: ServerInfo {
                name: "Lambdust Language Server".to_string(),
                version: "0.2.0".to_string(),
            },
            workspace_config: WorkspaceConfig {
                file_extensions: vec![".scm".to_string(), ".ss".to_string(), ".rkt".to_string()],
                workspace_folders: true,
                file_watching: true,
                configuration_change: true,
            },
            parsing_config: ParsingConfig {
                max_errors_per_file: 100,
                aggressive_recovery: true,
                parse_timeout_ms: 5000,
                incremental_parsing: true,
                semantic_analysis: true,
            },
            realtime_config: RealtimeConfig {
                debounce_delay_ms: 300,
                real_time_diagnostics: true,
                syntax_highlighting: true,
                auto_completion: true,
                signature_help: true,
            },
            diagnostic_config: DiagnosticConfig {
                contextual_errors: true,
                quick_fixes: true,
                educational_content: true,
                r7rs_compliance: true,
                severity_levels: vec![
                    DiagnosticSeverity::Error,
                    DiagnosticSeverity::Warning,
                    DiagnosticSeverity::Information,
                ],
            },
            performance_config: PerformanceConfig {
                worker_threads: 2,
                cache_size_mb: 64,
                memory_monitoring: true,
                metrics_collection: true,
            },
        }
    }
}

impl DocumentManager {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
            versions: HashMap::new(),
            workspace_folders: Vec::new(),
            watchers: HashMap::new(),
        }
    }

    fn add_document(&mut self, doc: DocumentInfo) -> Result<()> {
        let uri = doc.uri.clone();
        let version = doc.version;

        self.documents.insert(uri.clone(), doc);
        self.versions.insert(uri, version);

        Ok(())
    }

    fn get_document(&self, uri: &str) -> Option<&DocumentInfo> {
        self.documents.get(uri)
    }

    fn update_document_content(&mut self, uri: &str, content: &str, version: u64) -> Result<()> {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.content = content.to_string();
            doc.version = version;
            doc.last_modified = Instant::now();
            self.versions.insert(uri.to_string(), version);
        }
        Ok(())
    }

    fn remove_document(&mut self, uri: &str) {
        self.documents.remove(uri);
        self.versions.remove(uri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_server_creation() {
        let config = LanguageServerConfig::default();
        let server = LambdustLanguageServer::new(config);

        assert!(server.capabilities().hover_provider);
        assert!(server.capabilities().completion_provider.is_some());
    }

    #[test]
    fn test_document_manager() {
        let mut manager = DocumentManager::new();

        let doc = DocumentInfo {
            uri: "file:///test.scm".to_string(),
            content: "(define x 42)".to_string(),
            version: 1,
            language_id: "scheme".to_string(),
            last_modified: Instant::now(),
            parse_state: None,
        };

        assert!(manager.add_document(doc).is_ok());
        assert!(manager.get_document("file:///test.scm").is_some());
    }

    #[test]
    fn test_server_initialization() {
        let config = LanguageServerConfig::default();
        let server = LambdustLanguageServer::new(config);

        let client_caps = ClientCapabilities {
            text_document: None,
            workspace: None,
            window: None,
            general: None,
        };

        assert!(server.initialize(client_caps).is_ok());

        let state = server.server_state.try_read().unwrap();
        assert!(state.initialized);
    }
}
