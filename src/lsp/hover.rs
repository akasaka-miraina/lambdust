//! Hover provider for Lambdust LSP
//!
//! This module provides hover information including type information,
//! documentation, and signature help for symbols in Scheme code.

use crate::error::{LambdustError, Result};
use crate::interpreter::LambdustInterpreter;
use crate::lsp::position::{Position, Range};
use crate::lsp::document::Document;
use std::collections::HashMap;

/// Hover information for a symbol
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Content to display (Markdown)
    pub content: String,
    
    /// Range where hover applies
    pub range: Option<Range>,
    
    /// Additional metadata
    pub metadata: HoverMetadata,
}

/// Metadata about hover information
#[derive(Debug, Clone, Default)]
pub struct HoverMetadata {
    /// Symbol name
    pub symbol: Option<String>,
    
    /// Symbol type
    pub symbol_type: Option<SymbolType>,
    
    /// Signature (for functions)
    pub signature: Option<String>,
    
    /// Documentation source
    pub documentation_source: Option<String>,
    
    /// Related symbols
    pub related: Vec<String>,
}

/// Type of symbol
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    /// Built-in function
    BuiltinFunction,
    
    /// User-defined function
    UserFunction,
    
    /// Special form
    SpecialForm,
    
    /// Variable binding
    Variable,
    
    /// Constant
    Constant,
    
    /// Macro
    Macro,
    
    /// Module
    Module,
    
    /// Type
    Type,
}

/// Hover provider implementation
pub struct HoverProvider {
    /// Interpreter for symbol resolution
    interpreter: LambdustInterpreter,
    
    /// Built-in documentation cache
    builtin_docs: HashMap<String, HoverInfo>,
    
    /// Special form documentation
    special_form_docs: HashMap<String, HoverInfo>,
    
    /// User documentation cache
    user_docs: HashMap<String, HoverInfo>,
}

impl HoverProvider {
    /// Create a new hover provider
    pub fn new(interpreter: LambdustInterpreter) -> Result<Self> {
        let mut provider = Self {
            interpreter,
            builtin_docs: HashMap::new(),
            special_form_docs: HashMap::new(),
            user_docs: HashMap::new(),
        };
        
        provider.initialize_builtin_docs();
        provider.initialize_special_form_docs();
        
        Ok(provider)
    }
    
    /// Get hover information at position
    pub fn get_hover_info(&self, document: &Document, position: Position) -> Result<Option<HoverInfo>> {
        // Extract the symbol at the position
        if let Some((symbol, range)) = self.extract_symbol_at_position(document, position)? {
            // Look up hover information for the symbol
            if let Some(mut info) = self.lookup_symbol(&symbol) {
                info.range = Some(range);
                return Ok(Some(info));
            }
        }
        
        Ok(None)
    }
    
    /// Extract symbol at position with its range
    fn extract_symbol_at_position(&self, document: &Document, position: Position) -> Result<Option<(String, Range)>> {
        let line_content = document.get_line(position.line as usize)
            .ok_or_else(|| LambdustError::runtime_error("Invalid line"))?;
        
        let char_pos = position.character as usize;
        if char_pos > line_content.len() {
            return Ok(None);
        }
        
        // Find symbol boundaries
        let chars: Vec<char> = line_content.chars().collect();
        
        // Find start of symbol
        let mut start = char_pos;
        while start > 0 && Self::is_symbol_char(chars[start - 1]) {
            start -= 1;
        }
        
        // Find end of symbol
        let mut end = char_pos;
        while end < chars.len() && Self::is_symbol_char(chars[end]) {
            end += 1;
        }
        
        if start < end {
            let symbol: String = chars[start..end].iter().collect();
            let range = Range::new(
                Position::new(position.line, start as u32),
                Position::new(position.line, end as u32),
            );
            Ok(Some((symbol, range)))
        } else {
            Ok(None)
        }
    }
    
    /// Check if character is part of a symbol
    fn is_symbol_char(ch: char) -> bool {
        ch.is_alphanumeric() || 
        matches!(ch, '+' | '-' | '*' | '/' | '?' | '!' | '<' | '>' | '=' | '_' | ':')
    }
    
    /// Look up hover information for a symbol
    fn lookup_symbol(&self, symbol: &str) -> Option<HoverInfo> {
        // Check built-in functions first
        if let Some(info) = self.builtin_docs.get(symbol) {
            return Some(info.clone());
        }
        
        // Check special forms
        if let Some(info) = self.special_form_docs.get(symbol) {
            return Some(info.clone());
        }
        
        // Check user-defined symbols
        if let Some(info) = self.user_docs.get(symbol) {
            return Some(info.clone());
        }
        
        // Try to resolve from interpreter environment
        self.resolve_from_environment(symbol)
    }
    
    /// Resolve symbol from interpreter environment
    fn resolve_from_environment(&self, symbol: &str) -> Option<HoverInfo> {
        // Query interpreter environment for symbol information
        match self.interpreter.get_environment() {
            Ok(env) => {
                // Check if symbol is bound in environment
                if let Some(value) = env.get_binding(symbol) {
                    self.create_hover_from_value(symbol, &value)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
    
    /// Create hover info from a value in the environment
    fn create_hover_from_value(&self, symbol: &str, value: &crate::value::Value) -> Option<HoverInfo> {
        use crate::value::{Value, Procedure};
        
        let (content, symbol_type, signature) = match value {
            Value::Procedure(proc) => {
                match proc {
                    Procedure::Lambda { params, body, .. } => {
                        let param_str = params.join(" ");
                        let sig = format!("(λ ({}) body)", param_str);
                        let content = format!(
                            "**{}** - User-defined function\n\n```scheme\n{}\n```\n\nUser-defined lambda function with parameters: {}",
                            symbol, sig, param_str
                        );
                        (content, SymbolType::UserFunction, Some(sig))
                    }
                    Procedure::Builtin { name, .. } => {
                        let sig = format!("({})", name);
                        let content = format!(
                            "**{}** - Built-in function\n\n```scheme\n{}\n```\n\nBuilt-in function",
                            symbol, sig
                        );
                        (content, SymbolType::BuiltinFunction, Some(sig))
                    }
                    Procedure::Macro { .. } => {
                        let content = format!(
                            "**{}** - Macro\n\nUser-defined macro",
                            symbol
                        );
                        (content, SymbolType::Macro, None)
                    }
                }
            }
            Value::Number(n) => {
                let content = format!(
                    "**{}** - Number\n\nValue: {}\n\nNumerical constant",
                    symbol, n
                );
                (content, SymbolType::Constant, None)
            }
            Value::String(s) => {
                let content = format!(
                    "**{}** - String\n\nValue: \"{}\"\n\nString constant",
                    symbol, s
                );
                (content, SymbolType::Constant, None)
            }
            Value::Boolean(b) => {
                let content = format!(
                    "**{}** - Boolean\n\nValue: {}\n\nBoolean constant",
                    symbol, if *b { "#t" } else { "#f" }
                );
                (content, SymbolType::Constant, None)
            }
            Value::Symbol(s) => {
                let content = format!(
                    "**{}** - Symbol\n\nValue: {}\n\nSymbol binding",
                    symbol, s
                );
                (content, SymbolType::Variable, None)
            }
            Value::List(_) => {
                let content = format!(
                    "**{}** - List\n\nList data structure",
                    symbol
                );
                (content, SymbolType::Variable, None)
            }
            _ => {
                let content = format!(
                    "**{}** - Variable\n\nUser-defined variable",
                    symbol
                );
                (content, SymbolType::Variable, None)
            }
        };
        
        Some(HoverInfo {
            content,
            range: None,
            metadata: HoverMetadata {
                symbol: Some(symbol.to_string()),
                symbol_type: Some(symbol_type),
                signature,
                documentation_source: Some("environment".to_string()),
                related: Vec::new(),
            },
        })
    }
    
    /// Initialize built-in function documentation
    fn initialize_builtin_docs(&mut self) {
        let builtin_docs = vec![
            // Arithmetic functions
            ("+", "Addition", "(+ number ...)", 
             "Returns the sum of all arguments. If no arguments are given, returns 0."),
            ("-", "Subtraction", "(- number number ...)", 
             "Subtracts the second and subsequent arguments from the first."),
            ("*", "Multiplication", "(* number ...)", 
             "Returns the product of all arguments. If no arguments are given, returns 1."),
            ("/", "Division", "(/ number number ...)", 
             "Divides the first argument by the second and subsequent arguments."),
            
            // Comparison functions
            ("=", "Numerical equality", "(= number number ...)", 
             "Returns #t if all arguments are numerically equal."),
            ("<", "Less than", "(< number number ...)", 
             "Returns #t if the arguments are in strictly increasing order."),
            (">", "Greater than", "(> number number ...)", 
             "Returns #t if the arguments are in strictly decreasing order."),
            ("<=", "Less than or equal", "(<= number number ...)", 
             "Returns #t if the arguments are in non-decreasing order."),
            (">=", "Greater than or equal", "(>= number number ...)", 
             "Returns #t if the arguments are in non-increasing order."),
            
            // List functions
            ("cons", "Construct pair", "(cons obj1 obj2)", 
             "Returns a newly allocated pair whose car is obj1 and whose cdr is obj2."),
            ("car", "First element", "(car pair)", 
             "Returns the first element of the pair."),
            ("cdr", "Second element", "(cdr pair)", 
             "Returns the second element of the pair."),
            ("list", "Create list", "(list obj ...)", 
             "Returns a newly allocated list of its arguments."),
            ("length", "List length", "(length list)", 
             "Returns the length of the list."),
            ("append", "Append lists", "(append list ...)", 
             "Returns a list consisting of the elements of the first list followed by the elements of the other lists."),
            ("reverse", "Reverse list", "(reverse list)", 
             "Returns a newly allocated list consisting of the elements of list in reverse order."),
            
            // Type predicates
            ("null?", "Null predicate", "(null? obj)", 
             "Returns #t if obj is the empty list, otherwise returns #f."),
            ("pair?", "Pair predicate", "(pair? obj)", 
             "Returns #t if obj is a pair, otherwise returns #f."),
            ("list?", "List predicate", "(list? obj)", 
             "Returns #t if obj is a list, otherwise returns #f."),
            ("number?", "Number predicate", "(number? obj)", 
             "Returns #t if obj is a number, otherwise returns #f."),
            ("integer?", "Integer predicate", "(integer? obj)", 
             "Returns #t if obj is an integer, otherwise returns #f."),
            ("string?", "String predicate", "(string? obj)", 
             "Returns #t if obj is a string, otherwise returns #f."),
            ("symbol?", "Symbol predicate", "(symbol? obj)", 
             "Returns #t if obj is a symbol, otherwise returns #f."),
            ("boolean?", "Boolean predicate", "(boolean? obj)", 
             "Returns #t if obj is a boolean, otherwise returns #f."),
            ("procedure?", "Procedure predicate", "(procedure? obj)", 
             "Returns #t if obj is a procedure, otherwise returns #f."),
            
            // String functions
            ("string-length", "String length", "(string-length string)", 
             "Returns the number of characters in the string."),
            ("string-append", "String append", "(string-append string ...)", 
             "Returns a newly allocated string whose characters form the concatenation of the given strings."),
            ("string-ref", "String reference", "(string-ref string k)", 
             "Returns character k of string using zero-origin indexing."),
            ("substring", "Substring", "(substring string start end)", 
             "Returns a newly allocated string formed from the characters of string beginning with index start and ending with index end."),
            
            // I/O functions
            ("display", "Display", "(display obj)", 
             "Writes a representation of obj to the current output port."),
            ("write", "Write", "(write obj)", 
             "Writes a representation of obj to the current output port."),
            ("newline", "Newline", "(newline)", 
             "Writes an end of line to the current output port."),
            ("read", "Read", "(read)", 
             "Reads an external representation from the current input port."),
            
            // Control flow
            ("apply", "Apply procedure", "(apply proc arg1 ... args)", 
             "Applies proc to the elements of the list (append (list arg1 ...) args)."),
            ("eval", "Evaluate", "(eval expression environment-specifier)", 
             "Evaluates expression in the specified environment."),
            ("call/cc", "Call with current continuation", "(call/cc proc)", 
             "Calls proc with the current continuation as an argument."),
        ];
        
        for (name, title, signature, description) in builtin_docs {
            let content = format!(
                "**{}** - {}\n\n```scheme\n{}\n```\n\n{}",
                name, title, signature, description
            );
            
            let info = HoverInfo {
                content,
                range: None,
                metadata: HoverMetadata {
                    symbol: Some(name.to_string()),
                    symbol_type: Some(SymbolType::BuiltinFunction),
                    signature: Some(signature.to_string()),
                    documentation_source: Some("R7RS".to_string()),
                    related: Vec::new(),
                },
            };
            
            self.builtin_docs.insert(name.to_string(), info);
        }
    }
    
    /// Initialize special form documentation
    fn initialize_special_form_docs(&mut self) {
        let special_forms = vec![
            ("define", "Variable definition", "(define variable expression)", 
             "Defines a variable and binds it to the result of evaluating expression."),
            ("lambda", "Lambda expression", "(lambda formals body)", 
             "Creates a procedure. Formals specify the parameters, body is the procedure body."),
            ("if", "Conditional", "(if test consequent alternate)", 
             "Evaluates test. If true, evaluates and returns consequent, otherwise alternate."),
            ("cond", "Multi-way conditional", "(cond clause ...)", 
             "Each clause is of the form (test expression ...). Tests are evaluated in order."),
            ("case", "Case analysis", "(case key clause ...)", 
             "Key is evaluated and compared against the data in each clause."),
            ("and", "Logical AND", "(and test ...)", 
             "Evaluates tests from left to right until one returns #f or all are evaluated."),
            ("or", "Logical OR", "(or test ...)", 
             "Evaluates tests from left to right until one returns true or all are evaluated."),
            ("let", "Local binding", "(let bindings body)", 
             "Bindings is a list of (variable value) pairs. Body is evaluated in the extended environment."),
            ("let*", "Sequential binding", "(let* bindings body)", 
             "Like let, but bindings are performed sequentially."),
            ("letrec", "Recursive binding", "(letrec bindings body)", 
             "Like let, but variables are bound recursively."),
            ("begin", "Sequential evaluation", "(begin expression ...)", 
             "Evaluates expressions sequentially and returns the value of the last one."),
            ("do", "Iteration", "(do bindings test result body)", 
             "Iteration construct with variable bindings, test condition, and body."),
            ("quote", "Quote", "(quote datum)", 
             "Returns datum without evaluating it. Can be abbreviated as 'datum."),
            ("quasiquote", "Quasiquote", "(quasiquote template)", 
             "Like quote, but allows unquote and unquote-splicing. Can be abbreviated as `template."),
            ("unquote", "Unquote", "(unquote expression)", 
             "Used within quasiquote to evaluate expression. Can be abbreviated as ,expression."),
            ("unquote-splicing", "Unquote splicing", "(unquote-splicing expression)", 
             "Used within quasiquote to splice in a list. Can be abbreviated as ,@expression."),
            ("set!", "Assignment", "(set! variable expression)", 
             "Assigns the value of expression to variable."),
            ("syntax-rules", "Macro definition", "(syntax-rules literals rules)", 
             "Defines a macro using pattern-based transformation rules."),
        ];
        
        for (name, title, signature, description) in special_forms {
            let content = format!(
                "**{}** - {}\n\n```scheme\n{}\n```\n\n{}",
                name, title, signature, description
            );
            
            let info = HoverInfo {
                content,
                range: None,
                metadata: HoverMetadata {
                    symbol: Some(name.to_string()),
                    symbol_type: Some(SymbolType::SpecialForm),
                    signature: Some(signature.to_string()),
                    documentation_source: Some("R7RS".to_string()),
                    related: Vec::new(),
                },
            };
            
            self.special_form_docs.insert(name.to_string(), info);
        }
    }
    
    /// Add user documentation
    pub fn add_user_documentation(&mut self, symbol: String, info: HoverInfo) {
        self.user_docs.insert(symbol, info);
    }
    
    /// Remove user documentation
    pub fn remove_user_documentation(&mut self, symbol: &str) {
        self.user_docs.remove(symbol);
    }
    
    /// Clear user documentation
    pub fn clear_user_documentation(&mut self) {
        self.user_docs.clear();
    }
    
    /// Get all available symbols with documentation
    pub fn get_documented_symbols(&self) -> Vec<String> {
        let mut symbols = Vec::new();
        symbols.extend(self.builtin_docs.keys().cloned());
        symbols.extend(self.special_form_docs.keys().cloned());
        symbols.extend(self.user_docs.keys().cloned());
        symbols.sort();
        symbols
    }
}
