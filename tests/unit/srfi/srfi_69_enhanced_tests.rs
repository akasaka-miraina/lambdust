//! Enhanced SRFI 69 tests for hash table higher-order functions

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_hash_table_merge() {
    let mut evaluator = Evaluator::new();

    // Import SRFI 69
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(69))),
        ]),
    ]);

    evaluator
        .eval(
            import_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Create first hash table
    let ht1_expr = Expr::List(vec![Expr::Variable("make-hash-table".to_string())]);

    let ht1 = evaluator
        .eval(
            ht1_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Add some values to first table
    let set1_expr = Expr::List(vec![
        Expr::Variable("hash-table-set!".to_string()),
        Expr::Variable("ht1".to_string()),
        Expr::Literal(Literal::String("key1".to_string())),
        Expr::Literal(Literal::String("value1".to_string())),
    ]);

    // Define ht1 in environment
    evaluator.global_env.define("ht1".to_string(), ht1);

    evaluator
        .eval(
            set1_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Create second hash table
    let ht2_expr = Expr::List(vec![Expr::Variable("make-hash-table".to_string())]);

    let ht2 = evaluator
        .eval(
            ht2_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Define ht2 in environment
    evaluator.global_env.define("ht2".to_string(), ht2);

    // Add some values to second table
    let set2_expr = Expr::List(vec![
        Expr::Variable("hash-table-set!".to_string()),
        Expr::Variable("ht2".to_string()),
        Expr::Literal(Literal::String("key2".to_string())),
        Expr::Literal(Literal::String("value2".to_string())),
    ]);

    evaluator
        .eval(
            set2_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Merge ht2 into ht1
    let merge_expr = Expr::List(vec![
        Expr::Variable("hash-table-merge!".to_string()),
        Expr::Variable("ht1".to_string()),
        Expr::Variable("ht2".to_string()),
    ]);

    let result = evaluator.eval(
        merge_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());

    // Check that ht1 now contains both keys
    let get1_expr = Expr::List(vec![
        Expr::Variable("hash-table-ref".to_string()),
        Expr::Variable("ht1".to_string()),
        Expr::Literal(Literal::String("key1".to_string())),
    ]);

    let result1 = evaluator.eval(
        get1_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Value::String("value1".to_string()));

    let get2_expr = Expr::List(vec![
        Expr::Variable("hash-table-ref".to_string()),
        Expr::Variable("ht1".to_string()),
        Expr::Literal(Literal::String("key2".to_string())),
    ]);

    let result2 = evaluator.eval(
        get2_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Value::String("value2".to_string()));
}

#[test]
fn test_make_hash_table_with_custom_functions() {
    let mut evaluator = Evaluator::new();

    // Import SRFI 69
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(69))),
        ]),
    ]);

    evaluator
        .eval(
            import_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Create hash table with custom equality function
    let ht_expr = Expr::List(vec![
        Expr::Variable("make-hash-table".to_string()),
        Expr::Variable("equal?".to_string()),
        Expr::Variable("hash".to_string()),
    ]);

    let result = evaluator.eval(
        ht_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());

    // Check that result is a hash table
    match result.unwrap() {
        Value::HashTable(_) => (), // Success
        _ => panic!("Expected hash table result"),
    }
}

#[test]
fn test_hash_table_fold_with_builtin() {
    let mut evaluator = Evaluator::new();

    // Import SRFI 69
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(69))),
        ]),
    ]);

    evaluator
        .eval(
            import_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Create and populate hash table
    let ht_expr = Expr::List(vec![Expr::Variable("make-hash-table".to_string())]);

    let ht = evaluator
        .eval(
            ht_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    evaluator.global_env.define("ht".to_string(), ht);

    // Add some numeric values
    for i in 1..=3 {
        let set_expr = Expr::List(vec![
            Expr::Variable("hash-table-set!".to_string()),
            Expr::Variable("ht".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(i))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(i * 10))),
        ]);

        evaluator
            .eval(
                set_expr,
                evaluator.global_env.clone(),
                Continuation::Identity,
            )
            .unwrap();
    }

    // Test hash-table-fold with + (special form version)
    let fold_expr = Expr::List(vec![
        Expr::Variable("hash-table-fold".to_string()),
        Expr::Variable("ht".to_string()),
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
    ]);

    let result = evaluator.eval(
        fold_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // This should work as a special form with + builtin
    assert!(result.is_ok());
}

#[test]
fn test_hash_table_fold_with_lambda() {
    let mut evaluator = Evaluator::new();

    // Import SRFI 69
    let import_expr = Expr::List(vec![
        Expr::Variable("import".to_string()),
        Expr::List(vec![
            Expr::Variable("srfi".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(69))),
        ]),
    ]);

    evaluator
        .eval(
            import_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Create and populate hash table
    let ht_expr = Expr::List(vec![Expr::Variable("make-hash-table".to_string())]);

    let ht = evaluator
        .eval(
            ht_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    evaluator.global_env.define("ht".to_string(), ht);

    // Add some values
    let set1_expr = Expr::List(vec![
        Expr::Variable("hash-table-set!".to_string()),
        Expr::Variable("ht".to_string()),
        Expr::Literal(Literal::String("a".to_string())),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);

    evaluator
        .eval(
            set1_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    let set2_expr = Expr::List(vec![
        Expr::Variable("hash-table-set!".to_string()),
        Expr::Variable("ht".to_string()),
        Expr::Literal(Literal::String("b".to_string())),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);

    evaluator
        .eval(
            set2_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        )
        .unwrap();

    // Test hash-table-fold with lambda (special form version)
    let fold_expr = Expr::List(vec![
        Expr::Variable("hash-table-fold".to_string()),
        Expr::Variable("ht".to_string()),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![
                Expr::Variable("k".to_string()),
                Expr::Variable("v".to_string()),
                Expr::Variable("acc".to_string()),
            ]),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("v".to_string()),
                Expr::Variable("acc".to_string()),
            ]),
        ]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
    ]);

    let result = evaluator.eval(
        fold_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // This should work as a special form with lambda support
    assert!(result.is_ok());

    // Result should be 1 + 2 = 3
    assert_eq!(result.unwrap(), Value::Number(SchemeNumber::Integer(3)));
}
