//! Demonstration of Lambdust's enhanced error reporting system
//! 
//! This example shows how Lambdust provides detailed error information
//! including source location, stack traces, and context for debugging.

use lambdust::error::{LambdustError, SourcePosition, SourceSpan, StackFrame, FrameType};

fn main() {
    println!("=== Lambdust Enhanced Error Reporting Demo ===\n");

    // Demonstrate various error types with location information
    demo_lexer_error();
    demo_runtime_error_with_stack_trace();
    demo_type_error_with_context();
    demo_arity_error_with_location();
    demo_undefined_variable_error();
}

fn demo_lexer_error() {
    println!("1. Lexer Error with Position Information:");
    
    let error = LambdustError::LexerError {
        message: "Invalid number format".to_string(),
        location: SourceSpan::with_filename(
            SourcePosition::new(3, 15, 42),
            SourcePosition::new(3, 20, 47),
            "script.scm".to_string()
        ),
    };
    
    println!("{}", error.format_detailed());
    println!();
}

fn demo_runtime_error_with_stack_trace() {
    println!("2. Runtime Error with Stack Trace:");
    
    let error = LambdustError::RuntimeError {
        message: "Cannot access property of undefined object".to_string(),
        location: SourceSpan::with_filename(
            SourcePosition::new(15, 8, 234),
            SourcePosition::new(15, 25, 251),
            "main.scm".to_string()
        ),
        stack_trace: vec![
            StackFrame {
                name: "process-data".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(12, 4, 189),
                    SourcePosition::new(12, 16, 201),
                    "main.scm".to_string()
                ),
                frame_type: FrameType::Function,
            },
            StackFrame {
                name: "calculate-result".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(8, 12, 134),
                    SourcePosition::new(8, 28, 150),
                    "main.scm".to_string()
                ),
                frame_type: FrameType::Function,
            },
            StackFrame {
                name: "main".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(5, 1, 78),
                    SourcePosition::new(5, 5, 82),
                    "main.scm".to_string()
                ),
                frame_type: FrameType::TopLevel,
            },
        ],
    };
    
    println!("{}", error.format_detailed());
    println!();
}

fn demo_type_error_with_context() {
    println!("3. Type Error with Context:");
    
    let error = LambdustError::TypeError {
        message: "Expected number, got string \"hello\"".to_string(),
        location: SourceSpan::with_filename(
            SourcePosition::new(7, 10, 156),
            SourcePosition::new(7, 17, 163),
            "math.scm".to_string()
        ),
        stack_trace: vec![
            StackFrame {
                name: "+".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(7, 9, 155),
                    SourcePosition::new(7, 10, 156),
                    "math.scm".to_string()
                ),
                frame_type: FrameType::Builtin,
            },
            StackFrame {
                name: "sum-values".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(7, 5, 151),
                    SourcePosition::new(7, 15, 161),
                    "math.scm".to_string()
                ),
                frame_type: FrameType::Function,
            },
        ],
    };
    
    println!("{}", error.format_detailed());
    println!();
}

fn demo_arity_error_with_location() {
    println!("4. Arity Error with Function Information:");
    
    let error = LambdustError::ArityError {
        expected: 2,
        actual: 3,
        function: "cons".to_string(),
        location: SourceSpan::with_filename(
            SourcePosition::new(11, 5, 198),
            SourcePosition::new(11, 9, 202),
            "list-ops.scm".to_string()
        ),
        stack_trace: vec![
            StackFrame {
                name: "build-list".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(10, 1, 178),
                    SourcePosition::new(10, 11, 188),
                    "list-ops.scm".to_string()
                ),
                frame_type: FrameType::Function,
            },
        ],
    };
    
    println!("{}", error.format_detailed());
    println!();
}

fn demo_undefined_variable_error() {
    println!("5. Undefined Variable Error:");
    
    let error = LambdustError::UndefinedVariable {
        variable: "undefined-var".to_string(),
        location: SourceSpan::with_filename(
            SourcePosition::new(6, 8, 89),
            SourcePosition::new(6, 21, 102),
            "variables.scm".to_string()
        ),
        stack_trace: vec![
            StackFrame {
                name: "use-variable".to_string(),
                location: SourceSpan::with_filename(
                    SourcePosition::new(5, 1, 67),
                    SourcePosition::new(5, 13, 79),
                    "variables.scm".to_string()
                ),
                frame_type: FrameType::Function,
            },
        ],
    };
    
    println!("{}", error.format_detailed());
    println!();
}

fn _demonstrate_host_integration() {
    println!("6. Integration with Host Applications:");
    println!("   - Error information can be extracted programmatically");
    println!("   - Source positions enable IDE integration");
    println!("   - Stack traces help with debugging complex call chains");
    println!("   - Structured error data supports custom error displays");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_position_formatting() {
        let pos = SourcePosition::new(10, 5, 123);
        assert_eq!(pos.to_string(), "line 10, column 5");
        
        let unknown_pos = SourcePosition::unknown();
        assert_eq!(unknown_pos.to_string(), "<unknown position>");
    }

    #[test]
    fn test_source_span_formatting() {
        let span = SourceSpan::with_filename(
            SourcePosition::new(5, 10, 50),
            SourcePosition::new(5, 20, 60),
            "test.scm".to_string()
        );
        assert_eq!(span.to_string(), "test.scm:line 5, column 10");
    }

    #[test]
    fn test_stack_frame_formatting() {
        let frame = StackFrame {
            name: "test-func".to_string(),
            location: SourceSpan::new(
                SourcePosition::new(1, 1, 0),
                SourcePosition::new(1, 10, 9)
            ),
            frame_type: FrameType::Function,
        };
        
        let formatted = frame.to_string();
        assert!(formatted.contains("test-func"));
        assert!(formatted.contains("line 1, column 1"));
    }

    #[test]
    fn test_error_with_stack_frame() {
        let mut error = LambdustError::runtime_error("Test error");
        
        let frame = StackFrame {
            name: "test-func".to_string(),
            location: SourceSpan::unknown(),
            frame_type: FrameType::Function,
        };
        
        error = error.with_stack_frame(frame);
        
        let formatted = error.format_detailed();
        assert!(formatted.contains("Stack trace:"));
        assert!(formatted.contains("test-func"));
    }
}