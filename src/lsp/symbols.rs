//! Symbol provider for workspace navigation and symbol search
//!
//! This module provides symbol navigation, workspace symbol search,
//! and definition/reference finding capabilities for the LSP server.

use crate::error::{LambdustError, Result};
use crate::interpreter::LambdustInterpreter;
use crate::lsp::position::{Position, Range};
use crate::lsp::document::Document;
use std::collections::HashMap;

/// Symbol information for navigation
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    
    /// Symbol kind
    pub kind: SymbolKind,
    
    /// Location where symbol is defined
    pub location: Location,
    
    /// Container symbol (for nested symbols)
    pub container_name: Option<String>,
    
    /// Additional metadata
    pub metadata: SymbolMetadata,
}

/// Location reference
#[derive(Debug, Clone)]
pub struct Location {
    /// URI of the document
    pub uri: String,
    
    /// Range in the document
    pub range: Range,
}

/// Symbol kind classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Function definition
    Function,
    
    /// Variable definition
    Variable,
    
    /// Constant definition
    Constant,
    
    /// Macro definition
    Macro,
    
    /// Module or namespace
    Module,
    
    /// Class or type definition
    Class,
    
    /// Method definition
    Method,
    
    /// Property or field
    Property,
    
    /// Constructor
    Constructor,
    
    /// Enum value
    EnumMember,
    
    /// Interface
    Interface,
    
    /// Struct
    Struct,
    
    /// Event
    Event,
    
    /// Operator
    Operator,
    
    /// Type parameter
    TypeParameter,
}

/// Symbol metadata
#[derive(Debug, Clone, Default)]
pub struct SymbolMetadata {
    /// Symbol signature
    pub signature: Option<String>,
    
    /// Documentation
    pub documentation: Option<String>,
    
    /// Parameters (for functions)
    pub parameters: Vec<String>,
    
    /// Return type
    pub return_type: Option<String>,
    
    /// Symbol tags
    pub tags: Vec<SymbolTag>,
    
    /// Deprecated status
    pub deprecated: bool,
}

/// Symbol tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolTag {
    /// Deprecated symbol
    Deprecated,
    
    /// Internal/private symbol
    Internal,
    
    /// Experimental symbol
    Experimental,
}

/// Symbol search result
#[derive(Debug, Clone)]
pub struct SymbolInformation {
    /// Symbol name
    pub name: String,
    
    /// Symbol kind
    pub kind: SymbolKind,
    
    /// Location of the symbol
    pub location: Location,
    
    /// Container name
    pub container_name: Option<String>,
}

/// Symbol provider for navigation and search
pub struct SymbolProvider {
    /// Interpreter for symbol resolution
    interpreter: LambdustInterpreter,
    
    /// Symbol index for each document
    document_symbols: HashMap<String, Vec<Symbol>>,
    
    /// Global symbol index
    workspace_symbols: HashMap<String, Vec<Symbol>>,
    
    /// Symbol references
    symbol_references: HashMap<String, Vec<Location>>,
    
    /// Configuration
    config: SymbolConfig,
}

/// Configuration for symbol provider
#[derive(Debug, Clone)]
pub struct SymbolConfig {
    /// Maximum number of symbols to return in search
    pub max_symbols: usize,
    
    /// Enable fuzzy matching
    pub fuzzy_matching: bool,
    
    /// Include deprecated symbols
    pub include_deprecated: bool,
    
    /// Case sensitive search
    pub case_sensitive: bool,
    
    /// Include private symbols
    pub include_private: bool,
}

impl Default for SymbolConfig {
    fn default() -> Self {
        Self {
            max_symbols: 100,
            fuzzy_matching: true,
            include_deprecated: false,
            case_sensitive: false,
            include_private: false,
        }
    }
}

impl SymbolProvider {
    /// Create a new symbol provider
    pub fn new(interpreter: LambdustInterpreter) -> Result<Self> {
        Ok(Self {
            interpreter,
            document_symbols: HashMap::new(),
            workspace_symbols: HashMap::new(),
            symbol_references: HashMap::new(),
            config: SymbolConfig::default(),
        })
    }
    
    /// Get symbols in a document
    pub fn get_document_symbols(&mut self, document: &Document) -> Result<Vec<Symbol>> {
        let uri = &document.uri;
        
        // Check cache first
        if let Some(symbols) = self.document_symbols.get(uri) {
            return Ok(symbols.clone());
        }
        
        // Extract symbols from document
        let symbols = self.extract_symbols_from_document(document)?;
        
        // Cache the results
        self.document_symbols.insert(uri.clone(), symbols.clone());
        
        Ok(symbols)
    }
    
    /// Search for symbols in workspace
    pub fn search_workspace_symbols(&self, query: &str) -> Result<Vec<SymbolInformation>> {
        let mut results = Vec::new();
        
        for symbols in self.workspace_symbols.values() {
            for symbol in symbols {
                if self.matches_query(&symbol.name, query) {
                    if self.should_include_symbol(symbol) {
                        results.push(SymbolInformation {
                            name: symbol.name.clone(),
                            kind: symbol.kind,
                            location: symbol.location.clone(),
                            container_name: symbol.container_name.clone(),
                        });
                    }
                }
                
                if results.len() >= self.config.max_symbols {
                    break;
                }
            }
        }
        
        // Sort results by relevance
        results.sort_by(|a, b| {
            self.calculate_relevance(&a.name, query)
                .partial_cmp(&self.calculate_relevance(&b.name, query))
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        });
        
        Ok(results)
    }
    
    /// Find definition of symbol at position
    pub fn find_definition(&self, document: &Document, position: Position) -> Result<Option<Location>> {
        if let Some(symbol_name) = self.get_symbol_at_position(document, position)? {
            // Look for definition in workspace symbols
            for symbols in self.workspace_symbols.values() {
                for symbol in symbols {
                    if symbol.name == symbol_name {
                        return Ok(Some(symbol.location.clone()));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Find references to symbol at position
    pub fn find_references(
        &self,
        document: &Document,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<Location>> {
        if let Some(symbol_name) = self.get_symbol_at_position(document, position)? {
            let mut references = Vec::new();
            
            // Get references from cache
            if let Some(refs) = self.symbol_references.get(&symbol_name) {
                references.extend(refs.clone());
            }
            
            // Find declaration if requested
            if include_declaration {
                if let Some(definition) = self.find_definition(document, position)? {
                    references.push(definition);
                }
            }
            
            Ok(references)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Update symbols for a document
    pub fn update_document_symbols(&mut self, document: &Document) -> Result<()> {
        let symbols = self.extract_symbols_from_document(document)?;
        
        // Remove old symbols from workspace index
        if let Some(old_symbols) = self.document_symbols.get(&document.uri) {
            for symbol in old_symbols {
                if let Some(workspace_symbols) = self.workspace_symbols.get_mut(&symbol.name) {
                    workspace_symbols.retain(|s| s.location.uri != document.uri);
                    if workspace_symbols.is_empty() {
                        self.workspace_symbols.remove(&symbol.name);
                    }
                }
            }
        }
        
        // Add new symbols to workspace index
        for symbol in &symbols {
            self.workspace_symbols
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol.clone());
        }
        
        // Update document symbols cache
        self.document_symbols.insert(document.uri.clone(), symbols);
        
        Ok(())
    }
    
    /// Remove document symbols
    pub fn remove_document_symbols(&mut self, uri: &str) -> Result<()> {
        if let Some(symbols) = self.document_symbols.remove(uri) {
            // Remove from workspace index
            for symbol in symbols {
                if let Some(workspace_symbols) = self.workspace_symbols.get_mut(&symbol.name) {
                    workspace_symbols.retain(|s| s.location.uri != uri);
                    if workspace_symbols.is_empty() {
                        self.workspace_symbols.remove(&symbol.name);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract symbols from document content
    fn extract_symbols_from_document(&self, document: &Document) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let content = document.get_content();
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            symbols.extend(self.extract_symbols_from_line(line, line_num, &document.uri)?);
        }
        
        Ok(symbols)
    }
    
    /// Extract symbols from a single line
    fn extract_symbols_from_line(&self, line: &str, line_num: usize, uri: &str) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let trimmed = line.trim();
        
        // Look for define forms
        if trimmed.starts_with("(define ") {
            if let Some(symbol) = self.parse_define_form(trimmed, line_num, uri)? {
                symbols.push(symbol);
            }
        }
        
        // Look for lambda forms (function definitions)
        if trimmed.contains("lambda") {
            if let Some(symbol) = self.parse_lambda_form(trimmed, line_num, uri)? {
                symbols.push(symbol);
            }
        }
        
        // Look for macro definitions
        if trimmed.starts_with("(define-syntax ") || trimmed.starts_with("(syntax-rules") {
            if let Some(symbol) = self.parse_macro_form(trimmed, line_num, uri)? {
                symbols.push(symbol);
            }
        }
        
        Ok(symbols)
    }
    
    /// Parse define form to extract symbol
    fn parse_define_form(&self, line: &str, line_num: usize, uri: &str) -> Result<Option<Symbol>> {
        // Simple parsing for (define name value) or (define (name params) body)
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        if tokens.len() >= 3 && tokens[0] == "(define" {
            let name_token = tokens[1];
            
            if name_token.starts_with('(') {
                // Function definition: (define (name params) body)
                let name = name_token.trim_start_matches('(');
                if !name.is_empty() {
                    return Ok(Some(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        location: Location {
                            uri: uri.to_string(),
                            range: Range::new(
                                Position::new(line_num as u32, 0),
                                Position::new(line_num as u32, line.len() as u32),
                            ),
                        },
                        container_name: None,
                        metadata: SymbolMetadata {
                            signature: Some(name_token.to_string()),
                            ..Default::default()
                        },
                    }));
                }
            } else {
                // Variable definition: (define name value)
                return Ok(Some(Symbol {
                    name: name_token.to_string(),
                    kind: SymbolKind::Variable,
                    location: Location {
                        uri: uri.to_string(),
                        range: Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                    },
                    container_name: None,
                    metadata: SymbolMetadata::default(),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Parse lambda form to extract symbol (if named)
    fn parse_lambda_form(&self, line: &str, line_num: usize, uri: &str) -> Result<Option<Symbol>> {
        // This would parse lambda expressions that are bound to names
        // For now, we'll skip this as it requires more sophisticated parsing
        Ok(None)
    }
    
    /// Parse macro definition to extract symbol
    fn parse_macro_form(&self, line: &str, line_num: usize, uri: &str) -> Result<Option<Symbol>> {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        if tokens.len() >= 3 && tokens[0] == "(define-syntax" {
            let name = tokens[1];
            return Ok(Some(Symbol {
                name: name.to_string(),
                kind: SymbolKind::Macro,
                location: Location {
                    uri: uri.to_string(),
                    range: Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, line.len() as u32),
                    ),
                },
                container_name: None,
                metadata: SymbolMetadata::default(),
            }));
        }
        
        Ok(None)
    }
    
    /// Get symbol at position
    fn get_symbol_at_position(&self, document: &Document, position: Position) -> Result<Option<String>> {
        let line_content = document.get_line(position.line as usize)
            .ok_or_else(|| LambdustError::runtime_error("Invalid line"))?;
        
        let char_pos = position.character as usize;
        if char_pos > line_content.len() {
            return Ok(None);
        }
        
        // Find symbol boundaries (simple implementation)
        let chars: Vec<char> = line_content.chars().collect();
        
        let mut start = char_pos;
        while start > 0 && Self::is_symbol_char(chars[start - 1]) {
            start -= 1;
        }
        
        let mut end = char_pos;
        while end < chars.len() && Self::is_symbol_char(chars[end]) {
            end += 1;
        }
        
        if start < end {
            let symbol: String = chars[start..end].iter().collect();
            Ok(Some(symbol))
        } else {
            Ok(None)
        }
    }
    
    /// Check if character is part of a symbol
    fn is_symbol_char(ch: char) -> bool {
        ch.is_alphanumeric() || 
        matches!(ch, '+' | '-' | '*' | '/' | '?' | '!' | '<' | '>' | '=' | '_' | ':')
    }
    
    /// Check if query matches symbol name
    fn matches_query(&self, symbol_name: &str, query: &str) -> bool {
        if self.config.case_sensitive {
            if self.config.fuzzy_matching {
                self.fuzzy_match(symbol_name, query)
            } else {
                symbol_name.contains(query)
            }
        } else {
            let symbol_lower = symbol_name.to_lowercase();
            let query_lower = query.to_lowercase();
            
            if self.config.fuzzy_matching {
                self.fuzzy_match(&symbol_lower, &query_lower)
            } else {
                symbol_lower.contains(&query_lower)
            }
        }
    }
    
    /// Simple fuzzy matching
    fn fuzzy_match(&self, text: &str, pattern: &str) -> bool {
        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();
        
        let mut text_idx = 0;
        let mut pattern_idx = 0;
        
        while text_idx < text_chars.len() && pattern_idx < pattern_chars.len() {
            if text_chars[text_idx] == pattern_chars[pattern_idx] {
                pattern_idx += 1;
            }
            text_idx += 1;
        }
        
        pattern_idx == pattern_chars.len()
    }
    
    /// Calculate relevance score for search results
    fn calculate_relevance(&self, symbol_name: &str, query: &str) -> f64 {
        if symbol_name == query {
            return 1.0;
        }
        
        if symbol_name.starts_with(query) {
            return 0.8;
        }
        
        if symbol_name.contains(query) {
            return 0.6;
        }
        
        // Fuzzy match score
        if self.fuzzy_match(symbol_name, query) {
            return 0.4;
        }
        
        0.0
    }
    
    /// Check if symbol should be included in results
    fn should_include_symbol(&self, symbol: &Symbol) -> bool {
        if symbol.metadata.deprecated && !self.config.include_deprecated {
            return false;
        }
        
        if symbol.metadata.tags.contains(&SymbolTag::Internal) && !self.config.include_private {
            return false;
        }
        
        true
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: SymbolConfig) {
        self.config = config;
    }
    
    /// Clear all symbols
    pub fn clear_all_symbols(&mut self) {
        self.document_symbols.clear();
        self.workspace_symbols.clear();
        self.symbol_references.clear();
    }
}

impl SymbolKind {
    /// Convert to LSP symbol kind
    pub fn to_lsp_symbol_kind(&self) -> lsp_types::SymbolKind {
        match self {
            SymbolKind::Function => lsp_types::SymbolKind::FUNCTION,
            SymbolKind::Variable => lsp_types::SymbolKind::VARIABLE,
            SymbolKind::Constant => lsp_types::SymbolKind::CONSTANT,
            SymbolKind::Macro => lsp_types::SymbolKind::FUNCTION,
            SymbolKind::Module => lsp_types::SymbolKind::MODULE,
            SymbolKind::Class => lsp_types::SymbolKind::CLASS,
            SymbolKind::Method => lsp_types::SymbolKind::METHOD,
            SymbolKind::Property => lsp_types::SymbolKind::PROPERTY,
            SymbolKind::Constructor => lsp_types::SymbolKind::CONSTRUCTOR,
            SymbolKind::EnumMember => lsp_types::SymbolKind::ENUM_MEMBER,
            SymbolKind::Interface => lsp_types::SymbolKind::INTERFACE,
            SymbolKind::Struct => lsp_types::SymbolKind::STRUCT,
            SymbolKind::Event => lsp_types::SymbolKind::EVENT,
            SymbolKind::Operator => lsp_types::SymbolKind::OPERATOR,
            SymbolKind::TypeParameter => lsp_types::SymbolKind::TYPE_PARAMETER,
        }
    }
}
