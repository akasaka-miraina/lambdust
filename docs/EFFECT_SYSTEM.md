# Effect System Guide

Lambdust implements a sophisticated algebraic effect system that provides composable, type-safe side effects and monadic programming. This guide covers the complete effect system implementation and advanced usage patterns.

## Overview

The Lambdust effect system provides:

- **Algebraic Effects**: Composable effects with handler-based interpretation
- **Effect Handlers**: Pluggable interpretations for different execution contexts
- **Monadic Programming**: Full monad and monad transformer support
- **Effect Coordination**: Advanced coordination for concurrent effect execution
- **Effect Isolation**: Sandboxing and resource management for safe effect execution
- **Generational Effects**: Memory management integration with effect lifecycles

## Core Concepts

### Effects as Algebraic Operations

Effects are defined as algebraic operations with signatures:

```scheme
;; Define an effect type
(define-effect-type Console
  ;; Operations with their signatures
  (print : String -> Unit)
  (read-line : Unit -> String)
  (print-error : String -> Unit))

;; Define stateful effects
(define-effect-type (State s)
  (get : Unit -> s)
  (put : s -> Unit)
  (modify : (s -> s) -> Unit))

;; Define resource effects
(define-effect-type Resource
  (acquire : String -> Handle)
  (release : Handle -> Unit)
  (with-resource : String (Handle -> a) -> a))
```

### Effect Handlers

Handlers provide concrete interpretations for effects:

```scheme
;; Console handler using standard I/O
(define console-io-handler
  (handler Console
    [(print msg) 
     (display msg)
     (newline)]
    [(read-line)
     (read-line (current-input-port))]
    [(print-error msg)
     (display msg (current-error-port))
     (newline (current-error-port))]))

;; Console handler for testing (collects output)
(define (console-test-handler)
  (let ([output-buffer '()]
        [input-queue '()])
    (handler Console
      [(print msg)
       (set! output-buffer (cons msg output-buffer))]
      [(read-line)
       (if (null? input-queue)
           (error "No input available in test")
           (let ([input (car input-queue)])
             (set! input-queue (cdr input-queue))
             input))]
      [(print-error msg)
       (set! output-buffer (cons (string-append "ERROR: " msg) output-buffer))]
      ;; Provide access to collected output
      [get-output (reverse output-buffer)]
      [set-input! (lambda (inputs) (set! input-queue inputs))])))
```

## Basic Effect Usage

### Simple Effect Programs

```scheme
;; Basic effectful computation
(define (greet-user)
  (do [_ (print "What's your name?")]
      [name (read-line)]
      [_ (print (string-append "Hello, " name "!"))]
      (return name)))

;; Run with specific handler
(with-handler console-io-handler
  (greet-user))

;; Test the same program
(let ([test-handler (console-test-handler)])
  (test-handler 'set-input! '("Alice"))
  (with-handler test-handler
    (greet-user))
  ;; Check collected output
  (assert (equal? (test-handler 'get-output)
                  '("What's your name?" "Hello, Alice!"))))
```

### Stateful Computations

```scheme
;; Counter using State effect
(define (increment-counter)
  (do [current (get)]
      [_ (put (+ current 1))]
      (return current)))

(define (counter-program)
  (do [a (increment-counter)]
      [b (increment-counter)]  
      [c (increment-counter)]
      (return (list a b c))))

;; State handler
(define (state-handler initial-state)
  (let ([state initial-state])
    (handler (State Integer)
      [(get) state]
      [(put new-state) (set! state new-state)]
      [(modify f) (set! state (f state))]
      ;; Allow access to final state
      [final-state state])))

;; Usage
(let ([handler (state-handler 0)])
  (with-handler handler
    (let ([result (counter-program)])
      (display (format "Result: ~a" result))          ;; (0 1 2)  
      (display (format "Final state: ~a" (handler 'final-state)))))) ;; 3
```

## Advanced Effect Patterns

### Effect Composition

```scheme
;; Combine multiple effects
(define (logging-counter name)
  (do [_ (print (string-append "Starting counter: " name))]
      [initial (get)]
      [_ (print (format "Initial value: ~a" initial))]
      [result (increment-counter)]
      [_ (print (format "Incremented to: ~a" (+ initial 1)))]
      (return result)))

;; Use multiple handlers
(with-handler console-io-handler
  (with-handler (state-handler 10)
    (logging-counter "test-counter")))
```

### Custom Effect Definitions

```scheme
;; File system effect
(define-effect-type FileSystem
  (read-file : String -> String)
  (write-file : String String -> Unit)
  (file-exists? : String -> Boolean)
  (delete-file : String -> Unit))

;; Real file system handler
(define filesystem-handler
  (handler FileSystem
    [(read-file filename)
     (call-with-input-file filename
       (lambda (port) (read-string #f port)))]
    [(write-file filename content)
     (call-with-output-file filename
       (lambda (port) (display content port)))]
    [(file-exists? filename)
     (file-exists? filename)]
    [(delete-file filename)
     (delete-file filename)]))

;; In-memory file system for testing
(define (memory-filesystem-handler)
  (let ([files (make-hash-table)])
    (handler FileSystem
      [(read-file filename)
       (hash-table-ref files filename 
         (lambda () (error "File not found" filename)))]
      [(write-file filename content)
       (hash-table-set! files filename content)]
      [(file-exists? filename)
       (hash-table-exists? files filename)]
      [(delete-file filename)
       (hash-table-delete! files filename)]
      ;; Testing utilities
      [list-files (hash-table-keys files)]
      [clear! (hash-table-clear! files)])))
```

### Error Handling with Effects

```scheme
;; Exception effect
(define-effect-type (Exception e)
  (throw : e -> a)
  (catch : (Unit -> a) (e -> a) -> a))

;; Exception handler
(define (exception-handler)
  (handler (Exception String)
    [(throw error) (error error)]
    [(catch computation handler)
     (guard (condition
             [(string? condition) (handler condition)])
       (computation))]))

;; Safe file operations
(define (safe-read-file filename)
  (catch
    (lambda () (read-file filename))
    (lambda (error) 
      (print-error (format "Failed to read ~a: ~a" filename error))
      "")))

;; Usage with multiple effects
(define (process-config-file)
  (do [config-content (safe-read-file "config.json")]
      [_ (if (string-null? config-content)
             (print "Using default configuration")
             (print "Loaded custom configuration"))]
      [config (parse-json config-content)]
      (return config)))
```

## Monadic Programming

### Built-in Monads

```scheme
;; Maybe monad for nullable values
(define-monad Maybe
  (return : a -> (Maybe a))
  (bind : (Maybe a) (a -> (Maybe b)) -> (Maybe b)))

(define-type (Maybe a)
  (Nothing)
  (Just a))

(define-instance (Monad Maybe)
  (define (return x) (Just x))
  (define (bind maybe f)
    (match maybe
      [(Nothing) (Nothing)]
      [(Just x) (f x)])))

;; List monad for non-deterministic computation
(define-instance (Monad List)
  (define (return x) (list x))
  (define (bind lst f)
    (concatenate (map f lst))))

;; Usage examples
(define (safe-divide x y)
  (if (= y 0)
      (Nothing)
      (Just (/ x y))))

(define (computation)
  (do [x (Just 10)]
      [y (Just 2)] 
      [result (safe-divide x y)]
      (return (* result 2))))
;; Result: (Just 10)

;; Non-deterministic computation
(define (pythagorean-triples n)
  (do [x (range 1 n)]
      [y (range x n)]
      [z (range y n)]
      (if (= (+ (* x x) (* y y)) (* z z))
          (return (list x y z))
          (list))))  ;; Empty list = failure
```

### Monad Transformers

```scheme
;; StateT monad transformer
(define-monad-transformer StateT
  (lift-monad : (m a) -> (StateT s m a))
  (run-state : (StateT s m a) s -> (m (Pair a s))))

;; MaybeT transformer
(define-monad-transformer MaybeT
  (lift-monad : (m a) -> (MaybeT m a))
  (run-maybe : (MaybeT m a) -> (m (Maybe a))))

;; Combined transformers
(define-type (StateMaybeT s m a)
  (StateMaybeT (StateT s (MaybeT m) a)))

;; Example: stateful computation that might fail with logging
(define (stateful-division x y)
  (do [_ (lift-io (print (format "Dividing ~a by ~a" x y)))]
      [current-state (get)]
      [_ (put (+ current-state 1))]  ;; Count operations
      (if (= y 0)
          (lift-maybe (Nothing))     ;; Failure
          (return (/ x y)))))        ;; Success

;; Run the transformer stack
(define (run-computation)
  (let ([result (run-state 
                  (run-maybe
                    (run-io
                      (stateful-division 10 2)))
                  0)])
    result))
```

## Effect Coordination

### Concurrent Effects

```scheme
;; Parallel effect execution
(define (parallel-file-processing filenames)
  (parallel-map
    (lambda (filename)
      (do [content (read-file filename)]
          [processed (process-content content)]
          [output-name (string-append filename ".processed")]
          [_ (write-file output-name processed)]
          (return output-name)))
    filenames))

;; Effect synchronization
(define (coordinated-updates)
  (let ([barrier (make-barrier 3)])
    (parallel-eval
      (spawn (do [data1 (read-file "data1.txt")]
                 [_ (barrier-wait barrier)]
                 [_ (process-data data1)]
                 (return "Task 1 complete")))
      (spawn (do [data2 (read-file "data2.txt")]
                 [_ (barrier-wait barrier)]  
                 [_ (process-data data2)]
                 (return "Task 2 complete")))
      (spawn (do [config (read-file "config.json")]
                 [_ (barrier-wait barrier)]
                 [_ (apply-config config)]
                 (return "Task 3 complete"))))))
```

### Effect Isolation

```scheme
;; Sandboxed effect execution
(define (run-untrusted-code code)
  (with-effect-sandbox
    (sandbox-config
      (allow-effects [Console])           ;; Only allow console I/O
      (deny-effects [FileSystem Network]) ;; Block file and network access
      (resource-limits
        (memory-limit 10MB)
        (time-limit 30s)
        (cpu-limit 50%)))
    (eval-with-effects code)))

;; Resource-managed effects
(define (with-database-transaction f)
  (with-resource "database-connection"
    (lambda (conn)
      (do [_ (begin-transaction conn)]
          [result (guard (condition
                          [else 
                           (rollback-transaction conn)
                           (throw condition)])
                    (f conn))]
          [_ (commit-transaction conn)]
          (return result)))))
```

## Performance and Optimization

### Effect Compilation

```scheme
;; Effect fusion for performance
#:optimize-effects #t

(define (pipeline data)
  (do [step1 (map-effect transform1 data)]    ;; These effects can be fused
      [step2 (map-effect transform2 step1)]   ;; into a single operation
      [step3 (filter-effect predicate step2)]
      (return step3)))

;; Compiled to efficient single-pass operation
```

### Effect Specialization

```scheme
;; Specialized handlers for hot paths
(define fast-console-handler
  (handler Console
    [(print msg)
     ;; Specialized for performance-critical paths
     (fast-display msg)]    ;; Direct system call
    [(read-line)
     (fast-read-line)]))    ;; Optimized input

;; Effect inlining for pure computations
(define (pure-computation x)
  #:inline-effects #t
  (return (* x x)))        ;; Effect machinery eliminated
```

## Integration with Type System

### Effect Types

```scheme
;; Function types with effects
(define (read-config) : (FileSystem String)
  (read-file "config.json"))

(define (log-message msg : String) : (Console Unit)
  (print (string-append "[LOG] " msg)))

(define (interactive-setup) : (Console FileSystem (Config))
  (do [_ (log-message "Starting setup")]
      [name (do [_ (print "Enter name: ")]
                (read-line))]
      [config (make-config name)]
      [_ (write-file "user.config" (serialize config))]
      (return config)))
```

### Effect Inference

```scheme
;; Effects are inferred from usage
(define (process-user-data username)    ;; Inferred: FileSystem Console IO
  (do [user-file (string-append username ".json")]
      [_ (print (format "Processing ~a" username))]     ;; Console effect
      [data (read-file user-file)]                     ;; FileSystem effect  
      [processed (http-get processing-service data)]    ;; IO effect
      [_ (write-file (string-append username ".processed") processed)]
      (return processed)))
```

## Advanced Examples

### Complete Effect System Usage

```scheme
#!/usr/bin/env lambdust

(import (scheme base)
        (lambdust effects)
        (lambdust monads)
        (lambdust concurrency))

;; Define application-specific effects
(define-effect-type Database
  (query : String -> (List Record))
  (insert : Record -> Id)
  (update : Id Record -> Unit)
  (delete : Id -> Unit))

(define-effect-type Logging  
  (debug : String -> Unit)
  (info : String -> Unit)
  (warn : String -> Unit)
  (error : String -> Unit))

(define-effect-type WebServer
  (handle-request : Request -> Response)
  (start-server : Number -> Unit)
  (stop-server : Unit -> Unit))

;; Database handler with connection pooling
(define (database-handler connection-pool)
  (handler Database
    [(query sql)
     (with-connection connection-pool
       (lambda (conn) (execute-query conn sql)))]
    [(insert record)
     (with-connection connection-pool
       (lambda (conn) (execute-insert conn record)))]
    [(update id record)
     (with-connection connection-pool
       (lambda (conn) (execute-update conn id record)))]
    [(delete id)
     (with-connection connection-pool
       (lambda (conn) (execute-delete conn id)))]))

;; Structured logging handler
(define (logging-handler level)
  (handler Logging
    [(debug msg) (when (<= level 0) (log-output "DEBUG" msg))]
    [(info msg)  (when (<= level 1) (log-output "INFO" msg))]
    [(warn msg)  (when (<= level 2) (log-output "WARN" msg))]
    [(error msg) (when (<= level 3) (log-output "ERROR" msg))]))

;; Web application with effects
(define (user-service)
  (define (get-user id)
    (do [_ (debug (format "Fetching user ~a" id))]
        [users (query (format "SELECT * FROM users WHERE id = ~a" id))]
        (if (null? users)
            (do [_ (warn (format "User ~a not found" id))]
                (return (http-response 404 "User not found")))
            (do [_ (info (format "Found user ~a" id))]
                (return (http-response 200 (serialize (car users))))))))

  (define (create-user user-data)
    (do [_ (info "Creating new user")]
        [user (deserialize user-data)]
        [user-id (insert user)]
        [_ (info (format "Created user with ID ~a" user-id))]
        (return (http-response 201 (format "{\"id\": ~a}" user-id)))))

  ;; Route handler
  (lambda (request)
    (match (request-path request)
      [(string-append "/users/" id)
       (if (equal? (request-method request) "GET")
           (get-user (string->number id))
           (http-response 405 "Method not allowed"))]
      ["/users"
       (if (equal? (request-method request) "POST")
           (create-user (request-body request))
           (http-response 405 "Method not allowed"))]
      [_
       (http-response 404 "Not found")])))

;; Main application
(define (run-app)
  (let ([db-pool (create-connection-pool "postgresql://localhost/myapp")]
        [port 8080])
    (with-handler (database-handler db-pool)
      (with-handler (logging-handler 1) ;; Info level and above
        (with-handler web-server-handler
          (do [_ (info (format "Starting server on port ~a" port))]
              [service (user-service)]
              [_ (handle-request service)]  ;; Register handler
              [_ (start-server port)]
              [_ (info "Server started successfully")]
              ;; Keep server running
              (server-loop)))))))

;; Graceful shutdown
(define (shutdown-handler signal)
  (do [_ (info "Received shutdown signal")]
      [_ (stop-server)]
      [_ (info "Server stopped")]
      (exit 0)))

;; Application entry point
(when (script-file?)
  (install-signal-handler 'SIGTERM shutdown-handler)
  (install-signal-handler 'SIGINT shutdown-handler)
  (run-app))
```

### Effect System Testing

```scheme
;; Comprehensive effect testing framework
(define (test-user-service)
  ;; Mock database
  (let ([test-db (make-hash-table)]
        [next-id 1])
    (define mock-db-handler
      (handler Database
        [(query sql) (hash-table-values test-db)]
        [(insert record)
         (let ([id next-id])
           (set! next-id (+ next-id 1))
           (hash-table-set! test-db id record)
           id)]
        [(update id record) (hash-table-set! test-db id record)]
        [(delete id) (hash-table-delete! test-db id)]))
    
    ;; Mock logging (collect messages)
    (let ([log-messages '()])
      (define mock-logging-handler
        (handler Logging
          [(debug msg) (set! log-messages (cons (cons 'debug msg) log-messages))]
          [(info msg)  (set! log-messages (cons (cons 'info msg) log-messages))]
          [(warn msg)  (set! log-messages (cons (cons 'warn msg) log-messages))]
          [(error msg) (set! log-messages (cons (cons 'error msg) log-messages))]
          [get-messages (reverse log-messages)]
          [clear-messages! (set! log-messages '())]))
      
      ;; Run tests
      (with-handler mock-db-handler
        (with-handler mock-logging-handler
          (let ([service (user-service)])
            ;; Test user creation
            (let ([response (service (make-request "POST" "/users" "{\"name\": \"Alice\"}"))])
              (assert (= (response-status response) 201))
              (assert (member '(info . "Creating new user") 
                             (mock-logging-handler 'get-messages))))
            
            ;; Test user retrieval
            (let ([response (service (make-request "GET" "/users/1" ""))])
              (assert (= (response-status response) 200))
              (assert (string-contains? (response-body response) "Alice")))
            
            ;; Test user not found
            (let ([response (service (make-request "GET" "/users/999" ""))])
              (assert (= (response-status response) 404))
              (assert (member '(warn . "User 999 not found")
                             (mock-logging-handler 'get-messages))))))))))

;; Run tests
(test-user-service)
(display "All tests passed!")
```

This effect system provides a powerful foundation for building robust, testable, and maintainable applications while preserving the functional programming paradigm and enabling sophisticated program analysis and optimization.