use lambdust::environment::Environment;
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::value::{Procedure, Value};
use std::mem;

fn main() {
    println!("lambdust Memory Layout Analysis");
    println!("===============================");

    // 基本型のサイズ分析
    println!("Basic type sizes:");
    println!("  Value:           {} bytes", mem::size_of::<Value>());
    println!(
        "  Continuation:    {} bytes",
        mem::size_of::<Continuation>()
    );
    println!("  Procedure:       {} bytes", mem::size_of::<Procedure>());
    println!("  Environment:     {} bytes", mem::size_of::<Environment>());
    println!(
        "  Box<Continuation>: {} bytes",
        mem::size_of::<Box<Continuation>>()
    );
    println!(
        "  Rc<Environment>: {} bytes",
        mem::size_of::<std::rc::Rc<Environment>>()
    );

    // Valueのvariantサイズ分析
    println!("\nValue variant analysis:");
    let boolean_val = Value::Boolean(true);
    let number_val = Value::Number(lambdust::lexer::SchemeNumber::Integer(42));
    let string_val = Value::String("hello".to_string());
    let nil_val = Value::Nil;

    println!(
        "  Boolean:         {} bytes (stack)",
        mem::size_of_val(&boolean_val)
    );
    println!(
        "  Number:          {} bytes (stack)",
        mem::size_of_val(&number_val)
    );
    println!(
        "  String:          {} bytes (stack)",
        mem::size_of_val(&string_val)
    );
    println!(
        "  Nil:             {} bytes (stack)",
        mem::size_of_val(&nil_val)
    );

    // Continuationのvariantサイズ分析
    println!("\nContinuation variant analysis:");
    let identity_cont = Continuation::Identity;
    println!(
        "  Identity:        {} bytes",
        mem::size_of_val(&identity_cont)
    );

    // Box allocations
    println!("\nHeap allocation sizes:");
    println!(
        "  Box<Continuation>: {} bytes per allocation",
        mem::size_of::<Continuation>()
    );

    // 実際のEvaluatorでのメモリ使用量
    println!("\nEvaluator analysis:");
    let evaluator = Evaluator::new();
    println!("  Evaluator:       {} bytes", mem::size_of_val(&evaluator));

    // 深い継続チェーンのシミュレーション
    println!("\nContinuation chain memory usage simulation:");
    let mut total_size = 0;

    // Identity継続から始まる
    let mut current_cont = Continuation::Identity;
    total_size += mem::size_of_val(&current_cont);

    // 10層の深いチェーンを作成
    for i in 0..10 {
        current_cont = Continuation::Application {
            operator: Value::Nil,
            evaluated_args: vec![],
            remaining_args: vec![],
            env: std::rc::Rc::new(Environment::new()),
            parent: Box::new(current_cont),
        };
        total_size += mem::size_of_val(&current_cont);
        println!("  Depth {}: cumulative {} bytes", i + 1, total_size);
    }

    println!("\nMemory hotspot analysis:");
    println!("  Average continuation: ~{} bytes", total_size / 11);
    println!(
        "  Box overhead per level: ~{} bytes",
        mem::size_of::<Box<Continuation>>()
    );
    println!(
        "  Environment sharing: Rc<Environment> = {} bytes",
        mem::size_of::<std::rc::Rc<Environment>>()
    );

    // パフォーマンス影響の推定
    println!("\nPerformance impact estimation:");
    println!("  Stack frame vs Box allocation: ~10-100x slower"); // Box allocation is typically much slower than stack allocation
    println!("  Rc cloning cost: ~2-5 CPU cycles per clone");
    println!("  Deep continuation chain: O(n) memory, O(n) allocation time");

    // 最適化案
    println!("\nOptimization opportunities:");
    println!("  1. Stack-allocated continuations for tail calls");
    println!("  2. Environment copy-on-write optimization");
    println!("  3. Continuation pooling/reuse");
    println!("  4. Inline small continuations");
    println!("  5. Reduce enum size with Box<> for large variants");
}
