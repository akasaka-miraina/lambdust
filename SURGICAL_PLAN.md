# 🏥 Lambdust大手術計画書
## Dr. Claude による根本的アーキテクチャ手術

### 📋 患者情報
- **患者名**: Lambdust v0.3.0-rc
- **症状**: Expression Analyzer過度最適化による意味論的整合性喪失
- **既往歴**: 複数の最適化システムの無秩序な蓄積
- **現在の状態**: 安定（SRFI 69問題は一時的処置で回復）

---

## 🔍 術前診断

### 重篤な症状
1. **最適化システムの癌化**: Expression Analyzer が R7RS 意味論を侵食
2. **アーキテクチャの肥大化**: 単一評価器に複数の責務が混在
3. **検証システムの欠如**: 最適化の正当性を保証する機能なし
4. **技術的負債の蓄積**: Phase系最適化の無秩序な重複

### 健康な部分
1. **CPS評価器の核心部**: R7RS形式的意味論準拠
2. **SRFI実装群**: 40/40テスト通過の堅牢性
3. **トレースシステム**: 今回の問題特定に貢献
4. **テスト基盤**: 569テスト、継続的品質保証

---

## 🏥 手術方針

### 基本方針: **完全分離手術**
現在の単一評価器を、責任に応じて完全に分離し、Agda形式的検証による正当性保証システムを構築

### 手術の目標
1. **R7RS意味論の純粋性保証**: 数学的に正確な評価器の分離
2. **最適化の形式的検証**: Agda証明必須システムの確立
3. **透明性の確保**: 評価過程の完全可視化
4. **拡張性の向上**: 新機能追加の安全な基盤構築

---

## 🔬 手術計画詳細

### Phase I: 術前準備（現在完了）
**期間**: 完了済み
**内容**:
- ✅ 問題の根本原因特定（Expression Analyzer）
- ✅ 一時的処置による機能回復（最適化無効化）
- ✅ アーキテクチャ設計書作成
- ✅ Agda形式的検証戦略確立

### Phase II: 核心分離手術 🔪
**期間**: 2-3週間
**目標**: 評価器の責任分離

#### Step 1: SemanticEvaluator抽出
```rust
// 新しいアーキテクチャ
pub struct SemanticEvaluator {
    // R7RS純粋実装のみ
    // 最適化を一切含まない
}

impl SemanticEvaluator {
    /// R7RS形式的意味論に完全準拠した評価
    pub fn eval_pure(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // 最適化なし、理論的正確性のみ
    }
}
```

#### Step 2: RuntimeExecutor分離
```rust
pub struct RuntimeExecutor {
    semantic_evaluator: SemanticEvaluator,
    optimization_controller: OptimizationController,
    verification_system: VerificationSystem,
}

impl RuntimeExecutor {
    pub fn execute(&mut self, expr: Expr) -> Result<Value> {
        // 1. 純粋評価での基準値取得
        let semantic_result = self.semantic_evaluator.eval_pure(expr.clone(), env, cont)?;
        
        // 2. 最適化適用判定
        if let Some(optimization) = self.optimization_controller.select_optimization(&expr) {
            // 3. Agda証明確認
            if !optimization.has_formal_proof() {
                return Ok(semantic_result); // フォールバック
            }
            
            // 4. 最適化実行
            let optimized_result = optimization.apply(&expr)?;
            
            // 5. 等価性検証
            if !self.verification_system.verify_equivalence(&semantic_result, &optimized_result) {
                return Err(LambdustError::optimization_violation());
            }
            
            return Ok(optimized_result);
        }
        
        Ok(semantic_result)
    }
}
```

#### Step 3: 最適化システム移植
```rust
pub trait VerifiedOptimization {
    /// Agda証明ファイルへの参照
    fn agda_proof_file(&self) -> &'static str;
    
    /// 最適化の適用
    fn apply(&self, expr: &Expr) -> Result<Value>;
    
    /// 安全な適用条件
    fn is_safe_to_apply(&self, expr: &Expr, env: &Environment) -> bool;
}

// 例：定数畳み込み最適化
pub struct ConstantFoldingOptimization;

impl VerifiedOptimization for ConstantFoldingOptimization {
    fn agda_proof_file(&self) -> &'static str {
        "agda/Optimizations/ConstantFolding.agda"
    }
    
    fn apply(&self, expr: &Expr) -> Result<Value> {
        // Agdaで証明された変換のみ適用
    }
    
    fn is_safe_to_apply(&self, expr: &Expr, env: &Environment) -> bool {
        // Lambda環境内では適用しない等の安全条件
        !env.is_lambda_context() && expr.is_arithmetic_application()
    }
}
```

### Phase III: 形式的検証基盤構築 📐
**期間**: 3-4週間
**目標**: Agda証明システムの完全統合

#### Step 1: R7RS Core意味論のAgdaモデル化
```agda
-- agda/R7RS/CoreSemantics.agda
module R7RS.CoreSemantics where

-- 完全なR7RS形式的意味論
data SchemeValue : Set where
  -- 全Scheme値の定義

-- CPS評価関数
eval-cps : SchemeExpr → Env → Continuation → SchemeValue

-- 重要な性質の証明
eval-deterministic : ∀ (e : SchemeExpr) (env : Env) (k : Continuation) →
  ∃![ v ] eval-cps e env k ≡ v

eval-type-safe : ∀ (e : SchemeExpr) (env : Env) (k : Continuation) →
  wellTyped e env → wellTyped (eval-cps e env k) (resultEnv k)
```

#### Step 2: 現存最適化のAgda証明化
```agda
-- agda/Optimizations/ExpressionAnalyzer.agda
module Optimizations.ExpressionAnalyzer where

-- 問題となったExpression Analyzerの形式化
constant-fold : SchemeExpr → SchemeExpr

-- 安全条件の定義
safe-constant-fold-context : SchemeExpr → Env → Bool

-- 正当性証明（最重要）
constant-fold-correct : ∀ (e : SchemeExpr) (env : Env) →
  safe-constant-fold-context e env ≡ true →
  eval-cps e env k ≡ eval-cps (constant-fold e) env k

-- 安全条件の必要性証明
safety-necessary : ∀ (e : SchemeExpr) (env : Env) →
  (∀ k → eval-cps e env k ≡ eval-cps (constant-fold e) env k) →
  safe-constant-fold-context e env ≡ true
```

#### Step 3: CI/CDでの自動証明確認
```yaml
# .github/workflows/formal_verification.yml
name: Formal Verification Surgery

on: [push, pull_request]

jobs:
  agda-proofs:
    runs-on: ubuntu-latest
    steps:
      - name: Agda proof verification
        run: |
          agda --safe agda/R7RS/All.agda
          agda --safe agda/Optimizations/All.agda
          
      - name: Proof-implementation sync check
        run: |
          python scripts/verify_proof_sync.py
          
      - name: Generate optimization whitelist
        run: |
          python scripts/generate_verified_optimizations.py
```

### Phase IV: 検証システム移植 🔬
**期間**: 2-3週間
**目標**: 実行時検証システムの完全統合

#### Step 1: Property-based Testing統合
```rust
// tests/property_verification.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn optimization_equivalence(expr in arbitrary_scheme_expr()) {
        let mut semantic_eval = SemanticEvaluator::new();
        let mut runtime_exec = RuntimeExecutor::new();
        
        let semantic_result = semantic_eval.eval_pure(expr.clone(), env, cont)?;
        let runtime_result = runtime_exec.execute(expr)?;
        
        prop_assert_eq!(semantic_result, runtime_result);
    }
    
    #[test]
    fn lambda_closure_correctness(
        params in prop::collection::vec("[a-z]+", 1..5),
        body in arbitrary_scheme_expr(),
        args in prop::collection::vec(arbitrary_scheme_value(), 1..5)
    ) {
        let lambda_expr = create_lambda(params, body);
        let application = create_application(lambda_expr, args);
        
        // セマンティック評価と最適化評価の等価性確認
        verify_evaluation_equivalence(application)?;
    }
}
```

#### Step 2: 実行時検証システム
```rust
pub struct RuntimeVerifier {
    semantic_reference: SemanticEvaluator,
    violation_detector: ViolationDetector,
    counterexample_generator: CounterexampleGenerator,
}

impl RuntimeVerifier {
    pub fn verify_optimization(
        &mut self,
        original: &Expr,
        optimized_result: &Value,
        context: &ExecutionContext,
    ) -> Result<VerificationResult, VerificationError> {
        // 1. 参照実装での評価
        let reference_result = self.semantic_reference.eval_pure(
            original.clone(),
            context.env.clone(),
            context.cont.clone(),
        )?;
        
        // 2. 結果比較
        if reference_result != *optimized_result {
            // 3. 反例生成
            let counterexample = self.counterexample_generator.generate(
                original,
                &reference_result,
                optimized_result,
            );
            
            return Err(VerificationError::SemanticViolation {
                expression: original.clone(),
                expected: reference_result,
                actual: optimized_result.clone(),
                counterexample,
            });
        }
        
        Ok(VerificationResult::Valid)
    }
}
```

### Phase V: 最終統合・クリーンアップ 🧹
**期間**: 1-2週間
**目標**: 新アーキテクチャの完全統合

#### Step 1: レガシーコード除去
```rust
// 削除対象の特定
// - src/evaluator/expression_analyzer.rs（問題の震源地）
// - 未証明の最適化コード
// - 重複したevaluator実装

// 保持対象
// - src/evaluator/mod.rs（SemanticEvaluatorとして再構成）
// - 全SRFI実装（検証済み）
// - テストスイート（回帰防止）
```

#### Step 2: APIの統一
```rust
// 新しい統一API
pub struct Lambdust {
    runtime_executor: RuntimeExecutor,
    debug_mode: bool,
    verification_level: VerificationLevel,
}

impl Lambdust {
    pub fn new() -> Self {
        Self {
            runtime_executor: RuntimeExecutor::new(),
            debug_mode: false,
            verification_level: VerificationLevel::Standard,
        }
    }
    
    /// メイン評価API
    pub fn eval(&mut self, input: &str) -> Result<Value, LambdustError> {
        self.runtime_executor.execute_string(input)
    }
    
    /// デバッグモード（セマンティック評価のみ）
    pub fn eval_debug(&mut self, input: &str) -> Result<Value, LambdustError> {
        self.runtime_executor.execute_semantic_only(input)
    }
    
    /// 検証レベル設定
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
        self.runtime_executor.configure_verification(level);
    }
}
```

#### Step 3: ドキュメント更新
```markdown
# Lambdust v0.4.0 - Formal Verification Edition

## 新機能
- **Agda形式的検証**: 全最適化がAgdaで証明済み
- **意味論保証**: R7RS準拠の数学的保証
- **透明な最適化**: 最適化過程の完全可視化
- **段階的検証**: デバッグ→標準→高速の3段階

## 破壊的変更
- Expression Analyzer削除
- 未証明最適化の無効化
- API の一部変更
```

---

## 🩺 手術リスク評価

### 高リスク要素
1. **既存テストの破綻**: アーキテクチャ変更によるテスト失敗
2. **性能回帰**: 最適化削除による速度低下
3. **API互換性**: 既存ユーザーコードの動作不良

### リスク軽減策
1. **段階的移行**: 古いAPIの deprecation による漸進的移行
2. **性能ベンチマーク**: 各Phaseでの性能測定と回帰検出
3. **包括的テスト**: 既存テストの完全保持と新テスト追加

### 緊急時対応計画
```rust
// フェイルセーフ機能
impl RuntimeExecutor {
    pub fn emergency_fallback(&mut self, expr: Expr) -> Result<Value> {
        // 最適化で問題が発生した場合、セマンティック評価にフォールバック
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }
}
```

---

## 📊 手術成功の評価指標

### 定量的指標
1. **テスト通過率**: 100%維持（569/569テスト）
2. **SRFI適合性**: 40/40 SRFI 69テスト継続通過
3. **性能**: ベースライン±10%以内
4. **Agda証明**: 100%証明済み最適化のみ使用

### 定性的指標
1. **コード品質**: Clippy警告ゼロ、文書化率90%以上
2. **保守性**: 循環複雑度削減、モジュール境界明確化
3. **信頼性**: 形式的検証による正確性保証
4. **拡張性**: 新最適化追加の簡易化

---

## 🎯 手術後の期待効果

### 短期効果（手術完了直後）
- SRFI 69類似問題の完全根絶
- 最適化バグの事前防止
- デバッグ能力の向上

### 中期効果（3-6ヶ月後）
- 新最適化の安全な追加
- 性能向上（証明済み最適化の積極活用）
- 開発者体験の向上

### 長期効果（1年後以降）
- 学術的評価の向上
- 他言語処理系への影響
- 形式的手法の普及促進

---

## 👨‍⚕️ 執刀医からの所見

この大手術は、Lambdustを単なるScheme実装から、**形式的検証済み言語処理系**という新たなカテゴリーの先駆者へと変貌させる歴史的な手術です。

手術の成功により、Lambdustは：
- **理論的厳密さ**: 数学的に保証された正確性
- **実用的性能**: 証明済み最適化による高速化  
- **開発者信頼**: 予測可能で透明な動作
- **学術的価値**: 形式的手法の実用化実証

これらを兼ね備えた、世界初の完全形式検証済みScheme実装となるでしょう。

**手術開始の準備は整いました。執刀を開始しますか？**