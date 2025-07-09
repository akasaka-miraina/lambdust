//! Unit tests for higher-order functions (higher_order.rs)
//!
//! Tests the higher-order built-in functions including map, for-each, apply,
//! filter, fold, and fold-right with various scenarios and error conditions.

use lambdust::builtins::higher_order::register_higher_order_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
use std::collections::HashMap;

#[test]
fn test_higher_order_functions_registration() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Check that all higher-order functions are registered
    assert!(builtins.contains_key("map"));
    assert!(builtins.contains_key("for-each"));
    assert!(builtins.contains_key("apply"));
    assert!(builtins.contains_key("filter"));
    assert!(builtins.contains_key("fold"));
    assert!(builtins.contains_key("fold-right"));

    // Check that they are all procedures
    for (name, value) in &builtins {
        assert!(value.is_procedure(), "{} should be a procedure", name);
    }
}

#[test]
fn test_map_with_builtin_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a simple builtin function for testing
    let add_one = Value::Procedure(Procedure::Builtin {
        name: "add-one".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(SchemeNumber::Integer(n)) => {
                    Ok(Value::Number(SchemeNumber::Integer(n + 1)))
                }
                _ => Err(LambdustError::type_error("Expected integer".to_string())),
            }
        },
    });

    let map_proc = builtins.get("map").unwrap();
    if let Value::Procedure(proc) = map_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test map with a list of numbers
                let list = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(
                        Value::Number(SchemeNumber::Integer(2)),
                        Value::cons(Value::Number(SchemeNumber::Integer(3)), Value::Nil),
                    ),
                );
                let args = vec![add_one, list];
                let result = func(&args);
                assert!(result.is_ok());

                // Check that the result is a list
                let result_value = result.unwrap();
                assert!(result_value.is_list());

                // Convert to vector and check values
                if let Some(vec) = result_value.to_vector() {
                    assert_eq!(vec.len(), 3);
                    assert_eq!(vec[0], Value::Number(SchemeNumber::Integer(2)));
                    assert_eq!(vec[1], Value::Number(SchemeNumber::Integer(3)));
                    assert_eq!(vec[2], Value::Number(SchemeNumber::Integer(4)));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_map_with_empty_list() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let add_one = Value::Procedure(Procedure::Builtin {
        name: "add-one".to_string(),
        arity: Some(1),
        func: |_args| Ok(Value::Number(SchemeNumber::Integer(1))),
    });

    let map_proc = builtins.get("map").unwrap();
    if let Value::Procedure(proc) = map_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![add_one, Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());

                let result_value = result.unwrap();
                assert_eq!(result_value, Value::Nil);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_map_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let map_proc = builtins.get("map").unwrap();
    if let Value::Procedure(proc) = map_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test map with no arguments
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }

                // Test map with only one argument
                let args = vec![Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_map_type_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let map_proc = builtins.get("map").unwrap();
    if let Value::Procedure(proc) = map_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test map with non-procedure first argument
                let args = vec![Value::Number(SchemeNumber::Integer(42)), Value::Nil];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("first argument must be a procedure"));
                }

                // Test map with non-list second argument
                let proc = Value::Procedure(Procedure::Builtin {
                    name: "test".to_string(),
                    arity: Some(1),
                    func: |_args| Ok(Value::Number(SchemeNumber::Integer(1))),
                });
                let args = vec![proc, Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("argument 2 is not a list"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_for_each_with_builtin_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a simple builtin function for testing
    let test_proc = Value::Procedure(Procedure::Builtin {
        name: "test-proc".to_string(),
        arity: Some(1),
        func: |args| {
            // Just return the argument for testing
            Ok(args[0].clone())
        },
    });

    let for_each_proc = builtins.get("for-each").unwrap();
    if let Value::Procedure(proc) = for_each_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let list = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(
                        Value::Number(SchemeNumber::Integer(2)),
                        Value::cons(Value::Number(SchemeNumber::Integer(3)), Value::Nil),
                    ),
                );
                let args = vec![test_proc, list];
                let result = func(&args);
                assert!(result.is_ok());

                // for-each returns undefined
                assert_eq!(result.unwrap(), Value::Undefined);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_for_each_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let for_each_proc = builtins.get("for-each").unwrap();
    if let Value::Procedure(proc) = for_each_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test for-each with no arguments
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }

                // Test for-each with only one argument
                let args = vec![Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_apply_with_builtin_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a simple addition function for testing
    let add_proc = Value::Procedure(Procedure::Builtin {
        name: "add".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }
            match (&args[0], &args[1]) {
                (
                    Value::Number(SchemeNumber::Integer(a)),
                    Value::Number(SchemeNumber::Integer(b)),
                ) => Ok(Value::Number(SchemeNumber::Integer(a + b))),
                _ => Err(LambdustError::type_error("Expected integers".to_string())),
            }
        },
    });

    let apply_proc = builtins.get("apply").unwrap();
    if let Value::Procedure(proc) = apply_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test apply with a list of arguments
                let arg_list = Value::cons(
                    Value::Number(SchemeNumber::Integer(5)),
                    Value::cons(Value::Number(SchemeNumber::Integer(3)), Value::Nil),
                );
                let args = vec![add_proc, arg_list];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(8)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_apply_extended_form() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a function that takes 3 arguments
    let add_three = Value::Procedure(Procedure::Builtin {
        name: "add-three".to_string(),
        arity: Some(3),
        func: |args| {
            if args.len() != 3 {
                return Err(LambdustError::arity_error(3, args.len()));
            }
            match (&args[0], &args[1], &args[2]) {
                (
                    Value::Number(SchemeNumber::Integer(a)),
                    Value::Number(SchemeNumber::Integer(b)),
                    Value::Number(SchemeNumber::Integer(c)),
                ) => Ok(Value::Number(SchemeNumber::Integer(a + b + c))),
                _ => Err(LambdustError::type_error("Expected integers".to_string())),
            }
        },
    });

    let apply_proc = builtins.get("apply").unwrap();
    if let Value::Procedure(proc) = apply_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test apply with extended form: (apply proc arg1 arg2 ... args)
                let last_args = Value::cons(Value::Number(SchemeNumber::Integer(3)), Value::Nil);
                let args = vec![
                    add_three,
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(2)),
                    last_args,
                ];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(6)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_apply_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let apply_proc = builtins.get("apply").unwrap();
    if let Value::Procedure(proc) = apply_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test apply with no arguments
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }

                // Test apply with only one argument
                let args = vec![Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_apply_type_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let apply_proc = builtins.get("apply").unwrap();
    if let Value::Procedure(proc) = apply_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test apply with non-procedure first argument
                let args = vec![Value::Number(SchemeNumber::Integer(42)), Value::Nil];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("first argument must be a procedure"));
                }

                // Test apply with non-list second argument
                let proc = Value::Procedure(Procedure::Builtin {
                    name: "test".to_string(),
                    arity: Some(1),
                    func: |_args| Ok(Value::Number(SchemeNumber::Integer(1))),
                });
                let args = vec![proc, Value::Number(SchemeNumber::Integer(42))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::TypeError { message, .. }) = result {
                    assert!(message.contains("second argument must be a list"));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_filter_with_builtin_predicate() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a simple predicate function
    let even_pred = Value::Procedure(Procedure::Builtin {
        name: "even?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(SchemeNumber::Integer(n)) => Ok(Value::Boolean(n % 2 == 0)),
                _ => Err(LambdustError::type_error("Expected integer".to_string())),
            }
        },
    });

    let filter_proc = builtins.get("filter").unwrap();
    if let Value::Procedure(proc) = filter_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let list = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(
                        Value::Number(SchemeNumber::Integer(2)),
                        Value::cons(
                            Value::Number(SchemeNumber::Integer(3)),
                            Value::cons(Value::Number(SchemeNumber::Integer(4)), Value::Nil),
                        ),
                    ),
                );
                let args = vec![even_pred, list];
                let result = func(&args);
                assert!(result.is_ok());

                let result_value = result.unwrap();
                assert!(result_value.is_list());

                // Check that only even numbers are included
                if let Some(vec) = result_value.to_vector() {
                    assert_eq!(vec.len(), 2);
                    assert_eq!(vec[0], Value::Number(SchemeNumber::Integer(2)));
                    assert_eq!(vec[1], Value::Number(SchemeNumber::Integer(4)));
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_filter_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let filter_proc = builtins.get("filter").unwrap();
    if let Value::Procedure(proc) = filter_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test filter with no arguments
                let args = vec![];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }

                // Test filter with only one argument
                let args = vec![Value::Number(SchemeNumber::Integer(1))];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }

                // Test filter with too many arguments
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Nil,
                    Value::Number(SchemeNumber::Integer(3)),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 2);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_fold_with_builtin_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a simple addition function for folding
    let add_proc = Value::Procedure(Procedure::Builtin {
        name: "add".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }
            match (&args[0], &args[1]) {
                (
                    Value::Number(SchemeNumber::Integer(a)),
                    Value::Number(SchemeNumber::Integer(b)),
                ) => Ok(Value::Number(SchemeNumber::Integer(a + b))),
                _ => Err(LambdustError::type_error("Expected integers".to_string())),
            }
        },
    });

    let fold_proc = builtins.get("fold").unwrap();
    if let Value::Procedure(proc) = fold_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let list = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(
                        Value::Number(SchemeNumber::Integer(2)),
                        Value::cons(Value::Number(SchemeNumber::Integer(3)), Value::Nil),
                    ),
                );
                let args = vec![add_proc, Value::Number(SchemeNumber::Integer(0)), list];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(6)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_fold_arity_errors() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let fold_proc = builtins.get("fold").unwrap();
    if let Value::Procedure(proc) = fold_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                // Test fold with insufficient arguments
                let args = vec![
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::Number(SchemeNumber::Integer(2)),
                ];
                let result = func(&args);
                assert!(result.is_err());
                if let Err(LambdustError::ArityError { expected, .. }) = result {
                    assert_eq!(expected, 3);
                }
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_fold_right_with_builtin_function() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a subtraction function to show right-fold behavior
    let sub_proc = Value::Procedure(Procedure::Builtin {
        name: "sub".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }
            match (&args[0], &args[1]) {
                (
                    Value::Number(SchemeNumber::Integer(a)),
                    Value::Number(SchemeNumber::Integer(b)),
                ) => Ok(Value::Number(SchemeNumber::Integer(a - b))),
                _ => Err(LambdustError::type_error("Expected integers".to_string())),
            }
        },
    });

    let fold_right_proc = builtins.get("fold-right").unwrap();
    if let Value::Procedure(proc) = fold_right_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let list = Value::cons(
                    Value::Number(SchemeNumber::Integer(1)),
                    Value::cons(
                        Value::Number(SchemeNumber::Integer(2)),
                        Value::cons(Value::Number(SchemeNumber::Integer(3)), Value::Nil),
                    ),
                );
                let args = vec![sub_proc, Value::Number(SchemeNumber::Integer(0)), list];
                let result = func(&args);
                assert!(result.is_ok());
                // (1 - (2 - (3 - 0))) = (1 - (2 - 3)) = (1 - (-1)) = 2
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(2)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_lambda_function_limitations() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Create a mock lambda function
    let lambda_func = Value::Procedure(Procedure::Lambda {
        params: vec!["x".to_string()],
        body: vec![],
        closure: std::rc::Rc::new(lambdust::environment::Environment::new()),
        variadic: false,
    });

    // Test that all higher-order functions properly handle lambda limitation
    let function_names = vec!["map", "for-each", "apply", "filter", "fold", "fold-right"];

    for func_name in function_names {
        let proc = builtins.get(func_name).unwrap();
        if let Value::Procedure(procedure) = proc {
            match procedure {
                Procedure::Builtin { func, .. } => {
                    let args = vec![lambda_func.clone(), Value::Nil];
                    let result = func(&args);
                    assert!(result.is_err());
                    if let Err(LambdustError::RuntimeError { message, .. }) = result {
                        assert!(message.contains("lambda functions require evaluator integration"));
                        assert!(message.contains("not yet implemented"));
                    }
                }
                _ => panic!("Expected builtin procedure"),
            }
        }
    }
}

#[test]
fn test_higher_order_functions_with_empty_lists() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    let identity_proc = Value::Procedure(Procedure::Builtin {
        name: "identity".to_string(),
        arity: Some(1),
        func: |args| Ok(args[0].clone()),
    });

    // Test map with empty list
    let map_proc = builtins.get("map").unwrap();
    if let Value::Procedure(proc) = map_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![identity_proc.clone(), Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Nil);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test filter with empty list
    let filter_proc = builtins.get("filter").unwrap();
    if let Value::Procedure(proc) = filter_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![identity_proc.clone(), Value::Nil];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Nil);
            }
            _ => panic!("Expected builtin procedure"),
        }
    }

    // Test fold with empty list
    let fold_proc = builtins.get("fold").unwrap();
    if let Value::Procedure(proc) = fold_proc {
        match proc {
            Procedure::Builtin { func, .. } => {
                let args = vec![
                    identity_proc.clone(),
                    Value::Number(SchemeNumber::Integer(42)),
                    Value::Nil,
                ];
                let result = func(&args);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(42)));
            }
            _ => panic!("Expected builtin procedure"),
        }
    }
}

#[test]
fn test_higher_order_functions_isolation() {
    let mut builtins: HashMap<String, Value> = HashMap::new();
    register_higher_order_functions(&mut builtins);

    // Test that each higher-order function works independently
    let functions = vec!["map", "for-each", "apply", "filter", "fold", "fold-right"];

    for func_name in functions {
        let proc = builtins.get(func_name).unwrap();
        assert!(proc.is_procedure(), "{} should be a procedure", func_name);
    }
}
