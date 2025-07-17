use lambdust::environment::Environment;
use lambdust::error::Result;
use lambdust::evaluator::eval_with_formal_semantics;
use lambdust::lexer::tokenize;
use lambdust::parser::parse;
use lambdust::value::Value;
use lambdust::debug::DebugTracer;
use std::rc::Rc;

fn eval_str(input: &str) -> Result<Value> {
    let tokens = tokenize(input)?;
    let ast = parse(tokens)?;
    let env = Rc::new(Environment::with_builtins());
    eval_with_formal_semantics(ast, env)
}

fn main() {
    println!("=== Debug Trace Test ===\n");

    // Clear any previous traces
    DebugTracer::clear_trace_log();

    // Test the problematic case
    println!("Testing: (begin (define y 100) y)");
    let result = eval_str("(begin (define y 100) y)");
    match result {
        Ok(value) => println!("Result: {:?}\n", value),
        Err(e) => println!("Error: {:?}\n", e),
    }

    // Dump trace to file
    match DebugTracer::dump_trace_to_file("debug_trace.log") {
        Ok(()) => println!("Trace log written to debug_trace.log"),
        Err(e) => println!("Failed to write trace log: {:?}", e),
    }

    // Print summary of trace entries
    let trace_log = DebugTracer::get_trace_log();
    println!("Total trace entries: {}", trace_log.len());
    
    // Show first 10 entries
    for (i, entry) in trace_log.iter().take(10).enumerate() {
        println!("[{}] {}::{}:{} [{}] {}", 
            entry.step_id, entry.module, entry.method, entry.line,
            entry.level.as_str(), entry.message);
    }
    
    if trace_log.len() > 10 {
        println!("... and {} more entries", trace_log.len() - 10);
    }
}