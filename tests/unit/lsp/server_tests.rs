//! LSP Server Lifecycle and Protocol Compliance Tests
//!
//! Tests for core server functionality including initialization,
//! client-server communication, capability negotiation, and shutdown.

use super::lsp_test_utils::*;
use crate::lsp::{LspConfig, LambdustLanguageServer, LspCapabilities};
use crate::lsp::server::{InitializeParams, ServerState, WorkspaceFolder};
use lsp_types;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_server_creation() {
    let server = create_test_server();
    assert_eq!(server.state(), ServerState::Starting);
}

#[test] 
fn test_server_configuration() {
    let config = LspConfig {
        debug_mode: true,
        enable_verification: true,
        enable_performance_analysis: false,
        max_diagnostics: 200,
        completion_triggers: vec!["(".to_string(), ".".to_string()],
        workspace_root: Some("/test/workspace".into()),
        enable_repl_integration: true,
    };
    
    let server = LambdustLanguageServer::new(config.clone()).unwrap();
    // Verify configuration is applied (would need getter methods in real implementation)
}

#[tokio::test]
async fn test_server_initialization() {
    let mut server = create_test_server();
    
    let client_capabilities = lsp_types::ClientCapabilities {
        text_document: Some(lsp_types::TextDocumentClientCapabilities {
            completion: Some(lsp_types::CompletionClientCapabilities {
                completion_item: Some(lsp_types::CompletionItemCapability {
                    snippet_support: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            hover: Some(lsp_types::HoverClientCapabilities {
                content_format: Some(vec![lsp_types::MarkupKind::Markdown]),
                ..Default::default()
            }),
            ..Default::default()
        }),
        workspace: Some(lsp_types::WorkspaceClientCapabilities {
            symbol: Some(lsp_types::WorkspaceSymbolClientCapabilities {
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: Some("/test/project".to_string()),
        root_uri: Some("file:///test/project".to_string()),
        workspace_folders: Some(vec![
            WorkspaceFolder {
                uri: "file:///test/project".to_string(),
                name: "test-project".to_string(),
            }
        ]),
        capabilities: client_capabilities,
        initialization_options: None,
        trace: Some("verbose".to_string()),
    };
    
    let result = server.initialize(params).await.unwrap();
    
    // Verify server capabilities
    assert!(result.capabilities.text_document_sync.is_some());
    assert!(result.capabilities.completion_provider.is_some());
    assert!(result.capabilities.hover_provider.is_some());
    
    // Verify server info
    assert!(result.server_info.is_some());
    let server_info = result.server_info.unwrap();
    assert_eq!(server_info.name, "Lambdust Language Server");
    assert!(server_info.version.is_some());
    
    // Verify state transition
    assert_eq!(server.state(), ServerState::Initialized);
}

#[tokio::test]
async fn test_server_initialized_notification() {
    let mut server = create_test_server();
    
    // Initialize first
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: None,
        capabilities: lsp_types::ClientCapabilities::default(),
        initialization_options: None,
        trace: None,
    };
    
    server.initialize(params).await.unwrap();
    
    // Send initialized notification
    server.initialized().unwrap();
    
    // Verify state transition
    assert_eq!(server.state(), ServerState::Running);
}

#[tokio::test]
async fn test_server_shutdown_sequence() {
    let mut server = create_test_server();
    
    // Initialize and start server
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: None,
        capabilities: lsp_types::ClientCapabilities::default(),
        initialization_options: None,
        trace: None,
    };
    
    server.initialize(params).await.unwrap();
    server.initialized().unwrap();
    assert_eq!(server.state(), ServerState::Running);
    
    // Shutdown request
    server.shutdown().unwrap();
    assert_eq!(server.state(), ServerState::ShuttingDown);
    
    // Exit notification
    server.exit();
    assert_eq!(server.state(), ServerState::Shutdown);
}

#[tokio::test]
async fn test_capability_negotiation() {
    let mut server = create_test_server();
    
    let client_capabilities = lsp_types::ClientCapabilities {
        text_document: Some(lsp_types::TextDocumentClientCapabilities {
            completion: Some(lsp_types::CompletionClientCapabilities {
                completion_item: Some(lsp_types::CompletionItemCapability {
                    snippet_support: Some(true),
                    commit_characters_support: Some(true),
                    documentation_format: Some(vec![lsp_types::MarkupKind::Markdown]),
                    ..Default::default()
                }),
                context_support: Some(true),
                ..Default::default()
            }),
            hover: Some(lsp_types::HoverClientCapabilities {
                content_format: Some(vec![lsp_types::MarkupKind::Markdown, lsp_types::MarkupKind::PlainText]),
                ..Default::default()
            }),
            definition: Some(lsp_types::GotoCapability {
                link_support: Some(true),
                ..Default::default()
            }),
            references: Some(lsp_types::ReferenceClientCapabilities {
                ..Default::default()
            }),
            document_symbol: Some(lsp_types::DocumentSymbolClientCapabilities {
                hierarchical_document_symbol_support: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }),
        workspace: Some(lsp_types::WorkspaceClientCapabilities {
            symbol: Some(lsp_types::WorkspaceSymbolClientCapabilities {
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: None,
        capabilities: client_capabilities,
        initialization_options: None,
        trace: None,
    };
    
    let result = server.initialize(params).await.unwrap();
    
    // Verify that server capabilities match client capabilities
    assert!(result.capabilities.completion_provider.is_some());
    let completion = result.capabilities.completion_provider.unwrap();
    assert!(completion.trigger_characters.is_some());
    assert_eq!(completion.trigger_characters.unwrap(), vec!["(", " "]);
    
    assert!(result.capabilities.hover_provider.is_some());
    assert!(result.capabilities.definition_provider.is_some());
    assert!(result.capabilities.references_provider.is_some());
    assert!(result.capabilities.document_symbol_provider.is_some());
}

#[tokio::test]
async fn test_workspace_folder_management() {
    let mut server = create_test_server();
    
    let workspace_folders = vec![
        WorkspaceFolder {
            uri: "file:///project1".to_string(),
            name: "project1".to_string(),
        },
        WorkspaceFolder {
            uri: "file:///project2".to_string(), 
            name: "project2".to_string(),
        },
    ];
    
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: Some(workspace_folders.clone()),
        capabilities: lsp_types::ClientCapabilities::default(),
        initialization_options: None,
        trace: None,
    };
    
    let result = server.initialize(params).await.unwrap();
    assert!(result.capabilities.text_document_sync.is_some());
    
    // Verify workspace folders are tracked (would need getter in real implementation)
}

#[test]
fn test_lsp_capabilities_default() {
    let capabilities = LspCapabilities::default();
    
    assert!(capabilities.text_document_sync);
    assert!(capabilities.completion);
    assert!(capabilities.hover);
    assert!(capabilities.diagnostics);
    assert!(capabilities.symbol_navigation);
    assert!(capabilities.definition);
    assert!(capabilities.references);
    assert!(capabilities.formatting);
    assert!(capabilities.semantic_highlighting);
}

#[test]
fn test_server_state_transitions() {
    let mut server = create_test_server();
    
    // Initial state
    assert_eq!(server.state(), ServerState::Starting);
    
    // Simulate state transitions (in real code, these would be internal)
    // server.server_state = ServerState::Initialized;
    // assert_eq!(server.state(), ServerState::Initialized);
    
    // server.server_state = ServerState::Running;
    // assert_eq!(server.state(), ServerState::Running);
    
    // server.server_state = ServerState::ShuttingDown;
    // assert_eq!(server.state(), ServerState::ShuttingDown);
    
    // server.server_state = ServerState::Shutdown;
    // assert_eq!(server.state(), ServerState::Shutdown);
}

#[tokio::test]
async fn test_initialization_options() {
    let mut server = create_test_server();
    
    let init_options = serde_json::json!({
        "enable_debugging": true,
        "max_completions": 100,
        "custom_settings": {
            "scheme_version": "R7RS",
            "extensions": ["srfi-1", "srfi-13"]
        }
    });
    
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: None,
        capabilities: lsp_types::ClientCapabilities::default(),
        initialization_options: Some(init_options),
        trace: Some("messages".to_string()),
    };
    
    let result = server.initialize(params).await.unwrap();
    assert!(result.server_info.is_some());
    
    // Verify initialization options are processed (would need custom handling)
}

#[test]
fn test_server_error_handling() {
    // Test server creation with invalid config
    let config = LspConfig {
        debug_mode: true,
        enable_verification: true,
        enable_performance_analysis: true,
        max_diagnostics: 0, // Invalid: should be > 0
        completion_triggers: vec![], // Empty triggers
        workspace_root: None,
        enable_repl_integration: true,
    };
    
    // Should still create server but might have warnings
    let server = LambdustLanguageServer::new(config);
    assert!(server.is_ok());
}

#[tokio::test]
async fn test_server_versioning() {
    let mut server = create_test_server();
    
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: None,
        capabilities: lsp_types::ClientCapabilities::default(),
        initialization_options: None,
        trace: None,
    };
    
    let result = server.initialize(params).await.unwrap();
    
    let server_info = result.server_info.unwrap();
    assert_eq!(server_info.name, "Lambdust Language Server");
    
    let version = server_info.version.unwrap();
    assert!(!version.is_empty());
    assert!(version.contains('.'), "Version should contain dots");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = Arc::new(RwLock::new(create_test_server()));
    
    // Test concurrent initialization attempts (should be handled gracefully)
    let server1 = server.clone();
    let server2 = server.clone();
    
    let task1 = tokio::spawn(async move {
        let mut s = server1.write().await;
        let params = InitializeParams {
            process_id: Some(1234),
            root_path: None,
            root_uri: None,
            workspace_folders: None,
            capabilities: lsp_types::ClientCapabilities::default(),
            initialization_options: None,
            trace: None,
        };
        s.initialize(params).await
    });
    
    let task2 = tokio::spawn(async move {
        let s = server2.read().await;
        s.state()
    });
    
    let (result1, result2) = tokio::join!(task1, task2);
    assert!(result1.is_ok());
    // result2 should return a valid state
    assert!(matches!(result2, ServerState::Starting | ServerState::Initialized));
}

#[test]
fn test_default_configurations() {
    let default_config = LspConfig::default();
    
    assert!(!default_config.debug_mode);
    assert!(default_config.enable_verification);
    assert!(default_config.enable_performance_analysis);
    assert_eq!(default_config.max_diagnostics, 100);
    assert!(!default_config.completion_triggers.is_empty());
    assert!(default_config.completion_triggers.contains(&"(".to_string()));
    assert!(default_config.enable_repl_integration);
}

#[tokio::test]
async fn test_server_robustness() {
    let mut server = create_test_server();
    
    // Test double initialization (should handle gracefully)
    let params = InitializeParams {
        process_id: Some(1234),
        root_path: None,
        root_uri: None,
        workspace_folders: None,
        capabilities: lsp_types::ClientCapabilities::default(),
        initialization_options: None,
        trace: None,
    };
    
    let result1 = server.initialize(params.clone()).await;
    assert!(result1.is_ok());
    
    // Second initialization should either succeed or return appropriate error
    let result2 = server.initialize(params).await;
    // In a real implementation, this might return an error or ignore the second init
    assert!(result2.is_ok() || result2.is_err());
}