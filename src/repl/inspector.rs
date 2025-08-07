//! Code introspection and development tools for the enhanced REPL.

#![allow(dead_code, missing_docs)]

use crate::{Lambdust, Result, eval::Value};
use std::collections::HashMap;
use std::fmt;

/// Commands for code introspection
#[derive(Debug, Clone, PartialEq)]
pub enum IntrospectionCommand {
    Describe(String),
    Apropos(String),
    Source(String),
    Documentation(String),
    Type(String),
    Signature(String),
    Examples(String),
    ListBindings,
    ListModules,
    Profile(String),
    Trace(String),
    Untrace(String),
}

/// Information about a symbol or function
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub symbol_type: SymbolType,
    pub value: Option<Value>,
    pub documentation: Option<String>,
    pub signature: Option<String>,
    pub source_location: Option<SourceLocation>,
    pub module: Option<String>,
    pub examples: Vec<String>,
}

/// Types of symbols that can be inspected
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Macro,
    Variable,
    Constant,
    SpecialForm,
    Module,
    Type,
    Unknown,
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Profiling information for functions
#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub function_name: String,
    pub call_count: usize,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
    pub min_time: std::time::Duration,
    pub max_time: std::time::Duration,
}

/// The main code inspector
pub struct CodeInspector {
    symbol_database: HashMap<String, SymbolInfo>,
    profile_data: HashMap<String, ProfileInfo>,
    traced_functions: std::collections::HashSet<String>,
    documentation_cache: HashMap<String, String>,
}

impl CodeInspector {
    pub fn new() -> Self {
        let mut inspector = Self {
            symbol_database: HashMap::new(),
            profile_data: HashMap::new(),
            traced_functions: std::collections::HashSet::new(),
            documentation_cache: HashMap::new(),
        };

        inspector.initialize_builtin_docs();
        inspector
    }

    fn initialize_builtin_docs(&mut self) {
        // Initialize documentation for built-in functions
        let builtin_docs = vec![
            ("+", "Addition function. Adds all given numbers together.", "(+ number1 number2 ...)"),
            ("-", "Subtraction function. Subtracts subsequent numbers from the first.", "(- number1 number2 ...)"),
            ("*", "Multiplication function. Multiplies all given numbers together.", "(* number1 number2 ...)"),
            ("/", "Division function. Divides the first number by subsequent numbers.", "(/ number1 number2 ...)"),
            ("=", "Numeric equality predicate. Returns #t if all numbers are equal.", "(= number1 number2 ...)"),
            ("<", "Less-than predicate. Returns #t if numbers are in ascending order.", "(< number1 number2 ...)"),
            (">", "Greater-than predicate. Returns #t if numbers are in descending order.", "(> number1 number2 ...)"),
            
            ("car", "Returns the first element of a pair.", "(car pair)"),
            ("cdr", "Returns the second element of a pair.", "(cdr pair)"),
            ("cons", "Constructs a new pair from two objects.", "(cons obj1 obj2)"),
            ("list", "Creates a list from the given arguments.", "(list obj ...)"),
            ("length", "Returns the length of a proper list.", "(length list)"),
            ("append", "Concatenates lists together.", "(append list ...)"),
            ("reverse", "Returns a new list with elements in reverse order.", "(reverse list)"),
            
            ("map", "Applies a procedure to elements of lists, returning a new list.", "(map proc list1 list2 ...)"),
            ("filter", "Returns a list of elements that satisfy the predicate.", "(filter pred list)"),
            ("fold-left", "Reduces a list from left to right using a binary procedure.", "(fold-left proc init list1 list2 ...)"),
            ("fold-right", "Reduces a list from right to left using a binary procedure.", "(fold-right proc init list1 list2 ...)"),
            
            ("null?", "Returns #t if the object is the empty list.", "(null? obj)"),
            ("pair?", "Returns #t if the object is a pair.", "(pair? obj)"),
            ("list?", "Returns #t if the object is a proper list.", "(list? obj)"),
            ("number?", "Returns #t if the object is a number.", "(number? obj)"),
            ("string?", "Returns #t if the object is a string.", "(string? obj)"),
            ("symbol?", "Returns #t if the object is a symbol.", "(symbol? obj)"),
            ("boolean?", "Returns #t if the object is a boolean.", "(boolean? obj)"),
            ("procedure?", "Returns #t if the object is a procedure.", "(procedure? obj)"),
            
            ("define", "Binds a name to a value or creates a function.", "(define name value) or (define (name params) body)"),
            ("lambda", "Creates an anonymous procedure.", "(lambda (params) body)"),
            ("if", "Conditional expression. Evaluates test and returns then or else clause.", "(if test then else)"),
            ("cond", "Multi-way conditional with multiple test-result clauses.", "(cond (test result) ... (else result))"),
            ("let", "Creates local bindings for variables.", "(let ((var val) ...) body)"),
            ("let*", "Creates sequential local bindings.", "(let* ((var val) ...) body)"),
            ("letrec", "Creates recursive local bindings.", "(letrec ((var val) ...) body)"),
            
            ("display", "Writes an object to the output port in human-readable form.", "(display obj [port])"),
            ("write", "Writes an object to the output port in machine-readable form.", "(write obj [port])"),
            ("newline", "Writes a newline character to the output port.", "(newline [port])"),
            ("read", "Reads an object from the input port.", "(read [port])"),
        ];

        for (name, doc, sig) in builtin_docs {
            let symbol_info = SymbolInfo {
                name: name.to_string(),
                symbol_type: if name.starts_with("define") || name == "lambda" || name == "if" || name == "cond" || name == "let" || name == "let*" || name == "letrec" {
                    SymbolType::SpecialForm
                } else {
                    SymbolType::Function
                },
                value: None,
                documentation: Some(doc.to_string()),
                signature: Some(sig.to_string()),
                source_location: None,
                module: Some("scheme".to_string()),
                examples: Vec::new(),
            };
            
            self.symbol_database.insert(name.to_string(), symbol_info);
            self.documentation_cache.insert(name.to_string(), doc.to_string());
        }
    }

    pub fn inspect(&self, lambdust: &mut Lambdust, item: &str) -> Result<()> {
        if let Some(symbol_info) = self.symbol_database.get(item) {
            self.print_symbol_info(symbol_info);
        } else {
            // Try to introspect from the runtime
            match self.introspect_from_runtime(lambdust, item) {
                Ok(Some(info)) => self.print_symbol_info(&info),
                Ok(None) => println!("No information found for: {}", item),
                Err(e) => println!("Error inspecting {}: {}", item, e),
            }
        }
        Ok(())
    }

    fn print_symbol_info(&self, info: &SymbolInfo) {
        println!("🔍 {}", info.name);
        println!("   Type: {:?}", info.symbol_type);
        
        if let Some(ref doc) = info.documentation {
            println!("   Documentation: {}", doc);
        }
        
        if let Some(ref sig) = info.signature {
            println!("   Signature: {}", sig);
        }
        
        if let Some(ref module) = info.module {
            println!("   Module: {}", module);
        }
        
        if let Some(ref value) = info.value {
            println!("   Value: {}", value);
        }
        
        if let Some(ref location) = info.source_location {
            println!("   Source: {}:{}:{}", location.file, location.line, location.column);
        }
        
        if !info.examples.is_empty() {
            println!("   Examples:");
            for example in &info.examples {
                println!("     {}", example);
            }
        }
    }

    fn introspect_from_runtime(&self, _lambdust: &mut Lambdust, item: &str) -> Result<Option<SymbolInfo>> {
        // TODO: Implement runtime introspection
        // This would involve querying the runtime environment for information about the symbol
        println!("Runtime introspection not yet implemented for: {}", item);
        Ok(None)
    }

    pub fn apropos(&self, pattern: &str) -> Result<()> {
        let pattern_lower = pattern.to_lowercase();
        let mut matches = Vec::new();

        for (name, info) in &self.symbol_database {
            let name_lower = name.to_lowercase();
            let doc_matches = info.documentation
                .as_ref()
                .map(|doc| doc.to_lowercase().contains(&pattern_lower))
                .unwrap_or(false);

            if name_lower.contains(&pattern_lower) || doc_matches {
                matches.push((name, info));
            }
        }

        if matches.is_empty() {
            println!("No matches found for: {}", pattern);
        } else {
            println!("Matches for '{}' ({} found):", pattern, matches.len());
            for (name, info) in matches {
                println!("  {} - {:?}", name, info.symbol_type);
                if let Some(ref doc) = info.documentation {
                    let truncated_doc = if doc.len() > 60 {
                        format!("{}...", &doc[..57])
                    } else {
                        doc.clone())
                    };
                    println!("    {}", truncated_doc);
                }
            }
        }

        Ok(())
    }

    pub fn describe(&self, item: &str) -> Result<()> {
        if let Some(symbol_info) = self.symbol_database.get(item) {
            self.print_detailed_description(symbol_info);
        } else {
            println!("No description available for: {}", item);
        }
        Ok(())
    }

    fn print_detailed_description(&self, info: &SymbolInfo) {
        println!("📖 Detailed Description: {}", info.name);
        println!("{}", "=".repeat(50));
        
        println!("Type: {:?}", info.symbol_type);
        
        if let Some(ref sig) = info.signature {
            println!("\nSignature:");
            println!("  {}", sig);
        }
        
        if let Some(ref doc) = info.documentation {
            println!("\nDescription:");
            println!("  {}", doc);
        }
        
        if let Some(ref module) = info.module {
            println!("\nDefined in module: {}", module);
        }
        
        if !info.examples.is_empty() {
            println!("\nExamples:");
            for (i, example) in info.examples.iter().enumerate() {
                println!("  {}. {}", i + 1, example);
            }
        }
        
        // Show related functions
        self.show_related_functions(&info.name);
    }

    fn show_related_functions(&self, name: &str) {
        let mut related = Vec::new();
        
        // Find functions with similar names or in the same category
        for (other_name, _) in &self.symbol_database {
            if other_name != name {
                // Simple heuristic: same prefix or contains same keywords
                if self.are_related(name, other_name) {
                    related.push(other_name);
                }
            }
        }
        
        if !related.is_empty() {
            println!("\nRelated functions:");
            for related_name in related.iter().take(5) {
                println!("  {}", related_name);
            }
            if related.len() > 5 {
                println!("  ... and {} more", related.len() - 5);
            }
        }
    }

    fn are_related(&self, name1: &str, name2: &str) -> bool {
        // Simple heuristic for finding related functions
        if name1.len() < 2 || name2.len() < 2 {
            return false;
        }
        
        // Check for common prefixes
        let prefix_len = name1.chars().zip(name2.chars())
            .take_while(|(a, b)| a == b)
            .count();
        
        if prefix_len >= 3 {
            return true;
        }
        
        // Check for common suffixes (like predicates ending in ?)
        if (name1.ends_with('?') && name2.ends_with('?')) ||
           (name1.ends_with('!') && name2.ends_with('!')) {
            return true;
        }
        
        // Check for similar categories
        let categories = [
            ("string", vec!["string-", "char-"]),
            ("list", vec!["list", "car", "cdr", "cons", "append", "reverse"]),
            ("math", vec!["+", "-", "*", "/", "=", "<", ">", "<=", ">="]),
            ("io", vec!["read", "write", "display", "open", "close", "port"]),
        ];
        
        for (_, category_words) in &categories {
            let name1_in_category = category_words.iter().any(|word| name1.contains(word));
            let name2_in_category = category_words.iter().any(|word| name2.contains(word));
            
            if name1_in_category && name2_in_category {
                return true;
            }
        }
        
        false
    }

    pub fn list_bindings(&self, _lambdust: &Lambdust) -> Result<()> {
        println!("Available bindings:");
        
        let mut bindings: Vec<_> = self.symbol_database.keys().collect();
        bindings.sort();
        
        let mut current_category = None;
        
        for name in bindings {
            if let Some(info) = self.symbol_database.get(name) {
                let category = match info.symbol_type {
                    SymbolType::Function => "Functions",
                    SymbolType::Macro => "Macros",
                    SymbolType::SpecialForm => "Special Forms",
                    SymbolType::Variable => "Variables",
                    SymbolType::Constant => "Constants",
                    _ => "Other",
                };
                
                if current_category != Some(category) {
                    println!("\n{}:", category);
                    current_category = Some(category);
                }
                
                print!("  {}", name);
                if let Some(ref sig) = info.signature {
                    println!(" - {}", sig);
                } else {
                    println!();
                }
            }
        }
        
        Ok(())
    }

    pub fn start_profiling(&mut self, function_name: &str) -> Result<()> {
        // TODO: Implement actual profiling integration with the runtime
        self.profile_data.insert(function_name.to_string(), ProfileInfo {
            function_name: function_name.to_string(),
            call_count: 0,
            total_time: std::time::Duration::from_secs(0),
            average_time: std::time::Duration::from_secs(0),
            min_time: std::time::Duration::from_secs(u64::MAX),
            max_time: std::time::Duration::from_secs(0),
        });
        
        println!("Started profiling: {}", function_name);
        Ok(())
    }

    pub fn stop_profiling(&mut self, function_name: &str) -> Result<()> {
        if let Some(profile_info) = self.profile_data.remove(function_name) {
            self.print_profile_results(&profile_info);
        } else {
            println!("No profiling data found for: {}", function_name);
        }
        Ok(())
    }

    fn print_profile_results(&self, profile: &ProfileInfo) {
        println!("📊 Profile Results for: {}", profile.function_name);
        println!("   Calls: {}", profile.call_count);
        println!("   Total time: {:?}", profile.total_time);
        println!("   Average time: {:?}", profile.average_time);
        println!("   Min time: {:?}", profile.min_time);
        println!("   Max time: {:?}", profile.max_time);
    }

    pub fn start_tracing(&mut self, function_name: &str) -> Result<()> {
        self.traced_functions.insert(function_name.to_string());
        println!("Started tracing: {}", function_name);
        Ok(())
    }

    pub fn stop_tracing(&mut self, function_name: &str) -> Result<()> {
        if self.traced_functions.remove(function_name) {
            println!("Stopped tracing: {}", function_name);
        } else {
            println!("Function was not being traced: {}", function_name);
        }
        Ok(())
    }

    pub fn list_traced_functions(&self) -> Result<()> {
        if self.traced_functions.is_empty() {
            println!("No functions are currently being traced");
        } else {
            println!("Currently traced functions:");
            for function in &self.traced_functions {
                println!("  {}", function);
            }
        }
        Ok(())
    }

    pub fn add_symbol_info(&mut self, name: String, info: SymbolInfo) {
        self.symbol_database.insert(name, info);
    }

    pub fn update_symbol_info<F>(&mut self, name: &str, updater: F) 
    where
        F: FnOnce(&mut SymbolInfo),
    {
        if let Some(info) = self.symbol_database.get_mut(name) {
            updater(info);
        }
    }

    pub fn remove_symbol_info(&mut self, name: &str) -> Option<SymbolInfo> {
        self.symbol_database.remove(name)
    }

    pub fn get_symbol_info(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbol_database.get(name)
    }

    pub fn search_by_type(&self, symbol_type: SymbolType) -> Vec<&SymbolInfo> {
        self.symbol_database
            .values()
            .filter(|info| info.symbol_type == symbol_type)
            .collect()
    }
}

impl Default for CodeInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolType::Function => write!(f, "Function"),
            SymbolType::Macro => write!(f, "Macro"),
            SymbolType::Variable => write!(f, "Variable"),
            SymbolType::Constant => write!(f, "Constant"),
            SymbolType::SpecialForm => write!(f, "Special Form"),
            SymbolType::Module => write!(f, "Module"),
            SymbolType::Type => write!(f, "Type"),
            SymbolType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_initialization() {
        let inspector = CodeInspector::new();
        
        // Should have some built-in documentation
        assert!(!inspector.symbol_database.is_empty());
        assert!(inspector.symbol_database.contains_key("+"));
        assert!(inspector.symbol_database.contains_key("car"));
        assert!(inspector.symbol_database.contains_key("define"));
    }

    #[test]
    fn test_symbol_info_creation() {
        let info = SymbolInfo {
            name: "test-function".to_string(),
            symbol_type: SymbolType::Function,
            value: None,
            documentation: Some("A test function".to_string()),
            signature: Some("(test-function x)".to_string()),
            source_location: None,
            module: Some("test".to_string()),
            examples: vec!["(test-function 42)".to_string()],
        };
        
        assert_eq!(info.name, "test-function");
        assert_eq!(info.symbol_type, SymbolType::Function);
        assert_eq!(info.documentation, Some("A test function".to_string()));
    }

    #[test]
    fn test_related_functions() {
        let inspector = CodeInspector::new();
        
        // Test related function detection
        assert!(inspector.are_related("string-length", "string-append"));
        assert!(inspector.are_related("list?", "pair?"));
        assert!(inspector.are_related("+", "-"));
        assert!(!inspector.are_related("car", "newline"));
    }
}