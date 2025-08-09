//! Intelligent autocompletion system for the enhanced REPL.

#![allow(dead_code, missing_docs)]

use crate::{Lambdust, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Represents a completion suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    pub text: String,
    pub display: String,
    pub completion_type: CompletionType,
    pub description: Option<String>,
    pub signature: Option<String>,
}

impl Completion {
    pub fn new(text: String, completion_type: CompletionType) -> Self {
        let display = text.clone();
        Self {
            text,
            display,
            completion_type,
            description: None,
            signature: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn with_display(mut self, display: String) -> Self {
        self.display = display;
        self
    }
}

/// Types of completions available
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionType {
    Function,
    Macro,
    Variable,
    Keyword,
    SpecialForm,
    Library,
    Module,
    FilePath,
    BuiltinType,
    UserType,
}

/// Context information for generating completions
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub input: String,
    pub cursor_position: usize,
    pub current_word: String,
    pub preceding_word: Option<String>,
    pub in_string: bool,
    pub in_comment: bool,
    pub paren_depth: usize,
    pub context_type: ContextType,
}

/// Different contexts where completions might be needed
#[derive(Debug, Clone, PartialEq)]
pub enum ContextType {
    Expression,
    FunctionCall,
    MacroCall,
    Import,
    Define,
    Let,
    Lambda,
    FilePath,
}

/// The main completion provider
pub struct CompletionProvider {
    builtin_functions: HashMap<String, CompletionInfo>,
    builtin_macros: HashMap<String, CompletionInfo>,
    special_forms: HashMap<String, CompletionInfo>,
    keywords: HashSet<String>,
    user_definitions: HashMap<String, CompletionInfo>,
    imported_modules: HashMap<String, ModuleInfo>,
    srfi_modules: HashMap<String, SrfiModuleInfo>,
}

/// Information about a completion item
#[derive(Debug, Clone)]
pub struct CompletionInfo {
    pub name: String,
    pub completion_type: CompletionType,
    pub description: String,
    pub signature: Option<String>,
    pub module: Option<String>,
}

/// Information about an imported module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub exports: Vec<String>,
    pub path: Option<PathBuf>,
}

/// Information about SRFI modules
#[derive(Debug, Clone)]
pub struct SrfiModuleInfo {
    pub srfi_number: u32,
    pub title: String,
    pub exports: Vec<String>,
}

impl CompletionProvider {
    pub fn new(lambdust: &Lambdust) -> Result<Self> {
        let mut provider = Self {
            builtin_functions: HashMap::new(),
            builtin_macros: HashMap::new(),
            special_forms: HashMap::new(),
            keywords: HashSet::new(),
            user_definitions: HashMap::new(),
            imported_modules: HashMap::new(),
            srfi_modules: HashMap::new(),
        };

        provider.initialize_builtins()?;
        provider.initialize_special_forms()?;
        provider.initialize_keywords()?;
        provider.initialize_srfi_modules()?;
        provider.update_from_runtime(lambdust)?;

        Ok(provider)
    }

    fn initialize_builtins(&mut self) -> Result<()> {
        // Arithmetic functions
        self.add_builtin_function("+", "Addition", Some("(+ number1 number2 ...)"));
        self.add_builtin_function("-", "Subtraction", Some("(- number1 number2 ...)"));
        self.add_builtin_function("*", "Multiplication", Some("(* number1 number2 ...)"));
        self.add_builtin_function("/", "Division", Some("(/ number1 number2 ...)"));
        self.add_builtin_function("=", "Numeric equality", Some("(= number1 number2 ...)"));
        self.add_builtin_function("<", "Less than", Some("(< number1 number2 ...)"));
        self.add_builtin_function(">", "Greater than", Some("(> number1 number2 ...)"));
        self.add_builtin_function("<=", "Less than or equal", Some("(<= number1 number2 ...)"));
        self.add_builtin_function(">=", "Greater than or equal", Some("(>= number1 number2 ...)"));

        // List functions
        self.add_builtin_function("car", "First element of pair", Some("(car pair)"));
        self.add_builtin_function("cdr", "Rest of pair", Some("(cdr pair)"));
        self.add_builtin_function("cons", "Construct pair", Some("(cons obj1 obj2)"));
        self.add_builtin_function("list", "Create list", Some("(list obj ...)"));
        self.add_builtin_function("length", "List length", Some("(length list)"));
        self.add_builtin_function("append", "Append lists", Some("(append list ...)"));
        self.add_builtin_function("reverse", "Reverse list", Some("(reverse list)"));
        self.add_builtin_function("map", "Apply function to list elements", Some("(map proc list1 list2 ...)"));
        self.add_builtin_function("filter", "Filter list elements", Some("(filter pred list)"));
        self.add_builtin_function("fold-left", "Left fold over list", Some("(fold-left proc init list1 list2 ...)"));
        self.add_builtin_function("fold-right", "Right fold over list", Some("(fold-right proc init list1 list2 ...)"));

        // Predicates
        self.add_builtin_function("null?", "Test for empty list", Some("(null? obj)"));
        self.add_builtin_function("pair?", "Test for pair", Some("(pair? obj)"));
        self.add_builtin_function("list?", "Test for proper list", Some("(list? obj)"));
        self.add_builtin_function("number?", "Test for number", Some("(number? obj)"));
        self.add_builtin_function("string?", "Test for string", Some("(string? obj)"));
        self.add_builtin_function("symbol?", "Test for symbol", Some("(symbol? obj)"));
        self.add_builtin_function("boolean?", "Test for boolean", Some("(boolean? obj)"));
        self.add_builtin_function("procedure?", "Test for procedure", Some("(procedure? obj)"));

        // String functions
        self.add_builtin_function("string-length", "String length", Some("(string-length string)"));
        self.add_builtin_function("string-append", "Append strings", Some("(string-append string ...)"));
        self.add_builtin_function("substring", "Extract substring", Some("(substring string start end)"));
        self.add_builtin_function("string=?", "String equality", Some("(string=? string1 string2)"));
        self.add_builtin_function("string<?", "String less than", Some("(string<? string1 string2)"));

        // I/O functions
        self.add_builtin_function("display", "Display object", Some("(display obj [port])"));
        self.add_builtin_function("write", "Write object", Some("(write obj [port])"));
        self.add_builtin_function("newline", "Write newline", Some("(newline [port])"));
        self.add_builtin_function("read", "Read object", Some("(read [port])"));

        Ok(())
    }

    fn initialize_special_forms(&mut self) -> Result<()> {
        self.add_special_form("define", "Define variable or function", Some("(define name value) or (define (name params) body)"));
        self.add_special_form("lambda", "Create procedure", Some("(lambda (params) body)"));
        self.add_special_form("if", "Conditional expression", Some("(if test then else)"));
        self.add_special_form("cond", "Multi-way conditional", Some("(cond (test result) ...)"));
        self.add_special_form("case", "Dispatch on value", Some("(case key ((datums) result) ...)"));
        self.add_special_form("and", "Logical and", Some("(and test ...)"));
        self.add_special_form("or", "Logical or", Some("(or test ...)"));
        self.add_special_form("let", "Local binding", Some("(let ((var val) ...) body)"));
        self.add_special_form("let*", "Sequential local binding", Some("(let* ((var val) ...) body)"));
        self.add_special_form("letrec", "Recursive local binding", Some("(letrec ((var val) ...) body)"));
        self.add_special_form("begin", "Sequential evaluation", Some("(begin expr ...)"));
        self.add_special_form("quote", "Quote expression", Some("(quote expr) or 'expr"));
        self.add_special_form("quasiquote", "Quasi-quote expression", Some("(quasiquote expr) or `expr"));
        self.add_special_form("unquote", "Unquote in quasi-quote", Some("(unquote expr) or ,expr"));
        self.add_special_form("unquote-splicing", "Unquote-splicing in quasi-quote", Some("(unquote-splicing expr) or ,@expr"));
        self.add_special_form("set!", "Assignment", Some("(set! var value)"));

        Ok(())
    }

    fn initialize_keywords(&mut self) -> Result<()> {
        let keywords = vec![
            "else", "=>", "...", "unquote", "unquote-splicing",
            "#t", "#f", "#true", "#false"
        ];

        for keyword in keywords {
            self.keywords.insert(keyword.to_string());
        }

        Ok(())
    }

    fn initialize_srfi_modules(&mut self) -> Result<()> {
        // SRFI-1: List Library
        self.srfi_modules.insert("srfi-1".to_string(), SrfiModuleInfo {
            srfi_number: 1,
            title: "List Library".to_string(),
            exports: vec![
                "xcons".to_string(), "list-tabulate".to_string(), "list-copy".to_string(),
                "circular-list".to_string(), "iota".to_string(), "proper-list?".to_string(),
                "circular-list?".to_string(), "dotted-list?".to_string(), "not-pair?".to_string(),
                "null-list?".to_string(), "list=".to_string(), "first".to_string(), "second".to_string(),
                "third".to_string(), "fourth".to_string(), "fifth".to_string(), "sixth".to_string(),
                "seventh".to_string(), "eighth".to_string(), "ninth".to_string(), "tenth".to_string(),
                "take".to_string(), "drop".to_string(), "take-right".to_string(), "drop-right".to_string(),
                "take!".to_string(), "drop-right!".to_string(), "split-at".to_string(), "split-at!".to_string(),
                "last".to_string(), "last-pair".to_string(), "zip".to_string(), "unzip1".to_string(),
                "unzip2".to_string(), "unzip3".to_string(), "unzip4".to_string(), "unzip5".to_string(),
                "count".to_string(), "fold".to_string(), "unfold".to_string(), "pair-fold".to_string(),
                "reduce".to_string(), "unfold-right".to_string(), "pair-fold-right".to_string(),
                "reduce-right".to_string(), "append-map".to_string(), "append-map!".to_string(),
                "map!".to_string(), "pair-for-each".to_string(), "filter-map".to_string(),
                "map-in-order".to_string(), "filter!".to_string(), "partition".to_string(),
                "partition!".to_string(), "remove".to_string(), "remove!".to_string(),
                // ... more SRFI-1 exports
            ],
        });

        // SRFI-13: String Libraries
        self.srfi_modules.insert("srfi-13".to_string(), SrfiModuleInfo {
            srfi_number: 13,
            title: "String Libraries".to_string(),
            exports: vec![
                "string-null?".to_string(), "string-every".to_string(), "string-any".to_string(),
                "string-tabulate".to_string(), "string-unfold".to_string(), "string-unfold-right".to_string(),
                "reverse-list->string".to_string(), "string-take".to_string(), "string-drop".to_string(),
                "string-take-right".to_string(), "string-drop-right".to_string(), "string-pad".to_string(),
                "string-pad-right".to_string(), "string-trim".to_string(), "string-trim-right".to_string(),
                "string-trim-both".to_string(), "string-compare".to_string(), "string-compare-ci".to_string(),
                "string-hash".to_string(), "string-hash-ci".to_string(), "string-prefix-length".to_string(),
                "string-suffix-length".to_string(), "string-prefix-length-ci".to_string(),
                "string-suffix-length-ci".to_string(), "string-prefix?".to_string(), "string-suffix?".to_string(),
                "string-prefix-ci?".to_string(), "string-suffix-ci?".to_string(), "string-index".to_string(),
                "string-index-right".to_string(), "string-skip".to_string(), "string-skip-right".to_string(),
                "string-count".to_string(), "string-contains".to_string(), "string-contains-ci".to_string(),
                // ... more SRFI-13 exports
            ],
        });

        // SRFI-26: Notation for Specializing Parameters
        self.srfi_modules.insert("srfi-26".to_string(), SrfiModuleInfo {
            srfi_number: 26,
            title: "Notation for Specializing Parameters".to_string(),
            exports: vec![
                "cut".to_string(), "<>".to_string(), "<...>".to_string(), "cute".to_string(),
            ],
        });

        Ok(())
    }

    fn add_builtin_function(&mut self, name: &str, description: &str, signature: Option<&str>) {
        self.builtin_functions.insert(name.to_string(), CompletionInfo {
            name: name.to_string(),
            completion_type: CompletionType::Function,
            description: description.to_string(),
            signature: signature.map(|s| s.to_string()),
            module: None,
        });
    }

    fn add_special_form(&mut self, name: &str, description: &str, signature: Option<&str>) {
        self.special_forms.insert(name.to_string(), CompletionInfo {
            name: name.to_string(),
            completion_type: CompletionType::SpecialForm,
            description: description.to_string(),
            signature: signature.map(|s| s.to_string()),
            module: None,
        });
    }

    fn update_from_runtime(&mut self, _lambdust: &Lambdust) -> Result<()> {
        // TODO: Extract user-defined functions and variables from the runtime
        // This would involve introspecting the global environment
        Ok(())
    }

    pub fn get_completions(&self, context: &CompletionContext) -> Vec<Completion> {
        let mut completions = Vec::new();
        let prefix = &context.current_word.to_lowercase();

        // Skip completion if we're in a comment
        if context.in_comment {
            return completions;
        }

        // File path completions
        if context.in_string || context.context_type == ContextType::FilePath {
            completions.extend(self.get_file_completions(&context.current_word));
            return completions;
        }

        // Built-in functions
        for (name, info) in &self.builtin_functions {
            if name.to_lowercase().starts_with(prefix) {
                let mut completion = Completion::new(name.clone(), CompletionType::Function);
                if let Some(ref sig) = info.signature {
                    completion = completion.with_signature(sig.clone());
                }
                completion = completion.with_description(info.description.clone());
                completions.push(completion);
            }
        }

        // Special forms
        for (name, info) in &self.special_forms {
            if name.to_lowercase().starts_with(prefix) {
                let mut completion = Completion::new(name.clone(), CompletionType::SpecialForm);
                if let Some(ref sig) = info.signature {
                    completion = completion.with_signature(sig.clone());
                }
                completion = completion.with_description(info.description.clone());
                completions.push(completion);
            }
        }

        // Keywords
        for keyword in &self.keywords {
            if keyword.to_lowercase().starts_with(prefix) {
                completions.push(Completion::new(keyword.clone(), CompletionType::Keyword));
            }
        }

        // User definitions
        for (name, info) in &self.user_definitions {
            if name.to_lowercase().starts_with(prefix) {
                let mut completion = Completion::new(name.clone(), info.completion_type.clone());
                if let Some(ref sig) = info.signature {
                    completion = completion.with_signature(sig.clone());
                }
                completion = completion.with_description(info.description.clone());
                completions.push(completion);
            }
        }

        // SRFI module completions
        if context.context_type == ContextType::Import {
            for (name, info) in &self.srfi_modules {
                if name.to_lowercase().starts_with(prefix) {
                    let completion = Completion::new(name.clone(), CompletionType::Library)
                        .with_description(format!("SRFI-{}: {}", info.srfi_number, info.title));
                    completions.push(completion);
                }
            }
        }

        // Sort completions by relevance
        completions.sort_by(|a, b| {
            // Exact matches first
            let a_exact = a.text.to_lowercase() == *prefix;
            let b_exact = b.text.to_lowercase() == *prefix;
            
            if a_exact && !b_exact {
                return std::cmp::Ordering::Less;
            }
            if b_exact && !a_exact {
                return std::cmp::Ordering::Greater;
            }

            // Then by completion type priority
            let a_priority = Self::completion_type_priority(&a.completion_type);
            let b_priority = Self::completion_type_priority(&b.completion_type);
            
            if a_priority != b_priority {
                return a_priority.cmp(&b_priority);
            }

            // Finally by alphabetical order
            a.text.cmp(&b.text)
        });

        completions
    }

    fn completion_type_priority(completion_type: &CompletionType) -> u32 {
        match completion_type {
            CompletionType::SpecialForm => 0,
            CompletionType::Function => 1,
            CompletionType::Macro => 2,
            CompletionType::Variable => 3,
            CompletionType::Keyword => 4,
            CompletionType::Library => 5,
            CompletionType::Module => 6,
            CompletionType::BuiltinType => 7,
            CompletionType::UserType => 8,
            CompletionType::FilePath => 9,
        }
    }

    fn get_file_completions(&self, prefix: &str) -> Vec<Completion> {
        let mut completions = Vec::new();
        
        let path = Path::new(prefix);
        let (dir, filename_prefix) = if prefix.ends_with('/') || prefix.ends_with('\\') {
            (path, "")
        } else {
            (path.parent().unwrap_or(Path::new(".")), 
             path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
        };

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(filename_prefix) {
                        let full_path = if prefix.ends_with('/') || prefix.ends_with('\\') {
                            format!("{prefix}{name}")
                        } else if let Some(parent) = path.parent() {
                            format!("{}/{}", parent.display(), name)
                        } else {
                            name.to_string()
                        };

                        let display = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            format!("{name}/")
                        } else {
                            name.to_string()
                        };

                        completions.push(
                            Completion::new(full_path, CompletionType::FilePath)
                                .with_display(display)
                        );
                    }
                }
            }
        }

        completions
    }

    pub fn analyze_context(&self, input: &str, cursor_position: usize) -> CompletionContext {
        let chars: Vec<char> = input.chars().collect();
        let cursor_pos = cursor_position.min(chars.len());
        
        let mut in_string = false;
        let mut in_comment = false;
        let mut paren_depth: usize = 0;
        let mut current_word_start = cursor_pos;
        let preceding_word: Option<String> = None;

        // Analyze the input up to cursor position
        for (i, &ch) in chars[..cursor_pos].iter().enumerate() {
            match ch {
                '"' if i == 0 || chars[i-1] != '\\' => in_string = !in_string,
                ';' if !in_string => in_comment = true,
                '\n' => in_comment = false,
                '(' if !in_string && !in_comment => paren_depth += 1,
                ')' if !in_string && !in_comment => paren_depth = paren_depth.saturating_sub(1),
                ' ' | '\t' | '\r' if !in_string => {
                    if i < cursor_pos - 1 {
                        current_word_start = i + 1;
                    }
                }
                _ => {}
            }
        }

        // Find the current word
        while current_word_start < cursor_pos && 
              chars.get(current_word_start).is_some_and(|c| c.is_whitespace()) {
            current_word_start += 1;
        }

        let current_word: String = chars[current_word_start..cursor_pos].iter().collect();

        // Determine context type
        let context_type = if in_string {
            ContextType::FilePath
        } else {
            self.determine_context_type(&chars, cursor_pos, paren_depth)
        };

        CompletionContext {
            input: input.to_string(),
            cursor_position,
            current_word,
            preceding_word,
            in_string,
            in_comment,
            paren_depth,
            context_type,
        }
    }

    fn determine_context_type(&self, chars: &[char], cursor_pos: usize, _paren_depth: usize) -> ContextType {
        // Look backward to find context clues
        let mut i = cursor_pos;
        while i > 0 {
            i -= 1;
            if chars[i] == '(' {
                // Look at the next non-whitespace character
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                
                let word_start = j;
                while j < chars.len() && !chars[j].is_whitespace() && chars[j] != ')' {
                    j += 1;
                }
                
                if word_start < j {
                    let word: String = chars[word_start..j].iter().collect();
                    return match word.as_str() {
                        "import" | "include" => ContextType::Import,
                        "define" => ContextType::Define,
                        "let" | "let*" | "letrec" => ContextType::Let,
                        "lambda" => ContextType::Lambda,
                        _ => ContextType::FunctionCall,
                    };
                }
                
                return ContextType::Expression;
            }
        }
        
        ContextType::Expression
    }

    pub fn update_user_definitions(&mut self, definitions: HashMap<String, CompletionInfo>) {
        self.user_definitions = definitions;
    }

    pub fn add_user_definition(&mut self, name: String, info: CompletionInfo) {
        self.user_definitions.insert(name, info);
    }

    pub fn remove_user_definition(&mut self, name: &str) -> Option<CompletionInfo> {
        self.user_definitions.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_context_analysis() {
        let lambdust = crate::Lambdust::new();
        let provider = CompletionProvider::new(&lambdust).unwrap();
        
        // Test basic function completion
        let context = provider.analyze_context("(+ 1 2", 6);
        assert_eq!(context.current_word, "2");
        assert_eq!(context.paren_depth, 1);
        assert!(!context.in_string);
        
        // Test string context
        let context = provider.analyze_context("(load \"test", 11);
        assert_eq!(context.current_word, "test");
        assert!(context.in_string);
        
        // Test function name completion
        let context = provider.analyze_context("(def", 4);
        assert_eq!(context.current_word, "def");
        assert_eq!(context.context_type, ContextType::Expression);
    }

    #[test]
    fn test_completion_generation() {
        let lambdust = crate::Lambdust::new();
        let provider = CompletionProvider::new(&lambdust).unwrap();
        
        let context = CompletionContext {
            input: "(+".to_string(),
            cursor_position: 2,
            current_word: "+".to_string(),
            preceding_word: None,
            in_string: false,
            in_comment: false,
            paren_depth: 1,
            context_type: ContextType::Expression,
        };
        
        let completions = provider.get_completions(&context);
        assert!(!completions.is_empty());
        
        // Should find the + function
        let plus_completion = completions.iter().find(|c| c.text == "+");
        assert!(plus_completion.is_some());
        assert_eq!(plus_completion.unwrap().completion_type, CompletionType::Function);
    }
}