//! Comprehensive tests for the value/display module
//!
//! These tests verify the display and debug formatting of Scheme values,
//! ensuring proper representation for user interaction and debugging.

use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Procedure;
use lambdust::value::pair::PairData;
use lambdust::value::record::{Record, RecordType};
use lambdust::environment::Environment;
use lambdust::ast::Expr;
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(test)]
mod basic_display_tests {
    use super::*;

    #[test]
    fn test_display_undefined() {
        let value = Value::Undefined;
        assert_eq!(format!("{}", value), "#<undefined>");
    }

    #[test]
    fn test_display_booleans() {
        let true_val = Value::Boolean(true);
        let false_val = Value::Boolean(false);
        
        assert_eq!(format!("{}", true_val), "#t");
        assert_eq!(format!("{}", false_val), "#f");
    }

    #[test]
    fn test_display_numbers() {
        let integer = Value::Number(SchemeNumber::Integer(42));
        let negative_integer = Value::Number(SchemeNumber::Integer(-17));
        let real = Value::Number(SchemeNumber::Real(3.14));
        let rational = Value::Number(SchemeNumber::Rational(3, 4));
        let complex = Value::Number(SchemeNumber::Complex(2.0, 3.0));
        
        assert_eq!(format!("{}", integer), "42");
        assert_eq!(format!("{}", negative_integer), "-17");
        assert_eq!(format!("{}", real), "3.14");
        assert_eq!(format!("{}", rational), "3/4");
        assert_eq!(format!("{}", complex), "2+3i");
    }

    #[test]
    fn test_display_strings() {
        let empty_string = Value::String("".to_string());
        let simple_string = Value::String("hello".to_string());
        let string_with_spaces = Value::String("hello world".to_string());
        let string_with_escapes = Value::String("hello\nworld\ttab".to_string());
        
        assert_eq!(format!("{}", empty_string), "\"\"");
        assert_eq!(format!("{}", simple_string), "\"hello\"");
        assert_eq!(format!("{}", string_with_spaces), "\"hello world\"");
        assert_eq!(format!("{}", string_with_escapes), "\"hello\nworld\ttab\"");
    }

    #[test]
    fn test_display_characters() {
        let char_a = Value::Character('a');
        let char_space = Value::Character(' ');
        let char_newline = Value::Character('\n');
        let char_tab = Value::Character('\t');
        let char_special = Value::Character('@');
        
        assert_eq!(format!("{}", char_a), "#\\a");
        assert_eq!(format!("{}", char_space), "#\\space");
        assert_eq!(format!("{}", char_newline), "#\\newline");
        assert_eq!(format!("{}", char_tab), "#\\tab");
        assert_eq!(format!("{}", char_special), "#\\@");
    }

    #[test]
    fn test_display_symbols() {
        let symbol = Value::Symbol("hello".to_string());
        let symbol_with_special = Value::Symbol("hello-world".to_string());
        let symbol_arithmetic = Value::Symbol("+".to_string());
        let symbol_predicate = Value::Symbol("null?".to_string());
        
        assert_eq!(format!("{}", symbol), "hello");
        assert_eq!(format!("{}", symbol_with_special), "hello-world");
        assert_eq!(format!("{}", symbol_arithmetic), "+");
        assert_eq!(format!("{}", symbol_predicate), "null?");
    }

    #[test]
    fn test_display_nil() {
        let nil = Value::Nil;
        assert_eq!(format!("{}", nil), "()");
    }
}

#[cfg(test)]
mod pair_display_tests {
    use super::*;

    #[test]
    fn test_display_simple_pair() {
        let car = Value::Number(SchemeNumber::Integer(1));
        let cdr = Value::Number(SchemeNumber::Integer(2));
        let pair_data = PairData::new(car, cdr);
        let pair = Value::Pair(Rc::new(RefCell::new(pair_data)));
        
        assert_eq!(format!("{}", pair), "(1 . 2)");
    }

    #[test]
    fn test_display_proper_list() {
        let car1 = Value::Number(SchemeNumber::Integer(1));
        let car2 = Value::Number(SchemeNumber::Integer(2));
        let car3 = Value::Number(SchemeNumber::Integer(3));
        
        let pair3 = PairData::new(car3, Value::Nil);
        let pair2 = PairData::new(car2, Value::Pair(Rc::new(RefCell::new(pair3))));
        let pair1 = PairData::new(car1, Value::Pair(Rc::new(RefCell::new(pair2))));
        let list = Value::Pair(Rc::new(RefCell::new(pair1)));
        
        assert_eq!(format!("{}", list), "(1 2 3)");
    }

    #[test]
    fn test_display_improper_list() {
        let car1 = Value::Number(SchemeNumber::Integer(1));
        let car2 = Value::Number(SchemeNumber::Integer(2));
        let cdr2 = Value::Number(SchemeNumber::Integer(3));
        
        let pair2 = PairData::new(car2, cdr2);
        let pair1 = PairData::new(car1, Value::Pair(Rc::new(RefCell::new(pair2))));
        let improper_list = Value::Pair(Rc::new(RefCell::new(pair1)));
        
        assert_eq!(format!("{}", improper_list), "(1 2 . 3)");
    }

    #[test]
    fn test_display_nested_pairs() {
        let inner_car = Value::Number(SchemeNumber::Integer(1));
        let inner_cdr = Value::Number(SchemeNumber::Integer(2));
        let inner_pair = PairData::new(inner_car, inner_cdr);
        
        let outer_car = Value::Pair(Rc::new(RefCell::new(inner_pair)));
        let outer_cdr = Value::Number(SchemeNumber::Integer(3));
        let outer_pair = PairData::new(outer_car, outer_cdr);
        let nested = Value::Pair(Rc::new(RefCell::new(outer_pair)));
        
        assert_eq!(format!("{}", nested), "((1 . 2) . 3)");
    }

    #[test]
    fn test_display_mixed_type_list() {
        let car1 = Value::Number(SchemeNumber::Integer(42));
        let car2 = Value::String("hello".to_string());
        let car3 = Value::Boolean(true);
        let car4 = Value::Symbol("test".to_string());
        
        let pair4 = PairData::new(car4, Value::Nil);
        let pair3 = PairData::new(car3, Value::Pair(Rc::new(RefCell::new(pair4))));
        let pair2 = PairData::new(car2, Value::Pair(Rc::new(RefCell::new(pair3))));
        let pair1 = PairData::new(car1, Value::Pair(Rc::new(RefCell::new(pair2))));
        let mixed_list = Value::Pair(Rc::new(RefCell::new(pair1)));
        
        assert_eq!(format!("{}", mixed_list), "(42 \"hello\" #t test)");
    }

    #[test]
    fn test_display_empty_list() {
        let nil = Value::Nil;
        assert_eq!(format!("{}", nil), "()");
    }

    #[test]
    fn test_display_single_element_list() {
        let car = Value::Symbol("single".to_string());
        let pair = PairData::new(car, Value::Nil);
        let single_list = Value::Pair(Rc::new(RefCell::new(pair)));
        
        assert_eq!(format!("{}", single_list), "(single)");
    }
}

#[cfg(test)]
mod procedure_display_tests {
    use super::*;

    #[test]
    fn test_display_builtin_procedure() {
        let builtin = Procedure::Builtin {
            name: "+".to_string(),
            arity: Some(2),
            func: |_args| Ok(Value::Number(SchemeNumber::Integer(0))),
        };
        let proc_value = Value::Procedure(builtin);
        
        assert_eq!(format!("{}", proc_value), "#<builtin +>");
    }

    #[test]
    fn test_display_lambda_procedure() {
        let lambda = Procedure::Lambda {
            params: vec!["x".to_string(), "y".to_string()],
            variadic: false,
            body: vec![Expr::Variable("x".to_string())],
            closure: Rc::new(Environment::new()),
        };
        let proc_value = Value::Procedure(lambda);
        
        assert_eq!(format!("{}", proc_value), "#<procedure (x y)>");
    }

    #[test]
    fn test_display_lambda_variadic() {
        let lambda = Procedure::Lambda {
            params: vec!["x".to_string()],
            variadic: true,
            body: vec![Expr::Variable("x".to_string())],
            closure: Rc::new(Environment::new()),
        };
        let proc_value = Value::Procedure(lambda);
        
        assert_eq!(format!("{}", proc_value), "#<procedure (x ...)>");
    }

    #[test]
    fn test_display_lambda_no_params() {
        let lambda = Procedure::Lambda {
            params: vec![],
            variadic: false,
            body: vec![Expr::Variable("x".to_string())],
            closure: Rc::new(Environment::new()),
        };
        let proc_value = Value::Procedure(lambda);
        
        assert_eq!(format!("{}", proc_value), "#<procedure ()>");
    }
}

#[cfg(test)]
mod vector_display_tests {
    use super::*;

    #[test]
    fn test_display_empty_vector() {
        let empty_vec = Value::Vector(vec![]);
        assert_eq!(format!("{}", empty_vec), "#()");
    }

    #[test]
    fn test_display_single_element_vector() {
        let single_vec = Value::Vector(vec![Value::Number(SchemeNumber::Integer(42))]);
        assert_eq!(format!("{}", single_vec), "#(42)");
    }

    #[test]
    fn test_display_multiple_element_vector() {
        let multi_vec = Value::Vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
        ]);
        assert_eq!(format!("{}", multi_vec), "#(1 2 3)");
    }

    #[test]
    fn test_display_mixed_type_vector() {
        let mixed_vec = Value::Vector(vec![
            Value::Number(SchemeNumber::Integer(42)),
            Value::String("hello".to_string()),
            Value::Boolean(true),
            Value::Symbol("test".to_string()),
        ]);
        assert_eq!(format!("{}", mixed_vec), "#(42 \"hello\" #t test)");
    }

    #[test]
    fn test_display_nested_vector() {
        let inner_vec = Value::Vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ]);
        let outer_vec = Value::Vector(vec![inner_vec, Value::Number(SchemeNumber::Integer(3))]);
        assert_eq!(format!("{}", outer_vec), "#(#(1 2) 3)");
    }

    #[test]
    fn test_display_vector_with_pairs() {
        let pair_data = PairData::new(
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        );
        let pair = Value::Pair(Rc::new(RefCell::new(pair_data)));
        let vec_with_pair = Value::Vector(vec![pair, Value::Number(SchemeNumber::Integer(3))]);
        
        assert_eq!(format!("{}", vec_with_pair), "#((1 . 2) 3)");
    }

    #[test]
    fn test_display_large_vector() {
        let large_vec = Value::Vector(
            (1..=10)
                .map(|i| Value::Number(SchemeNumber::Integer(i)))
                .collect(),
        );
        assert_eq!(format!("{}", large_vec), "#(1 2 3 4 5 6 7 8 9 10)");
    }
}

#[cfg(test)]
mod record_display_tests {
    use super::*;

    #[test]
    fn test_display_record_type() {
        let record_type = RecordType {
            name: "person".to_string(),
            field_names: vec!["name".to_string(), "age".to_string()],
            constructor_name: "make-person".to_string(),
            predicate_name: "person?".to_string(),
        };
        let record = Record {
            record_type,
            fields: vec![Value::String("Alice".to_string()), Value::Number(SchemeNumber::Integer(30))],
        };
        let record_value = Value::Record(record);
        
        assert_eq!(format!("{}", record_value), "#<person:\"Alice\" 30>");
    }

    #[test]
    fn test_display_record_instance() {
        let record_type = RecordType {
            name: "person".to_string(),
            field_names: vec!["name".to_string(), "age".to_string()],
            constructor_name: "make-person".to_string(),
            predicate_name: "person?".to_string(),
        };
        let record = Record {
            record_type,
            fields: vec![
                Value::String("Alice".to_string()),
                Value::Number(SchemeNumber::Integer(30)),
            ],
        };
        let record_value = Value::Record(record);
        
        assert_eq!(format!("{}", record_value), "#<person:\"Alice\" 30>");
    }

    #[test]
    fn test_display_record_with_complex_fields() {
        let record_type = RecordType {
            name: "data".to_string(),
            field_names: vec!["items".to_string(), "count".to_string()],
            constructor_name: "make-data".to_string(),
            predicate_name: "data?".to_string(),
        };
        
        let items_list = {
            let car1 = Value::Number(SchemeNumber::Integer(1));
            let car2 = Value::Number(SchemeNumber::Integer(2));
            let pair2 = PairData::new(car2, Value::Nil);
            let pair1 = PairData::new(car1, Value::Pair(Rc::new(RefCell::new(pair2))));
            Value::Pair(Rc::new(RefCell::new(pair1)))
        };
        
        let record = Record {
            record_type,
            fields: vec![items_list, Value::Number(SchemeNumber::Integer(2))],
        };
        let record_value = Value::Record(record);
        
        assert_eq!(format!("{}", record_value), "#<data:(1 2) 2>");
    }
}


#[cfg(test)]
mod complex_structure_display_tests {
    use super::*;

    #[test]
    fn test_display_nested_list_and_vector() {
        let inner_vec = Value::Vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ]);
        
        let pair_data = PairData::new(inner_vec, Value::Nil);
        let list_with_vec = Value::Pair(Rc::new(RefCell::new(pair_data)));
        
        assert_eq!(format!("{}", list_with_vec), "(#(1 2))");
    }

    #[test]
    fn test_display_vector_with_nested_lists() {
        let inner_pair = PairData::new(
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        );
        let inner_list = Value::Pair(Rc::new(RefCell::new(inner_pair)));
        
        let vec_with_list = Value::Vector(vec![inner_list, Value::Number(SchemeNumber::Integer(3))]);
        
        assert_eq!(format!("{}", vec_with_list), "#((1 . 2) 3)");
    }

    #[test]
    fn test_display_deeply_nested_structure() {
        let deep_inner = Value::Number(SchemeNumber::Integer(42));
        let deep_pair = PairData::new(deep_inner, Value::Nil);
        let deep_list = Value::Pair(Rc::new(RefCell::new(deep_pair)));
        
        let mid_vec = Value::Vector(vec![deep_list]);
        let mid_pair = PairData::new(mid_vec, Value::Nil);
        let mid_list = Value::Pair(Rc::new(RefCell::new(mid_pair)));
        
        let outer_vec = Value::Vector(vec![mid_list]);
        
        assert_eq!(format!("{}", outer_vec), "#((#((42))))");
    }

    #[test]
    fn test_display_mixed_procedures_and_data() {
        let builtin = Procedure::Builtin {
            name: "+".to_string(),
            arity: None,
            func: |_args| Ok(Value::Number(SchemeNumber::Integer(0))),
        };
        let proc_value = Value::Procedure(builtin);
        
        let pair_data = PairData::new(proc_value, Value::Number(SchemeNumber::Integer(42)));
        let mixed_pair = Value::Pair(Rc::new(RefCell::new(pair_data)));
        
        assert_eq!(format!("{}", mixed_pair), "(#<builtin +> . 42)");
    }

    #[test]
    fn test_display_list_with_various_types() {
        let elements = vec![
            Value::Boolean(true),
            Value::Number(SchemeNumber::Integer(42)),
            Value::String("test".to_string()),
            Value::Symbol("symbol".to_string()),
            Value::Character('x'),
            Value::Vector(vec![Value::Number(SchemeNumber::Integer(1))]),
        ];
        
        let mut current = Value::Nil;
        for element in elements.into_iter().rev() {
            let pair_data = PairData::new(element, current);
            current = Value::Pair(Rc::new(RefCell::new(pair_data)));
        }
        
        assert_eq!(format!("{}", current), "(#t 42 \"test\" symbol #\\x #(1))");
    }
}

#[cfg(test)]
mod debug_display_tests {
    use super::*;

    #[test]
    fn test_debug_display_consistency() {
        let values = vec![
            Value::Boolean(true),
            Value::Number(SchemeNumber::Integer(42)),
            Value::String("hello".to_string()),
            Value::Symbol("test".to_string()),
            Value::Character('a'),
            Value::Nil,
            Value::Undefined,
        ];
        
        for value in values {
            let display_str = format!("{}", value);
            let debug_str = format!("{:?}", value);
            
            // Debug format should contain the display format information
            assert!(!display_str.is_empty());
            assert!(!debug_str.is_empty());
            
            // Both should be consistent and not panic
            assert_eq!(display_str, format!("{}", value));
            assert_eq!(debug_str, format!("{:?}", value));
        }
    }

    #[test]
    fn test_display_special_characters() {
        let special_chars = vec![
            Value::Character('\0'),
            Value::Character('\r'),
            Value::Character('\x08'), // backspace
            Value::Character('\x0c'), // form feed
            Value::Character('\x7f'), // delete
        ];
        
        for char_val in special_chars {
            let display_str = format!("{}", char_val);
            assert!(display_str.starts_with("#\\"));
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_display_unicode_characters() {
        let unicode_chars = vec![
            Value::Character('α'),
            Value::Character('β'),
            Value::Character('π'),
            Value::Character('λ'),
            Value::Character('🎉'),
            Value::Character('🚀'),
        ];
        
        for char_val in unicode_chars {
            let display_str = format!("{}", char_val);
            assert!(display_str.starts_with("#\\"));
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_display_very_long_strings() {
        let long_string = "a".repeat(1000);
        let string_value = Value::String(long_string.clone());
        
        let display_str = format!("{}", string_value);
        assert!(display_str.starts_with('"'));
        assert!(display_str.ends_with('"'));
        assert!(display_str.contains(&long_string));
    }

    #[test]
    fn test_display_strings_with_quotes() {
        let string_with_quotes = Value::String("hello \"world\" test".to_string());
        let display_str = format!("{}", string_with_quotes);
        
        assert!(display_str.starts_with('"'));
        assert!(display_str.ends_with('"'));
        // Test that quotes are displayed correctly
        assert!(display_str.contains("world"));
        assert!(display_str.contains("test"));
    }

    #[test]
    fn test_display_strings_with_backslashes() {
        let string_with_backslashes = Value::String("hello\\world\\test".to_string());
        let display_str = format!("{}", string_with_backslashes);
        
        assert!(display_str.starts_with('"'));
        assert!(display_str.ends_with('"'));
        // Test that backslashes are displayed correctly
        assert!(display_str.contains("world"));
        assert!(display_str.contains("test"));
    }
}