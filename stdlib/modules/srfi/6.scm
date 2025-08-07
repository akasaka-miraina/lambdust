;; SRFI-6: Basic String Ports
;; 
;; This library provides basic string port operations. String ports allow
;; reading from and writing to strings as if they were files or other I/O ports.
;; This functionality is now part of R7RS, so this module provides compatibility.
;;
;; Reference: https://srfi.schemers.org/srfi-6/srfi-6.html

(define-library (srfi 6)
  (import (scheme base))
  
  (export 
    open-input-string
    open-output-string
    get-output-string)

  (begin
    ;; SRFI-6 provides three fundamental string port operations:
    ;; 1. open-input-string - creates an input port from a string
    ;; 2. open-output-string - creates an output port that accumulates to a string
    ;; 3. get-output-string - extracts the accumulated string from an output port
    
    ;; These procedures are now part of R7RS (scheme base) and are implemented
    ;; in Lambdust's core I/O system. This module provides them for SRFI-6
    ;; compatibility and documents their behavior according to the SRFI specification.
    
    ;; ============= INPUT STRING PORTS =============
    
    ;; (open-input-string string) -> input-port
    ;; 
    ;; Takes a string and returns a textual input port that will deliver
    ;; characters from the string. The port will return EOF when all
    ;; characters have been read.
    ;;
    ;; If the string is modified after the port is created, the effect
    ;; on the port's behavior is unspecified.
    ;;
    ;; Examples:
    ;; (define port (open-input-string "hello world"))
    ;; (read-char port) => #\h
    ;; (read-string 5 port) => "ello "
    ;; (read-string 10 port) => "world"
    ;; (read-char port) => EOF
    
    ;; This procedure is provided by R7RS (scheme base)
    ;; We re-export it here for SRFI-6 compatibility
    
    ;; ============= OUTPUT STRING PORTS =============
    
    ;; (open-output-string) -> output-port
    ;;
    ;; Returns a textual output port that will accumulate characters
    ;; for retrieval by get-output-string. The port can be used with
    ;; any output procedures like write, display, write-char, etc.
    ;;
    ;; Example:
    ;; (define port (open-output-string))
    ;; (write-string "Hello " port)
    ;; (write-string "World!" port)
    ;; (get-output-string port) => "Hello World!"
    
    ;; This procedure is provided by R7RS (scheme base)
    ;; We re-export it here for SRFI-6 compatibility
    
    ;; (get-output-string port) -> string
    ;;
    ;; Given an output port created by open-output-string, returns a
    ;; string consisting of the characters that have been output to the
    ;; port so far. The port remains open and writable after this call.
    ;;
    ;; After calling get-output-string, output to the port will continue
    ;; to accumulate characters, but will start from the current accumulated
    ;; string rather than from empty.
    ;;
    ;; Note: The SRFI-6 specification is ambiguous about whether the port
    ;; should be reset after get-output-string. R7RS clarifies that the
    ;; port continues accumulating without reset.
    ;;
    ;; Examples:
    ;; (define port (open-output-string))
    ;; (display "Hello" port)
    ;; (get-output-string port) => "Hello"
    ;; (display " World" port)
    ;; (get-output-string port) => "Hello World"
    
    ;; This procedure is provided by R7RS (scheme base)
    ;; We re-export it here for SRFI-6 compatibility
    
    ;; ============= USAGE EXAMPLES =============
    
    ;; Example 1: String to list of tokens
    ;; (define (string-tokenize str delimiter)
    ;;   (let ((port (open-input-string str))
    ;;         (current-token (open-output-string))
    ;;         (tokens '()))
    ;;     (let loop ((ch (read-char port)))
    ;;       (cond
    ;;         ((eof-object? ch)
    ;;          (let ((final-token (get-output-string current-token)))
    ;;            (reverse (if (string=? final-token "")
    ;;                         tokens
    ;;                         (cons final-token tokens)))))
    ;;         ((char=? ch delimiter)
    ;;          (let ((token (get-output-string current-token)))
    ;;            (set! current-token (open-output-string))
    ;;            (loop (read-char port))))
    ;;         (else
    ;;          (write-char ch current-token)
    ;;          (loop (read-char port)))))))
    
    ;; Example 2: Building formatted output
    ;; (define (format-person name age city)
    ;;   (let ((port (open-output-string)))
    ;;     (display "Name: " port)
    ;;     (display name port)
    ;;     (display ", Age: " port)
    ;;     (display age port)
    ;;     (display ", City: " port)
    ;;     (display city port)
    ;;     (get-output-string port)))
    
    ;; Example 3: S-expression to string conversion
    ;; (define (sexp->string expr)
    ;;   (let ((port (open-output-string)))
    ;;     (write expr port)
    ;;     (get-output-string port)))
    
    ;; Example 4: String parsing with error recovery
    ;; (define (safe-read-from-string str)
    ;;   (guard (exception
    ;;           (else #f))
    ;;     (let ((port (open-input-string str)))
    ;;       (read port))))
    
    ;; ============= COMPATIBILITY NOTES =============
    
    ;; SRFI-6 vs R7RS differences:
    ;; 1. SRFI-6 was less specific about port behavior after get-output-string
    ;; 2. R7RS clarifies that ports remain open and continue accumulating
    ;; 3. Both specifications agree on the basic functionality
    
    ;; Performance considerations:
    ;; - String ports are typically implemented with dynamic string buffers
    ;; - Reading from string ports is usually faster than file I/O
    ;; - Writing to string ports may involve string copying/reallocation
    ;; - For large strings, consider using bytevector ports if available
    
    ;; Thread safety:
    ;; - String ports, like all ports, are not inherently thread-safe
    ;; - Concurrent access to the same string port requires external synchronization
    ;; - Each thread should typically use its own string ports
    
    ;; Memory management:
    ;; - String ports hold references to their string data
    ;; - Input string ports typically don't copy the input string
    ;; - Output string ports manage their own accumulation buffer
    ;; - Ports should be closed when no longer needed, though GC will handle them
    
    ;; Integration with other I/O:
    ;; (define (copy-string-to-file str filename)
    ;;   (let ((input-port (open-input-string str)))
    ;;     (call-with-output-file filename
    ;;       (lambda (output-port)
    ;;         (let loop ((ch (read-char input-port)))
    ;;           (unless (eof-object? ch)
    ;;             (write-char ch output-port)
    ;;             (loop (read-char input-port))))))))
    
    ;; Error handling patterns:
    ;; - Invalid string arguments to open-input-string should signal an error
    ;; - get-output-string with invalid port should signal an error
    ;; - EOF conditions on string input ports are normal, not errors
    ;; - String output ports generally don't have write failures
    
    ;; Advanced usage with call/cc:
    ;; (define (string-parser str)
    ;;   (call-with-current-continuation
    ;;     (lambda (return)
    ;;       (let ((port (open-input-string str)))
    ;;         (let loop ()
    ;;           (let ((ch (read-char port)))
    ;;             (cond
    ;;               ((eof-object? ch) (return 'end-of-string))
    ;;               ((char=? ch #\space) (loop))
    ;;               (else (return ch)))))))))
    ))