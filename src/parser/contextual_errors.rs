//! Contextual error messages for improved developer experience
//!
//! This module provides intelligent error message generation based on parsing context,
//! offering helpful suggestions, corrections, and educational information to developers.

use crate::ast::{Expr, Formals};
use crate::diagnostics::{Error, Span};
use crate::lexer::{Token, TokenKind};
use std::collections::{HashMap, HashSet};

/// Context-aware error message generator
pub struct ContextualErrorGenerator {
    /// Error templates by context
    error_templates: HashMap<ErrorContext, Vec<ErrorTemplate>>,
    /// Common typo corrections
    typo_corrections: HashMap<String, Vec<String>>,
    /// Context-sensitive suggestions
    context_suggestions: HashMap<ErrorContext, Vec<String>>,
    /// Educational hints by error type
    educational_hints: HashMap<String, String>,
    /// R7RS compliance messages
    r7rs_compliance: HashMap<String, String>,
}

/// Error context information for message generation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ErrorContext {
    /// Top-level program context
    TopLevel,
    /// Inside a define form
    Define {
        /// Whether this is defining a function
        is_function_definition: bool,
        /// Whether the definition has a name
        has_name: bool,
        /// Whether the definition has parameters
        has_parameters: bool,
    },
    /// Inside a lambda form
    Lambda {
        /// Whether the lambda has parameters
        has_parameters: bool,
        /// Whether the lambda has a body
        has_body: bool,
    },
    /// Inside a conditional (if, cond, case)
    Conditional {
        /// Type of conditional form
        form_type: ConditionalType,
        /// Index of the current clause
        clause_index: usize,
    },
    /// Inside a binding form (let, let*, letrec)
    Binding {
        /// Type of binding form
        form_type: BindingType,
        /// Whether we're in the bindings section
        in_bindings: bool,
        /// Index of the current binding
        binding_index: usize,
    },
    /// Inside a function application
    Application {
        /// Whether the operator is a known function
        operator_known: bool,
        /// Name of the operator (if known)
        operator_name: Option<String>,
        /// Index of the current parameter
        parameter_index: usize,
    },
    /// Inside a special form
    SpecialForm {
        /// Name of the special form
        form_name: String,
        /// Index of the current argument
        argument_index: usize,
    },
    /// Inside a macro definition
    MacroDefinition {
        /// Whether the macro has a pattern
        has_pattern: bool,
        /// Whether the macro has a template
        has_template: bool,
    },
    /// Inside a library definition
    LibraryDefinition {
        /// Whether the library has a name
        has_name: bool,
        /// Whether we're in the exports section
        in_exports: bool,
        /// Whether we're in the imports section
        in_imports: bool,
    },
    /// Generic expression context
    Expression,
}

/// Types of conditional forms
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ConditionalType {
    /// If expression (if condition then else)
    If,
    /// Cond expression (multiple clauses)
    Cond,
    /// Case expression (pattern matching)
    Case,
    /// When expression (single-clause conditional)
    When,
    /// Unless expression (negated when)
    Unless,
}

/// Types of binding forms
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BindingType {
    /// Let expression (parallel binding)
    Let,
    /// Let* expression (sequential binding)
    LetStar,
    /// Letrec expression (recursive binding)
    Letrec,
    /// Letrec* expression (sequential recursive binding)
    LetrecStar,
    /// Let-values expression (multiple value binding)
    LetValues,
}

/// Error message template
#[derive(Debug, Clone)]
pub struct ErrorTemplate {
    /// Primary error message
    pub message: String,
    /// Detailed explanation
    pub explanation: Option<String>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Educational content
    pub educational: Option<String>,
    /// R7RS reference
    pub r7rs_reference: Option<String>,
    /// Examples of correct usage
    pub examples: Vec<String>,
    /// Priority for template selection (higher = more specific)
    pub priority: i32,
}

/// Enhanced error information with context
#[derive(Debug, Clone)]
pub struct ContextualError {
    /// Base error
    pub base_error: Error,
    /// Context where error occurred
    pub context: ErrorContext,
    /// Enhanced message
    pub enhanced_message: String,
    /// Suggestions for fixing
    pub suggestions: Vec<ErrorSuggestion>,
    /// Related spans (for multi-location errors)
    pub related_spans: Vec<(Span, String)>,
    /// Educational content
    pub educational_content: Option<String>,
    /// Quick fixes available
    pub quick_fixes: Vec<QuickFix>,
}

/// Error suggestion with confidence level
#[derive(Debug, Clone)]
pub struct ErrorSuggestion {
    /// Suggestion description
    pub description: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Category of suggestion
    pub category: SuggestionCategory,
    /// Code example (if applicable)
    pub code_example: Option<String>,
}

/// Categories of error suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionCategory {
    /// Syntax correction
    SyntaxCorrection,
    /// Typo correction
    TypoCorrection,
    /// Missing element
    MissingElement,
    /// Incorrect usage
    IncorrectUsage,
    /// Performance improvement
    Performance,
    /// Best practice
    BestPractice,
    /// R7RS compliance
    R7rsCompliance,
}

/// Quick fix action
#[derive(Debug, Clone)]
pub struct QuickFix {
    /// Fix description
    pub title: String,
    /// Text changes to apply
    pub changes: Vec<TextChange>,
    /// Whether this is a preferred fix
    pub preferred: bool,
}

/// Text change for quick fixes
#[derive(Debug, Clone)]
pub struct TextChange {
    /// Span to replace
    pub span: Span,
    /// New text
    pub new_text: String,
}

impl ContextualErrorGenerator {
    /// Creates a new contextual error generator
    pub fn new() -> Self {
        let mut generator = Self {
            error_templates: HashMap::new(),
            typo_corrections: HashMap::new(),
            context_suggestions: HashMap::new(),
            educational_hints: HashMap::new(),
            r7rs_compliance: HashMap::new(),
        };

        generator.init_error_templates();
        generator.init_typo_corrections();
        generator.init_educational_hints();
        generator.init_r7rs_compliance();
        generator
    }

    /// Generates a contextual error from a base error
    pub fn generate_contextual_error(
        &self,
        base_error: Error,
        context: ErrorContext,
        tokens: &[Token],
        position: usize,
    ) -> ContextualError {
        let enhanced_message =
            self.generate_enhanced_message(&base_error, &context, tokens, position);
        let suggestions = self.generate_suggestions(&base_error, &context, tokens, position);
        let related_spans = self.find_related_spans(&base_error, &context, tokens, position);
        let educational_content = self.get_educational_content(&base_error, &context);
        let quick_fixes = self.generate_quick_fixes(&base_error, &context, tokens, position);

        ContextualError {
            base_error,
            context,
            enhanced_message,
            suggestions,
            related_spans,
            educational_content,
            quick_fixes,
        }
    }

    /// Generates an enhanced error message
    fn generate_enhanced_message(
        &self,
        base_error: &Error,
        context: &ErrorContext,
        tokens: &[Token],
        position: usize,
    ) -> String {
        // Look for specific templates for this context
        if let Some(templates) = self.error_templates.get(context) {
            if let Some(template) = templates.first() {
                return self.format_template_message(template, base_error, tokens, position);
            }
        }

        // Generate context-specific message
        match context {
            ErrorContext::Define { is_function_definition: true, has_name, has_parameters } => {
                if !has_name {
                    "Function definition is missing a name. Define syntax: (define (name parameters) body)".to_string()
                } else if !has_parameters {
                    "Function definition has a name but no parameter list. Use (define (name param1 param2 ...) body)".to_string()
                } else {
                    format!("Error in function definition: {base_error}")
                }
            }
            ErrorContext::Define { is_function_definition: false, has_name, .. } => {
                if !has_name {
                    "Variable definition is missing a name. Define syntax: (define name value)".to_string()
                } else {
                    format!("Error in variable definition: {base_error}")
                }
            }
            ErrorContext::Lambda { has_parameters, has_body } => {
                if !has_parameters {
                    "Lambda expression is missing parameter list. Lambda syntax: (lambda (params) body)".to_string()
                } else if !has_body {
                    "Lambda expression has parameters but no body. Add an expression after the parameter list.".to_string()
                } else {
                    format!("Error in lambda expression: {base_error}")
                }
            }
            ErrorContext::Conditional { form_type, clause_index } => {
                match form_type {
                    ConditionalType::If => {
                        match clause_index {
                            0 => "The 'if' condition is malformed. If syntax: (if condition then-expr else-expr)".to_string(),
                            1 => "The 'then' branch of the 'if' is malformed.".to_string(),
                            2 => "The 'else' branch of the 'if' is malformed.".to_string(),
                            _ => format!("Error in 'if' expression: {base_error}"),
                        }
                    }
                    ConditionalType::Cond => {
                        format!("Error in 'cond' clause {clause_index}: {base_error}")
                    }
                    ConditionalType::Case => {
                        format!("Error in 'case' clause {clause_index}: {base_error}")
                    }
                    ConditionalType::When => {
                        format!("Error in 'when' expression: {base_error}")
                    }
                    ConditionalType::Unless => {
                        format!("Error in 'unless' expression: {base_error}")
                    }
                }
            }
            ErrorContext::Application { operator_known: true, operator_name: Some(name), parameter_index } => {
                format!("Error in argument {parameter_index} to function '{name}': {base_error}")
            }
            ErrorContext::Application { operator_known: false, parameter_index, .. } => {
                format!("Error in argument {parameter_index} to function call: {base_error}")
            }
            _ => { let desc = self.context_description(context); format!("Error in {desc}: {base_error}") },
        }
    }

    /// Generates suggestions for fixing the error
    fn generate_suggestions(
        &self,
        base_error: &Error,
        context: &ErrorContext,
        tokens: &[Token],
        position: usize,
    ) -> Vec<ErrorSuggestion> {
        let mut suggestions = Vec::new();

        // Context-specific suggestions
        match context {
            ErrorContext::Define {
                is_function_definition: true,
                has_name: false,
                ..
            } => {
                suggestions.push(ErrorSuggestion {
                    description: "Add a function name after 'define'".to_string(),
                    confidence: 0.9,
                    category: SuggestionCategory::MissingElement,
                    code_example: Some("(define (my-function param) body)".to_string()),
                });
            }
            ErrorContext::Lambda {
                has_parameters: false,
                ..
            } => {
                suggestions.push(ErrorSuggestion {
                    description: "Add parameter list to lambda".to_string(),
                    confidence: 0.9,
                    category: SuggestionCategory::MissingElement,
                    code_example: Some("(lambda (x) (* x x))".to_string()),
                });
            }
            ErrorContext::Application {
                operator_name: Some(name),
                ..
            } => {
                if let Some(corrections) = self.typo_corrections.get(name) {
                    for correction in corrections {
                        suggestions.push(ErrorSuggestion {
                            description: format!("Did you mean '{correction}'?"),
                            confidence: 0.8,
                            category: SuggestionCategory::TypoCorrection,
                            code_example: None,
                        });
                    }
                }
            }
            _ => {}
        }

        // Token-based suggestions
        if position < tokens.len() {
            let current_token = &tokens[position];
            suggestions.extend(self.generate_token_suggestions(current_token, context));
        }

        // General suggestions from context
        if let Some(context_suggestions) = self.context_suggestions.get(context) {
            for suggestion in context_suggestions {
                suggestions.push(ErrorSuggestion {
                    description: suggestion.clone(),
                    confidence: 0.6,
                    category: SuggestionCategory::BestPractice,
                    code_example: None,
                });
            }
        }

        suggestions
    }

    /// Generates token-specific suggestions
    pub fn generate_token_suggestions(
        &self,
        token: &Token,
        context: &ErrorContext,
    ) -> Vec<ErrorSuggestion> {
        let mut suggestions = Vec::new();

        match &token.kind {
            TokenKind::Identifier => {
                let text = token.text();

                // Check for common typos
                if let Some(corrections) = self.typo_corrections.get(text) {
                    for correction in corrections {
                        suggestions.push(ErrorSuggestion {
                            description: format!("Did you mean '{correction}'?"),
                            confidence: 0.85,
                            category: SuggestionCategory::TypoCorrection,
                            code_example: None,
                        });
                    }
                }

                // Context-specific identifier suggestions
                if context == &ErrorContext::TopLevel
                    && text.chars().next().is_some_and(|c| c.is_uppercase())
                {
                    suggestions.push(ErrorSuggestion {
                        description: "Scheme identifiers typically use lowercase".to_string(),
                        confidence: 0.7,
                        category: SuggestionCategory::BestPractice,
                        code_example: Some(text.to_lowercase()),
                    });
                }
            }
            TokenKind::LeftParen => {
                if matches!(context, ErrorContext::TopLevel) {
                    suggestions.push(ErrorSuggestion {
                        description: "Start with a special form or function call".to_string(),
                        confidence: 0.8,
                        category: SuggestionCategory::BestPractice,
                        code_example: Some("(define name value)".to_string()),
                    });
                }
            }
            TokenKind::String => {
                // Check for possible quote usage confusion
                suggestions.push(ErrorSuggestion {
                    description: "Use quote (') for symbols, strings for text data".to_string(),
                    confidence: 0.6,
                    category: SuggestionCategory::BestPractice,
                    code_example: Some("'symbol vs \"string\"".to_string()),
                });
            }
            _ => {}
        }

        suggestions
    }

    /// Finds related spans for multi-location errors
    fn find_related_spans(
        &self,
        base_error: &Error,
        context: &ErrorContext,
        tokens: &[Token],
        position: usize,
    ) -> Vec<(Span, String)> {
        let mut related = Vec::new();

        // Look for unmatched delimiters
        if position < tokens.len() {
            let current_token = &tokens[position];

            if current_token.kind == TokenKind::RightParen {
                // Find matching opening paren
                if let Some(open_pos) = self.find_matching_open_paren(tokens, position) {
                    related.push((
                        tokens[open_pos].span,
                        "Opening parenthesis here".to_string(),
                    ));
                }
            }
        }

        // Context-specific related information
        if let ErrorContext::Define { .. } = context {
            // Find the define keyword
            for (i, token) in tokens.iter().enumerate() {
                if token.kind == TokenKind::Identifier && token.text() == "define" {
                    related.push((token.span, "Definition starts here".to_string()));
                    break;
                }
            }
        }

        related
    }

    /// Gets educational content for the error
    fn get_educational_content(
        &self,
        base_error: &Error,
        context: &ErrorContext,
    ) -> Option<String> {
        // Check for specific educational content
        let error_type = match base_error {
            Error::ParseError { .. } => "parse_error",
            Error::LexError { .. } => "lex_error",
            _ => return None,
        };

        if let Some(hint) = self.educational_hints.get(error_type) {
            return Some(hint.clone());
        }

        // Context-specific educational content
        match context {
            ErrorContext::Define { .. } => Some(
                "The 'define' form creates variable or function bindings. \
                 For variables: (define name value). \
                 For functions: (define (name params) body)."
                    .to_string(),
            ),
            ErrorContext::Lambda { .. } => Some(
                "Lambda expressions create anonymous functions. \
                 Syntax: (lambda (parameters) body). \
                 Example: (lambda (x) (* x x)) creates a squaring function."
                    .to_string(),
            ),
            ErrorContext::Conditional {
                form_type: ConditionalType::If,
                ..
            } => Some(
                "The 'if' form evaluates a condition and chooses between two expressions. \
                 Syntax: (if condition then-expr else-expr). \
                 The else-expr is optional."
                    .to_string(),
            ),
            _ => None,
        }
    }

    /// Generates quick fixes for the error
    fn generate_quick_fixes(
        &self,
        base_error: &Error,
        context: &ErrorContext,
        tokens: &[Token],
        position: usize,
    ) -> Vec<QuickFix> {
        let mut fixes = Vec::new();

        match context {
            ErrorContext::Define {
                has_name: false, ..
            } => {
                if position < tokens.len() {
                    let span = tokens[position].span;
                    fixes.push(QuickFix {
                        title: "Add variable name".to_string(),
                        changes: vec![TextChange {
                            span: Span::new(span.start, 0),
                            new_text: "name ".to_string(),
                        }],
                        preferred: true,
                    });
                }
            }
            ErrorContext::Lambda {
                has_parameters: false,
                ..
            } => {
                if position < tokens.len() {
                    let span = tokens[position].span;
                    fixes.push(QuickFix {
                        title: "Add parameter list".to_string(),
                        changes: vec![TextChange {
                            span: Span::new(span.start, 0),
                            new_text: "(x) ".to_string(),
                        }],
                        preferred: true,
                    });
                }
            }
            _ => {}
        }

        // Generic fixes for common issues
        if position < tokens.len() {
            let token = &tokens[position];

            if token.kind == TokenKind::LeftParen {
                // Suggest closing paren
                fixes.push(QuickFix {
                    title: "Add closing parenthesis".to_string(),
                    changes: vec![TextChange {
                        span: Span::new(token.span.end(), 0),
                        new_text: ")".to_string(),
                    }],
                    preferred: false,
                });
            }
        }

        fixes
    }

    /// Initializes error templates
    fn init_error_templates(&mut self) {
        // Define templates
        let define_template = ErrorTemplate {
            message: "Malformed define expression".to_string(),
            explanation: Some("The define form requires specific syntax".to_string()),
            suggestions: vec![
                "For variables: (define name value)".to_string(),
                "For functions: (define (name params) body)".to_string(),
            ],
            educational: Some("Define creates bindings in the current scope".to_string()),
            r7rs_reference: Some("R7RS Section 4.1.5".to_string()),
            examples: vec![
                "(define pi 3.14159)".to_string(),
                "(define (square x) (* x x))".to_string(),
            ],
            priority: 100,
        };

        self.error_templates.insert(
            ErrorContext::Define {
                is_function_definition: false,
                has_name: false,
                has_parameters: false,
            },
            vec![define_template],
        );
    }

    /// Initializes typo corrections
    fn init_typo_corrections(&mut self) {
        let corrections = vec![
            ("defin", vec!["define"]),
            ("lamda", vec!["lambda"]),
            ("lamba", vec!["lambda"]),
            ("iff", vec!["if"]),
            ("els", vec!["else"]),
            ("lest", vec!["let"]),
            ("conde", vec!["cond"]),
            ("cas", vec!["case"]),
            ("beginn", vec!["begin"]),
            ("qoute", vec!["quote"]),
            ("sett", vec!["set!"]),
        ];

        for (typo, fixes) in corrections {
            self.typo_corrections.insert(
                typo.to_string(),
                fixes.into_iter().map(String::from).collect(),
            );
        }
    }

    /// Initializes educational hints
    fn init_educational_hints(&mut self) {
        self.educational_hints.insert(
            "parse_error".to_string(),
            "Parse errors occur when the source code doesn't match the expected grammar. \
             Check for missing or extra parentheses, quotes, or other delimiters."
                .to_string(),
        );

        self.educational_hints.insert(
            "lex_error".to_string(),
            "Lexical errors occur when the tokenizer encounters invalid characters or sequences. \
             Check for invalid escape sequences or unterminated strings."
                .to_string(),
        );
    }

    /// Initializes R7RS compliance information
    fn init_r7rs_compliance(&mut self) {
        self.r7rs_compliance.insert(
            "define".to_string(),
            "R7RS Section 4.1.5: Variable definitions and function definitions use the define form.".to_string()
        );

        self.r7rs_compliance.insert(
            "lambda".to_string(),
            "R7RS Section 4.1.4: Lambda expressions create procedures.".to_string(),
        );
    }

    /// Helper methods
    fn format_template_message(
        &self,
        template: &ErrorTemplate,
        base_error: &Error,
        tokens: &[Token],
        position: usize,
    ) -> String {
        // Simple template formatting - could be enhanced with placeholders
        template.message.clone()
    }

    fn context_description(&self, context: &ErrorContext) -> String {
        match context {
            ErrorContext::TopLevel => "top-level expression",
            ErrorContext::Define { .. } => "define form",
            ErrorContext::Lambda { .. } => "lambda expression",
            ErrorContext::Conditional { form_type, .. } => match form_type {
                ConditionalType::If => "if expression",
                ConditionalType::Cond => "cond expression",
                ConditionalType::Case => "case expression",
                ConditionalType::When => "when expression",
                ConditionalType::Unless => "unless expression",
            },
            ErrorContext::Binding { form_type, .. } => match form_type {
                BindingType::Let => "let binding",
                BindingType::LetStar => "let* binding",
                BindingType::Letrec => "letrec binding",
                BindingType::LetrecStar => "letrec* binding",
                BindingType::LetValues => "let-values binding",
            },
            ErrorContext::Application { .. } => "function application",
            ErrorContext::SpecialForm { form_name, .. } => form_name,
            ErrorContext::MacroDefinition { .. } => "macro definition",
            ErrorContext::LibraryDefinition { .. } => "library definition",
            ErrorContext::Expression => "expression",
        }
        .to_string()
    }

    fn find_matching_open_paren(&self, tokens: &[Token], close_pos: usize) -> Option<usize> {
        let mut paren_count = 1;
        let mut pos = close_pos;

        while pos > 0 {
            pos -= 1;
            match tokens[pos].kind {
                TokenKind::RightParen => paren_count += 1,
                TokenKind::LeftParen => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        return Some(pos);
                    }
                }
                _ => {}
            }
        }

        None
    }
}

impl Default for ContextualErrorGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_contextual_error_generator_creation() {
        let generator = ContextualErrorGenerator::new();
        assert!(!generator.typo_corrections.is_empty());
        assert!(!generator.educational_hints.is_empty());
    }

    #[test]
    fn test_typo_correction_suggestions() {
        let generator = ContextualErrorGenerator::new();

        // Test token suggestion generation
        let token = Token::new(TokenKind::Identifier, Span::new(0, 5), "lamda".to_string());
        let context = ErrorContext::TopLevel;
        let suggestions = generator.generate_token_suggestions(&token, &context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.description.contains("lambda")));
    }

    #[test]
    fn test_context_specific_messages() {
        let generator = ContextualErrorGenerator::new();
        let base_error = Error::parse_error("test error", Span::new(0, 1));
        let context = ErrorContext::Define {
            is_function_definition: true,
            has_name: false,
            has_parameters: false,
        };

        let message = generator.generate_enhanced_message(&base_error, &context, &[], 0);
        assert!(message.contains("Function definition"));
        assert!(message.contains("missing a name"));
    }

    #[test]
    fn test_quick_fix_generation() {
        let generator = ContextualErrorGenerator::new();
        let base_error = Error::parse_error("test error", Span::new(0, 1));
        let context = ErrorContext::Define {
            is_function_definition: false,
            has_name: false,
            has_parameters: false,
        };
        let tokens = vec![Token::new(
            TokenKind::Identifier,
            Span::new(0, 6),
            "define".to_string(),
        )];

        let fixes = generator.generate_quick_fixes(&base_error, &context, &tokens, 0);
        assert!(!fixes.is_empty());
        assert!(fixes[0].title.contains("Add variable name"));
    }
}
