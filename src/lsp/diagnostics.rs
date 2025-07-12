//! Diagnostics engine for Lambdust LSP
//!
//! This module provides real-time syntax checking, semantic analysis,
//! and error reporting with precise location information for IDE integration.

use crate::error::{LambdustError, Result};
use crate::interpreter::LambdustInterpreter;
use crate::lexer::{lex, LexError};
use crate::lsp::position::{Position, Range, Span};
use crate::parser::parse;
use std::collections::HashMap;

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error that prevents compilation/evaluation
    Error,
    /// Warning about potential issues
    Warning,
    /// Information message
    Information,
    /// Hint for improvement
    Hint,
}

/// Diagnostic message with location information
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Range where the diagnostic applies
    pub range: Range,
    
    /// Severity of the diagnostic
    pub severity: DiagnosticSeverity,
    
    /// Diagnostic message
    pub message: String,
    
    /// Error code (if applicable)
    pub code: Option<String>,
    
    /// Source of the diagnostic
    pub source: String,
    
    /// Related information
    pub related_information: Vec<DiagnosticRelatedInformation>,
    
    /// Tags for additional context
    pub tags: Vec<DiagnosticTag>,
}

/// Related diagnostic information
#[derive(Debug, Clone)]
pub struct DiagnosticRelatedInformation {
    /// Location of related information
    pub location: Location,
    
    /// Related message
    pub message: String,
}

/// Location information
#[derive(Debug, Clone)]
pub struct Location {
    /// URI of the document
    pub uri: String,
    
    /// Range in the document
    pub range: Range,
}

/// Diagnostic tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticTag {
    /// Unused code
    Unnecessary,
    
    /// Deprecated code
    Deprecated,
}

/// Diagnostics engine for analyzing Scheme code
pub struct DiagnosticsEngine {
    /// Interpreter for semantic analysis
    interpreter: LambdustInterpreter,
    
    /// Cache of recent diagnostics
    diagnostics_cache: HashMap<String, Vec<Diagnostic>>,
    
    /// Configuration options
    config: DiagnosticsConfig,
}

/// Configuration for diagnostics engine
#[derive(Debug, Clone)]
pub struct DiagnosticsConfig {
    /// Enable syntax checking
    pub enable_syntax_check: bool,
    
    /// Enable semantic analysis
    pub enable_semantic_analysis: bool,
    
    /// Enable style warnings
    pub enable_style_warnings: bool,
    
    /// Enable performance hints
    pub enable_performance_hints: bool,
    
    /// Maximum number of diagnostics to report
    pub max_diagnostics: usize,
    
    /// Enable unused variable detection
    pub detect_unused_variables: bool,
    
    /// Enable deprecated feature warnings
    pub warn_deprecated: bool,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            enable_syntax_check: true,
            enable_semantic_analysis: true,
            enable_style_warnings: true,
            enable_performance_hints: false,
            max_diagnostics: 100,
            detect_unused_variables: true,
            warn_deprecated: true,
        }
    }
}

impl DiagnosticsEngine {
    /// Create a new diagnostics engine
    pub fn new(interpreter: LambdustInterpreter) -> Result<Self> {
        Ok(Self {
            interpreter,
            diagnostics_cache: HashMap::new(),
            config: DiagnosticsConfig::default(),
        })
    }
    
    /// Analyze a document and return diagnostics
    pub fn analyze_document(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        
        // Syntax analysis
        if self.config.enable_syntax_check {
            diagnostics.extend(self.check_syntax(document)?);
        }
        
        // Semantic analysis
        if self.config.enable_semantic_analysis {
            diagnostics.extend(self.check_semantics(document)?);
        }
        
        // Style analysis
        if self.config.enable_style_warnings {
            diagnostics.extend(self.check_style(document)?);
        }
        
        // Performance analysis
        if self.config.enable_performance_hints {
            diagnostics.extend(self.check_performance(document)?);
        }
        
        // Limit number of diagnostics
        diagnostics.truncate(self.config.max_diagnostics);
        
        Ok(diagnostics)
    }
    
    /// Check syntax errors
    fn check_syntax(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        // Lexical analysis
        match lex(content) {
            Ok(_tokens) => {
                // Parsing analysis
                match parse(content) {
                    Ok(_ast) => {
                        // No syntax errors
                    },
                    Err(parse_error) => {
                        if let Some(diagnostic) = self.parse_error_to_diagnostic(&parse_error, document) {
                            diagnostics.push(diagnostic);
                        }
                    }
                }
            },
            Err(lex_error) => {
                if let Some(diagnostic) = self.lex_error_to_diagnostic(&lex_error, document) {
                    diagnostics.push(diagnostic);
                }
            }
        }
        
        // Check for common syntax issues
        diagnostics.extend(self.check_bracket_balance(document)?);
        diagnostics.extend(self.check_string_literals(document)?);
        
        Ok(diagnostics)
    }
    
    /// Check semantic errors
    fn check_semantics(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        // Try to evaluate the code and catch semantic errors
        match self.interpreter.eval_string(content) {
            Ok(_) => {
                // No semantic errors
            },
            Err(error) => {
                if let Some(diagnostic) = self.runtime_error_to_diagnostic(&error, document) {
                    diagnostics.push(diagnostic);
                }
            }
        }
        
        // Check for undefined variables
        diagnostics.extend(self.check_undefined_variables(document)?);
        
        // Check for unused variables (if enabled)
        if self.config.detect_unused_variables {
            diagnostics.extend(self.check_unused_variables(document)?);
        }
        
        Ok(diagnostics)
    }
    
    /// Check style issues
    fn check_style(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        
        // Check naming conventions
        diagnostics.extend(self.check_naming_conventions(document)?);
        
        // Check indentation
        diagnostics.extend(self.check_indentation(document)?);
        
        // Check line length
        diagnostics.extend(self.check_line_length(document)?);
        
        Ok(diagnostics)
    }
    
    /// Check performance issues
    fn check_performance(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        
        // Check for inefficient patterns
        diagnostics.extend(self.check_inefficient_patterns(document)?);
        
        // Check for tail call opportunities
        diagnostics.extend(self.check_tail_call_opportunities(document)?);
        
        Ok(diagnostics)
    }
    
    /// Convert lex error to diagnostic
    fn lex_error_to_diagnostic(
        &self,
        error: &LexError,
        document: &crate::lsp::document::Document,
    ) -> Option<Diagnostic> {
        // Extract position information from error if available
        let range = Range::at_position(Position::new(0, 0)); // Simplified
        
        Some(Diagnostic {
            range,
            severity: DiagnosticSeverity::Error,
            message: format!("Lexical error: {}", error),
            code: Some("LEX001".to_string()),
            source: "lambdust".to_string(),
            related_information: Vec::new(),
            tags: Vec::new(),
        })
    }
    
    /// Convert parse error to diagnostic
    fn parse_error_to_diagnostic(
        &self,
        error: &ParseError,
        document: &crate::lsp::document::Document,
    ) -> Option<Diagnostic> {
        // Extract position information from error if available
        let range = Range::at_position(Position::new(0, 0)); // Simplified
        
        Some(Diagnostic {
            range,
            severity: DiagnosticSeverity::Error,
            message: format!("Parse error: {}", error),
            code: Some("PARSE001".to_string()),
            source: "lambdust".to_string(),
            related_information: Vec::new(),
            tags: Vec::new(),
        })
    }
    
    /// Convert runtime error to diagnostic
    fn runtime_error_to_diagnostic(
        &self,
        error: &LambdustError,
        document: &crate::lsp::document::Document,
    ) -> Option<Diagnostic> {
        let range = Range::at_position(Position::new(0, 0)); // Simplified
        
        Some(Diagnostic {
            range,
            severity: DiagnosticSeverity::Error,
            message: format!("Runtime error: {}", error),
            code: Some("RUNTIME001".to_string()),
            source: "lambdust".to_string(),
            related_information: Vec::new(),
            tags: Vec::new(),
        })
    }
    
    /// Check bracket balance
    fn check_bracket_balance(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        let mut bracket_stack = Vec::new();
        let mut position = Position::new(0, 0);
        
        for ch in content.chars() {
            match ch {
                '(' => bracket_stack.push((ch, position)),
                ')' => {
                    if bracket_stack.is_empty() {
                        diagnostics.push(Diagnostic {
                            range: Range::at_position(position),
                            severity: DiagnosticSeverity::Error,
                            message: "Unmatched closing parenthesis".to_string(),
                            code: Some("BRACKET001".to_string()),
                            source: "lambdust".to_string(),
                            related_information: Vec::new(),
                            tags: Vec::new(),
                        });
                    } else {
                        bracket_stack.pop();
                    }
                },
                '\n' => {
                    position.line += 1;
                    position.character = 0;
                    continue;
                },
                _ => {},
            }
            position.character += 1;
        }
        
        // Check for unclosed brackets
        for (bracket, pos) in bracket_stack {
            diagnostics.push(Diagnostic {
                range: Range::at_position(pos),
                severity: DiagnosticSeverity::Error,
                message: format!("Unclosed bracket: {}", bracket),
                code: Some("BRACKET002".to_string()),
                source: "lambdust".to_string(),
                related_information: Vec::new(),
                tags: Vec::new(),
            });
        }
        
        Ok(diagnostics)
    }
    
    /// Check string literals
    fn check_string_literals(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        let mut in_string = false;
        let mut string_start = Position::new(0, 0);
        let mut position = Position::new(0, 0);
        let mut escaped = false;
        
        for ch in content.chars() {
            match ch {
                '"' if !escaped => {
                    if in_string {
                        in_string = false;
                    } else {
                        in_string = true;
                        string_start = position;
                    }
                },
                '\\' if in_string && !escaped => {
                    escaped = true;
                },
                '\n' => {
                    if in_string {
                        diagnostics.push(Diagnostic {
                            range: Range::new(string_start, position),
                            severity: DiagnosticSeverity::Error,
                            message: "Unterminated string literal".to_string(),
                            code: Some("STRING001".to_string()),
                            source: "lambdust".to_string(),
                            related_information: Vec::new(),
                            tags: Vec::new(),
                        });
                        in_string = false;
                    }
                    position.line += 1;
                    position.character = 0;
                    escaped = false;
                    continue;
                },
                _ => {
                    escaped = false;
                },
            }
            position.character += 1;
        }
        
        // Check for unterminated string at end of file
        if in_string {
            diagnostics.push(Diagnostic {
                range: Range::new(string_start, position),
                severity: DiagnosticSeverity::Error,
                message: "Unterminated string literal at end of file".to_string(),
                code: Some("STRING002".to_string()),
                source: "lambdust".to_string(),
                related_information: Vec::new(),
                tags: Vec::new(),
            });
        }
        
        Ok(diagnostics)
    }
    
    /// Check for undefined variables
    fn check_undefined_variables(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        // TODO: Implement variable binding analysis
        Ok(Vec::new())
    }
    
    /// Check for unused variables
    fn check_unused_variables(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        // TODO: Implement unused variable detection
        Ok(Vec::new())
    }
    
    /// Check naming conventions
    fn check_naming_conventions(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        // TODO: Implement naming convention checks
        Ok(diagnostics)
    }
    
    /// Check indentation
    fn check_indentation(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        // TODO: Implement indentation checks
        Ok(diagnostics)
    }
    
    /// Check line length
    fn check_line_length(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let max_line_length = 100; // Configurable
        
        for (line_number, line) in document.get_content().lines().enumerate() {
            if line.len() > max_line_length {
                diagnostics.push(Diagnostic {
                    range: Range::new(
                        Position::new(line_number as u32, max_line_length as u32),
                        Position::new(line_number as u32, line.len() as u32),
                    ),
                    severity: DiagnosticSeverity::Information,
                    message: format!("Line exceeds {} characters", max_line_length),
                    code: Some("STYLE001".to_string()),
                    source: "lambdust".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }
        
        Ok(diagnostics)
    }
    
    /// Check for inefficient patterns
    fn check_inefficient_patterns(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        // TODO: Implement performance pattern detection
        Ok(diagnostics)
    }
    
    /// Check for tail call opportunities
    fn check_tail_call_opportunities(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        // TODO: Implement tail call analysis
        Ok(diagnostics)
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: DiagnosticsConfig) {
        self.config = config;
    }
    
    /// Clear diagnostics cache
    pub fn clear_cache(&mut self) {
        self.diagnostics_cache.clear();
    }
}

impl Diagnostic {
    /// Convert to LSP diagnostic
    pub fn to_lsp_diagnostic(&self) -> lsp_types::Diagnostic {
        use crate::lsp::position::PositionUtils;
        
        lsp_types::Diagnostic {
            range: PositionUtils::range_to_lsp_range(self.range),
            severity: Some(match self.severity {
                DiagnosticSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
                DiagnosticSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
                DiagnosticSeverity::Information => lsp_types::DiagnosticSeverity::INFORMATION,
                DiagnosticSeverity::Hint => lsp_types::DiagnosticSeverity::HINT,
            }),
            code: self.code.as_ref().map(|c| lsp_types::NumberOrString::String(c.clone())),
            code_description: None,
            source: Some(self.source.clone()),
            message: self.message.clone(),
            related_information: if self.related_information.is_empty() {
                None
            } else {
                Some(self.related_information.iter().map(|info| {
                    lsp_types::DiagnosticRelatedInformation {
                        location: lsp_types::Location {
                            uri: info.location.uri.parse().unwrap_or_default(),
                            range: PositionUtils::range_to_lsp_range(info.location.range),
                        },
                        message: info.message.clone(),
                    }
                }).collect())
            },
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags.iter().map(|tag| match tag {
                    DiagnosticTag::Unnecessary => lsp_types::DiagnosticTag::UNNECESSARY,
                    DiagnosticTag::Deprecated => lsp_types::DiagnosticTag::DEPRECATED,
                }).collect())
            },
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_engine_creation() {
        let interpreter = LambdustInterpreter::new();
        let engine = DiagnosticsEngine::new(interpreter);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_diagnostic_severity() {
        let diagnostic = Diagnostic {
            range: Range::at_position(Position::new(0, 0)),
            severity: DiagnosticSeverity::Error,
            message: "Test error".to_string(),
            code: Some("TEST001".to_string()),
            source: "test".to_string(),
            related_information: Vec::new(),
            tags: Vec::new(),
        };
        
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.message, "Test error");
    }
}