;; SRFI-9: Defining Record Types
;; 
;; This library provides a syntax for creating new data types with named fields.
;; Records created with this library are disjoint from all existing types.
;;
;; Reference: https://srfi.schemers.org/srfi-9/srfi-9.html

(define-library (srfi 9)
  (import (scheme base))
  
  (export
    define-record-type)

  (begin
    ;; The main macro for defining record types
    ;; 
    ;; Syntax: (define-record-type <type-name>
    ;;           (<constructor-name> <field-tag> ...)
    ;;           <predicate-name>
    ;;           (<field-tag> <accessor-name> [<modifier-name>]) ...)
    ;;
    ;; Example:
    ;; (define-record-type pare
    ;;   (kons x y)
    ;;   pare?
    ;;   (x kar set-kar!)
    ;;   (y kdr))
    
    (define-syntax define-record-type
      (syntax-rules ()
        ((_ type-name
            (constructor-name constructor-field ...)
            predicate-name
            (field-name accessor-name . modifier-spec) ...)
         (begin
           ;; Define the record type itself - using a unique tag
           (define type-tag (gensym 'record-type))
           
           ;; Define the constructor
           (define (constructor-name constructor-field ...)
             (vector type-tag constructor-field ...))
           
           ;; Define the predicate
           (define (predicate-name obj)
             (and (vector? obj)
                  (> (vector-length obj) 0)
                  (eq? (vector-ref obj 0) type-tag)))
           
           ;; Define field accessors and modifiers
           (define-record-accessors-and-modifiers
             type-tag predicate-name 1
             (field-name accessor-name . modifier-spec) ...)))))
    
    ;; Helper macro to define accessors and modifiers
    (define-syntax define-record-accessors-and-modifiers
      (syntax-rules ()
        ;; Base case: no more fields
        ((_ type-tag predicate-name index)
         (begin))
        
        ;; Field with accessor only
        ((_ type-tag predicate-name index 
            (field-name accessor-name) . rest)
         (begin
           (define (accessor-name record)
             (if (predicate-name record)
                 (vector-ref record index)
                 (error "Not a record of the expected type" record)))
           (define-record-accessors-and-modifiers
             type-tag predicate-name (+ index 1) . rest)))
        
        ;; Field with accessor and modifier
        ((_ type-tag predicate-name index
            (field-name accessor-name modifier-name) . rest)
         (begin
           (define (accessor-name record)
             (if (predicate-name record)
                 (vector-ref record index)
                 (error "Not a record of the expected type" record)))
           (define (modifier-name record value)
             (if (predicate-name record)
                 (vector-set! record index value)
                 (error "Not a record of the expected type" record)))
           (define-record-accessors-and-modifiers
             type-tag predicate-name (+ index 1) . rest)))))
    
    ;; Alternative implementation using association lists for debugging
    ;; This can be useful when vector-based implementation isn't available
    
    (define-syntax define-record-type-alist
      (syntax-rules ()
        ((_ type-name
            (constructor-name constructor-field ...)
            predicate-name
            (field-name accessor-name . modifier-spec) ...)
         (begin
           ;; Define the record type tag
           (define type-tag (gensym 'record-type))
           
           ;; Define the constructor
           (define (constructor-name constructor-field ...)
             (list type-tag
                   (cons 'constructor-field constructor-field) ...))
           
           ;; Define the predicate
           (define (predicate-name obj)
             (and (pair? obj)
                  (eq? (car obj) type-tag)))
           
           ;; Define field accessors and modifiers for alist representation
           (define-alist-accessors-and-modifiers
             type-tag predicate-name
             (field-name accessor-name . modifier-spec) ...)))))
    
    ;; Helper macro for alist-based record accessors
    (define-syntax define-alist-accessors-and-modifiers
      (syntax-rules ()
        ;; Base case: no more fields
        ((_ type-tag predicate-name)
         (begin))
        
        ;; Field with accessor only
        ((_ type-tag predicate-name
            (field-name accessor-name) . rest)
         (begin
           (define (accessor-name record)
             (if (predicate-name record)
                 (let ((field-alist (cdr record)))
                   (let ((entry (assq 'field-name field-alist)))
                     (if entry
                         (cdr entry)
                         (error "Field not found" 'field-name))))
                 (error "Not a record of the expected type" record)))
           (define-alist-accessors-and-modifiers
             type-tag predicate-name . rest)))
        
        ;; Field with accessor and modifier
        ((_ type-tag predicate-name
            (field-name accessor-name modifier-name) . rest)
         (begin
           (define (accessor-name record)
             (if (predicate-name record)
                 (let ((field-alist (cdr record)))
                   (let ((entry (assq 'field-name field-alist)))
                     (if entry
                         (cdr entry)
                         (error "Field not found" 'field-name))))
                 (error "Not a record of the expected type" record)))
           (define (modifier-name record value)
             (if (predicate-name record)
                 (let ((field-alist (cdr record)))
                   (let ((entry (assq 'field-name field-alist)))
                     (if entry
                         (set-cdr! entry value)
                         (error "Field not found" 'field-name))))
                 (error "Not a record of the expected type" record)))
           (define-alist-accessors-and-modifiers
             type-tag predicate-name . rest)))))
    
    ;; Simple implementation using pairs for two-field records
    ;; This demonstrates how SRFI-9 can be implemented with minimal primitives
    
    (define-syntax define-simple-pair-record
      (syntax-rules ()
        ((_ type-name
            (constructor-name field1 field2)
            predicate-name
            (field1 accessor1 . modifier1-spec)
            (field2 accessor2 . modifier2-spec))
         (begin
           ;; Use a unique tag for this record type
           (define type-tag (gensym 'simple-pair-record))
           
           ;; Constructor creates a tagged pair
           (define (constructor-name field1 field2)
             (cons type-tag (cons field1 field2)))
           
           ;; Predicate checks for the type tag
           (define (predicate-name obj)
             (and (pair? obj)
                  (eq? (car obj) type-tag)
                  (pair? (cdr obj))))
           
           ;; Field accessors
           (define (accessor1 record)
             (if (predicate-name record)
                 (cadr record)
                 (error "Not a record of the expected type" record)))
           
           (define (accessor2 record)
             (if (predicate-name record)
                 (cddr record)
                 (error "Not a record of the expected type" record)))
           
           ;; Optional modifiers
           (define-simple-pair-modifiers
             predicate-name (field1 accessor1 . modifier1-spec))
           (define-simple-pair-modifiers
             predicate-name (field2 accessor2 . modifier2-spec))))))
    
    ;; Helper for defining modifiers in simple pair records
    (define-syntax define-simple-pair-modifiers
      (syntax-rules ()
        ((_ predicate-name (field accessor))
         (begin))  ; No modifier
        ((_ predicate-name (field accessor modifier))
         (define (modifier record value)
           (if (predicate-name record)
               (if (eq? 'field 'field1)  ; First field
                   (set-car! (cdr record) value)
                   (set-cdr! (cdr record) value))  ; Second field
               (error "Not a record of the expected type" record))))))
    
    ;; Utility function to create a unique symbol (gensym)
    ;; This is a simplified version - in a full implementation,
    ;; this would ensure true uniqueness across the system
    (define gensym-counter 0)
    
    (define (gensym . prefix)
      (set! gensym-counter (+ gensym-counter 1))
      (if (null? prefix)
          (string->symbol (string-append "g" (number->string gensym-counter)))
          (string->symbol (string-append 
                          (symbol->string (car prefix))
                          (number->string gensym-counter)))))))