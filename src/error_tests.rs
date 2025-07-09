//! Comprehensive unit tests for the error handling system
//!
//! Tests the complete error handling functionality including error types,
//! source positions, stack traces, error context, and error reporting.

use crate::error::{
    ErrorContext, FrameType, LambdustError, Result, SourcePosition, SourceSpan, StackFrame,
};

#[cfg(test)]
mod source_position_tests {
    use super::*;

    #[test]
    fn test_source_position_new() {
        let pos = SourcePosition::new(10, 5, 100);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 100);
    }

    #[test]
    fn test_source_position_unknown() {
        let pos = SourcePosition::unknown();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
        assert_eq!(pos.offset, 0);
    }

    #[test]
    fn test_source_position_display_known() {
        let pos = SourcePosition::new(10, 5, 100);
        assert_eq!(format!("{}", pos), "line 10, column 5");
    }

    #[test]
    fn test_source_position_display_unknown() {
        let pos = SourcePosition::unknown();
        assert_eq!(format!("{}", pos), "<unknown position>");
    }

    #[test]
    fn test_source_position_equality() {
        let pos1 = SourcePosition::new(10, 5, 100);
        let pos2 = SourcePosition::new(10, 5, 100);
        let pos3 = SourcePosition::new(10, 5, 101);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_source_position_clone() {
        let pos = SourcePosition::new(10, 5, 100);
        let cloned = pos.clone();
        assert_eq!(pos, cloned);
    }

    #[test]
    fn test_source_position_edge_cases() {
        let pos_zero = SourcePosition::new(0, 0, 0);
        let pos_max = SourcePosition::new(usize::MAX, usize::MAX, usize::MAX);
        assert_eq!(pos_zero.line, 0);
        assert_eq!(pos_max.line, usize::MAX);
    }
}

#[cfg(test)]
mod source_span_tests {
    use super::*;

    #[test]
    fn test_source_span_new() {
        let start = SourcePosition::new(10, 5, 100);
        let end = SourcePosition::new(10, 15, 110);
        let span = SourceSpan::new(start.clone(), end.clone());
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
        assert_eq!(span.filename, None);
    }

    #[test]
    fn test_source_span_with_filename() {
        let start = SourcePosition::new(10, 5, 100);
        let end = SourcePosition::new(10, 15, 110);
        let span = SourceSpan::with_filename(start.clone(), end.clone(), "test.scm".to_string());
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
        assert_eq!(span.filename, Some("test.scm".to_string()));
    }

    #[test]
    fn test_source_span_unknown() {
        let span = SourceSpan::unknown();
        assert_eq!(span.start, SourcePosition::unknown());
        assert_eq!(span.end, SourcePosition::unknown());
        assert_eq!(span.filename, None);
    }

    #[test]
    fn test_source_span_point() {
        let pos = SourcePosition::new(10, 5, 100);
        let span = SourceSpan::point(pos.clone());
        assert_eq!(span.start, pos);
        assert_eq!(span.end, pos);
        assert_eq!(span.filename, None);
    }

    #[test]
    fn test_source_span_display_with_filename() {
        let start = SourcePosition::new(10, 5, 100);
        let end = SourcePosition::new(10, 15, 110);
        let span = SourceSpan::with_filename(start, end, "test.scm".to_string());
        assert_eq!(format!("{}", span), "test.scm:line 10, column 5");
    }

    #[test]
    fn test_source_span_display_without_filename() {
        let start = SourcePosition::new(10, 5, 100);
        let end = SourcePosition::new(10, 15, 110);
        let span = SourceSpan::new(start, end);
        assert_eq!(format!("{}", span), "line 10, column 5");
    }

    #[test]
    fn test_source_span_display_unknown() {
        let span = SourceSpan::unknown();
        assert_eq!(format!("{}", span), "<unknown position>");
    }

    #[test]
    fn test_source_span_equality() {
        let start = SourcePosition::new(10, 5, 100);
        let end = SourcePosition::new(10, 15, 110);
        let span1 = SourceSpan::new(start.clone(), end.clone());
        let span2 = SourceSpan::new(start.clone(), end.clone());
        let span3 = SourceSpan::with_filename(start, end, "test.scm".to_string());
        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }
}

#[cfg(test)]
mod stack_frame_tests {
    use super::*;

    #[test]
    fn test_stack_frame_creation() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let frame = StackFrame {
            name: "test-function".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        assert_eq!(frame.name, "test-function");
        assert_eq!(frame.location, location);
        assert_eq!(frame.frame_type, FrameType::Function);
    }

    #[test]
    fn test_frame_type_display_function() {
        let frame = StackFrame {
            name: "test-function".to_string(),
            location: SourceSpan::point(SourcePosition::new(10, 5, 100)),
            frame_type: FrameType::Function,
        };
        assert_eq!(
            format!("{}", frame),
            "  at test-function (line 10, column 5)"
        );
    }

    #[test]
    fn test_frame_type_display_builtin() {
        let frame = StackFrame {
            name: "+".to_string(),
            location: SourceSpan::point(SourcePosition::new(10, 5, 100)),
            frame_type: FrameType::Builtin,
        };
        assert_eq!(format!("{}", frame), "  at + <builtin> (line 10, column 5)");
    }

    #[test]
    fn test_frame_type_display_special_form() {
        let frame = StackFrame {
            name: "if".to_string(),
            location: SourceSpan::point(SourcePosition::new(10, 5, 100)),
            frame_type: FrameType::SpecialForm,
        };
        assert_eq!(format!("{}", frame), "  in if form (line 10, column 5)");
    }

    #[test]
    fn test_frame_type_display_macro() {
        let frame = StackFrame {
            name: "when".to_string(),
            location: SourceSpan::point(SourcePosition::new(10, 5, 100)),
            frame_type: FrameType::Macro,
        };
        assert_eq!(format!("{}", frame), "  in when macro (line 10, column 5)");
    }

    #[test]
    fn test_frame_type_display_top_level() {
        let frame = StackFrame {
            name: "top-level".to_string(),
            location: SourceSpan::point(SourcePosition::new(10, 5, 100)),
            frame_type: FrameType::TopLevel,
        };
        assert_eq!(format!("{}", frame), "  at top level (line 10, column 5)");
    }

    #[test]
    fn test_frame_type_equality() {
        assert_eq!(FrameType::Function, FrameType::Function);
        assert_eq!(FrameType::Builtin, FrameType::Builtin);
        assert_ne!(FrameType::Function, FrameType::Builtin);
    }

    #[test]
    fn test_stack_frame_equality() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let frame1 = StackFrame {
            name: "test".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let frame2 = StackFrame {
            name: "test".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let frame3 = StackFrame {
            name: "test".to_string(),
            location,
            frame_type: FrameType::Builtin,
        };
        assert_eq!(frame1, frame2);
        assert_ne!(frame1, frame3);
    }

    #[test]
    fn test_stack_frame_clone() {
        let frame = StackFrame {
            name: "test".to_string(),
            location: SourceSpan::point(SourcePosition::new(10, 5, 100)),
            frame_type: FrameType::Function,
        };
        let cloned = frame.clone();
        assert_eq!(frame, cloned);
    }
}

#[cfg(test)]
mod error_context_tests {
    use super::*;

    #[test]
    fn test_error_context_new() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let frame = StackFrame {
            name: "test".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let stack_trace = vec![frame];
        let context = ErrorContext::new(location.clone(), stack_trace.clone());
        assert_eq!(context.location, location);
        assert_eq!(context.stack_trace, stack_trace);
    }

    #[test]
    fn test_error_context_unknown() {
        let context = ErrorContext::unknown();
        assert_eq!(context.location, SourceSpan::unknown());
        assert_eq!(context.stack_trace, Vec::new());
    }

    #[test]
    fn test_error_context_clone() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let context = ErrorContext::new(location.clone(), Vec::new());
        let cloned = context.clone();
        assert_eq!(context, cloned);
    }

    #[test]
    fn test_error_context_equality() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let context1 = ErrorContext::new(location.clone(), Vec::new());
        let context2 = ErrorContext::new(location.clone(), Vec::new());
        let context3 = ErrorContext::unknown();
        assert_eq!(context1, context2);
        assert_ne!(context1, context3);
    }
}

#[cfg(test)]
mod lambdust_error_tests {
    use super::*;

    #[test]
    fn test_runtime_error_creation() {
        let error = LambdustError::runtime_error("test message");
        match error {
            LambdustError::RuntimeError { message, .. } => {
                assert_eq!(message, "test message");
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_type_error_creation() {
        let error = LambdustError::type_error("test message");
        match error {
            LambdustError::TypeError { message, .. } => {
                assert_eq!(message, "test message");
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_undefined_variable_creation() {
        let error = LambdustError::undefined_variable("test_var");
        match error {
            LambdustError::UndefinedVariable { variable, .. } => {
                assert_eq!(variable, "test_var");
            }
            _ => panic!("Expected UndefinedVariable"),
        }
    }

    #[test]
    fn test_syntax_error_creation() {
        let error = LambdustError::syntax_error("test message");
        match error {
            LambdustError::SyntaxError { message, .. } => {
                assert_eq!(message, "test message");
            }
            _ => panic!("Expected SyntaxError"),
        }
    }

    #[test]
    fn test_parse_error_creation() {
        let error = LambdustError::parse_error("test message");
        match error {
            LambdustError::ParseError { message, .. } => {
                assert_eq!(message, "test message");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_lexer_error_creation() {
        let error = LambdustError::lexer_error("test message");
        match error {
            LambdustError::LexerError { message, .. } => {
                assert_eq!(message, "test message");
            }
            _ => panic!("Expected LexerError"),
        }
    }

    #[test]
    fn test_arity_error_creation() {
        let error = LambdustError::arity_error(2, 3);
        match error {
            LambdustError::ArityError {
                expected, actual, ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected ArityError"),
        }
    }

    #[test]
    fn test_arity_error_range() {
        let error = LambdustError::arity_error_range(2, 4, 5);
        match error {
            LambdustError::ArityError {
                expected,
                actual,
                function,
                ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 5);
                assert_eq!(function, "expected 2-4 arguments");
            }
            _ => panic!("Expected ArityError"),
        }
    }

    #[test]
    fn test_arity_error_min() {
        let error = LambdustError::arity_error_min(2, 1);
        match error {
            LambdustError::ArityError {
                expected,
                actual,
                function,
                ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
                assert_eq!(function, "expected at least 2 arguments");
            }
            _ => panic!("Expected ArityError"),
        }
    }

    #[test]
    fn test_division_by_zero_creation() {
        let error = LambdustError::division_by_zero();
        match error {
            LambdustError::DivisionByZero { .. } => {
                // Test passes if we get the right variant
            }
            _ => panic!("Expected DivisionByZero"),
        }
    }

    #[test]
    fn test_stack_overflow_creation() {
        let error = LambdustError::stack_overflow();
        match error {
            LambdustError::StackOverflow { .. } => {
                // Test passes if we get the right variant
            }
            _ => panic!("Expected StackOverflow"),
        }
    }

    #[test]
    fn test_io_error_creation() {
        let error = LambdustError::io_error("test message");
        match error {
            LambdustError::IoError { message, .. } => {
                assert_eq!(message, "test message");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_io_error_from_std_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = LambdustError::from(io_error);
        match error {
            LambdustError::IoError { message, .. } => {
                assert!(message.contains("file not found"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_error_with_stack_frame() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let frame = StackFrame {
            name: "test".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let error = LambdustError::runtime_error("test").with_stack_frame(frame.clone());
        match error {
            LambdustError::RuntimeError { context, .. } => {
                assert_eq!(context.stack_trace.len(), 1);
                assert_eq!(context.stack_trace[0], frame);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_error_with_location() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let error = LambdustError::runtime_error("test").with_location(location.clone());
        match error {
            LambdustError::RuntimeError { context, .. } => {
                assert_eq!(context.location, location);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_error_clone() {
        let error = LambdustError::runtime_error("test");
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_error_equality() {
        let error1 = LambdustError::runtime_error("test");
        let error2 = LambdustError::runtime_error("test");
        let error3 = LambdustError::runtime_error("different");
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}

#[cfg(test)]
mod legacy_constructor_tests {
    use super::*;

    #[test]
    fn test_legacy_type_error_old() {
        let error = LambdustError::type_error_old("test".to_string());
        match error {
            LambdustError::TypeError { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_legacy_undefined_variable_old() {
        let error = LambdustError::undefined_variable_old("test".to_string());
        match error {
            LambdustError::UndefinedVariable { variable, .. } => {
                assert_eq!(variable, "test");
            }
            _ => panic!("Expected UndefinedVariable"),
        }
    }

    #[test]
    fn test_legacy_syntax_error_old() {
        let error = LambdustError::syntax_error_old("test".to_string());
        match error {
            LambdustError::SyntaxError { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected SyntaxError"),
        }
    }

    #[test]
    fn test_legacy_lexer_error_old() {
        let error = LambdustError::lexer_error_old("test".to_string());
        match error {
            LambdustError::LexerError { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected LexerError"),
        }
    }

    #[test]
    fn test_legacy_parse_error_old() {
        let error = LambdustError::parse_error_old("test".to_string());
        match error {
            LambdustError::ParseError { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_legacy_arity_error_old() {
        let error = LambdustError::arity_error_old(2, 3);
        match error {
            LambdustError::ArityError {
                expected, actual, ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected ArityError"),
        }
    }

    #[test]
    fn test_legacy_arity_error_struct() {
        let error = LambdustError::arity_error_struct(2, 3);
        match error {
            LambdustError::ArityError {
                expected,
                actual,
                function,
                ..
            } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 3);
                assert_eq!(function, "<unknown>");
            }
            _ => panic!("Expected ArityError"),
        }
    }

    #[test]
    fn test_legacy_division_by_zero_old() {
        let error = LambdustError::division_by_zero_old();
        match error {
            LambdustError::DivisionByZero { .. } => {
                // Test passes if we get the right variant
            }
            _ => panic!("Expected DivisionByZero"),
        }
    }

    #[test]
    fn test_legacy_stack_overflow_old() {
        let error = LambdustError::stack_overflow_old();
        match error {
            LambdustError::StackOverflow { .. } => {
                // Test passes if we get the right variant
            }
            _ => panic!("Expected StackOverflow"),
        }
    }

    #[test]
    fn test_legacy_macro_error_old() {
        let error = LambdustError::macro_error_old("test".to_string());
        match error {
            LambdustError::MacroError { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected MacroError"),
        }
    }

    #[test]
    fn test_legacy_io_error_old() {
        let error = LambdustError::io_error_old("test".to_string());
        match error {
            LambdustError::IoError { message, .. } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected IoError"),
        }
    }
}

#[cfg(test)]
mod error_display_tests {
    use super::*;

    #[test]
    fn test_runtime_error_display() {
        let error = LambdustError::runtime_error("test message");
        assert_eq!(format!("{}", error), "Runtime error: test message");
    }

    #[test]
    fn test_type_error_display() {
        let error = LambdustError::type_error("test message");
        assert_eq!(format!("{}", error), "Type error: test message");
    }

    #[test]
    fn test_undefined_variable_display() {
        let error = LambdustError::undefined_variable("test_var");
        assert_eq!(format!("{}", error), "Undefined variable: test_var");
    }

    #[test]
    fn test_syntax_error_display() {
        let error = LambdustError::syntax_error("test message");
        assert_eq!(format!("{}", error), "Syntax error: test message");
    }

    #[test]
    fn test_parse_error_display() {
        let error = LambdustError::parse_error("test message");
        assert_eq!(format!("{}", error), "Parse error: test message");
    }

    #[test]
    fn test_lexer_error_display() {
        let error = LambdustError::lexer_error("test message");
        assert_eq!(format!("{}", error), "Lexer error: test message");
    }

    #[test]
    fn test_arity_error_display() {
        let error = LambdustError::arity_error(2, 3);
        assert_eq!(format!("{}", error), "Arity error: expected 2, got 3");
    }

    #[test]
    fn test_division_by_zero_display() {
        let error = LambdustError::division_by_zero();
        assert_eq!(format!("{}", error), "Division by zero");
    }

    #[test]
    fn test_stack_overflow_display() {
        let error = LambdustError::stack_overflow();
        assert_eq!(format!("{}", error), "Stack overflow");
    }

    #[test]
    fn test_io_error_display() {
        let error = LambdustError::io_error("test message");
        assert_eq!(format!("{}", error), "I/O error: test message");
    }

    #[test]
    fn test_macro_error_display() {
        let error = LambdustError::MacroError {
            message: "test message".to_string(),
            context: Box::new(ErrorContext::unknown()),
        };
        assert_eq!(format!("{}", error), "Macro error: test message");
    }
}

#[cfg(test)]
mod error_detailed_formatting_tests {
    use super::*;

    #[test]
    fn test_runtime_error_detailed_no_context() {
        let error = LambdustError::runtime_error("test message");
        let detailed = error.format_detailed();
        assert!(detailed.contains("Error: Runtime error: test message"));
    }

    #[test]
    fn test_runtime_error_detailed_with_location() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let error = LambdustError::runtime_error("test message").with_location(location);
        let detailed = error.format_detailed();
        assert!(detailed.contains("Error: Runtime error: test message"));
        assert!(detailed.contains("at line 10, column 5"));
    }

    #[test]
    fn test_runtime_error_detailed_with_stack_trace() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let frame = StackFrame {
            name: "test-function".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let error = LambdustError::runtime_error("test message").with_stack_frame(frame);
        let detailed = error.format_detailed();
        assert!(detailed.contains("Error: Runtime error: test message"));
        assert!(detailed.contains("Stack trace:"));
        assert!(detailed.contains("at test-function"));
    }

    #[test]
    fn test_syntax_error_detailed_with_location() {
        let location = SourceSpan::with_filename(
            SourcePosition::new(10, 5, 100),
            SourcePosition::new(10, 15, 110),
            "test.scm".to_string(),
        );
        let error = LambdustError::syntax_error("test message").with_location(location);
        let detailed = error.format_detailed();
        assert!(detailed.contains("Error: Syntax error: test message"));
        assert!(detailed.contains("at test.scm:line 10, column 5"));
    }

    #[test]
    fn test_io_error_detailed_with_location() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let error = LambdustError::IoError {
            message: "test message".to_string(),
            location: Some(location),
        };
        let detailed = error.format_detailed();
        assert!(detailed.contains("Error: I/O error: test message"));
        assert!(detailed.contains("at line 10, column 5"));
    }

    #[test]
    fn test_multiple_stack_frames_detailed() {
        let location = SourceSpan::point(SourcePosition::new(10, 5, 100));
        let frame1 = StackFrame {
            name: "inner".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let frame2 = StackFrame {
            name: "outer".to_string(),
            location: location.clone(),
            frame_type: FrameType::Function,
        };
        let error = LambdustError::runtime_error("test message")
            .with_stack_frame(frame1)
            .with_stack_frame(frame2);
        let detailed = error.format_detailed();
        assert!(detailed.contains("Stack trace:"));
        assert!(detailed.contains("at inner"));
        assert!(detailed.contains("at outer"));
    }

    #[test]
    fn test_error_detailed_empty_stack_trace() {
        let error = LambdustError::runtime_error("test message");
        let detailed = error.format_detailed();
        assert!(detailed.contains("Error: Runtime error: test message"));
        assert!(!detailed.contains("Stack trace:"));
    }
}

#[cfg(test)]
mod result_type_tests {
    use super::*;

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(LambdustError::runtime_error("test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_result_map() {
        let result: Result<i32> = Ok(42);
        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.unwrap(), 84);
    }

    #[test]
    fn test_result_map_err() {
        let result: Result<i32> = Err(LambdustError::runtime_error("test"));
        let mapped = result.map_err(|e| LambdustError::type_error(format!("wrapped: {}", e)));
        assert!(mapped.is_err());
        assert!(format!("{}", mapped.unwrap_err()).contains("wrapped"));
    }

    #[test]
    fn test_result_and_then() {
        let result: Result<i32> = Ok(42);
        let chained = result.map(|x| x * 2);
        assert_eq!(chained, Ok(84));
    }

    #[test]
    fn test_result_or_else() {
        let result: Result<i32> = Err(LambdustError::runtime_error("test"));
        let alternative: Result<i32> = result.or(Ok(42));
        assert_eq!(alternative, Ok(42));
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_very_long_error_message() {
        let long_message = "a".repeat(10000);
        let error = LambdustError::runtime_error(long_message.clone());
        match error {
            LambdustError::RuntimeError { message, .. } => {
                assert_eq!(message, long_message);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_unicode_error_message() {
        let unicode_message = "エラー: 変数が見つかりません";
        let error = LambdustError::runtime_error(unicode_message);
        match error {
            LambdustError::RuntimeError { message, .. } => {
                assert_eq!(message, unicode_message);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_empty_error_message() {
        let error = LambdustError::runtime_error("");
        match error {
            LambdustError::RuntimeError { message, .. } => {
                assert_eq!(message, "");
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_error_with_newlines() {
        let error = LambdustError::runtime_error("line1\nline2\nline3");
        let detailed = error.format_detailed();
        assert!(detailed.contains("line1"));
        assert!(detailed.contains("line2"));
        assert!(detailed.contains("line3"));
    }

    #[test]
    fn test_error_with_special_characters() {
        let error = LambdustError::runtime_error("error: \"quoted\" text with \t tabs");
        let detailed = error.format_detailed();
        assert!(detailed.contains("\"quoted\""));
        assert!(detailed.contains("\t"));
    }

    #[test]
    fn test_maximum_arity_values() {
        let error = LambdustError::arity_error(usize::MAX, usize::MAX - 1);
        match error {
            LambdustError::ArityError {
                expected, actual, ..
            } => {
                assert_eq!(expected, usize::MAX);
                assert_eq!(actual, usize::MAX - 1);
            }
            _ => panic!("Expected ArityError"),
        }
    }

    #[test]
    fn test_error_chaining() {
        let error = LambdustError::runtime_error("original")
            .with_location(SourceSpan::point(SourcePosition::new(1, 1, 0)))
            .with_stack_frame(StackFrame {
                name: "frame1".to_string(),
                location: SourceSpan::point(SourcePosition::new(1, 1, 0)),
                frame_type: FrameType::Function,
            })
            .with_stack_frame(StackFrame {
                name: "frame2".to_string(),
                location: SourceSpan::point(SourcePosition::new(2, 1, 10)),
                frame_type: FrameType::Function,
            });
        let detailed = error.format_detailed();
        assert!(detailed.contains("original"));
        assert!(detailed.contains("frame1"));
        assert!(detailed.contains("frame2"));
    }
}
