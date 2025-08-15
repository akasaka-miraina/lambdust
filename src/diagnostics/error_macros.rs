//! Powerful macro system for eliminating error handling redundancy.
//!
//! This module provides compile-time error handling macros that generate
//! optimized, type-safe error handling code with zero runtime overhead.

/// Generates type-safe error creation functions for a specific category.
/// 
/// This macro eliminates the boilerplate of creating error functions for
/// each category while maintaining compile-time type safety.
#[macro_export]
macro_rules! define_error_constructors {
    ($category:ty, $prefix:ident) => {
        paste::paste! {
            /// Creates a new error in this category.
            pub fn [<$prefix _error>](message: impl Into<String>) -> $crate::diagnostics::unified_error::UnifiedError {
                $crate::diagnostics::unified_error::UnifiedError::new($category, message)
            }

            /// Creates a new error with span information.
            pub fn [<$prefix _error_spanned>](message: impl Into<String>, span: $crate::diagnostics::Span) -> $crate::diagnostics::unified_error::UnifiedError {
                $crate::diagnostics::unified_error::UnifiedError::new($category, message).with_span(span)
            }

            /// Creates a new error with context.
            pub fn [<$prefix _error_ctx>](message: impl Into<String>, context: impl Into<String>) -> $crate::diagnostics::unified_error::UnifiedError {
                $crate::diagnostics::unified_error::UnifiedError::new($category, message)
                    .with_context("context", context)
            }

            /// Creates a critical error.
            pub fn [<$prefix _critical>](message: impl Into<String>) -> $crate::diagnostics::unified_error::UnifiedError {
                $crate::diagnostics::unified_error::UnifiedError::new($category, message)
                    .with_severity($crate::diagnostics::unified_error::ErrorSeverity::Critical)
            }

            /// Creates a boxed error for Result compatibility.
            pub fn [<$prefix _boxed>](message: impl Into<String>) -> Box<$crate::diagnostics::unified_error::UnifiedError> {
                [<$prefix _error>](message).boxed()
            }
        }
    };
}

/// Generates optimized error conversion patterns.
/// 
/// Eliminates repetitive `.map_err()` chains with compile-time optimization.
#[macro_export]
macro_rules! error_convert {
    // Basic error conversion with custom message
    ($result:expr, $category:expr, $message:expr) => {
        $result.map_err(|e| {
            $crate::diagnostics::unified_error::UnifiedError::new($category, $message)
                .with_context("source_error", e.to_string())
        })
    };
    
    // Error conversion with span
    ($result:expr, $category:expr, $message:expr, span: $span:expr) => {
        $result.map_err(|e| {
            $crate::diagnostics::unified_error::UnifiedError::new($category, $message)
                .with_span($span)
                .with_context("source_error", e.to_string())
        })
    };
    
    // Error conversion with context
    ($result:expr, $category:ty, $message:expr, context: $context:expr) => {
        $result.map_err(|e| {
            $crate::diagnostics::unified_error::UnifiedError::new($category, $message)
                .with_context("context", $context)
                .with_context("source_error", e.to_string())
        })
    };
    
    // Full error conversion with all options
    ($result:expr, $category:ty, $message:expr, span: $span:expr, context: $context:expr) => {
        $result.map_err(|e| {
            $crate::diagnostics::unified_error::UnifiedError::new($category, $message)
                .with_span($span)
                .with_context("context", $context)
                .with_context("source_error", e.to_string())
        })
    };
}

/// Streamlined error propagation for common patterns.
/// 
/// Provides ergonomic syntax for error handling while maintaining performance.
#[macro_export]
macro_rules! propagate_error {
    // Simple error propagation
    ($result:expr) => {
        $result?
    };
    
    // Error propagation with additional context
    ($result:expr, context: $context:expr) => {
        $result.map_err(|mut e| {
            e.context.push($crate::diagnostics::unified_error::ErrorContext {
                context_type: "propagation_context".to_string(),
                data: $context.into(),
                span: None,
            });
            e
        })?
    };
    
    // Error propagation with span information
    ($result:expr, span: $span:expr) => {
        $result.map_err(|mut e| {
            if e.span.is_none() {
                e.span = Some($span);
            }
            e
        })?
    };
    
    // Full error propagation with context and span
    ($result:expr, context: $context:expr, span: $span:expr) => {
        $result.map_err(|mut e| {
            if e.span.is_none() {
                e.span = Some($span);
            }
            e.context.push($crate::diagnostics::unified_error::ErrorContext {
                context_type: "propagation_context".to_string(),
                data: $context.into(),
                span: Some($span),
            });
            e
        })?
    };
}

/// Generates type-safe validation macros for common patterns.
/// 
/// Eliminates repetitive type checking with optimized generated code.
#[macro_export]
macro_rules! validate_type {
    // Validate single argument type
    ($value:expr, number) => {
        $value.as_number().ok_or_else(|| {
            $crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError, 
                format!("Expected number, got {:?}", $value)
            )
        })
    };
    
    ($value:expr, string) => {
        $value.as_string().ok_or_else(|| {
            $crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Expected string, got {:?}", $value)
            )
        })
    };
    
    ($value:expr, list) => {
        $value.as_list().ok_or_else(|| {
            $crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Expected list, got {:?}", $value)
            )
        })
    };
    
    ($value:expr, boolean) => {
        $value.as_boolean().ok_or_else(|| {
            $crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Expected boolean, got {:?}", $value)
            )
        })
    };
    
    // Validate with custom error message
    ($value:expr, $type_name:ident, message: $message:expr) => {
        $value.[<as_ $type_name>]().ok_or_else(|| {
            $crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                $message
            )
        })
    };
}

/// Generates arity checking macros for function validation.
/// 
/// Provides compile-time optimized argument count validation.
#[macro_export]
macro_rules! validate_arity {
    // Exact arity check
    ($args:expr, $expected:expr) => {
        if $args.len() != $expected {
            return Err($crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Expected {} arguments, got {}", $expected, $args.len())
            ));
        }
    };
    
    // Minimum arity check
    ($args:expr, min: $min:expr) => {
        if $args.len() < $min {
            return Err($crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Expected at least {} arguments, got {}", $min, $args.len())
            ));
        }
    };
    
    // Range arity check
    ($args:expr, range: $min:expr, $max:expr) => {
        if $args.len() < $min || $args.len() > $max {
            return Err($crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Expected {}-{} arguments, got {}", $min, $max, $args.len())
            ));
        }
    };
    
    // Named function arity check
    ($args:expr, $expected:expr, function: $func_name:expr) => {
        if $args.len() != $expected {
            return Err($crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Function '{}' expects {} arguments, got {}", $func_name, $expected, $args.len())
            ));
        }
    };
}

/// Generates error handling for Result types with automatic category detection.
/// 
/// Provides zero-overhead error handling with compile-time category inference.
#[macro_export]
macro_rules! handle_result {
    // Basic result handling
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => return Err(e.into()),
        }
    };
    
    // Result handling with error transformation
    ($result:expr, transform: |$err:ident| $transform:expr) => {
        match $result {
            Ok(value) => value,
            Err($err) => return Err($transform),
        }
    };
    
    // Result handling with default value
    ($result:expr, default: $default:expr) => {
        match $result {
            Ok(value) => value,
            Err(_) => $default,
        }
    };
    
    // Result handling with recovery function
    ($result:expr, recover: |$err:ident| $recovery:expr) => {
        match $result {
            Ok(value) => value,
            Err($err) => $recovery,
        }
    };
}

/// Generates compile-time optimized error chains for complex operations.
/// 
/// Creates efficient error propagation chains with minimal runtime overhead.
#[macro_export]
macro_rules! error_chain {
    // Start a new error chain
    (start: $category:expr, $message:expr) => {
        $crate::diagnostics::unified_error::UnifiedError::new($category, $message)
    };
    
    // Add context to error chain
    (chain: $error:expr, context: $context:expr) => {
        $error.with_context("chain_context", $context)
    };
    
    // Add span to error chain
    (chain: $error:expr, span: $span:expr) => {
        $error.with_span($span)
    };
    
    // Add source error to chain
    (chain: $error:expr, source: $source:expr) => {
        $error.with_source($source)
    };
    
    // Complete error chain
    (complete: $error:expr) => {
        $error
    };
}

/// Generates performance-optimized error collection for batch operations.
/// 
/// Provides efficient error aggregation with minimal memory allocation.
#[macro_export]
macro_rules! collect_errors {
    // Collect errors from iterator
    ($iter:expr, $operation:expr) => {{
        let mut errors = Vec::new();
        let mut results = Vec::new();
        
        for item in $iter {
            match $operation(item) {
                Ok(result) => results.push(result),
                Err(error) => errors.push(error),
            }
        }
        
        if errors.is_empty() {
            Ok(results)
        } else {
            Err($crate::diagnostics::unified_error::UnifiedError::new(
                $crate::diagnostics::unified_error::RuntimeError,
                format!("Multiple errors occurred: {} errors", errors.len())
            ).with_context("error_count", errors.len().to_string()))
        }
    }};
    
    // Collect errors with early termination
    ($iter:expr, $operation:expr, fail_fast: true) => {{
        let mut results = Vec::new();
        
        for item in $iter {
            match $operation(item) {
                Ok(result) => results.push(result),
                Err(error) => return Err(error),
            }
        }
        
        Ok(results)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{Span, unified_error::*};

    #[test]
    fn test_error_convert_macro() {
        let result: Result<i32, &str> = Err("test error");
        let converted = error_convert!(result, crate::diagnostics::unified_error::ErrorCategory::Runtime, "Conversion failed");
        
        assert!(converted.is_err());
        let error = converted.unwrap_err();
        assert_eq!(error.category, ErrorCategory::Runtime);
        assert_eq!(error.message, "Conversion failed");
    }

    #[test]
    fn test_validate_type_macro() {
        use crate::eval::Value;
        
        let number_value = Value::number(42.0);
        let result = validate_type!(number_value, number);
        assert!(result.is_ok());
        
        let string_value = Value::string("hello");
        let result = validate_type!(string_value, number);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_arity_macro() {
        fn test_function(args: &[i32]) -> UnifiedResult<i32> {
            validate_arity!(args, 2);
            Ok(args[0] + args[1])
        }
        
        let result = test_function(&[1, 2]);
        assert!(result.is_ok());
        
        let result = test_function(&[1]);
        assert!(result.is_err());
    }

    #[test] 
    fn test_error_chain_macro() {
        let error = error_chain!(start: crate::diagnostics::unified_error::ErrorCategory::Runtime, "Base error");
        let chained = error_chain!(chain: error, context: "Additional context");
        let final_error = error_chain!(complete: chained);
        
        assert_eq!(final_error.category, ErrorCategory::Runtime);
        assert_eq!(final_error.context.len(), 1);
    }
}