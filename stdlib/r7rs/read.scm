;; R7RS Standard Library - Read Module  
;; Provides input procedures for (scheme read)

(define-library (scheme read)
  (export
    ;; Core input procedures
    read
    
    ;; Character input
    read-char peek-char char-ready?
    
    ;; Line input
    read-line
    
    ;; String input  
    read-string
    
    ;; Binary input
    read-u8 peek-u8 u8-ready?
    read-bytevector read-bytevector!)

  (begin
    ;; Most read procedures are implemented as Rust primitives
    ;; and are automatically available in the global environment.
    
    ;; These procedures should be available from the Rust stdlib:
    ;; - read: reads Scheme expressions from input
    ;; - read-char: reads a single character
    ;; - peek-char: peeks at next character without consuming it
    ;; - char-ready?: tests if character input is available
    ;; - read-line: reads a line of text
    ;; - read-string: reads a string of specified length
    ;; - read-u8: reads a single byte
    ;; - peek-u8: peeks at next byte without consuming it  
    ;; - u8-ready?: tests if byte input is available
    ;; - read-bytevector: reads bytes into a bytevector
    ;; - read-bytevector!: reads bytes into an existing bytevector
    
    ;; The Rust primitives handle:
    ;; - Lexical analysis and parsing for read
    ;; - Character encoding (UTF-8)  
    ;; - Port management and EOF detection
    ;; - Buffering and lookahead for peek operations
    ;; - Error handling for malformed input
    ))