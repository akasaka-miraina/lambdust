use lambdust::Interpreter;

#[test]
fn test_do_step_expression_bug() {
    let mut interpreter = Interpreter::new();

    // This should compute factorial
    // i starts at 1, increments by 1 each time: 1, 2, 3, 4, 5, 6
    // fact starts at 1, multiplies by i each time: 1, 1*1=1, 1*2=2, 2*3=6, 6*4=24, 24*5=120
    let result = interpreter
        .eval("(do ((i 1 (+ i 1)) (fact 1 (* fact i))) ((> i 5) fact))")
        .unwrap();

    println!("Factorial result: {:?}", result);
    assert_eq!(result, lambdust::Value::from(120i64));
}
