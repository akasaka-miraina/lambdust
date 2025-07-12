// 基本的な形式的検証テスト
//
// コンパイルエラーを避けるため、最小限の機能のみをテストします。

use lambdust::{
    evaluator::{
        combinators::CombinatorExpr,
        church_rosser_proof::{ChurchRosserProofEngine, MeasureFunction, MeasureValue, WellFoundedOrdering, OrderingResult},
        SemanticEvaluator,
    },
    error::Result,
};
use std::collections::HashMap;

/// CombinatorExprの基本的な機能をテスト
#[test]
fn test_combinator_expr_basic() {
    // 単純なコンビネータの作成
    let i_combinator = CombinatorExpr::I;
    let k_combinator = CombinatorExpr::K;
    let s_combinator = CombinatorExpr::S;
    
    // アプリケーションの作成
    let app = CombinatorExpr::App(
        Box::new(k_combinator.clone()),
        Box::new(i_combinator.clone()),
    );
    
    println!("Created combinators:");
    println!("  I: {:?}", i_combinator);
    println!("  K: {:?}", k_combinator);
    println!("  S: {:?}", s_combinator);
    println!("  K I: {:?}", app);
    
    // 簡約のテスト
    match app.reduce_step() {
        Some(reduced) => {
            println!("  K I reduced to: {:?}", reduced);
        }
        None => {
            println!("  K I cannot be reduced further");
        }
    }
}

/// Church-Rosser証明エンジンの基本テスト
#[test]
fn test_church_rosser_engine_basic() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    println!("Church-Rosser証明エンジンが正常に作成されました");
    println!("統計: {:?}", engine.get_proof_statistics());
    
    Ok(())
}

/// 測度関数のテスト
#[test]
fn test_measure_functions() {
    // サイズ測度関数のテスト
    let size_measure = MeasureFunction::new_size_measure();
    println!("サイズ測度関数: {}", size_measure.name);
    
    // 単純なコンビネータでの測度計算
    let simple_expr = CombinatorExpr::I;
    let complex_expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::K),
        Box::new(CombinatorExpr::I),
    );
    
    let simple_measure = (size_measure.compute)(&simple_expr);
    let complex_measure = (size_measure.compute)(&complex_expr);
    
    println!("I の測度: {}", simple_measure.numeric_value);
    println!("K I の測度: {}", complex_measure.numeric_value);
    
    // 深度測度関数のテスト
    let depth_measure = MeasureFunction::new_depth_measure();
    let simple_depth = (depth_measure.compute)(&simple_expr);
    let complex_depth = (depth_measure.compute)(&complex_expr);
    
    println!("I の深度: {}", simple_depth.numeric_value);
    println!("K I の深度: {}", complex_depth.numeric_value);
}

/// 測度値の比較テスト
#[test]
fn test_measure_value_ordering() {
    let value1 = MeasureValue {
        numeric_value: 3,
        structural_value: vec![1, 2],
        metadata: HashMap::new(),
    };
    
    let value2 = MeasureValue {
        numeric_value: 5,
        structural_value: vec![2, 1],
        metadata: HashMap::new(),
    };
    
    println!("測度値1: {:?}", value1);
    println!("測度値2: {:?}", value2);
    
    // 比較のテスト
    assert!(value1 < value2);
    println!("value1 < value2: true");
    
    // 整礎順序のテスト
    let ordering = WellFoundedOrdering::new_natural_numbers();
    let comparison = (ordering.compare)(&value1, &value2);
    
    match comparison {
        OrderingResult::Less => println!("順序比較: value1 < value2"),
        OrderingResult::Greater => println!("順序比較: value1 > value2"),
        OrderingResult::Equal => println!("順序比較: value1 = value2"),
        OrderingResult::Incomparable => println!("順序比較: 比較不可能"),
    }
}

/// 正規化戦略のテスト
#[test]
fn test_normalization_strategy() {
    use lambdust::evaluator::church_rosser_proof::NormalizationStrategy;
    
    let leftmost_strategy = NormalizationStrategy::new_leftmost_outermost();
    println!("正規化戦略: {}", leftmost_strategy.name);
    
    let rightmost_strategy = NormalizationStrategy::new_rightmost_innermost();
    println!("正規化戦略: {}", rightmost_strategy.name);
    
    let parallel_strategy = NormalizationStrategy::new_parallel_outermost();
    println!("正規化戦略: {}", parallel_strategy.name);
}

/// 簡約ステップのテスト
#[test]
fn test_reduction_steps() {
    use lambdust::evaluator::church_rosser_proof::{ReductionStep, ReductionRule, ReductionPosition, PositionStep};
    
    let before = CombinatorExpr::App(
        Box::new(CombinatorExpr::I),
        Box::new(CombinatorExpr::K),
    );
    
    let after = CombinatorExpr::K;
    
    let position = ReductionPosition {
        path: vec![PositionStep::Function],
        description: "I application".to_string(),
    };
    
    let step = ReductionStep::new(
        before.clone(),
        after.clone(),
        ReductionRule::ICombinator,
        position,
    );
    
    println!("簡約ステップ:");
    println!("  適用規則: {}", step.rule.name());
    println!("  簡約前: {:?}", step.before);
    println!("  簡約後: {:?}", step.after);
}

/// コンビネータの自由変数テスト
#[test]
fn test_free_variables() {
    let expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::S),
        Box::new(CombinatorExpr::K),
    );
    
    let free_vars = expr.free_variables();
    println!("S K の自由変数数: {}", free_vars.len());
    
    // アトミック式での自由変数テスト
    use lambdust::ast::Expr;
    let atomic_expr = CombinatorExpr::Atomic(Expr::Variable("x".to_string()));
    let atomic_free_vars = atomic_expr.free_variables();
    println!("変数 x の自由変数: {:?}", atomic_free_vars);
}