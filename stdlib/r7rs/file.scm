;; R7RS Standard Library - File Module
;; Provides file I/O procedures for (scheme file)

(define-library (scheme file)
  (export
    ;; File port operations
    open-input-file open-output-file open-binary-input-file open-binary-output-file
    close-port close-input-port close-output-port
    
    ;; File system operations
    file-exists? delete-file
    
    ;; Port predicates
    input-port? output-port? textual-port? binary-port? port?
    
    ;; Current ports
    current-input-port current-output-port current-error-port
    
    ;; Port operations with files
    call-with-input-file call-with-output-file
    with-input-from-file with-output-to-file)

  (begin
    ;; Most file procedures are implemented as Rust primitives
    ;; and are automatically available in the global environment.
    
    ;; These procedures should be available from the Rust stdlib:
    ;; - open-input-file: opens a file for reading
    ;; - open-output-file: opens a file for writing  
    ;; - open-binary-input-file: opens a binary file for reading
    ;; - open-binary-output-file: opens a binary file for writing
    ;; - close-port, close-input-port, close-output-port: closes ports
    ;; - file-exists?: tests if a file exists
    ;; - delete-file: deletes a file
    ;; - Port predicates: input-port?, output-port?, etc.
    ;; - current-input-port, current-output-port, current-error-port: current ports
    
    ;; Implementation of convenience procedures that build on primitives:
    
    (define (call-with-input-file filename proc)
      "Calls proc with an input port opened on filename, ensuring the port is closed."
      (let ((port (open-input-file filename)))
        (let ((result (proc port)))
          (close-input-port port)
          result)))
          
    (define (call-with-output-file filename proc)
      "Calls proc with an output port opened on filename, ensuring the port is closed."
      (let ((port (open-output-file filename)))
        (let ((result (proc port)))
          (close-output-port port)
          result)))
          
    (define (with-input-from-file filename thunk)
      "Calls thunk with current-input-port set to file."
      (let ((old-input (current-input-port))
            (port (open-input-file filename)))
        (dynamic-wind
          (lambda () (set-current-input-port! port))
          thunk
          (lambda () 
            (close-input-port port)
            (set-current-input-port! old-input)))))
            
    (define (with-output-to-file filename thunk)
      "Calls thunk with current-output-port set to file."
      (let ((old-output (current-output-port))
            (port (open-output-file filename)))
        (dynamic-wind
          (lambda () (set-current-output-port! port))
          thunk
          (lambda ()
            (close-output-port port)
            (set-current-output-port! old-output)))))
    
    ;; Note: The dynamic-wind and parameter setting procedures above
    ;; assume the existence of set-current-input-port! and set-current-output-port!
    ;; These may need to be implemented as Rust primitives or using parameters.
    ))