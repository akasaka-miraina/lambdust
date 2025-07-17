//! Diagnostics engine for Lambdust LSP
//!
//! This module provides real-time syntax checking, semantic analysis,
//! and error reporting with precise location information for IDE integration.

use crate::error::{LambdustError, Result};
use crate::interpreter::LambdustInterpreter;
use crate::lexer::{lex, LexError};
use crate::lsp::position::{Position, Range};
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
        error: &LambdustError,
        document: &crate::lsp::document::Document,
    ) -> Option<Diagnostic> {
        match error {
            LambdustError::ParseError { message, location } => {
                let range = self.source_span_to_range(location);
                
                Some(Diagnostic {
                    range,
                    severity: DiagnosticSeverity::Error,
                    message: message.clone(),
                    code: Some("PARSE001".to_string()),
                    source: "lambdust".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                })
            }
            _ => None,
        }
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
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        // Parse the content to extract symbol usage
        match parse(content) {
            Ok(expressions) => {
                let mut defined_vars = std::collections::HashSet::new();
                let mut used_vars = std::collections::HashSet::new();
                
                // First pass: collect all defined variables
                for expr in &expressions {
                    self.collect_defined_variables(expr, &mut defined_vars);
                }
                
                // Add built-in variables
                self.add_builtin_variables(&mut defined_vars);
                
                // Second pass: collect all used variables
                for expr in &expressions {
                    self.collect_used_variables(expr, &mut used_vars);
                }
                
                // Find undefined variables
                for used_var in used_vars {
                    if !defined_vars.contains(&used_var) {
                        // Find the location of the undefined variable
                        if let Some(position) = self.find_variable_position(content, &used_var) {
                            diagnostics.push(Diagnostic {
                                range: Range::at_position(position),
                                severity: DiagnosticSeverity::Error,
                                message: format!("Undefined variable: {}", used_var),
                                code: Some("UNDEF001".to_string()),
                                source: "lambdust".to_string(),
                                related_information: Vec::new(),
                                tags: Vec::new(),
                            });
                        }
                    }
                }
            }
            Err(_) => {
                // If parsing fails, we can't do undefined variable analysis
            }
        }
        
        Ok(diagnostics)
    }
    
    /// Check for unused variables
    fn check_unused_variables(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        match parse(content) {
            Ok(expressions) => {
                let mut defined_vars = std::collections::HashMap::new();
                let mut used_vars = std::collections::HashSet::new();
                
                // Collect defined variables with their positions
                for expr in &expressions {
                    self.collect_defined_variables_with_positions(expr, &mut defined_vars, content);
                }
                
                // Collect used variables
                for expr in &expressions {
                    self.collect_used_variables(expr, &mut used_vars);
                }
                
                // Find unused variables
                for (var_name, position) in defined_vars {
                    if !used_vars.contains(&var_name) && !self.is_builtin_or_export(&var_name) {
                        diagnostics.push(Diagnostic {
                            range: Range::at_position(position),
                            severity: DiagnosticSeverity::Information,
                            message: format!("Unused variable: {}", var_name),
                            code: Some("UNUSED001".to_string()),
                            source: "lambdust".to_string(),
                            related_information: Vec::new(),
                            tags: vec![DiagnosticTag::Unnecessary],
                        });
                    }
                }
            }
            Err(_) => {
                // If parsing fails, we can't do unused variable analysis
            }
        }
        
        Ok(diagnostics)
    }
    
    /// Check naming conventions
    fn check_naming_conventions(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        // Check for proper predicate naming (should end with ?)
        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = self.extract_function_definitions(line) {
                for (name, position) in captures {
                    // Check predicate naming convention
                    if self.looks_like_predicate(&name) && !name.ends_with('?') {
                        diagnostics.push(Diagnostic {
                            range: Range::at_position(Position::new(line_num as u32, position as u32)),
                            severity: DiagnosticSeverity::Information,
                            message: format!("Predicate function '{}' should end with '?'", name),
                            code: Some("STYLE002".to_string()),
                            source: "lambdust".to_string(),
                            related_information: Vec::new(),
                            tags: Vec::new(),
                        });
                    }
                    
                    // Check for CamelCase (should use kebab-case)
                    if name.chars().any(|c| c.is_uppercase()) {
                        diagnostics.push(Diagnostic {
                            range: Range::at_position(Position::new(line_num as u32, position as u32)),
                            severity: DiagnosticSeverity::Information,
                            message: format!("Use kebab-case instead of CamelCase for '{}'", name),
                            code: Some("STYLE003".to_string()),
                            source: "lambdust".to_string(),
                            related_information: Vec::new(),
                            tags: Vec::new(),
                        });
                    }
                    
                    // Check for underscore usage (should use hyphens)
                    if name.contains('_') {
                        diagnostics.push(Diagnostic {
                            range: Range::at_position(Position::new(line_num as u32, position as u32)),
                            severity: DiagnosticSeverity::Information,
                            message: format!("Use hyphens instead of underscores in '{}'", name),
                            code: Some("STYLE004".to_string()),
                            source: "lambdust".to_string(),
                            related_information: Vec::new(),
                            tags: Vec::new(),
                        });
                    }
                }
            }
        }
        
        Ok(diagnostics)
    }
    
    /// Check indentation
    fn check_indentation(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        let mut expected_indent = 0;
        let mut paren_stack = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue; // Skip empty lines and comments
            }
            
            let actual_indent = line.len() - trimmed.len();
            
            // Update expected indentation based on parentheses
            for ch in line.chars() {
                match ch {
                    '(' => {
                        paren_stack.push(expected_indent);
                        expected_indent += 2; // Standard 2-space indentation
                    }
                    ')' => {
                        if let Some(prev_indent) = paren_stack.pop() {
                            expected_indent = prev_indent;
                        }
                    }
                    _ => {}
                }
            }
            
            // Check if indentation is correct
            if actual_indent != expected_indent && !paren_stack.is_empty() {
                diagnostics.push(Diagnostic {
                    range: Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, actual_indent as u32),
                    ),
                    severity: DiagnosticSeverity::Information,
                    message: format!("Expected {} spaces, found {}", expected_indent, actual_indent),
                    code: Some("STYLE005".to_string()),
                    source: "lambdust".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }
        
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
        let content = document.get_content();
        
        for (line_num, line) in content.lines().enumerate() {
            // Check for inefficient list operations
            if line.contains("(append") && line.contains("(cons") {
                diagnostics.push(Diagnostic {
                    range: Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, line.len() as u32),
                    ),
                    severity: DiagnosticSeverity::Information,
                    message: "Consider using more efficient list construction patterns".to_string(),
                    code: Some("PERF001".to_string()),
                    source: "lambdust".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
            
            // Check for nested map operations
            if line.matches("(map").count() > 1 {
                diagnostics.push(Diagnostic {
                    range: Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, line.len() as u32),
                    ),
                    severity: DiagnosticSeverity::Information,
                    message: "Nested map operations can be combined for better performance".to_string(),
                    code: Some("PERF002".to_string()),
                    source: "lambdust".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
            
            // Check for repeated calculations
            if line.contains("(length") && line.matches("(length").count() > 1 {
                diagnostics.push(Diagnostic {
                    range: Range::new(
                        Position::new(line_num as u32, 0),
                        Position::new(line_num as u32, line.len() as u32),
                    ),
                    severity: DiagnosticSeverity::Information,
                    message: "Consider caching length calculation to avoid repeated computation".to_string(),
                    code: Some("PERF003".to_string()),
                    source: "lambdust".to_string(),
                    related_information: Vec::new(),
                    tags: Vec::new(),
                });
            }
        }
        
        Ok(diagnostics)
    }
    
    /// Check for tail call opportunities
    fn check_tail_call_opportunities(&self, document: &crate::lsp::document::Document) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let content = document.get_content();
        
        match parse(content) {
            Ok(expressions) => {
                for expr in expressions {
                    self.analyze_tail_call_opportunities(expr, &mut diagnostics, 0);
                }
            }
            Err(_) => {
                // If parsing fails, we can't do tail call analysis
            }
        }
        
        Ok(diagnostics)
    }
    
    /// Analyze expression for tail call opportunities
    fn analyze_tail_call_opportunities(
        &self,
        expr: crate::ast::Expr,
        diagnostics: &mut Vec<Diagnostic>,
        line_num: u32,
    ) {
        use crate::ast::Expr;
        
        match expr {
            Expr::List(exprs) if exprs.len() >= 2 => {
                if let Expr::Variable(name) = &exprs[0] {
                    match name.as_str() {
                        "define" if exprs.len() >= 3 => {
                            // Check if this is a function definition
                            if let Expr::List(_) = &exprs[1] {
                                // Check the function body for non-tail recursion
                                if let Some(func_name) = self.extract_function_name(&exprs[1]) {
                                    if self.has_non_tail_recursion(&exprs[2], &func_name) {
                                        diagnostics.push(Diagnostic {
                                            range: Range::at_position(Position::new(line_num, 0)),
                                            severity: DiagnosticSeverity::Information,
                                            message: format!(
                                                "Function '{}' can be optimized with tail call optimization",
                                                func_name
                                            ),
                                            code: Some("PERF004".to_string()),
                                            source: "lambdust".to_string(),
                                            related_information: Vec::new(),
                                            tags: Vec::new(),
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                
                // Recursively analyze sub-expressions
                for (i, sub_expr) in exprs.into_iter().enumerate() {
                    self.analyze_tail_call_opportunities(sub_expr, diagnostics, line_num + i as u32);
                }
            }
            _ => {}
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: DiagnosticsConfig) {
        self.config = config;
    }
    
    /// Clear diagnostics cache
    pub fn clear_cache(&mut self) {
        self.diagnostics_cache.clear();
    }
    
    /// Convert source span to LSP range
    fn source_span_to_range(&self, span: &crate::error::SourceSpan) -> Range {
        Range::new(
            Position::new(
                span.start.line.saturating_sub(1) as u32, // Convert to 0-based
                span.start.column.saturating_sub(1) as u32, // Convert to 0-based
            ),
            Position::new(
                span.end.line.saturating_sub(1) as u32, // Convert to 0-based
                span.end.column.saturating_sub(1) as u32, // Convert to 0-based
            ),
        )
    }
    
    /// Collect defined variables from expression
    fn collect_defined_variables(&self, expr: &crate::ast::Expr, defined: &mut std::collections::HashSet<String>) {
        use crate::ast::Expr;
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    match name.as_str() {
                        "define" if exprs.len() >= 3 => {
                            if let Expr::Variable(var_name) = &exprs[1] {
                                defined.insert(var_name.clone());
                            } else if let Expr::List(def_exprs) = &exprs[1] {
                                if let Some(Expr::Variable(func_name)) = def_exprs.first() {
                                    defined.insert(func_name.clone());
                                }
                            }
                        }
                        "let" | "let*" | "letrec" if exprs.len() >= 3 => {
                            if let Expr::List(bindings) = &exprs[1] {
                                for binding in bindings {
                                    if let Expr::List(binding_exprs) = binding {
                                        if let Some(Expr::Variable(var_name)) = binding_exprs.first() {
                                            defined.insert(var_name.clone());
                                        }
                                    }
                                }
                            }
                        }
                        "lambda" if exprs.len() >= 3 => {
                            if let Expr::List(params) = &exprs[1] {
                                for param in params {
                                    if let Expr::Variable(param_name) = param {
                                        defined.insert(param_name.clone());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                
                // Recursively check sub-expressions
                for sub_expr in exprs {
                    self.collect_defined_variables(sub_expr, defined);
                }
            }
            _ => {}
        }
    }
    
    /// Collect used variables from expression
    fn collect_used_variables(&self, expr: &crate::ast::Expr, used: &mut std::collections::HashSet<String>) {
        use crate::ast::Expr;
        match expr {
            Expr::Variable(name) => {
                used.insert(name.clone());
            }
            Expr::List(exprs) => {
                for sub_expr in exprs {
                    self.collect_used_variables(sub_expr, used);
                }
            }
            _ => {}
        }
    }
    
    /// Collect defined variables with their positions
    fn collect_defined_variables_with_positions(
        &self,
        expr: &crate::ast::Expr,
        defined: &mut std::collections::HashMap<String, Position>,
        content: &str,
    ) {
        use crate::ast::Expr;
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    match name.as_str() {
                        "define" if exprs.len() >= 3 => {
                            if let Expr::Variable(var_name) = &exprs[1] {
                                if let Some(pos) = self.find_variable_position(content, var_name) {
                                    defined.insert(var_name.clone(), pos);
                                }
                            }
                        }
                        "let" | "let*" | "letrec" if exprs.len() >= 3 => {
                            if let Expr::List(bindings) = &exprs[1] {
                                for binding in bindings {
                                    if let Expr::List(binding_exprs) = binding {
                                        if let Some(Expr::Variable(var_name)) = binding_exprs.first() {
                                            if let Some(pos) = self.find_variable_position(content, var_name) {
                                                defined.insert(var_name.clone(), pos);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                
                // Recursively check sub-expressions
                for sub_expr in exprs {
                    self.collect_defined_variables_with_positions(sub_expr, defined, content);
                }
            }
            _ => {}
        }
    }
    
    /// Add built-in variables to the defined set
    fn add_builtin_variables(&self, defined: &mut std::collections::HashSet<String>) {
        // Add common Scheme built-ins
        let builtins = [
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
            "cons", "car", "cdr", "list", "null?", "pair?",
            "not", "and", "or", "if", "cond", "case",
            "eq?", "eqv?", "equal?", "symbol?", "number?",
            "string?", "boolean?", "procedure?",
            "append", "reverse", "length", "member", "assoc",
            "map", "for-each", "apply", "eval",
            "display", "newline", "write", "read",
            "vector", "vector?", "vector-ref", "vector-set!",
            "string", "string-ref", "string-set!", "string-length",
            "substring", "string-append",
        ];
        
        for builtin in &builtins {
            defined.insert(builtin.to_string());
        }
    }
    
    /// Find the position of a variable in the content
    fn find_variable_position(&self, content: &str, var_name: &str) -> Option<Position> {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(col) = line.find(var_name) {
                return Some(Position::new(line_num as u32, col as u32));
            }
        }
        None
    }
    
    /// Check if a variable is a built-in or export
    fn is_builtin_or_export(&self, var_name: &str) -> bool {
        // Check if it's a built-in
        matches!(var_name,
            "+" | "-" | "*" | "/" | "=" | "<" | ">" | "<=" | ">=" |
            "cons" | "car" | "cdr" | "list" | "null?" | "pair?" |
            "not" | "and" | "or" | "if" | "cond" | "case" |
            "eq?" | "eqv?" | "equal?" | "symbol?" | "number?" |
            "string?" | "boolean?" | "procedure?" |
            "append" | "reverse" | "length" | "member" | "assoc" |
            "map" | "for-each" | "apply" | "eval" |
            "display" | "newline" | "write" | "read" |
            "vector" | "vector?" | "vector-ref" | "vector-set!" |
            "string" | "string-ref" | "string-set!" | "string-length" |
            "substring" | "string-append"
        ) || var_name.starts_with("export-") // Convention for exported variables
    }
    
    /// Extract function definitions from a line
    fn extract_function_definitions(&self, line: &str) -> Option<Vec<(String, usize)>> {
        let mut results = Vec::new();
        
        // Look for (define (function-name ...)) patterns
        if line.trim().starts_with("(define (") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 3 {
                if let Some(name_part) = tokens[1].strip_prefix('(') {
                    let name = name_part.to_string();
                    if let Some(pos) = line.find(&name) {
                        results.push((name, pos));
                    }
                }
            }
        }
        
        // Look for (define function-name ...) patterns
        if line.trim().starts_with("(define ") && !line.trim().starts_with("(define (") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 3 {
                let name = tokens[1].to_string();
                if let Some(pos) = line.find(&name) {
                    results.push((name, pos));
                }
            }
        }
        
        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }
    
    /// Check if a name looks like a predicate
    fn looks_like_predicate(&self, name: &str) -> bool {
        // Common predicate patterns
        name.contains("is-") || 
        name.contains("has-") || 
        name.contains("can-") ||
        name.ends_with("-p") ||
        matches!(name, "null" | "zero" | "positive" | "negative" | "even" | "odd" | "empty")
    }
    
    /// Extract function name from define form
    fn extract_function_name(&self, expr: &crate::ast::Expr) -> Option<String> {
        use crate::ast::Expr;
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    Some(name.clone())
                } else {
                    None
                }
            }
            Expr::Variable(name) => Some(name.clone()),
            _ => None,
        }
    }
    
    /// Check if expression has non-tail recursion
    fn has_non_tail_recursion(&self, expr: &crate::ast::Expr, func_name: &str) -> bool {
        use crate::ast::Expr;
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                // Check if this is a recursive call in non-tail position
                if let Expr::Variable(name) = &exprs[0] {
                    if name == func_name {
                        // This is a recursive call - check if it's in tail position
                        // For simplicity, assume it's non-tail if it's part of an arithmetic operation
                        return false; // This would need more sophisticated analysis
                    }
                }
                
                // Check for recursive calls in arithmetic operations (non-tail)
                if let Expr::Variable(op) = &exprs[0] {
                    if matches!(op.as_str(), "+" | "-" | "*" | "/" | "cons" | "append") {
                        for arg in &exprs[1..] {
                            if self.contains_recursive_call(arg, func_name) {
                                return true; // Non-tail recursion found
                            }
                        }
                    }
                }
                
                // Recursively check sub-expressions
                for expr in exprs {
                    if self.has_non_tail_recursion(expr, func_name) {
                        return true;
                    }
                }
                
                false
            }
            _ => false,
        }
    }
    
    /// Check if expression contains a recursive call
    fn contains_recursive_call(&self, expr: &crate::ast::Expr, func_name: &str) -> bool {
        use crate::ast::Expr;
        match expr {
            Expr::Variable(name) => name == func_name,
            Expr::List(exprs) => {
                for expr in exprs {
                    if self.contains_recursive_call(expr, func_name) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
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
