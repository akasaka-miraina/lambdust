//! Tab Completion and Syntax Highlighting for REPL
//!
//! このモジュールはREPLのタブ補完とシンタックスハイライト機能を
//! 提供します。

use lambdust::interpreter::LambdustInterpreter;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{Context, Result as RustylineResult};
use rustyline::highlight::CmdKind;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;

/// Scheme completion helper for enhanced REPL experience
pub struct SchemeHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    builtin_functions: HashSet<String>,
    special_forms: HashSet<String>,
}

impl SchemeHelper {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut builtin_functions = HashSet::new();
        let mut special_forms = HashSet::new();

        // R7RS builtin functions
        for func in &[
            // Arithmetic
            "+",
            "-",
            "*",
            "/",
            "quotient",
            "remainder",
            "modulo",
            "abs",
            "floor",
            "ceiling",
            "sqrt",
            "expt",
            "min",
            "max",
            "exact?",
            "inexact?",
            "number?",
            "integer?",
            "real?",
            "rational?",
            "complex?",
            "exact->inexact",
            "inexact->exact",
            "number->string",
            "string->number",
            // Comparison
            "=",
            "<",
            ">",
            "<=",
            ">=",
            "eq?",
            "eqv?",
            "equal?",
            // List operations
            "car",
            "cdr",
            "cons",
            "list",
            "append",
            "reverse",
            "length",
            "null?",
            "pair?",
            "list?",
            "set-car!",
            "set-cdr!",
            "list->vector",
            "list->string",
            // String operations
            "string?",
            "string=?",
            "string<?",
            "string>?",
            "string<=?",
            "string>=?",
            "string-ci=?",
            "string-ci<?",
            "string-ci>?",
            "string-ci<=?",
            "string-ci>=?",
            "make-string",
            "string-length",
            "string-ref",
            "string-set!",
            "substring",
            "string-append",
            "string->list",
            "string-copy",
            "string-fill!",
            // Character operations
            "char?",
            "char=?",
            "char<?",
            "char>?",
            "char<=?",
            "char>=?",
            "char-ci=?",
            "char-ci<?",
            "char-ci>?",
            "char-ci<=?",
            "char-ci>=?",
            "char-alphabetic?",
            "char-numeric?",
            "char-whitespace?",
            "char-upper-case?",
            "char-lower-case?",
            "char-upcase",
            "char-downcase",
            "char->integer",
            "integer->char",
            // Vector operations
            "vector?",
            "make-vector",
            "vector",
            "vector-length",
            "vector-ref",
            "vector-set!",
            "vector->list",
            "list->vector",
            "vector-copy",
            "vector-fill!",
            // I/O
            "read",
            "write",
            "display",
            "newline",
            "read-char",
            "write-char",
            "peek-char",
            "eof-object?",
            "char-ready?",
            "load",
            // Higher-order functions
            "map",
            "for-each",
            "apply",
            "fold",
            "fold-right",
            "filter",
            // Control
            "call/cc",
            "call-with-current-continuation",
            "values",
            "call-with-values",
            "dynamic-wind",
            "raise",
            "with-exception-handler",
            "error",
            // Type predicates
            "boolean?",
            "symbol?",
            "procedure?",
            "port?",
            "input-port?",
            "output-port?",
            // Record types (SRFI 9)
            "make-record",
            "record-of-type?",
            "record-field",
            "record-set-field!",
            // SRFI functions
            "take",
            "drop",
            "concatenate",
            "delete-duplicates",
            "find",
            "any",
            "every",
            "string-null?",
            "string-hash",
            "string-hash-ci",
            "string-prefix?",
            "string-suffix?",
            "string-contains",
            "string-take",
            "string-drop",
            "string-concatenate",
            "make-hash-table",
            "hash-table?",
            "hash-table-set!",
            "hash-table-ref",
            "hash-table-delete!",
            "hash-table-size",
            "hash-table-exists?",
            "hash-table-keys",
            "hash-table-values",
            "hash-table->alist",
            "alist->hash-table",
            "hash",
            "string-hash",
        ] {
            builtin_functions.insert(func.to_string());
        }

        // Special forms
        for form in &[
            "define",
            "lambda",
            "if",
            "cond",
            "case",
            "and",
            "or",
            "when",
            "unless",
            "begin",
            "do",
            "let",
            "let*",
            "letrec",
            "letrec*",
            "set!",
            "quote",
            "quasiquote",
            "unquote",
            "unquote-splicing",
            "syntax-rules",
            "define-syntax",
            "guard",
            "define-record-type",
            "delay",
            "lazy",
            "force",
            "promise?",
        ] {
            special_forms.insert(form.to_string());
        }

        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            builtin_functions,
            special_forms,
        }
    }

    #[allow(dead_code)]
    pub fn update_builtin_functions(&mut self, interpreter: &LambdustInterpreter) {
        // Add host functions
        for func_name in interpreter.list_host_functions() {
            self.builtin_functions.insert(func_name.clone());
        }

        // Add scheme functions
        for func_name in interpreter.list_scheme_functions() {
            self.builtin_functions.insert(func_name.clone());
        }
    }
}

impl Completer for SchemeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        // Find the start of the current word
        let mut start = pos;
        while start > 0 {
            let ch = line.chars().nth(start - 1).unwrap_or(' ');
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            start -= 1;
        }

        let word = &line[start..pos];
        if word.is_empty() {
            return Ok((start, vec![]));
        }

        let mut candidates = Vec::new();

        // Complete builtin functions
        for func in &self.builtin_functions {
            if func.starts_with(word) {
                candidates.push(Pair {
                    display: func.clone(),
                    replacement: func.clone(),
                });
            }
        }

        // Complete special forms
        for form in &self.special_forms {
            if form.starts_with(word) {
                candidates.push(Pair {
                    display: format!("{} (special form)", form),
                    replacement: form.clone(),
                });
            }
        }

        // If no matches and word looks like a filename, try file completion
        if candidates.is_empty() && (word.contains('/') || word.contains('.')) {
            return self.completer.complete(line, pos, _ctx);
        }

        // Sort candidates alphabetically
        candidates.sort_by(|a, b| a.display.cmp(&b.display));

        Ok((start, candidates))
    }
}

impl Hinter for SchemeHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for SchemeHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Simple syntax highlighting for demonstration
        let mut result = String::new();
        let chars = line.chars();
        let mut in_string = false;
        let mut in_comment = false;

        for ch in chars {
            match ch {
                // String literals
                '"' if !in_comment => {
                    if !in_string {
                        result.push_str("\x1b[33m"); // Yellow for strings
                        in_string = true;
                    } else {
                        in_string = false;
                        result.push(ch);
                        result.push_str("\x1b[0m"); // Reset color
                        continue;
                    }
                }

                // Comments
                ';' if !in_string => {
                    result.push_str("\x1b[90m"); // Gray for comments
                    in_comment = true;
                }

                // Numbers (simplified detection)
                c if c.is_ascii_digit() && !in_string && !in_comment => {
                    result.push_str("\x1b[94m"); // Light blue for numbers
                    result.push(c);
                    result.push_str("\x1b[0m");
                    continue;
                }

                // Reset comment flag at end of line
                '\n' => {
                    if in_comment {
                        result.push(ch);
                        result.push_str("\x1b[0m");
                        in_comment = false;
                        continue;
                    }
                }

                _ => {}
            }

            result.push(ch);
        }

        // Reset color at end if still in string or comment
        if in_string || in_comment {
            result.push_str("\x1b[0m");
        }

        Owned(result)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Validator for SchemeHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> RustylineResult<rustyline::validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl rustyline::Helper for SchemeHelper {}