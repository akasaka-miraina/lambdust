use lambdust::environment::Environment;
use lambdust::error::Result;
use lambdust::evaluator::eval_with_formal_semantics;
use lambdust::lexer::tokenize;
use lambdust::parser::parse;
use lambdust::value::Value;
use std::rc::Rc;

fn eval_str(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    let ast = parse(tokens)?;
    let env = Rc::new(Environment::with_builtins());
    eval_with_formal_semantics(ast, env)
}

fn main() {
    println!("=== 環境変数管理システム調査 ===\n");

    // Test 1: 基本的な変数定義
    println!("Test 1: 基本的な変数定義");
    let result1 = eval_str("(define x 42)");
    match result1 {
        Ok(value) => println!("define x 42 => {:?}", value),
        Err(e) => println!("define x 42 => ERROR: {:?}", e),
    }

    // Test 2: 変数参照
    println!("\nTest 2: 変数参照");
    let result2 = eval_str(r#"
        (begin
          (define y 100)
          y)
    "#);
    match result2 {
        Ok(value) => println!("begin define y, y => {:?}", value),
        Err(e) => println!("begin define y, y => ERROR: {:?}", e),
    }

    // Test 3: 同一環境での複数定義・参照
    println!("\nTest 3: 同一環境での複数定義・参照");
    let result3 = eval_str(r#"
        (begin
          (define a 1)
          (define b 2)
          (+ a b))
    "#);
    match result3 {
        Ok(value) => println!("multiple define + reference => {:?}", value),
        Err(e) => println!("multiple define + reference => ERROR: {:?}", e),
    }

    // Test 4: 関数定義と呼び出し
    println!("\nTest 4: 関数定義と呼び出し");
    let result4 = eval_str(r#"
        (begin
          (define (square x) (* x x))
          (square 5))
    "#);
    match result4 {
        Ok(value) => println!("function define + call => {:?}", value),
        Err(e) => println!("function define + call => ERROR: {:?}", e),
    }

    // Test 5: 組み込み関数との混在
    println!("\nTest 5: 組み込み関数との混在");
    let result5 = eval_str(r#"
        (begin
          (define z 10)
          (+ z 5))
    "#);
    match result5 {
        Ok(value) => println!("user variable + builtin => {:?}", value),
        Err(e) => println!("user variable + builtin => ERROR: {:?}", e),
    }

    // Test 6: 複雑な式での環境テスト
    println!("\nTest 6: 複雑な式での環境テスト");
    let result6 = eval_str(r#"
        (begin
          (define make-adder 
            (lambda (n) 
              (lambda (x) (+ x n))))
          (define add10 (make-adder 10))
          (add10 5))
    "#);
    match result6 {
        Ok(value) => println!("closure test => {:?}", value),
        Err(e) => println!("closure test => ERROR: {:?}", e),
    }

    println!("\n=== ホスト組み込みパターンテスト ===\n");

    // Test 7: Environment直接操作パターン（ホスト組み込み模擬）
    println!("Test 7: Environment直接操作パターン");
    let env = Rc::new(Environment::with_builtins());
    
    // ホストアプリから変数を直接定義
    env.define("host_var".to_string(), Value::String("from_host".to_string()));
    
    // Scheme側から参照できるかテスト
    let tokens = tokenize("host_var").unwrap();
    let ast = parse(tokens).unwrap();
    let result7 = eval_with_formal_semantics(ast, env.clone());
    match result7 {
        Ok(value) => println!("host direct define => {:?}", value),
        Err(e) => println!("host direct define => ERROR: {:?}", e),
    }

    // Test 8: 混在パターン（define + host define）
    println!("\nTest 8: 混在パターン");
    let result8_str = r#"
        (begin
          (define scheme_var 123)
          (+ scheme_var 1))
    "#;
    let tokens8 = tokenize(result8_str).unwrap();
    let ast8 = parse(tokens8).unwrap();
    let result8 = eval_with_formal_semantics(ast8, env);
    match result8 {
        Ok(value) => println!("mixed environment => {:?}", value),
        Err(e) => println!("mixed environment => ERROR: {:?}", e),
    }
}