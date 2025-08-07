;; R7RS Standard Library - Load Module
;; Provides file loading procedures for (scheme load)

(define-library (scheme load)
  (export
    ;; File loading
    load)

  (begin
    ;; load reads and evaluates Scheme expressions from a file
    ;; in the current interaction environment
    (define (load filename . environment)
      "Loads and evaluates expressions from filename."
      ;; The load procedure should:
      ;; 1. Open the file for reading
      ;; 2. Read each expression from the file
      ;; 3. Evaluate each expression in the given environment
      ;;    (or interaction environment if none specified)
      ;; 4. Close the file when done
      ;; 5. Handle errors appropriately
      
      (let ((env (if (null? environment)
                     (interaction-environment)
                     (car environment))))
        ;; This implementation assumes basic I/O and eval procedures exist
        (call-with-input-file filename
          (lambda (port)
            (let loop ((result #f))
              (let ((expr (read port)))
                (if (eof-object? expr)
                    result
                    (loop (eval expr env)))))))))
    
    ;; Helper procedures that load depends on:
    ;; - interaction-environment (from scheme eval)
    ;; - call-with-input-file (from scheme file) 
    ;; - read (from scheme read)
    ;; - eval (from scheme eval)
    ;; - eof-object? (from scheme base)
    
    ;; Note: This is a pure Scheme implementation that relies on
    ;; the availability of lower-level I/O and evaluation primitives.
    ;; A Rust implementation would be more efficient and could provide
    ;; better error reporting and security features.
    ))