use lambdust::Lambdust;

fn main() {
    println!("=== Lambdust Basic Functionality Verification ===");

    let mut lambdust = Lambdust::new();
    
    let test_cases = vec![
        ("42", "Number literal"),
        ("\"hello world\"", "String literal"),
        ("#t", "Boolean true"),
        ("#f", "Boolean false"),
        ("(+ 1 2)", "Simple addition"),
        ("(* 6 7)", "Simple multiplication"),
        ("(- 10 3)", "Simple subtraction"),
        ("(/ 12 3)", "Simple division"),
        ("(if #t 100 200)", "Conditional true branch"),
        ("(if #f 100 200)", "Conditional false branch"),
        ("(quote hello)", "Quote form"),
        ("'world", "Quote shorthand"),
        ("(list 1 2 3)", "List construction"),
        ("(cons 1 2)", "Pair construction"),
        ("(car (cons 1 2))", "Car operation"),
        ("(cdr (cons 1 2))", "Cdr operation"),
    ];
    
    let mut passed = 0;
    let total = test_cases.len();
    
    for (i, (code, description)) in test_cases.iter().enumerate() {
        print!("{:2}. Testing {:<25}", i + 1, description);
        match lambdust.eval(code, Some("test")) {
            Ok(result) => {
                println!(" ✓ -> {:?}", result);
                passed += 1;
            }
            Err(e) => {
                println!(" ✗ -> Error: {:?}", e);
            }
        }
    }
    
    println!("\n=== Summary ===");
    println!("Passed: {}/{} tests", passed, total);
    if passed == total {
        println!("🎉 All tests passed! Core functionality is working.");
    } else {
        println!("⚠️  Some tests failed. Core functionality needs attention.");
    }
}