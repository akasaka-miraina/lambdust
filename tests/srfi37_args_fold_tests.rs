//! Tests for SRFI-37: args-fold implementation
//!
//! This module contains comprehensive tests for the SRFI-37 args-fold
//! argument processing system, covering:
//! - Option creation and descriptor handling
//! - Short and long option processing
//! - Options with required/optional arguments
//! - Clustered short options (-abc)
//! - Unrecognized option handling
//! - Operand processing
//! - Complex argument processing scenarios
//! - Error handling and edge cases

use lambdust::diagnostics::{Error, ErrorKind, Result};
use lambdust::effects::Effect;
use lambdust::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use lambdust::stdlib::srfi37_args_fold::{args_fold, bind_srfi37_procedures, option};
use std::sync::Arc;

/// Helper function to create a simple test environment
fn create_test_environment() -> Arc<ThreadSafeEnvironment> {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    bind_srfi37_procedures(&env);
    env
}

/// Helper function to create a test processor that accumulates values
fn create_accumulator_proc(tag: &str) -> Value {
    let tag = tag.to_string();
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: format!("{}-handler", tag),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::Native(move |args| {
            // Extract the current seed (last argument)
            let seed = args.last().unwrap_or(&Value::Nil);

            // Create a tagged entry for this processing event
            let entry = Value::List(vec![
                Value::symbol(tag.clone()),
                args[0].clone(), // The processed argument/option
            ]);

            // Add to seed list or create new list
            match seed {
                Value::List(list) => {
                    let mut new_list = list.clone();
                    new_list.push(entry);
                    Ok(Value::List(new_list))
                }
                Value::Nil => Ok(Value::List(vec![entry])),
                _ => Ok(Value::List(vec![entry, seed.clone()])),
            }
        }),
        effects: &[Effect::Pure],
    }))
}

/// Helper function to create a simple processor that returns a constant
fn create_constant_proc(constant: Value) -> Value {
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "constant-handler".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::Native(move |_args| Ok(constant.clone())),
        effects: &[Effect::Pure],
    }))
}

#[test]
fn test_option_creation_basic() {
    // Test basic option creation with single name
    let names = Value::List(vec![Value::character('h')]);
    let args = [
        names,
        Value::boolean(false), // required-arg?
        Value::boolean(false), // optional-arg?
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(desc)) = result {
        assert_eq!(desc.len(), 5);
        assert!(matches!(desc[0], Value::symbol(ref s) if s == "option-descriptor"));
        assert!(matches!(desc[2], Value::boolean(false))); // required-arg?
        assert!(matches!(desc[3], Value::boolean(false))); // optional-arg?
    } else {
        panic!("Expected option descriptor list");
    }
}

#[test]
fn test_option_creation_multiple_names() {
    // Test option with both short and long names
    let names = Value::List(vec![
        Value::character('h'),
        Value::string("help".to_string()),
    ]);

    let args = [
        names,
        Value::boolean(false),
        Value::boolean(false),
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(desc)) = result {
        if let Value::List(names_list) = &desc[1] {
            assert_eq!(names_list.len(), 2);
            assert!(matches!(names_list[0], Value::string(ref s) if s == "h"));
            assert!(matches!(names_list[1], Value::string(ref s) if s == "help"));
        } else {
            panic!("Expected names list in option descriptor");
        }
    } else {
        panic!("Expected option descriptor list");
    }
}

#[test]
fn test_option_with_required_arg() {
    let names = Value::List(vec![Value::character('f')]);
    let args = [
        names,
        Value::boolean(true),  // required-arg?
        Value::boolean(false), // optional-arg?
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(desc)) = result {
        assert!(matches!(desc[2], Value::boolean(true))); // required-arg?
        assert!(matches!(desc[3], Value::boolean(false))); // optional-arg?
    }
}

#[test]
fn test_option_with_optional_arg() {
    let names = Value::List(vec![Value::character('o')]);
    let args = [
        names,
        Value::boolean(false), // required-arg?
        Value::boolean(true),  // optional-arg?
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(desc)) = result {
        assert!(matches!(desc[2], Value::boolean(false))); // required-arg?
        assert!(matches!(desc[3], Value::boolean(true))); // optional-arg?
    }
}

#[test]
fn test_option_error_both_required_and_optional() {
    let names = Value::List(vec![Value::character('x')]);
    let args = [
        names,
        Value::boolean(true), // required-arg?
        Value::boolean(true), // optional-arg? - ERROR!
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_err());

    if let Err(error) = result {
        assert!(matches!(error.kind(), ErrorKind::ArgumentError));
    }
}

#[test]
fn test_option_error_wrong_arity() {
    // Test with wrong number of arguments
    let result = option(&[Value::Nil]);
    assert!(result.is_err());

    let result = option(&[]);
    assert!(result.is_err());
}

#[test]
fn test_option_error_invalid_names() {
    // Test with non-list names
    let args = [
        Value::string("not-a-list".to_string()),
        Value::boolean(false),
        Value::boolean(false),
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_err());

    // Test with invalid name types in list
    let names = Value::List(vec![
        Value::integer(42), // Invalid - should be character or string
    ]);
    let args = [
        names,
        Value::boolean(false),
        Value::boolean(false),
        create_constant_proc(Value::Nil),
    ];

    let result = option(&args);
    assert!(result.is_err());
}

#[test]
fn test_args_fold_basic_short_option() {
    // Test basic short option processing: -h
    let args_list = Value::List(vec![Value::string("-h".to_string())]);

    let help_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("h".to_string())]),
        Value::boolean(false), // not required
        Value::boolean(false), // not optional
        create_accumulator_proc("help"),
    ]);

    let options = Value::List(vec![help_option]);
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 1);
        if let Value::List(event) = &events[0] {
            assert!(matches!(event[0], Value::symbol(ref s) if s == "help"));
        }
    }
}

#[test]
fn test_args_fold_basic_long_option() {
    // Test basic long option processing: --help
    let args_list = Value::List(vec![Value::string("--help".to_string())]);

    let help_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("help".to_string())]),
        Value::boolean(false),
        Value::boolean(false),
        create_accumulator_proc("help"),
    ]);

    let options = Value::List(vec![help_option]);
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 1);
        if let Value::List(event) = &events[0] {
            assert!(matches!(event[0], Value::symbol(ref s) if s == "help"));
        }
    }
}

#[test]
fn test_args_fold_operand_processing() {
    // Test operand (non-option) processing
    let args_list = Value::List(vec![
        Value::string("file.txt".to_string()),
        Value::string("another-file.txt".to_string()),
    ]);

    let options = Value::List(vec![]); // No options
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 2);
        // Both should be operand events
        for event in events {
            if let Value::List(event_parts) = event {
                assert!(matches!(event_parts[0], Value::symbol(ref s) if s == "operand"));
            }
        }
    }
}

#[test]
fn test_args_fold_option_with_required_argument() {
    // Test option that requires an argument: -f file.txt
    let args_list = Value::List(vec![
        Value::string("-f".to_string()),
        Value::string("input.txt".to_string()),
    ]);

    let file_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("f".to_string())]),
        Value::boolean(true),  // required argument
        Value::boolean(false), // not optional
        create_accumulator_proc("file"),
    ]);

    let options = Value::List(vec![file_option]);
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 1);
        if let Value::List(event) = &events[0] {
            assert!(matches!(event[0], Value::symbol(ref s) if s == "file"));
        }
    }
}

#[test]
fn test_args_fold_long_option_with_embedded_argument() {
    // Test long option with embedded argument: --file=input.txt
    let args_list = Value::List(vec![Value::string("--file=input.txt".to_string())]);

    let file_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("file".to_string())]),
        Value::boolean(true),  // required argument
        Value::boolean(false), // not optional
        create_accumulator_proc("file"),
    ]);

    let options = Value::List(vec![file_option]);
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 1);
        if let Value::List(event) = &events[0] {
            assert!(matches!(event[0], Value::symbol(ref s) if s == "file"));
        }
    }
}

#[test]
fn test_args_fold_unrecognized_option() {
    // Test unrecognized option handling
    let args_list = Value::List(vec![
        Value::string("--unknown".to_string()),
        Value::string("-x".to_string()),
    ]);

    let options = Value::List(vec![]); // No defined options
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 2);
        // Both should be unrecognized events
        for event in events {
            if let Value::List(event_parts) = event {
                assert!(matches!(event_parts[0], Value::symbol(ref s) if s == "unrecognized"));
            }
        }
    }
}

#[test]
fn test_args_fold_double_dash_separator() {
    // Test -- separator handling
    let args_list = Value::List(vec![
        Value::string("-h".to_string()),
        Value::string("--".to_string()),
        Value::string("-v".to_string()), // This should be treated as operand after --
        Value::string("file.txt".to_string()),
    ]);

    let help_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("h".to_string())]),
        Value::boolean(false),
        Value::boolean(false),
        create_accumulator_proc("help"),
    ]);

    let options = Value::List(vec![help_option]);
    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        assert_eq!(events.len(), 3);
        // First should be help option, then two operands
        if let Value::List(first_event) = &events[0] {
            assert!(matches!(first_event[0], Value::symbol(ref s) if s == "help"));
        }
        if let Value::List(second_event) = &events[1] {
            assert!(matches!(second_event[0], Value::symbol(ref s) if s == "operand"));
        }
        if let Value::List(third_event) = &events[2] {
            assert!(matches!(third_event[0], Value::symbol(ref s) if s == "operand"));
        }
    }
}

#[test]
fn test_args_fold_error_cases() {
    // Test error: wrong number of arguments
    let result = args_fold(&[]);
    assert!(result.is_err());

    let result = args_fold(&[Value::Nil, Value::Nil]);
    assert!(result.is_err());

    // Test error: non-list arguments
    let args = [
        Value::string("not-a-list".to_string()), // Should be list
        Value::List(vec![]),
        create_constant_proc(Value::Nil),
        create_constant_proc(Value::Nil),
    ];
    let result = args_fold(&args);
    assert!(result.is_err());

    // Test error: non-string in arguments list
    let args_list = Value::List(vec![
        Value::integer(42), // Should be string
    ]);
    let args = [
        args_list,
        Value::List(vec![]),
        create_constant_proc(Value::Nil),
        create_constant_proc(Value::Nil),
    ];
    let result = args_fold(&args);
    assert!(result.is_err());
}

#[test]
fn test_args_fold_complex_scenario() {
    // Test a complex, realistic scenario
    let args_list = Value::List(vec![
        Value::string("-v".to_string()),
        Value::string("--output".to_string()),
        Value::string("result.txt".to_string()),
        Value::string("-abc".to_string()), // Clustered options
        Value::string("input.txt".to_string()),
        Value::string("--flag".to_string()),
    ]);

    let verbose_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("v".to_string())]),
        Value::boolean(false),
        Value::boolean(false),
        create_accumulator_proc("verbose"),
    ]);

    let output_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("output".to_string())]),
        Value::boolean(true), // requires argument
        Value::boolean(false),
        create_accumulator_proc("output"),
    ]);

    let flag_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("flag".to_string())]),
        Value::boolean(false),
        Value::boolean(false),
        create_accumulator_proc("flag"),
    ]);

    let a_option = Value::List(vec![
        Value::symbol("option-descriptor".to_string()),
        Value::List(vec![Value::string("a".to_string())]),
        Value::boolean(false),
        Value::boolean(false),
        create_accumulator_proc("a-opt"),
    ]);

    let options = Value::List(vec![verbose_option, output_option, flag_option, a_option]);

    let unrecognized = create_accumulator_proc("unrecognized");
    let operand = create_accumulator_proc("operand");

    let args = [
        args_list,
        options,
        unrecognized,
        operand,
        Value::List(vec![]), // initial seed
    ];

    let result = args_fold(&args);
    assert!(result.is_ok());

    if let Ok(Value::List(events)) = result {
        // Should process: -v, --output result.txt, -a (from -abc), -b/-c (unrecognized), input.txt (operand), --flag
        assert!(events.len() >= 5);

        // Check that we have the expected event types
        let event_types: Vec<String> = events
            .iter()
            .filter_map(|event| {
                if let Value::List(parts) = event {
                    if let Value::symbol(tag) = &parts[0] {
                        Some(tag.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Should contain verbose, output, a-opt, and operand events
        assert!(event_types.contains(&"verbose".to_string()));
        assert!(event_types.contains(&"output".to_string()));
        assert!(event_types.contains(&"a-opt".to_string()));
        assert!(event_types.contains(&"operand".to_string()));
        assert!(event_types.contains(&"flag".to_string()));
    }
}

#[test]
fn test_environment_integration() {
    // Test that SRFI-37 procedures are properly bound in environment
    let env = create_test_environment();

    // Check that option procedure is bound
    let option_val = env.lookup("option");
    assert!(option_val.is_some());
    assert!(matches!(option_val.unwrap(), Value::Primitive(_)));

    // Check that args-fold procedure is bound
    let args_fold_val = env.lookup("args-fold");
    assert!(args_fold_val.is_some());
    assert!(matches!(args_fold_val.unwrap(), Value::Primitive(_)));
}
