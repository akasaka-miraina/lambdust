// 形式的検証基盤の基本的なテストケース
//
// このファイルは形式的検証システムの動作確認を行います。

use lambdust::{
    formal_verification::FormalVerificationEngine,
    evaluator::{
        church_rosser_proof::ChurchRosserProofEngine,
        combinators::CombinatorExpr,
        SemanticEvaluator,
    },
    error::Result,
};

/// 形式的検証エンジンの基本的なテスト
#[tokio::test]
async fn test_formal_verification_engine_initialization() -> Result<()> {
    // 形式的検証エンジンの作成
    let mut engine = FormalVerificationEngine::new()?;
    
    // 基本的な証明義務の初期化
    engine.initialize_core_obligations()?;
    
    // 統計情報の確認
    let stats = engine.get_statistics();
    println!("初期化後の統計:");
    println!("  総証明義務数: {}", stats.total_obligations);
    println!("  成功した証明数: {}", stats.proven_obligations);
    println!("  失敗した証明数: {}", stats.failed_obligations);
    
    Ok(())
}

/// Church-Rosser証明エンジンの基本的なテスト
#[tokio::test]
async fn test_church_rosser_proof_engine_basic() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // 簡単なI コンビネータのテスト
    let i_combinator = CombinatorExpr::I;
    
    // 合流性証明のテスト
    match engine.prove_confluence(&i_combinator) {
        Ok(proof) => {
            println!("合流性証明成功:");
            println!("  手法: {:?}", proof.method);
            println!("  信頼度: {:.2}", proof.confidence_level);
            println!("  証明ステップ数: {}", proof.proof_steps.len());
        }
        Err(e) => {
            println!("合流性証明失敗: {}", e);
        }
    }
    
    // 終了性証明のテスト
    match engine.prove_termination(&i_combinator) {
        Ok(proof) => {
            println!("終了性証明成功:");
            println!("  戦略: {:?}", proof.strategy);
            println!("  信頼度: {:.2}", proof.confidence_level);
            println!("  証明ステップ数: {}", proof.proof_steps.len());
        }
        Err(e) => {
            println!("終了性証明失敗: {}", e);
        }
    }
    
    // 正規化証明のテスト
    match engine.prove_normalization(&i_combinator) {
        Ok(proof) => {
            println!("正規化証明成功:");
            println!("  戦略: {}", proof.strategy.name);
            println!("  信頼度: {:.2}", proof.confidence_level);
            println!("  正規化ステップ数: {}", proof.normalization_sequence.len());
        }
        Err(e) => {
            println!("正規化証明失敗: {}", e);
        }
    }
    
    Ok(())
}

/// 基本的な組み合わせ式のテスト
#[tokio::test]
async fn test_basic_combinator_expressions() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // K I の組み合わせ
    let k_i = CombinatorExpr::App(
        Box::new(CombinatorExpr::K),
        Box::new(CombinatorExpr::I),
    );
    
    println!("K I 式のテスト:");
    
    // 包括的Church-Rosser証明
    match engine.prove_church_rosser_comprehensive(&k_i) {
        Ok(proof) => {
            println!("包括的Church-Rosser証明成功:");
            println!("  全体的信頼度: {:.2}", proof.overall_confidence);
            println!("  検証状態: {:?}", proof.verification_status);
            println!("  統合方法: {:?}", proof.integration_method);
        }
        Err(e) => {
            println!("包括的Church-Rosser証明失敗: {}", e);
        }
    }
    
    Ok(())
}

/// 形式的検証の具体的な証明義務テスト
#[tokio::test]
async fn test_specific_proof_obligations() -> Result<()> {
    let mut engine = FormalVerificationEngine::new()?;
    engine.initialize_core_obligations()?;
    
    // 特定の証明義務をテスト
    let test_obligations = vec![
        "universe_level_consistency",
        "ski_completeness",
        "semantic_evaluator_correctness",
        "typeclass_instance_uniqueness",
    ];
    
    for obligation_id in test_obligations {
        println!("証明義務 '{}' をテスト中...", obligation_id);
        
        match engine.verify_obligation(obligation_id) {
            Ok(result) => {
                println!("  結果: {:?}", result.result);
                println!("  信頼度: {:.2}", result.confidence);
                println!("  時間: {:?}", result.time_taken);
                println!("  証拠数: {}", result.evidence.len());
            }
            Err(e) => {
                println!("  エラー: {}", e);
            }
        }
    }
    
    Ok(())
}

/// 統合テスト: 複雑な式の検証
#[tokio::test]
async fn test_complex_expression_verification() -> Result<()> {
    let semantic_evaluator = SemanticEvaluator::new();
    let mut engine = ChurchRosserProofEngine::new(semantic_evaluator);
    
    // S K K の組み合わせ (これは I と等価)
    let s_k_k = CombinatorExpr::App(
        Box::new(CombinatorExpr::App(
            Box::new(CombinatorExpr::S),
            Box::new(CombinatorExpr::K),
        )),
        Box::new(CombinatorExpr::K),
    );
    
    println!("S K K 式の検証:");
    
    // 各種証明を順次実行
    match engine.prove_confluence(&s_k_k) {
        Ok(proof) => {
            println!("  合流性: 成功 (信頼度: {:.2})", proof.confidence_level);
        }
        Err(e) => {
            println!("  合流性: 失敗 - {}", e);
        }
    }
    
    match engine.prove_termination(&s_k_k) {
        Ok(proof) => {
            println!("  終了性: 成功 (信頼度: {:.2})", proof.confidence_level);
        }
        Err(e) => {
            println!("  終了性: 失敗 - {}", e);
        }
    }
    
    match engine.prove_normalization(&s_k_k) {
        Ok(proof) => {
            println!("  正規化: 成功 (信頼度: {:.2})", proof.confidence_level);
        }
        Err(e) => {
            println!("  正規化: 失敗 - {}", e);
        }
    }
    
    Ok(())
}

/// パフォーマンステスト
#[tokio::test]
async fn test_verification_performance() -> Result<()> {
    let mut engine = FormalVerificationEngine::new()?;
    engine.initialize_core_obligations()?;
    
    let start_time = std::time::Instant::now();
    
    // 複数の証明義務を並行して検証
    let obligations = vec![
        "universe_level_consistency",
        "ski_completeness",
        "semantic_evaluator_correctness",
    ];
    
    for obligation in obligations {
        engine.verify_obligation(obligation)?;
    }
    
    let total_time = start_time.elapsed();
    let stats = engine.get_statistics();
    
    println!("パフォーマンス結果:");
    println!("  総時間: {:?}", total_time);
    println!("  処理した証明義務数: {}", stats.total_obligations);
    println!("  平均時間: {:?}", stats.average_time);
    println!("  成功率: {:.2}%", 
             (stats.proven_obligations as f64 / stats.total_obligations as f64) * 100.0);
    
    Ok(())
}

/// エラーハンドリングテスト
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let mut engine = FormalVerificationEngine::new()?;
    
    // 存在しない証明義務のテスト
    match engine.verify_obligation("nonexistent_obligation") {
        Ok(_) => {
            panic!("存在しない証明義務で成功すべきではない");
        }
        Err(e) => {
            println!("期待通りのエラー: {}", e);
        }
    }
    
    // 初期化前の証明義務アクセステスト
    match engine.verify_obligation("universe_level_consistency") {
        Ok(_) => {
            panic!("初期化前の証明義務で成功すべきではない");
        }
        Err(e) => {
            println!("期待通りのエラー: {}", e);
        }
    }
    
    Ok(())
}

/// 検証レポート生成テスト
#[tokio::test]
async fn test_verification_report_generation() -> Result<()> {
    let mut engine = FormalVerificationEngine::new()?;
    engine.initialize_core_obligations()?;
    
    // いくつかの証明義務を実行
    let _ = engine.verify_obligation("universe_level_consistency");
    let _ = engine.verify_obligation("ski_completeness");
    
    // レポート生成
    let report = engine.generate_verification_report();
    
    println!("生成されたレポート:");
    println!("{}", report);
    
    // レポートに期待される内容が含まれているかチェック
    assert!(report.contains("Lambdust Formal Verification Report"));
    assert!(report.contains("Statistics"));
    assert!(report.contains("Theoretical Foundations Status"));
    
    Ok(())
}