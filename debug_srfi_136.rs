use lambdust::environment::Environment;
use lambdust::error::Result;
use lambdust::evaluator::eval_with_formal_semantics;
use lambdust::lexer::tokenize;
use lambdust::parser::parse;
use lambdust::srfi::{SrfiImport, SrfiRegistry};
use lambdust::value::Value;
use std::rc::Rc;

fn eval_str_with_srfi_136(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    let ast = parse(tokens)?;
    let env = Rc::new(Environment::with_builtins());
    
    // Import SRFI 136
    let registry = SrfiRegistry::with_standard_srfis();
    let import = SrfiImport::new(136);
    let exports = registry.import_srfi(&import)?;
    
    for (name, value) in exports {
        env.define(name, value);
    }
    
    eval_with_formal_semantics(ast, env)
}

fn main() {
    println!("Testing SRFI 136 make-record-type-descriptor...");
    
    // Test 1: simple create
    let result = eval_str_with_srfi_136("(make-record-type-descriptor 'point '#(x y))");
    match result {
        Ok(value) => println!("Test 1 result: {:?}", value),
        Err(e) => println!("Test 1 error: {:?}", e),
    }
    
    // Test 2: define and reference
    let result2 = eval_str_with_srfi_136(r#"
        (begin
          (define point-rtd (make-record-type-descriptor 'point '#(x y)))
          (record-type-descriptor? point-rtd))
    "#);
    match result2 {
        Ok(value) => println!("Test 2 result: {:?}", value),
        Err(e) => println!("Test 2 error: {:?}", e),
    }
    
    // Test 3: just define
    let result3 = eval_str_with_srfi_136(r#"
        (define point-rtd (make-record-type-descriptor 'point '#(x y)))
    "#);
    match result3 {
        Ok(value) => println!("Test 3 result: {:?}", value),
        Err(e) => println!("Test 3 error: {:?}", e),
    }
}