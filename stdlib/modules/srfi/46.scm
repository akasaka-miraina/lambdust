;; SRFI-46: Basic syntax-rules Extensions
;;
;; This SRFI provides two extensions to syntax-rules:
;; 1. Custom ellipsis identifiers: (syntax-rules ellipsis (literals...) rules...)  
;; 2. Tail patterns: (a b ... c d) - patterns with elements after ellipsis
;;
;; These extensions make it easier to write complex macros and simplify
;; the implementation of future SRFIs that depend on advanced macro facilities.

(define-library (srfi 46)
  (export syntax-rules)
  
  (import (scheme base))
  
  (begin
    ;; SRFI-46 extends the standard syntax-rules with:
    ;; - Optional custom ellipsis identifier as first argument
    ;; - Support for tail patterns (elements after ellipsis)
    ;;
    ;; Standard R7RS syntax:
    ;;   (syntax-rules (literal ...) 
    ;;     (pattern template) ...)
    ;;
    ;; SRFI-46 extended syntax:
    ;;   (syntax-rules ellipsis (literal ...)
    ;;     (pattern template) ...)
    ;;
    ;; Where ellipsis can be any identifier (e.g., ':::')
    
    ;; The implementation is provided by the core Lambdust macro system
    ;; in src/macro_system/syntax_rules.rs which now supports:
    ;; 1. Custom ellipsis detection via parse_custom_ellipsis()
    ;; 2. Ellipsis token threading through all parsing functions
    ;; 3. Tail pattern support in parse_list_pattern()
    
    ;; Examples of SRFI-46 usage:
    
    ;; Example 1: Custom ellipsis to avoid conflicts
    ;; (define-syntax my-macro
    ;;   (syntax-rules ::: (=>)
    ;;     ((my-macro (a b :::) => result)
    ;;      (list 'result a b :::))))
    
    ;; Example 2: Tail patterns 
    ;; (define-syntax list-with-suffix
    ;;   (syntax-rules ()
    ;;     ((list-with-suffix prefix ... suffix)
    ;;      (append (list prefix ...) (list suffix)))))
    
    ;; Example 3: Complex pattern matching
    ;; (define-syntax case-like
    ;;   (syntax-rules +++ (else)
    ;;     ((case-like expr 
    ;;        (pattern +++ guard result) +++
    ;;        (else default))
    ;;      (cond
    ;;        ((match-pattern expr pattern +++ guard) result) +++
    ;;        (else default)))))
    
    ;; Note: The actual syntax-rules implementation is built into the
    ;; Lambdust interpreter. This library serves as documentation and
    ;; compatibility layer for SRFI-46 compliance.
    
    ;; Test cases are provided in the comprehensive test suite to verify:
    ;; - Custom ellipsis identifier recognition  
    ;; - Tail pattern parsing and expansion
    ;; - Backward compatibility with standard R7RS syntax-rules
    ;; - Proper hygienic macro expansion with SRFI-46 extensions
    
    ;; Implementation status: COMPLETE
    ;; - Core parsing logic: ✓ Implemented in syntax_rules.rs
    ;; - Custom ellipsis support: ✓ parse_custom_ellipsis() 
    ;; - Tail pattern support: ✓ Enhanced parse_list_pattern()
    ;; - Template expansion: ✓ Custom ellipsis token threading
    ;; - R7RS compatibility: ✓ Maintains backward compatibility
    ;; - Hygienic properties: ✓ Preserved with SRFI-46 extensions
    
    ))