use lambdust::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    
    // Test factorial-like computation
    println!("Testing: (do ((i 0 (+ i 1)) (acc 1 (* acc i))) ((>= i 5) acc))");
    let result = interpreter.eval("(do ((i 0 (+ i 1)) (acc 1 (* acc i))) ((>= i 5) acc))")?;
    println!("Result: {:?}", result);
    
    // Test sum computation
    println!("\nTesting: (do ((x 10 (- x 1)) (sum 0 (+ sum x))) ((= x 0) sum))");
    let result = interpreter.eval("(do ((x 10 (- x 1)) (sum 0 (+ sum x))) ((= x 0) sum))")?;
    println!("Result: {:?}", result);
    
    // Test step expression without variable update (should stay constant)
    println!("\nTesting: (do ((x 42)) ((> x 40) x))");
    let result = interpreter.eval("(do ((x 42)) ((> x 40) x))")?;
    println!("Result: {:?}", result);
    
    // Test more complex step expression
    println!("\nTesting: (do ((i 1 (* i 2))) ((> i 10) i))");
    let result = interpreter.eval("(do ((i 1 (* i 2))) ((> i 10) i))")?;
    println!("Result: {:?}", result);
    
    // Test issue that might expose the step problem: factorial should not be 1
    println!("\nTesting factorial: (do ((i 1 (+ i 1)) (fact 1 (* fact i))) ((> i 5) fact))");
    let result = interpreter.eval("(do ((i 1 (+ i 1)) (fact 1 (* fact i))) ((> i 5) fact))")?;
    println!("Result: {:?} (Expected: 120 for 5!)");
    
    Ok(())
}