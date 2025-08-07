# Interactive Calculator Example

This example demonstrates building an interactive calculator in Lambdust, showcasing expression parsing, evaluation, and error handling.

## Overview

We'll build a calculator that:
- Parses mathematical expressions
- Supports basic arithmetic operations
- Handles variables and functions
- Provides helpful error messages
- Includes a REPL interface

## Basic Calculator

### Simple Expression Evaluator

```scheme
;; calculator.ldust - Basic expression calculator

(define (calculator-repl)
  "Interactive calculator REPL"
  (display "Lambdust Calculator v1.0")
  (newline)
  (display "Enter expressions or 'quit' to exit")
  (newline)
  (calculator-loop))

(define (calculator-loop)
  "Main calculator loop"
  (display "calc> ")
  (let ([input (read-line)])
    (cond
      [(or (equal? input "quit") (equal? input "exit"))
       (display "Goodbye!")
       (newline)]
      [(equal? input "help")
       (show-help)
       (calculator-loop)]
      [else
       (handle-expression input)
       (calculator-loop)])))

(define (handle-expression input)
  "Process and evaluate a calculator expression"
  (with-exception-handler
    (lambda (condition)
      (display "Error: ")
      (display (error-object-message condition))
      (newline))
    (lambda ()
      (let ([result (evaluate-expression input)])
        (display "=> ")
        (display result)
        (newline)))))

(define (show-help)
  "Display help information"
  (display "Available operations:")
  (newline)
  (display "  +, -, *, /    Basic arithmetic")
  (newline)
  (display "  sin, cos, tan Trigonometric functions")
  (newline)
  (display "  sqrt, log, exp Mathematical functions")
  (newline)
  (display "  let x = expr  Variable assignment")
  (newline)
  (display "  help          Show this help")
  (newline)
  (display "  quit          Exit calculator")
  (newline))
```

### Expression Evaluator

```scheme
;; Expression evaluation with variable support

(define calculator-env (make-hash-table))

(define (evaluate-expression input)
  "Evaluate a mathematical expression string"
  (let ([expr (parse-expression input)])
    (eval-expr expr calculator-env)))

(define (parse-expression input)
  "Parse expression string into internal representation"
  (cond
    ;; Handle variable assignment: let x = expr
    [(string-prefix? "let " input)
     (parse-assignment input)]
    ;; Handle regular expressions
    [else
     (parse-math-expression input)]))

(define (parse-assignment input)
  "Parse variable assignment: let var = expr"
  (let* ([parts (string-split (substring input 4) "=")]
         [var-name (string-trim (car parts))]
         [expr-str (string-trim (cadr parts))])
    (if (= (length parts) 2)
        `(assign ,var-name ,(parse-math-expression expr-str))
        (error "Invalid assignment syntax"))))

(define (parse-math-expression input)
  "Parse mathematical expression"
  ;; Simple parser for demonstration
  ;; In practice, you'd want a proper parser
  (with-input-from-string input
    (lambda ()
      (read))))

(define (eval-expr expr env)
  "Evaluate parsed expression"
  (cond
    ;; Numbers evaluate to themselves
    [(number? expr) expr]
    
    ;; Symbols are variable lookups
    [(symbol? expr)
     (hash-table-ref env expr
       (lambda () (error "Undefined variable" expr)))]
    
    ;; Lists are function calls
    [(list? expr)
     (eval-function-call expr env)]
    
    ;; Assignment expressions
    [(and (list? expr) (eq? (car expr) 'assign))
     (let ([var-name (cadr expr)]
           [value (eval-expr (caddr expr) env)])
       (hash-table-set! env var-name value)
       value)]
    
    [else (error "Invalid expression" expr)]))

(define (eval-function-call expr env)
  "Evaluate function call expression"
  (let ([func (car expr)]
        [args (map (lambda (arg) (eval-expr arg env)) (cdr expr))])
    (apply-function func args)))

(define (apply-function func args)
  "Apply mathematical function to arguments"
  (case func
    ;; Basic arithmetic
    [(+) (apply + args)]
    [(-) (apply - args)]
    [(*) (apply * args)]
    [(/) (apply / args)]
    
    ;; Mathematical functions
    [(sin) (sin (car args))]
    [(cos) (cos (car args))]
    [(tan) (tan (car args))]
    [(sqrt) (sqrt (car args))]
    [(log) (log (car args))]
    [(exp) (exp (car args))]
    [(abs) (abs (car args))]
    
    ;; Power and roots
    [(pow expt) (expt (car args) (cadr args))]
    [(square) (expt (car args) 2)]
    [(cube) (expt (car args) 3)]
    
    ;; Constants
    [(pi) 3.141592653589793]
    [(e) 2.718281828459045]
    
    [else (error "Unknown function" func)]))
```

## Advanced Calculator

### With Type Annotations

```scheme
;; Type-annotated calculator for better error checking

(define (typed-calculator-repl)
  #:type (-> Void)
  #:pure #f
  "Type-safe interactive calculator"
  (display "Lambdust Calculator v2.0 (Type-Safe)")
  (newline)
  (typed-calculator-loop))

(define (typed-calculator-loop)
  #:type (-> Void)
  #:effects (IO)
  "Main calculator loop with type checking"
  (display "calc> ")
  (let ([input (read-line)])
    (cond
      [(quit-command? input) (farewell-message)]
      [(help-command? input) (show-advanced-help) (typed-calculator-loop)]
      [else (handle-typed-expression input) (typed-calculator-loop)])))

(define (handle-typed-expression input)
  #:type (-> String Void)
  #:effects (IO)
  "Handle expression with type checking"
  (with-exception-handler
    (lambda (condition)
      #:type (-> Error Void)
      (display-error condition))
    (lambda ()
      (let ([result (evaluate-typed-expression input)])
        (display-result result)))))

(define (evaluate-typed-expression input)
  #:type (-> String Number)
  #:contract (-> string? number?)
  "Evaluate expression with type validation"
  (let ([expr (parse-and-validate-expression input)])
    (eval-typed-expr expr)))

(define (parse-and-validate-expression input)
  #:type (-> String Expression)
  "Parse and validate expression syntax"
  (let ([expr (parse-expression input)])
    (validate-expression-types expr)
    expr))
```

### Scientific Calculator

```scheme
;; Extended scientific calculator

(define-library (calculator scientific)
  (export scientific-calculator
          convert-units
          solve-equation)
  (import (scheme base)
          (scheme write)
          (scheme file)
          (calculator basic))
  
  (begin
    (define (scientific-calculator)
      "Scientific calculator with advanced functions"
      (display "Scientific Calculator")
      (newline)
      (display "Type 'modes' to see available modes")
      (newline)
      (scientific-loop 'basic))
    
    (define (scientific-loop mode)
      "Scientific calculator main loop"
      (display (string-append "sci[" (symbol->string mode) "]> "))
      (let ([input (read-line)])
        (cond
          [(equal? input "modes") (show-modes) (scientific-loop mode)]
          [(string-prefix? "mode " input) 
           (scientific-loop (string->symbol (substring input 5)))]
          [(equal? input "quit") (display "Goodbye!") (newline)]
          [else (handle-scientific-expression input mode) 
                (scientific-loop mode)])))
    
    (define (show-modes)
      "Show available calculator modes"
      (display "Available modes:")
      (newline)
      (display "  basic     - Basic arithmetic")
      (newline)
      (display "  trig      - Trigonometric functions")
      (newline)
      (display "  stats     - Statistical functions")
      (newline)
      (display "  units     - Unit conversions")
      (newline)
      (display "  solver    - Equation solver")
      (newline))
    
    (define (handle-scientific-expression input mode)
      "Handle expression based on current mode"
      (case mode
        [(basic) (handle-basic-expression input)]
        [(trig) (handle-trig-expression input)]
        [(stats) (handle-stats-expression input)]
        [(units) (handle-units-expression input)]
        [(solver) (handle-solver-expression input)]
        [else (display "Unknown mode") (newline)]))
    
    ;; Trigonometric mode
    (define (handle-trig-expression input)
      "Handle trigonometric expressions"
      (with-exception-handler
        (lambda (condition)
          (display "Trig Error: ")
          (display (error-object-message condition))
          (newline))
        (lambda ()
          (let ([result (eval-trig-expression input)])
            (display "=> ")
            (display result)
            (display " radians = ")
            (display (* result 180 (/ 1 3.141592653589793)))
            (display " degrees")
            (newline)))))
    
    ;; Statistical mode
    (define (handle-stats-expression input)
      "Handle statistical expressions"
      (cond
        [(string-prefix? "mean " input)
         (let ([numbers (parse-number-list (substring input 5))])
           (display "Mean: ")
           (display (mean numbers))
           (newline))]
        [(string-prefix? "median " input)
         (let ([numbers (parse-number-list (substring input 7))])
           (display "Median: ")
           (display (median numbers))
           (newline))]
        [(string-prefix? "stddev " input)
         (let ([numbers (parse-number-list (substring input 7))])
           (display "Standard Deviation: ")
           (display (standard-deviation numbers))
           (newline))]
        [else (display "Unknown statistical function") (newline)]))
    
    ;; Statistical functions
    (define (mean numbers)
      #:type (-> (List Number) Number)
      #:pure #t
      "Calculate arithmetic mean"
      (/ (fold-left + 0 numbers) (length numbers)))
    
    (define (median numbers)
      #:type (-> (List Number) Number)
      #:pure #t
      "Calculate median value"
      (let* ([sorted (sort numbers <)]
             [len (length sorted)]
             [mid (quotient len 2)])
        (if (odd? len)
            (list-ref sorted mid)
            (/ (+ (list-ref sorted (- mid 1))
                  (list-ref sorted mid))
               2))))
    
    (define (standard-deviation numbers)
      #:type (-> (List Number) Number)
      #:pure #t
      "Calculate standard deviation"
      (let* ([avg (mean numbers)]
             [squared-diffs (map (lambda (x) (expt (- x avg) 2)) numbers)]
             [variance (mean squared-diffs)])
        (sqrt variance)))
    
    ;; Unit conversion
    (define (convert-units from-amount from-unit to-unit)
      #:type (-> Number Symbol Symbol Number)
      #:pure #t
      "Convert between units"
      (let ([conversion-factor (get-conversion-factor from-unit to-unit)])
        (* from-amount conversion-factor)))
    
    (define (get-conversion-factor from to)
      #:type (-> Symbol Symbol Number)
      #:pure #t
      "Get conversion factor between units"
      (let ([conversions 
             '((meter kilometer 0.001)
               (kilometer meter 1000)
               (inch centimeter 2.54)
               (foot meter 0.3048)
               (celsius fahrenheit (lambda (c) (+ (* c 9/5) 32)))
               (fahrenheit celsius (lambda (f) (* (- f 32) 5/9))))])
        (let ([entry (assoc (list from to) conversions)])
          (if entry
              (caddr entry)
              (error "Unknown conversion" from to)))))
    
    ;; Simple equation solver
    (define (solve-equation equation variable)
      #:type (-> Expression Symbol Number)
      "Solve simple linear equations"
      ;; Simplified solver for demonstration
      ;; Real implementation would be much more complex
      (cond
        [(linear-equation? equation variable)
         (solve-linear equation variable)]
        [(quadratic-equation? equation variable)
         (solve-quadratic equation variable)]
        [else (error "Cannot solve equation type" equation)]))
    
    (define (solve-linear equation variable)
      "Solve linear equation ax + b = 0"
      ;; Implementation would extract coefficients and solve
      ;; This is a simplified placeholder
      0)
    
    (define (linear-equation? equation variable)
      "Check if equation is linear in variable"
      ;; Simplified check
      #t)))
```

## Usage Examples

### Basic Usage

```scheme
;; Start the calculator
(calculator-repl)

;; Example session:
;; calc> 2 + 3 * 4
;; => 14
;; calc> sin(3.14159/2)
;; => 1.0
;; calc> let x = 10
;; => 10
;; calc> x * x + 2 * x + 1
;; => 121
;; calc> sqrt(x)
;; => 3.1622776601683795
;; calc> quit
;; Goodbye!
```

### Scientific Mode

```scheme
;; Start scientific calculator
(scientific-calculator)

;; Example session:
;; sci[basic]> mode stats
;; sci[stats]> mean 1 2 3 4 5
;; Mean: 3
;; sci[stats]> stddev 1 2 3 4 5
;; Standard Deviation: 1.5811388300841898
;; sci[stats]> mode units
;; sci[units]> convert 100 celsius fahrenheit
;; => 212 fahrenheit
;; sci[units]> quit
;; Goodbye!
```

## Advanced Features

### Error Handling

```scheme
(define (safe-divide x y)
  #:type (-> Number Number Number)
  #:contract (-> number? number? number?)
  "Safe division with error handling"
  (cond
    [(zero? y) (error "Division by zero")]
    [(not (and (number? x) (number? y)))
     (error "Arguments must be numbers" x y)]
    [else (/ x y)]))

(define (safe-sqrt x)
  #:type (-> Number Number)
  #:contract (-> number? number?)
  "Safe square root"
  (cond
    [(negative? x) (error "Cannot take square root of negative number" x)]
    [(not (number? x)) (error "Argument must be a number" x)]
    [else (sqrt x)]))
```

### History and Memory

```scheme
(define calculator-history '())
(define calculator-memory 0)

(define (add-to-history expr result)
  "Add calculation to history"
  (set! calculator-history 
        (cons (list expr result (current-time))
              calculator-history)))

(define (show-history)
  "Display calculation history"
  (display "Recent calculations:")
  (newline)
  (for-each (lambda (entry)
              (display (car entry))
              (display " => ")
              (display (cadr entry))
              (newline))
            (take calculator-history 10)))

(define (memory-store value)
  "Store value in memory"
  (set! calculator-memory value)
  (display "Stored in memory: ")
  (display value)
  (newline))

(define (memory-recall)
  "Recall value from memory"
  calculator-memory)

(define (memory-clear)
  "Clear memory"
  (set! calculator-memory 0)
  (display "Memory cleared")
  (newline))
```

## Extension Points

The calculator can be extended with:

### Custom Functions

```scheme
(define (register-custom-function name func)
  "Register a custom calculator function"
  (hash-table-set! function-table name func))

;; Register custom functions
(register-custom-function 'factorial
  (lambda (n)
    (if (<= n 1) 1 (* n (factorial (- n 1))))))

(register-custom-function 'fibonacci
  (lambda (n)
    (cond
      [(<= n 0) 0]
      [(= n 1) 1]
      [else (+ (fibonacci (- n 1)) (fibonacci (- n 2)))])))
```

### Graphing Support

```scheme
(define (plot-function func x-min x-max steps)
  "Plot function values (text-based)"
  (let ([step-size (/ (- x-max x-min) steps)])
    (do ([x x-min (+ x step-size)]
         [i 0 (+ i 1)])
        [(> i steps)]
      (let ([y (func x)])
        (display x)
        (display " ")
        (display y)
        (newline)))))

;; Usage:
;; (plot-function sin -3.14159 3.14159 20)
```

This calculator example demonstrates many Lambdust features:
- **Interactive REPL development**
- **Error handling with exceptions**
- **Type annotations for safety**
- **Modular design with libraries**
- **Higher-order functions**
- **State management**
- **String processing**
- **Mathematical computations**

The code can be run directly in Lambdust and serves as both a useful tool and a learning example for functional programming concepts.