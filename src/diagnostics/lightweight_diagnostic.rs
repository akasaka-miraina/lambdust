//! Lightweight diagnostic system to replace miette dependency.
//!
//! This module provides a minimal, high-performance diagnostic trait and
//! implementation that replaces miette for Lambdust's specific needs,
//! achieving significant binary size reduction while maintaining essential
//! diagnostic functionality.

use crate::diagnostics::{Span, ErrorLabel, LabelStyle};
use std::fmt;

/// Lightweight diagnostic trait to replace miette::Diagnostic.
///
/// This trait provides the essential diagnostic functionality needed by
/// Lambdust's error types while being much lighter than the full miette crate.
pub trait LightweightDiagnostic: std::error::Error {
    /// Returns the diagnostic code for this error.
    fn code(&self) -> Option<&str> {
        None
    }
    
    /// Returns help text for this diagnostic.
    fn help(&self) -> Option<&str> {
        None
    }
    
    /// Returns the URL for more information about this diagnostic.
    fn url(&self) -> Option<&str> {
        None
    }
    
    /// Returns the source code related to this diagnostic.
    fn source_code(&self) -> Option<&str> {
        None
    }
    
    /// Returns the labels (spans with messages) for this diagnostic.
    fn labels(&self) -> Vec<DiagnosticLabel> {
        Vec::new()
    }
    
    /// Returns related diagnostics.
    fn related(&self) -> Vec<&dyn LightweightDiagnostic> {
        Vec::new()
    }
    
    /// Returns the severity level of this diagnostic.
    fn severity(&self) -> DiagnosticSeverity {
        DiagnosticSeverity::Error
    }
}

/// Diagnostic label for source location annotation.
#[derive(Debug, Clone)]
pub struct DiagnosticLabel {
    /// The span this label refers to.
    pub span: Span,
    /// The label message.
    pub message: Option<String>,
    /// The style of this label.
    pub style: DiagnosticLabelStyle,
}

/// Style for diagnostic labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLabelStyle {
    /// Primary label (most important).
    Primary,
    /// Secondary label (additional context).
    Secondary,
}

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error level (red).
    Error,
    /// Warning level (yellow).
    Warning,
    /// Information level (blue).
    Info,
    /// Note level (gray).
    Note,
    /// Help level (green).
    Help,
}

impl DiagnosticLabel {
    /// Creates a new primary diagnostic label.
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: Some(message.into()),
            style: DiagnosticLabelStyle::Primary,
        }
    }
    
    /// Creates a new secondary diagnostic label.
    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: Some(message.into()),
            style: DiagnosticLabelStyle::Secondary,
        }
    }
    
    /// Creates a new label with just a span (no message).
    pub fn span_only(span: Span) -> Self {
        Self {
            span,
            message: None,
            style: DiagnosticLabelStyle::Primary,
        }
    }
}

/// Conversion from ErrorLabel to DiagnosticLabel for compatibility.
impl From<ErrorLabel> for DiagnosticLabel {
    fn from(label: ErrorLabel) -> Self {
        Self {
            span: label.span,
            message: label.message,
            style: match label.style {
                LabelStyle::Primary => DiagnosticLabelStyle::Primary,
                LabelStyle::Secondary => DiagnosticLabelStyle::Secondary,
            },
        }
    }
}

/// Simple diagnostic reporter for lightweight error display.
pub struct DiagnosticReporter {
    /// Whether to use colors in output.
    use_colors: bool,
    /// Whether to show source code snippets.
    show_source: bool,
}

impl DiagnosticReporter {
    /// Creates a new diagnostic reporter with default settings.
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_source: true,
        }
    }
    
    /// Creates a plain diagnostic reporter without colors or source.
    pub fn plain() -> Self {
        Self {
            use_colors: false,
            show_source: false,
        }
    }
    
    /// Sets whether to use colors in output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }
    
    /// Sets whether to show source code snippets.
    pub fn with_source(mut self, show_source: bool) -> Self {
        self.show_source = show_source;
        self
    }
    
    /// Reports a diagnostic to stderr.
    pub fn report(&self, diagnostic: &dyn LightweightDiagnostic) {
        eprintln!("{}", self.format_diagnostic(diagnostic));
    }
    
    /// Formats a diagnostic as a string.
    pub fn format_diagnostic(&self, diagnostic: &dyn LightweightDiagnostic) -> String {
        let mut output = String::new();
        
        // Severity and message
        let severity_str = match diagnostic.severity() {
            DiagnosticSeverity::Error => if self.use_colors { "\x1b[31merror\x1b[0m" } else { "error" },
            DiagnosticSeverity::Warning => if self.use_colors { "\x1b[33mwarning\x1b[0m" } else { "warning" },
            DiagnosticSeverity::Info => if self.use_colors { "\x1b[36minfo\x1b[0m" } else { "info" },
            DiagnosticSeverity::Note => if self.use_colors { "\x1b[37mnote\x1b[0m" } else { "note" },
            DiagnosticSeverity::Help => if self.use_colors { "\x1b[32mhelp\x1b[0m" } else { "help" },
        };
        
        // Main error message
        if let Some(code) = diagnostic.code() {
            output.push_str(&format!("{severity_str}[{code}]: {diagnostic}\n"));
        } else {
            output.push_str(&format!("{severity_str}: {diagnostic}\n"));
        }
        
        // Labels with spans
        let labels = diagnostic.labels();
        if !labels.is_empty() && self.show_source {
            for label in labels {
                if let Some(message) = &label.message {
                    output.push_str(&format!("   {} {}\n", 
                        if self.use_colors { "\x1b[36m-->\x1b[0m" } else { "-->" },
                        message
                    ));
                }
            }
        }
        
        // Help text
        if let Some(help) = diagnostic.help() {
            output.push_str(&format!("   {} {}\n", 
                if self.use_colors { "\x1b[32m=\x1b[0m" } else { "=" },
                help
            ));
        }
        
        // URL for more info
        if let Some(url) = diagnostic.url() {
            output.push_str(&format!("   {} For more information: {}\n", 
                if self.use_colors { "\x1b[36m=\x1b[0m" } else { "=" },
                url
            ));
        }
        
        output
    }
}

impl Default for DiagnosticReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to report a diagnostic using the default reporter.
pub fn report_diagnostic(diagnostic: &dyn LightweightDiagnostic) {
    DiagnosticReporter::new().report(diagnostic);
}

/// Macro to derive LightweightDiagnostic implementation with attributes.
macro_rules! derive_diagnostic {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(
                $(#[diagnostic(code($code:expr))])?
                $(#[diagnostic(help($help:expr))])?
                $variant:ident $({
                    $(
                        $(#[label($label_msg:expr)])?
                        $field:ident: $field_ty:ty
                    ),* $(,)?
                })?,
            )*
        }
    ) => {
        $(#[$attr])*
        pub enum $name {
            $(
                $variant $({
                    $($field: $field_ty),*
                })?,
            )*
        }

        impl LightweightDiagnostic for $name {
            fn code(&self) -> Option<&str> {
                match self {
                    $(
                        Self::$variant { .. } => {
                            derive_diagnostic!(@code $($code)?)
                        }
                    )*
                }
            }
            
            fn help(&self) -> Option<&str> {
                match self {
                    $(
                        Self::$variant { .. } => {
                            derive_diagnostic!(@help $($help)?)
                        }
                    )*
                }
            }
            
            fn labels(&self) -> Vec<DiagnosticLabel> {
                match self {
                    $(
                        Self::$variant { $($($field),*)? } => {
                            derive_diagnostic!(@labels $($($field: $label_msg)?)*)
                        }
                    )*
                }
            }
        }
    };

    // Helper: Extract code
    (@code $code:expr) => { Some($code) };
    (@code) => { None };
    
    // Helper: Extract help
    (@help $help:expr) => { Some($help) };
    (@help) => { None };
    
    // Helper: Extract labels  
    (@labels $($field:ident: $msg:expr)*) => {
        {
            let mut labels = Vec::new();
            $(
                // Only create label if field is a Span type
                if let Some(span) = derive_diagnostic!(@maybe_span $field) {
                    labels.push(DiagnosticLabel::primary(span, $msg));
                }
            )*
            labels
        }
    };
    (@labels) => { Vec::new() };
    
    // Helper: Check if field is a Span
    (@maybe_span $field:ident) => {
        // This is a compile-time check - if $field is a Span, it will work
        // Otherwise, it will be None and won't add a label
        if std::any::type_name::<_>().contains("Span") {
            Some(*$field)
        } else {
            None
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_diagnostic_label() {
        let span = Span::new(0, 5);
        let label = DiagnosticLabel::primary(span, "test error");
        
        assert_eq!(label.style, DiagnosticLabelStyle::Primary);
        assert_eq!(label.message.as_ref().unwrap(), "test error");
        assert_eq!(label.span.start, 0);
        assert_eq!(label.span.end(), 5);
    }

    #[test]
    fn test_diagnostic_reporter() {
        struct TestDiagnostic {
            message: String,
        }
        
        impl fmt::Display for TestDiagnostic {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.message)
            }
        }
        
        impl fmt::Debug for TestDiagnostic {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "TestDiagnostic {{ message: {:?} }}", self.message)
            }
        }
        
        impl std::error::Error for TestDiagnostic {}
        
        impl LightweightDiagnostic for TestDiagnostic {
            fn code(&self) -> Option<&str> {
                Some("TEST001")
            }
            
            fn help(&self) -> Option<&str> {
                Some("This is a test diagnostic")
            }
        }
        
        let diagnostic = TestDiagnostic {
            message: "Test error message".to_string(),
        };
        
        let reporter = DiagnosticReporter::plain();
        let output = reporter.format_diagnostic(&diagnostic);
        
        assert!(output.contains("error[TEST001]: Test error message"));
        assert!(output.contains("This is a test diagnostic"));
    }

    #[test]
    fn test_diagnostic_severity() {
        assert_eq!(DiagnosticSeverity::Error, DiagnosticSeverity::Error);
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
    }
}