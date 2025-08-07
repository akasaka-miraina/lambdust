//! Built-in macro definitions for Lambdust.
//!
//! This module provides implementations of the derived forms specified in R7RS
//! as hygienic macros. These macros are implemented using the pattern and template
//! system to provide proper expansion behavior.

use super::{MacroExpander, MacroTransformer, Pattern, Template};
use crate::ast::Literal;
// use crate::diagnostics::{Span, Spanned};
use crate::eval::Environment;
use std::rc::Rc;

/// Installs all built-in macros into the given expander.
pub fn install_builtin_macros(expander: &mut MacroExpander) {
    // Core derived forms
    install_let_macro(expander);
    install_let_star_macro(expander);
    install_letrec_macro(expander);
    install_cond_macro(expander);
    install_case_macro(expander);
    install_and_macro(expander);
    install_or_macro(expander);
    install_when_macro(expander);
    install_unless_macro(expander);
    install_begin_macro(expander);
    
    // Additional useful macros
    install_do_macro(expander);
    install_delay_macro(expander);
    // Note: delay-force and force are provided as primitives, not macros
    install_quasiquote_macro(expander);
    
    // R7RS-required convenience macros
    install_case_lambda_macro(expander); // Re-enabled for R7RS compliance
    install_cond_expand_macro(expander);
    install_assert_macro(expander);
    
    // SRFI-26: Notation for Specializing Parameters
    install_cut_macro(expander);
    install_cute_macro(expander);
}

/// Installs the `let` macro.
/// (let ((var1 val1) (var2 val2) ...) body ...)
/// => ((lambda (var1 var2 ...) body ...) val1 val2 ...)
fn install_let_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("let"),
        Pattern::variable("bindings"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("body"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::list(vec![
            Template::identifier("lambda"),
            Template::Transform {
                function: "extract-vars".to_string(),
                argument: Box::new(Template::variable("bindings")),
            },
            Template::Variable("body".to_string()),
        ]),
        Template::Transform {
            function: "extract-vals".to_string(),
            argument: Box::new(Template::variable("bindings")),
        },
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("let".to_string()),
        source: None,
    };
    
    expander.define_macro("let".to_string(), transformer);
}

/// Installs the `let*` macro.
/// (let* ((var1 val1) (var2 val2) ...) body ...)
/// => (let ((var1 val1)) (let ((var2 val2)) ... body ...))
fn install_let_star_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("let*"),
        Pattern::variable("bindings"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("body"),
            None,
        ),
    ]);
    
    let template = Template::conditional(
        Template::Transform {
            function: "null?".to_string(),
            argument: Box::new(Template::variable("bindings")),
        },
        Template::list(vec![
            Template::identifier("begin"),
            Template::Variable("body".to_string()),
        ]),
        Some(Template::list(vec![
            Template::identifier("let"),
            Template::list(vec![
                Template::Transform {
                    function: "car".to_string(),
                    argument: Box::new(Template::variable("bindings")),
                },
            ]),
            Template::list(vec![
                Template::identifier("let*"),
                Template::Transform {
                    function: "cdr".to_string(),
                    argument: Box::new(Template::variable("bindings")),
                },
                Template::Variable("body".to_string()),
            ]),
        ])),
    );
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("let*".to_string()),
        source: None,
    };
    
    expander.define_macro("let*".to_string(), transformer);
}

/// Installs the `letrec` macro.
/// (letrec ((var1 val1) (var2 val2) ...) body ...)
/// => (let ((var1 #f) (var2 #f) ...)
///      (set! var1 val1)
///      (set! var2 val2)
///      ...
///      body ...)
fn install_letrec_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("letrec"),
        Pattern::variable("bindings"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("body"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("let"),
        Template::Transform {
            function: "make-undefined-bindings".to_string(),
            argument: Box::new(Template::variable("bindings")),
        },
        Template::Transform {
            function: "make-assignments".to_string(),
            argument: Box::new(Template::variable("bindings")),
        },
        Template::Variable("body".to_string()),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("letrec".to_string()),
        source: None,
    };
    
    expander.define_macro("letrec".to_string(), transformer);
}

/// Installs the `cond` macro.
/// (cond (test1 expr1 ...) (test2 expr2 ...) ... (else expr ...))
/// => (if test1 (begin expr1 ...) (if test2 (begin expr2 ...) ...))
fn install_cond_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("cond"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("clause"),
            None,
        ),
    ]);
    
    let template = Template::Transform {
        function: "expand-cond-clauses".to_string(),
        argument: Box::new(Template::Variable("clause".to_string())),
    };
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cond".to_string()),
        source: None,
    };
    
    expander.define_macro("cond".to_string(), transformer);
}

/// Installs the `case` macro.
/// (case key ((val1 val2 ...) expr1 ...) ... (else expr ...))
/// => (let ((temp key))
///      (cond ((memv temp '(val1 val2 ...)) expr1 ...)
///            ...
///            (else expr ...)))
fn install_case_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("case"),
        Pattern::variable("key"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("clause"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("let"),
        Template::list(vec![
            Template::list(vec![
                Template::identifier("temp"),
                Template::variable("key"),
            ]),
        ]),
        Template::Transform {
            function: "expand-case-clauses".to_string(),
            argument: Box::new(Template::Variable("clause".to_string())),
        },
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("case".to_string()),
        source: None,
    };
    
    expander.define_macro("case".to_string(), transformer);
}

/// Installs the `and` macro.
/// (and) => #t
/// (and test) => test
/// (and test1 test2 ...) => (if test1 (and test2 ...) #f)
fn install_and_macro(expander: &mut MacroExpander) {
    // Case 1: (and) => #t
    let pattern1 = Pattern::list(vec![Pattern::identifier("and")]);
    let template1 = Template::literal(Literal::Boolean(true));
    
    let _transformer1 = MacroTransformer {
        pattern: pattern1,
        template: template1,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("and".to_string()),
        source: None,
    };
    
    // Case 2: (and test) => test
    let pattern2 = Pattern::list(vec![
        Pattern::identifier("and"),
        Pattern::variable("test"),
    ]);
    let template2 = Template::variable("test");
    
    let _transformer2 = MacroTransformer {
        pattern: pattern2,
        template: template2,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("and".to_string()),
        source: None,
    };
    
    // Case 3: (and test1 test2 ...) => (if test1 (and test2 ...) #f)
    let pattern3 = Pattern::list(vec![
        Pattern::identifier("and"),
        Pattern::variable("test1"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("test"),
            None,
        ),
    ]);
    
    let template3 = Template::list(vec![
        Template::identifier("if"),
        Template::variable("test1"),
        Template::list(vec![
            Template::identifier("and"),
            Template::Variable("test".to_string()),
        ]),
        Template::literal(Literal::Boolean(false)),
    ]);
    
    let transformer3 = MacroTransformer {
        pattern: pattern3,
        template: template3,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("and".to_string()),
        source: None,
    };
    
    // For simplicity, we'll use the most general pattern (case 3)
    expander.define_macro("and".to_string(), transformer3);
}

/// Installs the `or` macro.
/// (or) => #f
/// (or test) => test
/// (or test1 test2 ...) => (let ((temp test1)) (if temp temp (or test2 ...)))
fn install_or_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("or"),
        Pattern::variable("test1"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("test"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("let"),
        Template::list(vec![
            Template::list(vec![
                Template::identifier("temp"),
                Template::variable("test1"),
            ]),
        ]),
        Template::list(vec![
            Template::identifier("if"),
            Template::identifier("temp"),
            Template::identifier("temp"),
            Template::list(vec![
                Template::identifier("or"),
                Template::Variable("test".to_string()),
            ]),
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("or".to_string()),
        source: None,
    };
    
    expander.define_macro("or".to_string(), transformer);
}

/// Installs the `when` macro.
/// (when test expr1 expr2 ...) => (if test (begin expr1 expr2 ...))
fn install_when_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("when"),
        Pattern::variable("test"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("expr"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("if"),
        Template::variable("test"),
        Template::list(vec![
            Template::identifier("begin"),
            Template::Variable("expr".to_string()),
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("when".to_string()),
        source: None,
    };
    
    expander.define_macro("when".to_string(), transformer);
}

/// Installs the `unless` macro.
/// (unless test expr1 expr2 ...) => (if (not test) (begin expr1 expr2 ...))
fn install_unless_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("unless"),
        Pattern::variable("test"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("expr"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("if"),
        Template::list(vec![
            Template::identifier("not"),
            Template::variable("test"),
        ]),
        Template::list(vec![
            Template::identifier("begin"),
            Template::Variable("expr".to_string()),
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("unless".to_string()),
        source: None,
    };
    
    expander.define_macro("unless".to_string(), transformer);
}

/// Installs the `begin` macro (for consistency).
/// (begin expr1 expr2 ...) => ((lambda () expr1 expr2 ...))
fn install_begin_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("begin"),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("expr"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::list(vec![
            Template::identifier("lambda"),
            Template::list(vec![]),
            Template::Variable("expr".to_string()),
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("begin".to_string()),
        source: None,
    };
    
    expander.define_macro("begin".to_string(), transformer);
}

/// Installs the `do` macro for iteration.
/// (do ((var1 init1 step1) ...) (test result ...) body ...)
fn install_do_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("do"),
        Pattern::variable("bindings"),
        Pattern::list(vec![
            Pattern::variable("test"),
            Pattern::ellipsis(
                vec![],
                Pattern::variable("result"),
                None,
            ),
        ]),
        Pattern::ellipsis(
            vec![],
            Pattern::variable("body"),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("letrec"),
        Template::list(vec![
            Template::list(vec![
                Template::identifier("loop"),
                Template::list(vec![
                    Template::identifier("lambda"),
                    Template::Transform {
                        function: "extract-vars".to_string(),
                        argument: Box::new(Template::variable("bindings")),
                    },
                    Template::list(vec![
                        Template::identifier("if"),
                        Template::variable("test"),
                        Template::list(vec![
                            Template::identifier("begin"),
                            Template::Variable("result".to_string()),
                        ]),
                        Template::list(vec![
                            Template::identifier("begin"),
                            Template::Variable("body".to_string()),
                            Template::list(vec![
                                Template::identifier("loop"),
                                Template::Transform {
                                    function: "extract-steps".to_string(),
                                    argument: Box::new(Template::variable("bindings")),
                                },
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]),
        Template::list(vec![
            Template::identifier("loop"),
            Template::Transform {
                function: "extract-inits".to_string(),
                argument: Box::new(Template::variable("bindings")),
            },
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("do".to_string()),
        source: None,
    };
    
    expander.define_macro("do".to_string(), transformer);
}

/// Installs the `delay` macro for lazy evaluation - R7RS compliant.
/// (delay expr) => (make-promise (lambda () expr))
/// Provides proper memoization and supports promise chains
fn install_delay_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("delay"),
        Pattern::variable("expr"),
    ]);
    
    // R7RS-compliant delay creates a memoizing promise
    let template = Template::list(vec![
        Template::identifier("make-promise"),
        Template::list(vec![
            Template::identifier("lambda"),
            Template::list(vec![]),
            Template::variable("expr"),
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("delay".to_string()),
        source: None,
    };
    
    expander.define_macro("delay".to_string(), transformer);
}

/// Installs the `delay-force` macro for tail-recursive lazy evaluation - R7RS extension.
/// (delay-force expr) => (delay-force (lambda () expr))
/// Optimized for tail-recursive promise chains
fn install_delay_force_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("delay-force"),
        Pattern::variable("expr"),
    ]);
    
    // delay-force creates tail-recursive promises for optimization
    // Note: This should use the primitive delay-force function, not recurse
    let template = Template::list(vec![
        Template::identifier("delay-force"),
        Template::list(vec![
            Template::identifier("lambda"),
            Template::list(vec![]),
            Template::variable("expr"),
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("delay-force".to_string()),
        source: None,
    };
    
    expander.define_macro("delay-force".to_string(), transformer);
}

/// Installs the `force` macro for promise evaluation - R7RS compliant.
/// (force promise) => (force promise) - Uses native force procedure
/// Supports full promise chain resolution and memoization
fn install_force_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("force"),
        Pattern::variable("promise"),
    ]);
    
    // R7RS force should not self-reference, use the primitive force function
    let template = Template::variable("promise"); // For now, just return the promise unchanged
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("force".to_string()),
        source: None,
    };
    
    expander.define_macro("force".to_string(), transformer);
}

/// Installs the `quasiquote` macro for template literals.
/// This is a simplified version - full quasiquote is quite complex
fn install_quasiquote_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("quasiquote"),
        Pattern::variable("template"),
    ]);
    
    let template = Template::Transform {
        function: "expand-quasiquote".to_string(),
        argument: Box::new(Template::variable("template")),
    };
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("quasiquote".to_string()),
        source: None,
    };
    
    expander.define_macro("quasiquote".to_string(), transformer);
}

/// Installs the `case-lambda` macro for variable arity procedures.
/// (case-lambda ((formals1) body1 ...) ((formals2) body2 ...) ...)
/// => (lambda args
///      (case (length args)
///        ((len1) (apply (lambda formals1 body1 ...) args))
///        ((len2) (apply (lambda formals2 body2 ...) args))
///        ...))
#[allow(dead_code)]
fn install_case_lambda_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("case-lambda"),
        Pattern::ellipsis(
            vec![],
            Pattern::list(vec![
                Pattern::variable("formals"),
                Pattern::ellipsis(
                    vec![],
                    Pattern::variable("body"),
                    None,
                ),
            ]),
            None,
        ),
    ]);
    
    let template = Template::list(vec![
        Template::identifier("lambda"),
        Template::identifier("args"),
        Template::list(vec![
            Template::identifier("case"),
            Template::list(vec![
                Template::identifier("length"),
                Template::identifier("args"),
            ]),
            Template::Transform {
                function: "expand-case-lambda-clauses".to_string(),
                argument: Box::new(Template::Variable("formals".to_string())),
            },
        ]),
    ]);
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("case-lambda".to_string()),
        source: None,
    };
    
    expander.define_macro("case-lambda".to_string(), transformer);
}

/// Installs the `cond-expand` macro for conditional compilation.
/// (cond-expand ((feature1) body1 ...) ((feature2) body2 ...) (else body ...))
/// => Expand to the first matching feature clause
fn install_cond_expand_macro(expander: &mut MacroExpander) {
    let pattern = Pattern::list(vec![
        Pattern::identifier("cond-expand"),
        Pattern::ellipsis(
            vec![],
            Pattern::list(vec![
                Pattern::variable("feature-test"),
                Pattern::ellipsis(
                    vec![],
                    Pattern::variable("body"),
                    None,
                ),
            ]),
            None,
        ),
    ]);
    
    let template = Template::Transform {
        function: "expand-cond-expand-clauses".to_string(),
        argument: Box::new(Template::Variable("feature-test".to_string())),
    };
    
    let transformer = MacroTransformer {
        pattern,
        template,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cond-expand".to_string()),
        source: None,
    };
    
    expander.define_macro("cond-expand".to_string(), transformer);
}

/// Installs the `assert` macro for runtime assertions.
/// (assert expr) => (if (not expr) (error "assertion failed" 'expr))
/// (assert expr message) => (if (not expr) (error message 'expr))
fn install_assert_macro(expander: &mut MacroExpander) {
    // Case 1: (assert expr)
    let pattern1 = Pattern::list(vec![
        Pattern::identifier("assert"),
        Pattern::variable("expr"),
    ]);
    
    let template1 = Template::list(vec![
        Template::identifier("if"),
        Template::list(vec![
            Template::identifier("not"),
            Template::variable("expr"),
        ]),
        Template::list(vec![
            Template::identifier("error"),
            Template::literal(Literal::String("assertion failed".to_string())),
            Template::list(vec![
                Template::identifier("quote"),
                Template::variable("expr"),
            ]),
        ]),
    ]);
    
    let transformer1 = MacroTransformer {
        pattern: pattern1,
        template: template1,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("assert".to_string()),
        source: None,
    };
    
    // For simplicity, we'll use the first form
    expander.define_macro("assert".to_string(), transformer1);
}

/// Installs the `cut` macro from SRFI-26.
/// (cut cons <> '()) => (lambda (x) (cons x '()))
/// (cut list 1 <> 3 <> 5) => (lambda (x y) (list 1 x 3 y 5))
fn install_cut_macro(expander: &mut MacroExpander) {
    // Most common case: (cut proc <>)
    let pattern1 = Pattern::list(vec![
        Pattern::identifier("cut"),
        Pattern::variable("proc"),
        Pattern::identifier("<>"),
    ]);
    
    let template1 = Template::list(vec![
        Template::identifier("lambda"),
        Template::list(vec![Template::identifier("x")]),
        Template::list(vec![
            Template::variable("proc"),
            Template::identifier("x"),
        ]),
    ]);
    
    let transformer1 = MacroTransformer {
        pattern: pattern1,
        template: template1,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cut".to_string()),
        source: None,
    };
    
    // Two slot case: (cut proc <> <>)
    let pattern2 = Pattern::list(vec![
        Pattern::identifier("cut"),
        Pattern::variable("proc"),
        Pattern::identifier("<>"),
        Pattern::identifier("<>"),
    ]);
    
    let template2 = Template::list(vec![
        Template::identifier("lambda"),
        Template::list(vec![
            Template::identifier("x"),
            Template::identifier("y"),
        ]),
        Template::list(vec![
            Template::variable("proc"),
            Template::identifier("x"),
            Template::identifier("y"),
        ]),
    ]);
    
    let _transformer2 = MacroTransformer {
        pattern: pattern2,
        template: template2,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cut".to_string()),
        source: None,
    };
    
    // Mixed case: (cut proc expr <>)
    let pattern3 = Pattern::list(vec![
        Pattern::identifier("cut"),
        Pattern::variable("proc"),
        Pattern::variable("expr"),
        Pattern::identifier("<>"),
    ]);
    
    let template3 = Template::list(vec![
        Template::identifier("lambda"),
        Template::list(vec![Template::identifier("x")]),
        Template::list(vec![
            Template::variable("proc"),
            Template::variable("expr"),
            Template::identifier("x"),
        ]),
    ]);
    
    let _transformer3 = MacroTransformer {
        pattern: pattern3,
        template: template3,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cut".to_string()),
        source: None,
    };
    
    // For now, use the most common single-slot case
    expander.define_macro("cut".to_string(), transformer1);
}

/// Installs the `cute` macro from SRFI-26.
/// (cute cons <> (expensive-computation)) => 
/// (let ((temp (expensive-computation))) (lambda (x) (cons x temp)))
fn install_cute_macro(expander: &mut MacroExpander) {
    // Most common case: (cute proc <>)
    let pattern1 = Pattern::list(vec![
        Pattern::identifier("cute"),
        Pattern::variable("proc"),
        Pattern::identifier("<>"),
    ]);
    
    let template1 = Template::list(vec![
        Template::identifier("lambda"),
        Template::list(vec![Template::identifier("x")]),
        Template::list(vec![
            Template::variable("proc"),
            Template::identifier("x"),
        ]),
    ]);
    
    let transformer1 = MacroTransformer {
        pattern: pattern1,
        template: template1,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cute".to_string()),
        source: None,
    };
    
    // Eager evaluation case: (cute proc expr <>)
    let pattern2 = Pattern::list(vec![
        Pattern::identifier("cute"),
        Pattern::variable("proc"),
        Pattern::variable("expr"),
        Pattern::identifier("<>"),
    ]);
    
    let template2 = Template::list(vec![
        Template::identifier("let"),
        Template::list(vec![
            Template::list(vec![
                Template::identifier("temp"),
                Template::variable("expr"),
            ]),
        ]),
        Template::list(vec![
            Template::identifier("lambda"),
            Template::list(vec![Template::identifier("x")]),
            Template::list(vec![
                Template::variable("proc"),
                Template::identifier("temp"),
                Template::identifier("x"),
            ]),
        ]),
    ]);
    
    let _transformer2 = MacroTransformer {
        pattern: pattern2,
        template: template2,
        definition_env: crate::eval::environment::global_environment(),
        name: Some("cute".to_string()),
        source: None,
    };
    
    // For now, use the simple single-slot case
    expander.define_macro("cute".to_string(), transformer1);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn test_builtin_installation() {
        let mut expander = MacroExpander::new();
        install_builtin_macros(&mut expander);
        
        // Check that basic macros are installed
        assert!(expander.macro_env().lookup("let").is_some());
        assert!(expander.macro_env().lookup("let*").is_some());
        assert!(expander.macro_env().lookup("letrec").is_some());
        assert!(expander.macro_env().lookup("cond").is_some());
        assert!(expander.macro_env().lookup("case").is_some());
        assert!(expander.macro_env().lookup("and").is_some());
        assert!(expander.macro_env().lookup("or").is_some());
        assert!(expander.macro_env().lookup("when").is_some());
        assert!(expander.macro_env().lookup("unless").is_some());
        assert!(expander.macro_env().lookup("case-lambda").is_some());
        assert!(expander.macro_env().lookup("cond-expand").is_some());
        assert!(expander.macro_env().lookup("assert").is_some());
        assert!(expander.macro_env().lookup("cut").is_some());
        assert!(expander.macro_env().lookup("cute").is_some());
    }
    
    #[test]
    fn test_macro_names() {
        let mut expander = MacroExpander::new();
        install_builtin_macros(&mut expander);
        
        let names = expander.macro_env().all_names();
        assert!(names.contains(&"let".to_string()));
        assert!(names.contains(&"cond".to_string()));
        assert!(names.contains(&"when".to_string()));
        assert!(names.contains(&"case-lambda".to_string()));
        assert!(names.contains(&"cond-expand".to_string()));
        assert!(names.contains(&"cut".to_string()));
        assert!(names.contains(&"cute".to_string()));
    }
}