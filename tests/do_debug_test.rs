use lambdust::Interpreter;

#[test]
fn test_do_step_debug() {
    let mut interpreter = Interpreter::new();

    // Test the first do loop from our earlier failed test
    println!("Testing: (do ((i 0 (+ i 1)) (acc 1 (* acc i))) ((>= i 5) acc))");
    let result = interpreter
        .eval("(do ((i 0 (+ i 1)) (acc 1 (* acc i))) ((>= i 5) acc))")
        .unwrap();
    println!("Result: {:?}", result);
    // Expected: i: 0,1,2,3,4,5; acc: 1,1*0=0,0*1=0,0*2=0,0*3=0,0*4=0 -> should be 0
    assert_eq!(result, lambdust::Value::from(0i64));

    // Test sum case
    println!("\nTesting: (do ((x 10 (- x 1)) (sum 0 (+ sum x))) ((= x 0) sum))");
    let result = interpreter
        .eval("(do ((x 10 (- x 1)) (sum 0 (+ sum x))) ((= x 0) sum))")
        .unwrap();
    println!("Result: {:?}", result);
    // Expected: x: 10,9,8,7,6,5,4,3,2,1,0; sum: 0,0+10=10,10+9=19,19+8=27,27+7=34,34+6=40,40+5=45,45+4=49,49+3=52,52+2=54,54+1=55 -> should be 55
    assert_eq!(result, lambdust::Value::from(55i64));

    // Test more complex step
    println!("\nTesting: (do ((i 1 (* i 2))) ((> i 10) i))");
    let result = interpreter
        .eval("(do ((i 1 (* i 2))) ((> i 10) i))")
        .unwrap();
    println!("Result: {:?}", result);
    // Expected: i: 1,2,4,8,16 -> should be 16
    assert_eq!(result, lambdust::Value::from(16i64));
}
