# Getting Started with Lambdust

Welcome to Lambdust! This guide will help you install, configure, and start using Lambdust for your functional programming projects.

## Installation

### Prerequisites

- **Rust**: Lambdust is written in Rust. Install Rust from [rustup.rs](https://rustup.rs/)
- **Git**: For cloning the repository

### Building from Source

Currently, Lambdust is available by building from source:

```bash
# Clone the repository
git clone https://github.com/username/lambdust.git
cd lambdust

# Build the project
cargo build --release

# Optional: Install globally
cargo install --path .
```

### Verify Installation

Test your installation:

```bash
# Check version
./target/release/lambdust --version

# Run a simple expression
./target/release/lambdust --eval "(+ 1 2 3)"
```

## First Steps

### Interactive REPL

The easiest way to start exploring Lambdust is through the interactive REPL (Read-Eval-Print Loop):

```bash
# Start the basic REPL
cargo run --features repl

# Or start the enhanced REPL with syntax highlighting and completion
cargo run --features enhanced-repl
```

You'll see a prompt like this:

```
Lambdust v0.1.0 - A Scheme dialect with gradual typing
Type (exit) to quit

λ> 
```

Try some basic expressions:

```scheme
λ> (+ 1 2 3)
6

λ> (* 5 5)
25

λ> (define greeting "Hello, Lambdust!")
λ> greeting
"Hello, Lambdust!"

λ> (define (square x) (* x x))
λ> (square 7)
49
```

### Running Files

Create a file called `hello.ldust`:

```scheme
;; hello.ldust
(display "Hello, World!")
(newline)

(define (factorial n)
  #:type (-> Number Number)
  #:pure #t
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(display "Factorial of 5 is: ")
(display (factorial 5))
(newline)
```

Run it:

```bash
cargo run hello.ldust
```

### Command-Line Options

Lambdust provides several command-line options:

```bash
# Run a file
lambdust program.ldust

# Evaluate an expression
lambdust --eval "(define x 10) (+ x 5)"

# Start REPL
lambdust --repl

# Show help
lambdust --help

# Enable verbose output
lambdust --verbose program.ldust
```

## Language Basics

### Core Syntax

Lambdust uses S-expression syntax like other Lisps:

```scheme
;; Comments start with semicolon
(function-name argument1 argument2 ...)

;; Nested expressions
(+ (* 2 3) (/ 8 4))  ; => 8

;; Lists
'(1 2 3 4 5)         ; quoted list
(list 1 2 3 4 5)     ; constructed list
```

### Variables and Functions

```scheme
;; Variable definition
(define pi 3.14159)
(define greeting "Hello")

;; Function definition
(define (add x y)
  (+ x y))

;; Anonymous functions (lambda)
(lambda (x) (* x x))
((lambda (x) (* x x)) 5)  ; => 25
```

### Conditional Expressions

```scheme
;; if expression
(define (abs x)
  (if (< x 0)
      (- x)
      x))

;; cond expression (multiple conditions)
(define (grade score)
  (cond
    [(>= score 90) "A"]
    [(>= score 80) "B"]
    [(>= score 70) "C"]
    [(>= score 60) "D"]
    [else "F"]))
```

### Lists and Higher-Order Functions

```scheme
;; List operations
(define numbers '(1 2 3 4 5))
(car numbers)         ; => 1
(cdr numbers)         ; => (2 3 4 5)
(cons 0 numbers)      ; => (0 1 2 3 4 5)

;; Higher-order functions
(map (lambda (x) (* x x)) numbers)      ; => (1 4 9 16 25)
(filter odd? numbers)                   ; => (1 3 5)
(fold-left + 0 numbers)                 ; => 15
```

## Lambdust-Specific Features

### Type Annotations

Lambdust supports gradual typing with optional type annotations:

```scheme
;; Dynamic typing (default)
(define (multiply x y) (* x y))

;; Static typing with annotations
(define (typed-multiply x y)
  #:type (-> Number Number Number)
  (* x y))

;; Pure function annotation
(define (square x)
  #:type (-> Number Number)
  #:pure #t
  (* x x))
```

### Effect System

Side effects are automatically tracked:

```scheme
;; Pure function - no effects
(define (add x y)
  #:pure #t
  (+ x y))

;; Function with I/O effects
(define (greet name)
  (display "Hello, ")
  (display name)
  (newline))

;; Function with state effects
(define counter
  (let ([count 0])
    (lambda ()
      (set! count (+ count 1))
      count)))
```

### Module System

Organize code with modules:

```scheme
;; Import R7RS standard modules
(import (scheme base)
        (scheme write)
        (scheme file))

;; Define and export functions
(define-library (my-utils)
  (export square cube)
  (import (scheme base))
  
  (begin
    (define (square x) (* x x))
    (define (cube x) (* x x x))))

;; Use the module
(import (my-utils))
(square 4)  ; => 16
```

## Development Environment

### Editor Support

While Lambdust-specific editor plugins are in development, you can use existing Scheme/Lisp modes:

- **Emacs**: Use `scheme-mode` or `geiser-mode`
- **VS Code**: Install "Scheme" or "Lisp" extensions
- **Vim**: Use `vim-scheme` or similar plugins

### REPL Features

The enhanced REPL provides:

- **Syntax highlighting**: Keywords and types are highlighted
- **Auto-completion**: Tab completion for functions and variables
- **History**: Access previous commands with up/down arrows
- **Multi-line editing**: Edit complex expressions across multiple lines

### Debugging

Lambdust provides helpful error messages:

```scheme
λ> (+ 1 "hello")
Error: +: contract violation
  expected: number
  given: "hello"
  at: <repl>:1:3
```

## Example Programs

### Calculator

```scheme
(define (calculate op x y)
  (cond
    [(equal? op '+) (+ x y)]
    [(equal? op '-) (- x y)]
    [(equal? op '*) (* x y)]
    [(equal? op '/) (/ x y)]
    [else (error "Unknown operation" op)]))

(calculate '+ 10 5)   ; => 15
(calculate '* 3 4)    ; => 12
```

### List Processing

```scheme
(define (sum-of-squares lst)
  #:type (-> (List Number) Number)
  #:pure #t
  (fold-left + 0 (map (lambda (x) (* x x)) lst)))

(sum-of-squares '(1 2 3 4))  ; => 30
```

### File Processing

```scheme
(define (count-lines filename)
  (call-with-input-file filename
    (lambda (port)
      (let loop ([count 0])
        (if (eof-object? (read-line port))
            count
            (loop (+ count 1)))))))
```

## Next Steps

Now that you have Lambdust up and running:

1. **Explore the Language**: Work through the [Basic Programming Tutorial](../tutorials/basic-programming.md)
2. **Learn Type System**: Read about [Gradual Typing](type-system.md)
3. **Understand Effects**: Learn about [Effect Management](effect-system.md)
4. **Check Examples**: Browse [code examples](../examples/) for inspiration
5. **Read API Reference**: Consult the [Standard Library documentation](../api/stdlib/)

## Getting Help

- **Documentation**: Browse this documentation for detailed information
- **Issues**: Report problems on [GitHub Issues](https://github.com/username/lambdust/issues)
- **Discussions**: Ask questions on [GitHub Discussions](https://github.com/username/lambdust/discussions)
- **Examples**: Check the [examples directory](../examples/) for more code samples

Welcome to the Lambdust community! Happy functional programming! 🎉