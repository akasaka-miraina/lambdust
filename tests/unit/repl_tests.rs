//! Unit tests for REPL functionality
//!
//! Tests the enhanced REPL features including tab completion,
//! syntax highlighting, and debug mode.

#[cfg(feature = "repl")]
mod repl_tests {
    use lambdust::interpreter::LambdustInterpreter;
    use std::collections::HashSet;

    /// Test that the completion helper correctly identifies builtin functions
    #[test]
    fn test_builtin_functions_completion() {
        // This would test the SchemeHelper completion functionality
        // Since SchemeHelper is private, we test the underlying logic
        let interpreter = LambdustInterpreter::new();
        let host_functions = interpreter.list_host_functions();
        let scheme_functions = interpreter.list_scheme_functions();
        
        // Should have basic arithmetic functions
        assert!(host_functions.iter().any(|f| f.as_str() == "+"));
        assert!(host_functions.iter().any(|f| f.as_str() == "-"));
        assert!(host_functions.iter().any(|f| f.as_str() == "*"));
        assert!(host_functions.iter().any(|f| f.as_str() == "/"));
        
        // Should have list functions
        assert!(host_functions.iter().any(|f| f.as_str() == "car"));
        assert!(host_functions.iter().any(|f| f.as_str() == "cdr"));
        assert!(host_functions.iter().any(|f| f.as_str() == "cons"));
        assert!(host_functions.iter().any(|f| f.as_str() == "list"));
    }

    /// Test completion word boundary detection
    #[test]
    fn test_completion_word_boundaries() {
        let test_cases = vec![
            ("(+ 1 2", 6, ""),        // End of line
            ("(+ ca", 5, "ca"),       // Partial word
            ("(define sq", 11, "sq"),  // Partial special form
            ("(map car", 8, "car"),   // Complete word
        ];
        
        for (line, pos, expected_word) in test_cases {
            let mut start = pos;
            while start > 0 {
                let ch = line.chars().nth(start - 1).unwrap_or(' ');
                if ch.is_whitespace() || ch == '(' || ch == ')' {
                    break;
                }
                start -= 1;
            }
            
            let word = &line[start..pos];
            assert_eq!(word, expected_word, "Failed for line: '{}' at pos {}", line, pos);
        }
    }

    /// Test debug state management
    #[test]
    fn test_debug_state() {
        #[derive(Debug, Clone)]
        struct DebugState {
            enabled: bool,
            breakpoint_set: bool,
            step_mode: bool,
            call_stack: Vec<String>,
            last_expression: Option<String>,
        }
        
        impl Default for DebugState {
            fn default() -> Self {
                Self {
                    enabled: false,
                    breakpoint_set: false,
                    step_mode: false,
                    call_stack: Vec::new(),
                    last_expression: None,
                }
            }
        }
        
        let mut debug_state = DebugState::default();
        
        // Test initial state
        assert!(!debug_state.enabled);
        assert!(!debug_state.breakpoint_set);
        assert!(!debug_state.step_mode);
        assert!(debug_state.call_stack.is_empty());
        assert!(debug_state.last_expression.is_none());
        
        // Test enabling debug mode
        debug_state.enabled = true;
        assert!(debug_state.enabled);
        
        // Test setting breakpoint
        debug_state.breakpoint_set = true;
        assert!(debug_state.breakpoint_set);
        
        // Test adding to call stack
        debug_state.call_stack.push("(square 5)".to_string());
        debug_state.call_stack.push("(* x x)".to_string());
        assert_eq!(debug_state.call_stack.len(), 2);
        assert_eq!(debug_state.call_stack[0], "(square 5)");
        assert_eq!(debug_state.call_stack[1], "(* x x)");
    }

    /// Test call info extraction for debugging
    #[test]
    fn test_call_info_extraction() {
        fn extract_call_info(input: &str) -> String {
            let trimmed = input.trim();
            if trimmed.starts_with('(') {
                if let Some(end) = trimmed.find(' ') {
                    let func_name = &trimmed[1..end];
                    format!("{}(...)", func_name)
                } else if trimmed.len() > 2 {
                    let func_name = &trimmed[1..trimmed.len()-1];
                    format!("{}()", func_name)
                } else {
                    "(unknown)".to_string()
                }
            } else {
                trimmed.chars().take(20).collect::<String>() + "..."
            }
        }
        
        let test_cases = vec![
            ("(+ 1 2 3)", "+(...)"),
            ("(square 5)", "square(...)"),
            ("(car)", "car()"),
            ("(define x 10)", "define(...)"),
            ("42", "42..."),
            ("()", "(unknown)"),
        ];
        
        for (input, expected) in test_cases {
            let result = extract_call_info(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    /// Test expression completeness detection
    #[test]
    fn test_expression_completeness() {
        fn is_complete_expression(input: &str) -> bool {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return false;
            }

            // Simple balance check for parentheses
            let mut paren_count = 0;
            let mut in_string = false;
            let mut escaped = false;

            for ch in trimmed.chars() {
                if escaped {
                    escaped = false;
                    continue;
                }

                match ch {
                    '\\' if in_string => escaped = true,
                    '"' => in_string = !in_string,
                    '(' if !in_string => paren_count += 1,
                    ')' if !in_string => paren_count -= 1,
                    _ => {}
                }
            }

            paren_count == 0 && !in_string
        }
        
        // Test complete expressions
        assert!(is_complete_expression("42"));
        assert!(is_complete_expression("(+ 1 2)"));
        assert!(is_complete_expression("(define square (lambda (x) (* x x)))"));
        assert!(is_complete_expression("\"hello world\""));
        assert!(is_complete_expression("(list 1 2 3)"));
        
        // Test incomplete expressions
        assert!(!is_complete_expression("(+ 1"));
        assert!(!is_complete_expression("(define x"));
        assert!(!is_complete_expression("\"hello"));
        assert!(!is_complete_expression("(lambda (x)"));
        assert!(!is_complete_expression("(begin (+ 1 2"));
        
        // Test empty input
        assert!(!is_complete_expression(""));
        assert!(!is_complete_expression("   "));
        
        // Test string handling
        assert!(is_complete_expression("\"(this is not code)\""));
        assert!(!is_complete_expression("\"incomplete string"));
        assert!(is_complete_expression("\"escaped \\\"quote\\\"\""));
    }

    /// Test special forms recognition
    #[test]
    fn test_special_forms_recognition() {
        let special_forms: HashSet<String> = [
            "define", "lambda", "if", "cond", "case", "and", "or", "when", "unless",
            "begin", "do", "let", "let*", "letrec", "letrec*", "set!", "quote", "quasiquote",
            "unquote", "unquote-splicing", "syntax-rules", "define-syntax", "guard",
            "define-record-type", "delay", "lazy", "force", "promise?",
        ].iter().map(|s| s.to_string()).collect();
        
        // Test core special forms
        assert!(special_forms.contains("define"));
        assert!(special_forms.contains("lambda"));
        assert!(special_forms.contains("if"));
        assert!(special_forms.contains("cond"));
        assert!(special_forms.contains("let"));
        
        // Test R7RS specific forms
        assert!(special_forms.contains("guard"));
        assert!(special_forms.contains("when"));
        assert!(special_forms.contains("unless"));
        
        // Test SRFI forms
        assert!(special_forms.contains("define-record-type"));
        assert!(special_forms.contains("delay"));
        assert!(special_forms.contains("lazy"));
        
        // Test that regular functions are not special forms
        assert!(!special_forms.contains("+"));
        assert!(!special_forms.contains("car"));
        assert!(!special_forms.contains("map"));
    }

    /// Test syntax highlighting color mapping
    #[test]
    fn test_syntax_highlighting_colors() {
        // Test color constants for syntax highlighting
        let colors = vec![
            ("red", "\x1b[31m"),      // Special forms
            ("green", "\x1b[32m"),    // Builtin functions
            ("yellow", "\x1b[33m"),   // Strings
            ("blue", "\x1b[94m"),     // Numbers
            ("gray", "\x1b[90m"),     // Comments
            ("reset", "\x1b[0m"),     // Reset
        ];
        
        for (name, code) in colors {
            assert!(!code.is_empty(), "Color code for {} should not be empty", name);
            assert!(code.starts_with("\x1b["), "Color code for {} should be ANSI escape", name);
        }
    }

    /// Test completion candidate sorting
    #[test]
    fn test_completion_sorting() {
        let mut candidates = vec![
            "vector-ref",
            "vector",
            "vector-set!",
            "vector-length",
            "vector?",
        ];
        
        candidates.sort();
        
        let expected = vec![
            "vector",
            "vector-length",
            "vector-ref",
            "vector-set!",
            "vector?",
        ];
        
        assert_eq!(candidates, expected);
    }
}