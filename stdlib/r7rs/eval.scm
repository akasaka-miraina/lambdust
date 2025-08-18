;; R7RS Standard Library - Eval Module
;; Provides dynamic evaluation procedures for (scheme eval)

(define-library (scheme eval)
  (export
    ;; Dynamic evaluation
    eval
    
    ;; Environment operations
    environment
    scheme-report-environment
    null-environment
    interaction-environment)

  (begin
    ;; All procedures in this module are implemented as Rust primitives
    ;; in src/stdlib/eval_operations.rs and are automatically available
    ;; when this module is imported.
    
    ;; eval - Implemented as a Rust primitive that evaluates arbitrary 
    ;; Scheme expressions in a given environment with security controls
    
    ;; environment - Implemented as a Rust primitive that creates a new 
    ;; evaluation environment from import sets by loading and importing
    ;; the specified modules
    
    ;; environment-bound? - Implemented as a Rust primitive that checks
    ;; if a symbol is bound in the given environment
    
    ;; scheme-report-environment - Implemented as a Rust primitive that
    ;; returns an R5RS-compatible environment for the given version
    
    ;; null-environment - Implemented as a Rust primitive that returns
    ;; a minimal environment with only special forms and syntax
    
    ;; interaction-environment - Implemented as a Rust primitive that
    ;; returns the current top-level/REPL environment
    
    ;; These procedures provide complete R7RS (scheme eval) compliance
    ;; and enable dynamic code evaluation with proper security controls.
    ))