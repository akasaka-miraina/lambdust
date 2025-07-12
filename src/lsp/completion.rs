//! Code completion provider for Lambdust LSP
//!
//! This module provides intelligent code completion by leveraging the existing
//! REPL completion system and extending it with context-aware suggestions,
//! type information, and documentation.

use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::interpreter::LambdustInterpreter;
use crate::lsp::position::{Position, Range};
use std::collections::HashMap;
use std::rc::Rc;

/// Completion item kind for different types of completions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionItemKind {
    /// Built-in function
    Function,
    /// Special form (define, lambda, if, etc.)
    Keyword,
    /// User-defined function
    Method,
    /// Variable binding
    Variable,
    /// Constant value
    Constant,
    /// Macro
    Macro,
    /// Module or library
    Module,
    /// Type or class
    Class,
    /// File path
    File,
    /// Snippet template
    Snippet,
}

/// A single completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The label displayed in the completion list
    pub label: String,
    
    /// The kind of completion item
    pub kind: CompletionItemKind,
    
    /// Detailed information about the item
    pub detail: Option<String>,
    
    /// Documentation string
    pub documentation: Option<String>,
    
    /// Text to insert when completing
    pub insert_text: Option<String>,
    
    /// Additional text edits (for imports, etc.)
    pub additional_text_edits: Vec<TextEdit>,
    
    /// Sorting priority (lower = higher priority)
    pub sort_priority: u32,
    
    /// Whether this item should be pre-selected
    pub preselect: bool,
    
    /// Range to replace when inserting
    pub replace_range: Option<Range>,
}

/// Text edit for completion
#[derive(Debug, Clone)]
pub struct TextEdit {
    /// Range to replace
    pub range: Range,
    /// New text
    pub new_text: String,
}

/// Completion context information
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Position where completion was triggered
    pub position: Position,
    
    /// Character that triggered completion (if any)
    pub trigger_character: Option<char>,
    
    /// Is this a re-trigger of completion?
    pub is_retrigger: bool,
    
    /// Current line content
    pub line_content: String,
    
    /// Text before cursor on current line
    pub prefix: String,
    
    /// Current expression context
    pub expression_context: ExpressionContext,
    
    /// Available bindings in current scope
    pub scope_bindings: Vec<String>,
}

/// Expression context for completion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionContext {
    /// At the beginning of an expression (after '(')
    ExpressionStart,
    
    /// In function position (first element of list)
    FunctionPosition,
    
    /// In argument position
    ArgumentPosition { function_name: Option<String>, arg_index: usize },
    
    /// In top level (not inside any expression)
    TopLevel,
    
    /// In variable reference position
    VariableReference,
    
    /// In string literal
    StringLiteral,
    
    /// In comment
    Comment,
}

/// Code completion provider
pub struct CompletionProvider {
    /// Interpreter for environment access
    interpreter: LambdustInterpreter,
    
    /// Cached builtin completions
    builtin_completions: HashMap<String, CompletionItem>,
    
    /// Special form completions
    special_form_completions: HashMap<String, CompletionItem>,
    
    /// Snippet completions
    snippet_completions: HashMap<String, CompletionItem>,
}

impl CompletionProvider {
    /// Create a new completion provider
    pub fn new() -> Result<Self> {
        let interpreter = LambdustInterpreter::new();
        let mut provider = Self {
            interpreter,
            builtin_completions: HashMap::new(),
            special_form_completions: HashMap::new(),
            snippet_completions: HashMap::new(),
        };
        
        provider.initialize_builtin_completions()?;
        provider.initialize_special_form_completions();
        provider.initialize_snippet_completions();
        
        Ok(provider)
    }
    
    /// Get completions for a given context
    pub fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();
        
        match context.expression_context {
            ExpressionContext::ExpressionStart | ExpressionContext::FunctionPosition => {
                // Add function and special form completions
                completions.extend(self.get_function_completions(&context.prefix));
                completions.extend(self.get_special_form_completions(&context.prefix));
                completions.extend(self.get_user_function_completions(&context.prefix)?);
            },
            
            ExpressionContext::ArgumentPosition { ref function_name, arg_index } => {
                // Context-aware argument completion
                completions.extend(self.get_argument_completions(
                    function_name.as_deref(),
                    arg_index,
                    &context.prefix
                )?);
            },
            
            ExpressionContext::TopLevel => {
                // Add definitions, special forms, and snippets
                completions.extend(self.get_definition_completions(&context.prefix));
                completions.extend(self.get_snippet_completions(&context.prefix));
            },
            
            ExpressionContext::VariableReference => {
                // Add variable completions
                completions.extend(self.get_variable_completions(&context.prefix, &context.scope_bindings));
            },
            
            ExpressionContext::StringLiteral => {
                // File path completions for load, etc.
                completions.extend(self.get_file_path_completions(&context.prefix));
            },
            
            ExpressionContext::Comment => {
                // No completions in comments
            },
        }
        
        // Sort and limit completions
        completions.sort_by_key(|item| (item.sort_priority, item.label.clone()));
        completions.truncate(50); // Limit to 50 items
        
        Ok(completions)
    }
    
    /// Initialize builtin function completions
    fn initialize_builtin_completions(&mut self) -> Result<()> {
        let builtin_functions = vec![
            // Arithmetic
            ("+", "Addition of numbers", "(+ number ...)"),
            ("-", "Subtraction of numbers", "(- number ...)"),
            ("*", "Multiplication of numbers", "(* number ...)"),
            ("/", "Division of numbers", "(/ number ...)"),
            ("quotient", "Integer division", "(quotient n1 n2)"),
            ("remainder", "Remainder of division", "(remainder n1 n2)"),
            ("modulo", "Modulo operation", "(modulo n1 n2)"),
            
            // Comparison
            ("=", "Numerical equality", "(= number ...)"),
            ("<", "Numerical less than", "(< number ...)"),
            (">", "Numerical greater than", "(> number ...)"),
            ("<=", "Numerical less than or equal", "(<= number ...)"),
            (">=", "Numerical greater than or equal", "(>= number ...)"),
            
            // List operations
            ("cons", "Construct a pair", "(cons obj1 obj2)"),
            ("car", "First element of pair", "(car pair)"),
            ("cdr", "Second element of pair", "(cdr pair)"),
            ("list", "Create a list", "(list obj ...)"),
            ("length", "Length of list", "(length list)"),
            ("append", "Append lists", "(append list ...)"),
            ("reverse", "Reverse a list", "(reverse list)"),
            
            // Type predicates
            ("null?", "Test for empty list", "(null? obj)"),
            ("pair?", "Test for pair", "(pair? obj)"),
            ("list?", "Test for list", "(list? obj)"),
            ("number?", "Test for number", "(number? obj)"),
            ("integer?", "Test for integer", "(integer? obj)"),
            ("string?", "Test for string", "(string? obj)"),
            ("symbol?", "Test for symbol", "(symbol? obj)"),
            ("boolean?", "Test for boolean", "(boolean? obj)"),
            ("procedure?", "Test for procedure", "(procedure? obj)"),
            
            // String operations
            ("string-length", "Length of string", "(string-length string)"),
            ("string-append", "Concatenate strings", "(string-append string ...)"),
            ("string-ref", "Character at index", "(string-ref string k)"),
            ("substring", "Extract substring", "(substring string start end)"),
            
            // I/O operations
            ("display", "Display object", "(display obj)"),
            ("write", "Write object", "(write obj)"),
            ("newline", "Write newline", "(newline)"),
            ("read", "Read expression", "(read)"),
            
            // Control
            ("apply", "Apply procedure to arguments", "(apply proc args)"),
            ("eval", "Evaluate expression", "(eval expr env)"),
            ("call/cc", "Call with current continuation", "(call/cc proc)"),
        ];
        
        for (name, doc, signature) in builtin_functions {
            let item = CompletionItem {
                label: name.to_string(),
                kind: CompletionItemKind::Function,
                detail: Some(signature.to_string()),
                documentation: Some(doc.to_string()),
                insert_text: Some(name.to_string()),
                additional_text_edits: Vec::new(),
                sort_priority: 10,
                preselect: false,
                replace_range: None,
            };
            self.builtin_completions.insert(name.to_string(), item);
        }
        
        Ok(())
    }
    
    /// Initialize special form completions
    fn initialize_special_form_completions(&mut self) {
        let special_forms = vec![
            ("define", "Define a variable or function", "(define name value)"),
            ("lambda", "Create a procedure", "(lambda (params) body)"),
            ("if", "Conditional expression", "(if test then else)"),
            ("cond", "Multi-way conditional", "(cond (test expr) ...)"),
            ("case", "Case analysis", "(case key ((datum ...) expr) ...)"),
            ("and", "Logical AND", "(and test ...)"),
            ("or", "Logical OR", "(or test ...)"),
            ("let", "Local bindings", "(let ((var val) ...) body)"),
            ("let*", "Sequential local bindings", "(let* ((var val) ...) body)"),
            ("letrec", "Recursive local bindings", "(letrec ((var val) ...) body)"),
            ("begin", "Sequential evaluation", "(begin expr ...)"),
            ("do", "Iteration", "(do ((var init step) ...) (test result) body)"),
            ("quote", "Quote expression", "(quote expr)"),
            ("quasiquote", "Quasi-quote expression", "(quasiquote expr)"),
            ("unquote", "Unquote in quasiquote", "(unquote expr)"),
            ("unquote-splicing", "Unquote-splicing", "(unquote-splicing expr)"),
            ("set!", "Assignment", "(set! var value)"),
            ("syntax-rules", "Define macro", "(syntax-rules (literals) rule ...)"),
        ];
        
        for (name, doc, signature) in special_forms {
            let item = CompletionItem {
                label: name.to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some(signature.to_string()),
                documentation: Some(doc.to_string()),
                insert_text: Some(name.to_string()),
                additional_text_edits: Vec::new(),
                sort_priority: 5, // Higher priority than functions
                preselect: false,
                replace_range: None,
            };
            self.special_form_completions.insert(name.to_string(), item);
        }
    }
    
    /// Initialize snippet completions
    fn initialize_snippet_completions(&mut self) {
        let snippets = vec![
            ("defun", "Define function", "(define (${1:name} ${2:params})\n  ${3:body})"),
            ("defvar", "Define variable", "(define ${1:name} ${2:value})"),
            ("lambda", "Lambda function", "(lambda (${1:params}) ${2:body})"),
            ("let", "Let binding", "(let ((${1:var} ${2:value}))\n  ${3:body})"),
            ("cond", "Conditional", "(cond\n  ((${1:test}) ${2:result})\n  (else ${3:default}))"),
            ("match", "Pattern matching", "(case ${1:expr}\n  ((${2:pattern}) ${3:result})\n  (else ${4:default}))"),
        ];
        
        for (name, doc, template) in snippets {
            let item = CompletionItem {
                label: name.to_string(),
                kind: CompletionItemKind::Snippet,
                detail: Some(doc.to_string()),
                documentation: Some(format!("Snippet: {}", doc)),
                insert_text: Some(template.to_string()),
                additional_text_edits: Vec::new(),
                sort_priority: 15,
                preselect: false,
                replace_range: None,
            };
            self.snippet_completions.insert(name.to_string(), item);
        }
    }
    
    /// Get function completions matching prefix
    fn get_function_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        self.builtin_completions
            .values()
            .filter(|item| item.label.starts_with(prefix))
            .cloned()
            .collect()
    }
    
    /// Get special form completions matching prefix
    fn get_special_form_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        self.special_form_completions
            .values()
            .filter(|item| item.label.starts_with(prefix))
            .cloned()
            .collect()
    }
    
    /// Get user-defined function completions
    fn get_user_function_completions(&self, prefix: &str) -> Result<Vec<CompletionItem>> {
        // TODO: Extract user-defined functions from interpreter environment
        // For now, return empty list
        let mut completions = Vec::new();
        
        // This would use the interpreter's environment to find user-defined functions
        // let env = self.interpreter.get_environment();
        // for (name, _value) in env.bindings() {
        //     if name.starts_with(prefix) && is_procedure(value) {
        //         completions.push(CompletionItem { ... });
        //     }
        // }
        
        Ok(completions)
    }
    
    /// Get argument completions for specific functions
    fn get_argument_completions(
        &self,
        function_name: Option<&str>,
        arg_index: usize,
        prefix: &str
    ) -> Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();
        
        if let Some(func_name) = function_name {
            match func_name {
                "define" if arg_index == 0 => {
                    // First argument to define should be a symbol
                    completions.push(CompletionItem {
                        label: format!("{}variable", prefix),
                        kind: CompletionItemKind::Variable,
                        detail: Some("Variable name".to_string()),
                        documentation: Some("Name for the variable being defined".to_string()),
                        insert_text: Some(format!("{}variable", prefix)),
                        additional_text_edits: Vec::new(),
                        sort_priority: 5,
                        preselect: false,
                        replace_range: None,
                    });
                },
                
                "load" if arg_index == 0 => {
                    // File path completion for load
                    completions.extend(self.get_file_path_completions(prefix));
                },
                
                _ => {
                    // Generic completions
                    completions.extend(self.get_variable_completions(prefix, &[]));
                }
            }
        }
        
        Ok(completions)
    }
    
    /// Get definition completions for top-level
    fn get_definition_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        self.special_form_completions
            .values()
            .filter(|item| {
                matches!(item.label.as_str(), "define" | "define-syntax" | "define-macro") &&
                item.label.starts_with(prefix)
            })
            .cloned()
            .collect()
    }
    
    /// Get snippet completions
    fn get_snippet_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        self.snippet_completions
            .values()
            .filter(|item| item.label.starts_with(prefix))
            .cloned()
            .collect()
    }
    
    /// Get variable completions
    fn get_variable_completions(&self, prefix: &str, scope_bindings: &[String]) -> Vec<CompletionItem> {
        scope_bindings
            .iter()
            .filter(|name| name.starts_with(prefix))
            .map(|name| CompletionItem {
                label: name.clone(),
                kind: CompletionItemKind::Variable,
                detail: Some("Variable".to_string()),
                documentation: Some(format!("Variable: {}", name)),
                insert_text: Some(name.clone()),
                additional_text_edits: Vec::new(),
                sort_priority: 8,
                preselect: false,
                replace_range: None,
            })
            .collect()
    }
    
    /// Get file path completions
    fn get_file_path_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        // TODO: Implement file path completion
        // This would scan the file system for matching files
        Vec::new()
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback if initialization fails
            Self {
                interpreter: LambdustInterpreter::new(),
                builtin_completions: HashMap::new(),
                special_form_completions: HashMap::new(),
                snippet_completions: HashMap::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_provider_creation() {
        let provider = CompletionProvider::new().unwrap();
        assert!(!provider.builtin_completions.is_empty());
        assert!(!provider.special_form_completions.is_empty());
    }

    #[test]
    fn test_builtin_completions() {
        let provider = CompletionProvider::new().unwrap();
        let completions = provider.get_function_completions("c");
        
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|item| item.label == "cons"));
        assert!(completions.iter().any(|item| item.label == "car"));
        assert!(completions.iter().any(|item| item.label == "cdr"));
    }

    #[test]
    fn test_special_form_completions() {
        let provider = CompletionProvider::new().unwrap();
        let completions = provider.get_special_form_completions("l");
        
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|item| item.label == "lambda"));
        assert!(completions.iter().any(|item| item.label == "let"));
    }

    #[test]
    fn test_completion_context() {
        let context = CompletionContext {
            position: Position::new(0, 5),
            trigger_character: Some('('),
            is_retrigger: false,
            line_content: "(cons ".to_string(),
            prefix: "".to_string(),
            expression_context: ExpressionContext::FunctionPosition,
            scope_bindings: vec!["x".to_string(), "y".to_string()],
        };
        
        let provider = CompletionProvider::new().unwrap();
        let completions = provider.get_completions(&context).unwrap();
        
        assert!(!completions.is_empty());
    }
}