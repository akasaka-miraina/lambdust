# Macro Expansion Functions Design

## Overview

This document outlines the design for `macro-expand` and `macro-expand-1` built-in functions in Lambdust, inspired by Common Lisp's macroexpansion facilities but adapted for Scheme and enhanced with hygienic macro support.

## Common Lisp MACROEXPAND Inspiration

### Key Features from Common Lisp
1. **Two-level expansion**: `macroexpand-1` (single step) vs `macroexpand` (complete)
2. **Multiple return values**: expanded form + boolean flag indicating if expansion occurred  
3. **Environment awareness**: Respects local macro definitions
4. **Non-recursive expansion**: `macroexpand-1` expands only the outermost macro call

### Adaptations for Scheme/Lambdust
1. **Single return value**: Use pair `(expanded-form . expanded?)` instead of multiple values
2. **Hygienic macro support**: Show both original and hygienic symbol names
3. **Detailed expansion info**: Include expansion context and macro provenance
4. **REPL-friendly output**: Pretty-printed expansion with color coding (if supported)

## Function Specifications

### `macro-expand-1`

```scheme
(macro-expand-1 form [environment]) → (expanded-form . expanded?)
```

**Purpose**: Expand a macro form exactly one level.

**Parameters**:
- `form`: The form to potentially expand
- `environment` (optional): Environment for macro lookup (defaults to current)

**Return Value**: 
- A pair where:
  - `car` is the expanded form (or original if no expansion)
  - `cdr` is `#t` if expansion occurred, `#f` otherwise

**Behavior**:
- If `form` is a macro call, expand it once and return `(expanded . #t)`
- If `form` is not a macro call, return `(form . #f)`
- Respects both global and local macro definitions
- Preserves hygienic symbol information in expansion

### `macro-expand`

```scheme
(macro-expand form [environment]) → (expanded-form . expanded?)
```

**Purpose**: Completely expand a form until no more macro expansions are possible.

**Parameters**: Same as `macro-expand-1`

**Return Value**: Same as `macro-expand-1`

**Behavior**:
- Repeatedly applies `macro-expand-1` until no further expansion occurs
- Returns the fully expanded form
- Tracks total expansion steps for debugging

### `macro-expand-all` (Extension)

```scheme
(macro-expand-all form [environment]) → expanded-form
```

**Purpose**: Recursively expand all macro calls within a form, including nested subforms.

**Behavior**:
- Unlike `macro-expand`, this walks the entire form structure
- Expands macros in `let` bodies, `lambda` bodies, etc.
- Useful for complete macro debugging

## Implementation Design

### Core Expansion Engine

```rust
/// Result of macro expansion attempt
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    /// The expanded form (or original if no expansion)
    pub form: Expr,
    /// Whether expansion occurred
    pub expanded: bool,
    /// Information about the expansion (for debugging)
    pub expansion_info: Option<ExpansionInfo>,
}

/// Detailed information about macro expansion
#[derive(Debug, Clone)]
pub struct ExpansionInfo {
    /// Name of the macro that was expanded
    pub macro_name: String,
    /// Type of macro (syntax-rules, hygienic, etc.)
    pub macro_type: MacroType,
    /// Expansion depth (for nested expansions)
    pub depth: usize,
    /// Original symbols → hygienic symbols mapping
    pub symbol_mapping: HashMap<String, HygienicSymbol>,
    /// Time taken for expansion (for performance debugging)
    pub expansion_time: Duration,
}

/// Macro expansion engine
pub struct MacroExpander {
    /// Environment for macro lookup
    environment: Rc<HygienicEnvironment>,
    /// Maximum expansion depth (prevents infinite recursion)
    max_depth: usize,
    /// Whether to preserve expansion metadata
    preserve_metadata: bool,
}

impl MacroExpander {
    /// Expand macro exactly one level
    pub fn expand_once(&self, form: &Expr) -> Result<ExpansionResult> {
        // Implementation details...
    }
    
    /// Expand macro completely
    pub fn expand_completely(&self, form: &Expr) -> Result<ExpansionResult> {
        // Implementation details...
    }
    
    /// Expand all macros recursively
    pub fn expand_all(&self, form: &Expr) -> Result<Expr> {
        // Implementation details...
    }
}
```

### Built-in Function Integration

```rust
/// Register macro expansion functions as builtins
pub fn register_macro_expansion_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("macro-expand-1".to_string(), macro_expand_1_builtin());
    builtins.insert("macro-expand".to_string(), macro_expand_builtin());
    builtins.insert("macro-expand-all".to_string(), macro_expand_all_builtin());
}

fn macro_expand_1_builtin() -> Value {
    make_builtin_procedure("macro-expand-1", Some(1), |args| {
        check_arity(args, 1)?;
        let form = &args[0];
        
        // Convert Value to Expr for expansion
        let expr = value_to_expr(form)?;
        
        // Get current environment
        let env = get_current_hygienic_environment()?;
        
        // Create expander
        let expander = MacroExpander::new(env);
        
        // Expand once
        let result = expander.expand_once(&expr)?;
        
        // Convert back to Value and return as pair
        let expanded_value = expr_to_value(&result.form)?;
        let expanded_flag = Value::Boolean(result.expanded);
        
        Ok(Value::Pair(Rc::new(RefCell::new(PairData {
            car: expanded_value,
            cdr: expanded_flag,
        }))))
    })
}
```

## REPL Integration

### Enhanced Display for Debugging

```scheme
> (define-syntax when
    (syntax-rules ()
      [(when test expr ...)
       (if test (begin expr ...))]))

> (macro-expand-1 '(when #t (display "hello")))
((if#123 #t (begin#124 (display "hello"))) . #t)

> (macro-expand-1 '(+ 1 2))
((+ 1 2) . #f)

> (macro-expand '(when (= x 0) (when #t (display "nested"))))
((if#123 (= x 0) (begin#124 (if#125 #t (begin#126 (display "nested"))))) . #t)
```

### Pretty-Printed Output with Metadata

```scheme
> (macro-expand-1 '(when #t (display "hello")) 'verbose)
Expansion Result:
  Original:  (when #t (display "hello"))
  Expanded:  (if#123 #t (begin#124 (display "hello")))
  Macro:     when (hygienic syntax-rules)
  Symbols:   if → if#123, begin → begin#124
  Time:      0.15ms
  Expanded:  #t
```

## Error Handling

### Expansion Errors

```scheme
> (macro-expand-1 '(undefined-macro x y))
Error: Unknown macro 'undefined-macro'
  Available macros: when, unless, cond, case, ...

> (macro-expand '(recursive-macro x))
Error: Macro expansion exceeded maximum depth (256)
  Expansion chain: recursive-macro → recursive-macro → ...
  Possible infinite recursion in macro definition
```

### Malformed Macro Calls

```scheme
> (macro-expand-1 '(when))
Error: Macro 'when' requires at least 1 argument, got 0
  Usage: (when test expr ...)
  
> (macro-expand-1 '(syntax-error-macro "bad" "args"))
Error: Macro expansion failed: No matching pattern
  Available patterns:
    (syntax-error-macro number?)
    (syntax-error-macro string? string?)
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod macro_expand_tests {
    #[test]
    fn test_macro_expand_1_simple() {
        // Test basic macro expansion
    }
    
    #[test]
    fn test_macro_expand_1_no_expansion() {
        // Test when form is not a macro
    }
    
    #[test]
    fn test_macro_expand_complete() {
        // Test complete expansion
    }
    
    #[test]
    fn test_hygienic_symbol_preservation() {
        // Test that hygienic symbols are properly shown
    }
    
    #[test]
    fn test_nested_macro_expansion() {
        // Test expansion of nested macro calls
    }
    
    #[test]
    fn test_expansion_limits() {
        // Test infinite recursion prevention
    }
}
```

### Integration Tests

```scheme
;; Test basic functionality
(define-syntax simple-when
  (syntax-rules ()
    [(simple-when test expr)
     (if test expr)]))

(assert-equal 
  (macro-expand-1 '(simple-when #t "hello"))
  '((if#N #t "hello") . #t))

;; Test nested expansion
(assert-equal 
  (macro-expand '(simple-when #t (simple-when #f "nested")))
  '((if#N #t (if#M #f "nested")) . #t))
```

## Benefits

1. **Debugging Support**: Essential for understanding macro behavior in development
2. **Educational Value**: Helps users learn how macros work
3. **IDE Integration**: Enables sophisticated macro debugging tools
4. **REPL Enhancement**: Makes interactive development more productive
5. **Hygienic Transparency**: Shows how symbol collision prevention works
6. **Performance Analysis**: Expansion timing helps optimize macro-heavy code

This design provides powerful macro introspection capabilities while maintaining compatibility with Lambdust's hygienic macro system and performance goals.

## Usage Guide

### Available Functions

#### `macro-expand-1`
Expands a macro form exactly one level.

**Syntax:** `(macro-expand-1 form)`

**Returns:** A pair `(expanded-form . expanded?)` where:
- `expanded-form` is the result of expansion (or original if no expansion)
- `expanded?` is `#t` if expansion occurred, `#f` otherwise

#### `macro-expand`
Completely expands a form until no more macro expansions are possible.

**Syntax:** `(macro-expand form)`

**Returns:** Same format as `macro-expand-1`

#### `macro-expand-all`
Recursively expands all macro calls throughout a form structure.

**Syntax:** `(macro-expand-all form)`

**Returns:** The fully expanded form (no expansion flag)

### REPL Usage Examples

#### Basic Macro Expansion

```scheme
;; Define a simple macro
> (define-syntax when
    (syntax-rules ()
      [(when test expr ...)
       (if test (begin expr ...))]))

;; Expand once
> (macro-expand-1 '(when #t (display "hello")))
((if#1 #t (begin#2 (display "hello"))) . #t)

;; No expansion for non-macros
> (macro-expand-1 '(+ 1 2))
((+ 1 2) . #f)
```

#### Nested Macro Expansion

```scheme
;; Define unless macro using when
> (define-syntax unless
    (syntax-rules ()
      [(unless test expr ...)
       (when (not test) expr ...)]))

;; Single level expansion
> (macro-expand-1 '(unless #f (display "hello")))
((when#3 (not #f) (display "hello")) . #t)

;; Complete expansion
> (macro-expand '(unless #f (display "hello")))
((if#4 (not #f) (begin#5 (display "hello"))) . #t)
```

#### Complex Form Analysis

```scheme
;; Analyze complex expressions
> (macro-expand-all '(let ((x 1))
                       (when (> x 0)
                         (unless (= x 2)
                           (display x)))))

;; Shows all macro expansions throughout the form
```

### Hygienic Symbol Tracking

Lambdust's macro expansion shows hygienic symbol renaming:

```scheme
> (define-syntax swap
    (syntax-rules ()
      [(swap x y)
       (let ((temp x))
         (set! x y)
         (set! y temp))]))

> (macro-expand-1 '(swap a b))
((let#6 ((temp#7 a))
  (set! a b)
  (set! b temp#7)) . #t)
```

Notice how:
- `let` becomes `let#6` (hygienic identifier)
- `temp` becomes `temp#7` (prevents collision with user variables)
- User variables `a` and `b` remain unchanged

### Debugging Workflows

#### 1. Understanding Macro Behavior

```scheme
;; Step-by-step expansion
> (define-syntax complex-macro ...)

> (macro-expand-1 '(complex-macro args...))
;; See first level

> (macro-expand '(complex-macro args...))
;; See complete expansion

> (macro-expand-all '(some-form-using-complex-macro))
;; See expansion in context
```

#### 2. Macro Development Workflow

```scheme
;; 1. Define macro
> (define-syntax my-macro
    (syntax-rules () ...))

;; 2. Test expansion
> (macro-expand-1 '(my-macro test-args))

;; 3. Check for hygiene issues
> (let ((temp 42))
    (macro-expand-1 '(my-macro temp)))

;; 4. Verify complete behavior
> (macro-expand '(my-macro (nested-macro args)))
```

#### 3. Performance Analysis

```scheme
;; Check expansion overhead
> (time (macro-expand-all large-form))

;; Verify expansion depth
> (macro-expand deeply-nested-macro-form)
```

### IDE Integration

#### VS Code / Language Server

```json
{
  "lambdust.macroExpansion": {
    "showHygienicIds": true,
    "expandOnHover": true,
    "maxExpansionDepth": 10
  }
}
```

#### Emacs Integration

```elisp
(defun lambdust-expand-macro-at-point ()
  "Expand macro at cursor position"
  (interactive)
  (let ((form (sexp-at-point)))
    (lambdust-send-to-repl 
     (format "(macro-expand-1 '%s)" form))))
```

### Error Messages and Troubleshooting

#### Common Errors

```scheme
> (macro-expand-1 '(undefined-macro x))
Error: Unknown macro 'undefined-macro'
Available macros: when, unless, cond, case, ...

> (macro-expand '(recursive-macro))
Error: Macro expansion exceeded maximum depth (256)
Possible infinite recursion in macro definition
```

#### Malformed Calls

```scheme
> (macro-expand-1 '(when))
Error: Macro 'when' requires at least 1 argument, got 0
Usage: (when test expr ...)
```

### Advanced Features

#### Expansion Metadata

```scheme
;; Get detailed expansion information (future feature)
> (macro-expand-1 '(when #t body) 'verbose)
{
  :expanded-form (if#1 #t (begin#2 body))
  :expanded? #t
  :macro-name "when"
  :macro-type "syntax-rules"
  :expansion-time "0.15ms"
  :symbol-mapping {
    "if" "if#1"
    "begin" "begin#2"
  }
}
```

#### Environment-Specific Expansion

```scheme
;; Expand in specific lexical environment
> (let-syntax ((local-macro ...))
    (macro-expand-1 '(local-macro args) current-environment))
```

### Best Practices

#### 1. Macro Debugging

- Always test `macro-expand-1` before `macro-expand`
- Check hygienic symbol generation
- Verify expansion in different contexts
- Test with edge cases and empty argument lists

#### 2. Development Workflow

- Use `macro-expand-1` for step-by-step debugging
- Use `macro-expand-all` to understand full program behavior
- Check expansion performance for macro-heavy code
- Verify hygiene by testing with conflicting variable names

#### 3. Documentation

- Include expansion examples in macro documentation
- Show both single-step and complete expansions
- Document any non-hygienic behavior explicitly
- Provide troubleshooting examples

### Implementation Notes

- Built on Lambdust's hygienic macro system
- Integrates with existing environment system
- Preserves expansion metadata for debugging
- Prevents infinite recursion with depth limits
- Compatible with all macro types (syntax-rules, syntax-case)

### Future Enhancements

- Syntax highlighting for expanded forms
- Interactive expansion in IDE
- Macro expansion history
- Performance profiling for macro-heavy code
- Custom expansion display formats

This macro expansion system provides powerful tools for understanding, debugging, and developing sophisticated macro-based Scheme programs in Lambdust.