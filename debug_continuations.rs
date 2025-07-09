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
    println!("=== 継続詳細調査 ===\n");

    // SingleBegin条件テスト (remaining.len() == 1)
    println!("Test 1: SingleBegin条件 (len=1)");
    let result1 = eval_str("(begin 42)");  // remaining=[], len=0
    println!("(begin 42) => {:?}\n", result1);

    println!("Test 2: SingleBegin条件 (len=1)");  
    let result2 = eval_str("(begin 1 2)");  // remaining=[2], len=1
    println!("(begin 1 2) => {:?}\n", result2);

    println!("Test 3: Non-SingleBegin (len=2)");
    let result3 = eval_str("(begin 1 2 3)");  // remaining=[2,3], len=2
    println!("(begin 1 2 3) => {:?}\n", result3);

    // defineとの組み合わせ
    println!("Test 4: define + value (SingleBegin?)");
    let result4 = eval_str("(begin (define x 100) 200)");
    println!("(begin (define x 100) 200) => {:?}\n", result4);

    println!("Test 5: define + value + value (Non-SingleBegin)");
    let result5 = eval_str("(begin (define y 100) 200 300)");
    println!("(begin (define y 100) 200 300) => {:?}\n", result5);
}