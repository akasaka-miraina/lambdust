use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

fn main() {
    let mut evaluator = Evaluator::new();
    
    // Simple test: create lambda and call it
    let lambda_expr = Expr::List(vec\![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec\![
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]),
        Expr::List(vec\![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]),
    ]);
    
    // Create lambda
    let lambda_proc = evaluator.eval(
        lambda_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    ).unwrap();
    
    println\!("Lambda created: {:?}", lambda_proc);
    
    // Apply lambda with arguments
    let apply_expr = Expr::List(vec\![
        Expr::Variable("apply".to_string()),
        lambda_proc.to_expr(),
        Expr::List(vec\![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
        ]),
    ]);
    
    let result = evaluator.eval(
        apply_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    println\!("Result: {:?}", result);
}
EOF < /dev/null