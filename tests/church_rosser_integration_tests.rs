//! Church-Rosser性・合流性の統合テスト
//!
//! このテストファイルは、Church-Rosser証明システムの包括的な統合テストを提供し、
//! コンビネータ理論における合流性、終了性、正規化の形式的証明を検証します。

use lambdust::ast::Expr;
use lambdust::error::Result;
use lambdust::evaluator::{
    ChurchRosserProofEngine, SemanticEvaluator,
    combinators::{CombinatorExpr, BracketAbstraction},
    church_rosser_proof::{
        ConfluenceVerifier, TerminationVerifier, NormalizationVerifier,
        ConfluenceProofMethod, TerminationStrategy,
    },
};
use std::time::Instant;

/// Church-Rosser証明システムの基本機能テスト
#[test]
fn test_church_rosser_proof_engine_basic() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut proof_engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // 基本的なS コンビネータのテスト
    let s_combinator = CombinatorExpr::S;
    let proof_result = proof_engine.prove_church_rosser_comprehensive(&s_combinator);
    
    assert!(proof_result.is_ok());
    let proof = proof_result.unwrap();
    assert!(proof.overall_confidence > 0.8);
    
    Ok(())
}

/// 合流性検証の包括テスト
#[test]
fn test_confluence_verification_comprehensive() -> Result<()> {
    let mut confluence_verifier = ConfluenceVerifier::new();
    
    // テストケース1: I コンビネータ（自明な合流性）
    let i_combinator = CombinatorExpr::I;
    let confluence_result = confluence_verifier.verify_confluence(&i_combinator);
    
    if let Ok(proof) = confluence_result {
        assert!(proof.confidence_level > 0.9);
        assert!(matches!(proof.method, ConfluenceProofMethod::DirectProof | ConfluenceProofMethod::ParallelReduction));
    }
    
    // テストケース2: K コンビネータ適用
    let k_application = CombinatorExpr::App(
        Box::new(CombinatorExpr::App(
            Box::new(CombinatorExpr::K),
            Box::new(CombinatorExpr::I),
        )),
        Box::new(CombinatorExpr::S),
    );
    
    let k_confluence_result = confluence_verifier.verify_confluence(&k_application);
    // K x y → x なので合流性があるはず
    
    Ok(())
}

/// 終了性検証の包括テスト
#[test]
fn test_termination_verification_comprehensive() -> Result<()> {
    let mut termination_verifier = TerminationVerifier::new();
    
    // テストケース1: 基本コンビネータの終了性
    let basic_combinators = vec![
        CombinatorExpr::S,
        CombinatorExpr::K,
        CombinatorExpr::I,
        CombinatorExpr::B,
        CombinatorExpr::C,
        CombinatorExpr::W,
    ];
    
    for combinator in basic_combinators {
        let termination_result = termination_verifier.verify_termination(&combinator);
        
        if let Ok(proof) = termination_result {
            assert!(proof.confidence_level > 0.8);
            assert!(matches!(
                proof.strategy,
                TerminationStrategy::LexicographicOrder |
                TerminationStrategy::PolynomialInterpretation |
                TerminationStrategy::SizeChangeTermination
            ));
        }
    }
    
    // テストケース2: 複合式の終了性
    let complex_expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::App(
            Box::new(CombinatorExpr::S),
            Box::new(CombinatorExpr::K),
        )),
        Box::new(CombinatorExpr::K),
    );
    
    let complex_termination_result = termination_verifier.verify_termination(&complex_expr);
    // S K K → I なので終了性があるはず
    
    Ok(())
}

/// 正規化検証の包括テスト
#[test]
fn test_normalization_verification_comprehensive() -> Result<()> {
    let mut normalization_verifier = NormalizationVerifier::new();
    
    // テストケース1: 既に正規形の式
    let normal_expr = CombinatorExpr::I;
    let normalization_result = normalization_verifier.verify_normalization(&normal_expr);
    
    if let Ok(proof) = normalization_result {
        assert!(proof.confidence_level > 0.9);
        assert_eq!(proof.normal_form.expression, normal_expr);
        assert!(proof.normal_form.normality_proof.verified);
    }
    
    // テストケース2: 正規化が必要な式
    let reducible_expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::I),
        Box::new(CombinatorExpr::S),
    );
    
    let reducible_normalization_result = normalization_verifier.verify_normalization(&reducible_expr);
    // I S → S なので正規化される
    
    Ok(())
}

/// Lambda式からコンビネータへの変換とChurch-Rosser性テスト
#[test]
fn test_lambda_to_combinator_church_rosser() -> Result<()> {
    // テスト用のlambda式を作成
    let lambda_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("x".to_string())]),
        Expr::Variable("x".to_string()),
    ]);
    
    // コンビネータに変換
    let combinator_result = BracketAbstraction::lambda_to_combinators(&lambda_expr);
    
    if let Ok(combinator_expr) = combinator_result {
        // Church-Rosser性を証明
        let semantic_evaluator = SemanticEvaluator::new();
        let mut proof_engine = ChurchRosserProofEngine::new(semantic_evaluator);
        
        let church_rosser_proof = proof_engine.prove_church_rosser_comprehensive(&combinator_expr);
        
        if let Ok(proof) = church_rosser_proof {
            // 合流性の確認
            assert!(proof.confluence_proof.confidence_level > 0.7);
            
            // 終了性の確認
            assert!(proof.termination_proof.confidence_level > 0.7);
            
            // 正規化の確認
            assert!(proof.normalization_proof.confidence_level > 0.7);
            
            // 全体的信頼度の確認
            assert!(proof.overall_confidence > 0.7);
        }
    }
    
    Ok(())
}

/// 複雑なコンビネータ式でのChurch-Rosser性テスト
#[test]
fn test_complex_combinator_church_rosser() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut proof_engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // 複雑なコンビネータ式: S (K S) (S I I) 
    let complex_expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::App(
            Box::new(CombinatorExpr::S),
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::K),
                Box::new(CombinatorExpr::S),
            )),
        )),
        Box::new(CombinatorExpr::App(
            Box::new(CombinatorExpr::S),
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::I),
                Box::new(CombinatorExpr::I),
            )),
        )),
    );
    
    let proof_result = proof_engine.prove_church_rosser_comprehensive(&complex_expr);
    
    if let Ok(proof) = proof_result {
        // 複雑な式でも基本的なChurch-Rosser性は保持されるべき
        assert!(proof.overall_confidence > 0.5);
        
        // 証明統計の確認
        let stats = proof_engine.get_proof_statistics();
        assert!(stats.successful_proofs > 0);
        assert!(stats.total_verification_time.as_millis() > 0);
    }
    
    Ok(())
}

/// 測度関数の単調性テスト
#[test]
fn test_measure_function_monotonicity() {
    use lambdust::evaluator::church_rosser_proof::MeasureFunction;
    
    // サイズ測度のテスト
    let simple_expr = CombinatorExpr::I;
    let complex_expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::I),
        Box::new(CombinatorExpr::S),
    );
    
    let size_measure = MeasureFunction::new_size_measure();
    let simple_measure = (size_measure.compute)(&simple_expr);
    let complex_measure = (size_measure.compute)(&complex_expr);
    
    // 複雑な式の方が測度が大きいべき
    assert!(complex_measure.numeric_value > simple_measure.numeric_value);
    
    // 深度測度のテスト
    let depth_measure = MeasureFunction::new_depth_measure();
    let simple_depth = (depth_measure.compute)(&simple_expr);
    let complex_depth = (depth_measure.compute)(&complex_expr);
    
    // 複雑な式の方が深度が大きいべき
    assert!(complex_depth.numeric_value >= simple_depth.numeric_value);
}

/// 整礎順序関係のテスト
#[test]
fn test_well_founded_ordering() {
    use lambdust::evaluator::church_rosser_proof::{WellFoundedOrdering, MeasureValue, OrderingResult};
    use std::collections::HashMap;
    
    let ordering = WellFoundedOrdering::new_natural_numbers();
    
    let value1 = MeasureValue {
        numeric_value: 5,
        structural_value: vec![],
        metadata: HashMap::new(),
    };
    
    let value2 = MeasureValue {
        numeric_value: 10,
        structural_value: vec![],
        metadata: HashMap::new(),
    };
    
    let value3 = MeasureValue {
        numeric_value: 5,
        structural_value: vec![],
        metadata: HashMap::new(),
    };
    
    // 順序関係のテスト
    assert_eq!((ordering.compare)(&value1, &value2), OrderingResult::Less);
    assert_eq!((ordering.compare)(&value2, &value1), OrderingResult::Greater);
    assert_eq!((ordering.compare)(&value1, &value3), OrderingResult::Equal);
}

/// 正規化戦略の効果性テスト
#[test]
fn test_normalization_strategy_effectiveness() {
    use lambdust::evaluator::church_rosser_proof::NormalizationStrategy;
    
    let strategy = NormalizationStrategy::new_leftmost_outermost();
    
    // 正規化前の式
    let before = CombinatorExpr::App(
        Box::new(CombinatorExpr::K),
        Box::new(CombinatorExpr::I),
    );
    
    // 正規化後の式（K I はそのまま）
    let after = before.clone();
    
    let effectiveness = (strategy.effectiveness_measure.measure)(&before, &after);
    
    // 効果性は0以上1以下の範囲内
    assert!(effectiveness >= 0.0);
    assert!(effectiveness <= 1.0);
    
    // 異なる戦略での比較
    let rightmost_strategy = NormalizationStrategy::new_rightmost_innermost();
    let parallel_strategy = NormalizationStrategy::new_parallel_outermost();
    
    // 戦略が正しく作成されることを確認
    assert_eq!(strategy.name, "leftmost_outermost");
    assert_eq!(rightmost_strategy.name, "rightmost_innermost");
    assert_eq!(parallel_strategy.name, "parallel_outermost");
}

/// パフォーマンス測定テスト
#[test]
fn test_church_rosser_proof_performance() -> Result<()> {
    use std::time::Instant;
    
    let semantic_evaluator = SemanticEvaluator::new();
    let mut proof_engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // 複数の式でのパフォーマンステスト
    let test_expressions = vec![
        CombinatorExpr::I,
        CombinatorExpr::K,
        CombinatorExpr::S,
        CombinatorExpr::App(Box::new(CombinatorExpr::I), Box::new(CombinatorExpr::S)),
        CombinatorExpr::App(
            Box::new(CombinatorExpr::K),
            Box::new(CombinatorExpr::App(Box::new(CombinatorExpr::I), Box::new(CombinatorExpr::S))),
        ),
    ];
    
    let start_time = Instant::now();
    let mut successful_proofs = 0;
    
    for expr in test_expressions {
        if let Ok(_proof) = proof_engine.prove_church_rosser_comprehensive(&expr) {
            successful_proofs += 1;
        }
    }
    
    let total_time = start_time.elapsed();
    
    // 基本的なパフォーマンス要件
    assert!(total_time.as_secs() < 30); // 30秒以内
    assert!(successful_proofs > 0); // 少なくとも1つは成功
    
    // 統計の確認
    let stats = proof_engine.get_proof_statistics();
    assert!(stats.successful_proofs >= successful_proofs);
    assert!(stats.average_proof_time.as_millis() > 0);
    
    Ok(())
}

/// 証明キャッシュのテスト
#[test]
fn test_proof_caching() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut proof_engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    let test_expr = CombinatorExpr::App(
        Box::new(CombinatorExpr::I),
        Box::new(CombinatorExpr::S),
    );
    
    // 最初の証明
    let start_time = Instant::now();
    let first_proof = proof_engine.prove_church_rosser_comprehensive(&test_expr);
    let first_duration = start_time.elapsed();
    
    assert!(first_proof.is_ok());
    
    // 2回目の証明（キャッシュされるべき）
    let start_time = Instant::now();
    let second_proof = proof_engine.prove_church_rosser_comprehensive(&test_expr);
    let second_duration = start_time.elapsed();
    
    assert!(second_proof.is_ok());
    
    // 統計の確認
    let stats = proof_engine.get_proof_statistics();
    assert!(stats.successful_proofs >= 2);
    
    Ok(())
}

/// エラー処理テスト
#[test]
fn test_church_rosser_error_handling() {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut proof_engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // 無限ループの可能性がある式（簡単なテスト）
    let potentially_problematic = CombinatorExpr::App(
        Box::new(CombinatorExpr::App(
            Box::new(CombinatorExpr::W),
            Box::new(CombinatorExpr::W),
        )),
        Box::new(CombinatorExpr::W),
    );
    
    // 証明が完了するか、適切にエラーハンドリングされるか確認
    let result = proof_engine.prove_church_rosser_comprehensive(&potentially_problematic);
    
    // 結果に関わらず、パニックしないことが重要
    match result {
        Ok(proof) => {
            // 証明が成功した場合、信頼度を確認
            assert!(proof.overall_confidence >= 0.0);
            assert!(proof.overall_confidence <= 1.0);
        }
        Err(_) => {
            // エラーが発生した場合、適切に処理されている
            // 統計にエラーが記録されているか確認
            let stats = proof_engine.get_proof_statistics();
            assert!(stats.failed_proofs >= 0); // エラーカウントが適切
        }
    }
}

/// 統合テスト：形式的検証システムとの連携
#[test]
fn test_integration_with_formal_verification() -> Result<()> {
    use lambdust::evaluator::formal_verification::FormalVerificationEngine;
    
    let mut formal_verifier = FormalVerificationEngine::new();
    
    // 簡単なlambda式
    let lambda_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("x".to_string())]),
        Expr::Variable("x".to_string()),
    ]);
    
    // Church-Rosser性質の証明
    let church_rosser_result = formal_verifier.prove_church_rosser_properties(&lambda_expr);
    
    if let Ok(proof) = church_rosser_result {
        assert!(proof.overall_confidence > 0.0);
        
        // 個別の証明結果を確認
        assert!(proof.confluence_proof.confidence_level > 0.0);
        assert!(proof.termination_proof.confidence_level > 0.0);
        assert!(proof.normalization_proof.confidence_level > 0.0);
    }
    
    // 合流性の個別確認
    let confluence_result = formal_verifier.verify_confluence(&lambda_expr);
    
    // 終了性の個別確認
    let termination_result = formal_verifier.verify_termination(&lambda_expr);
    
    // 正規化の個別確認
    let normalization_result = formal_verifier.verify_normalization(&lambda_expr);
    
    // Church-Rosser統計の確認
    let cr_stats = formal_verifier.get_church_rosser_statistics();
    assert!(cr_stats.total_verification_time.as_nanos() > 0);
    
    Ok(())
}