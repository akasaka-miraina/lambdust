;; R7RS Standard Library - Write Module
;; Provides output procedures avoiding primitive name collisions

(define-library (scheme write)
  (export 
    ;; Core output procedures  
    display write
    
    ;; Character and string output
    write-char write-string
    
    ;; Line termination
    newline
    
    ;; Output flushing  
    flush-output-port)

  (begin
    ;; All R7RS write functions are implemented as Rust primitives.
    ;; This module simply re-exports them to avoid name collisions.
    #t))