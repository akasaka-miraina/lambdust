//! Helper functions for Literal string interning.

/// Determines if a string is common enough to warrant interning.
/// This heuristic helps decide between regular and interned string storage.
pub fn is_common_string(s: &str) -> bool {
    // Empty string and very short strings
    if s.is_empty() || s.len() == 1 {
        return true;
    }
    
    // Common Scheme patterns
    const COMMON_STRINGS: &[&str] = &[
        // Error messages and common strings
        "error", "warning", "info", "debug",
        "true", "false", "nil", "null",
        "undefined", "unspecified",
        
        // Common operators as strings  
        "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
        "and", "or", "not",
        
        // Common keywords that might appear as strings
        "if", "then", "else", "let", "define", "lambda",
        "quote", "unquote", "quasiquote",
        
        // Common type names
        "number", "string", "symbol", "boolean", "pair", "list", "vector",
        "procedure", "port", "character", "bytevector",
        
        // File and I/O related
        "read", "write", "open", "close", "input", "output",
        "stdin", "stdout", "stderr",
        
        // Common format strings
        "~a", "~s", "~d", "~x", "~o", "~b", "~f", "~e", "~g",
        "~%", "~~", "~n", "~t",
    ];
    
    COMMON_STRINGS.contains(&s)
}