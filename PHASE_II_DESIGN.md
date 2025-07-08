# 🏥 Phase II: 核心分離手術 - 詳細設計書
## SemanticEvaluator抽出とRuntimeExecutor統合

### 📋 手術前の現状分析

#### 現在の評価器構造（src/evaluator/mod.rs）
```rust
// 現在の統合構造
impl Evaluator {
    pub fn eval(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // ❌ 問題: 最適化ロジックと意味論が混在
        // ✅ 応急処置: Expression Analyzer無効化済み
        
        // 純粋R7RS部分（保持対象）
        match expr {
            Expr::Literal(lit) => self.eval_literal(lit, cont),
            Expr::Variable(name) => self.eval_variable(name, env, cont),
            Expr::List(exprs) => self.eval_application(exprs, env, cont),
        }
    }
}
```

#### 分離すべき最適化モジュール
```
evaluator/
├── expression_analyzer.rs     # ❌ 削除対象（問題の震源地）
├── jit_loop_optimization.rs   # ⚡ 分離対象（最適化）
├── trampoline.rs              # ⚡ 分離対象（最適化）
├── continuation_pooling.rs    # ⚡ 分離対象（最適化）
├── inline_evaluation.rs      # ⚡ 分離対象（最適化）
├── tail_call_optimization.rs # ⚡ 分離対象（最適化）
└── llvm_backend.rs           # ⚡ 分離対象（最適化）
```

---

## 🔬 Step 1: SemanticEvaluator設計

### 基本構造
```rust
/// 純粋なR7RS形式的意味論に準拠した評価器
/// 最適化を一切含まず、理論的正確性のみを追求
pub struct SemanticEvaluator {
    /// 基本環境（R7RS標準ライブラリ）
    global_env: Rc<Environment>,
    
    /// 例外ハンドラーチェーン
    exception_handlers: Vec<ExceptionHandler>,
    
    /// 動的風システム（dynamic-wind）
    dynamic_points: Vec<DynamicPoint>,
    
    /// デバッグ情報（本番では無効化）
    #[cfg(debug_assertions)]
    debug_tracer: DebugTracer,
    
    /// 再帰深度監視（スタックオーバーフロー防止）
    recursion_depth: usize,
    max_recursion_depth: usize,
}
```

### 核心評価メソッド
```rust
impl SemanticEvaluator {
    /// R7RS形式的意味論に完全準拠した評価
    /// E[e]ρκσ - 式e、環境ρ、継続κ、ストアσ
    pub fn eval_pure(
        &mut self, 
        expr: Expr, 
        env: Rc<Environment>, 
        cont: Continuation
    ) -> Result<Value> {
        // スタックオーバーフロー検査
        self.check_recursion_depth()?;
        
        // R7RS形式的意味論の直接実装
        match expr {
            // 定数: E[K]ρκσ = κ(K[K])
            Expr::Literal(lit) => {
                let value = self.literal_to_value(lit)?;
                self.apply_continuation_pure(cont, value)
            }
            
            // 変数: E[I]ρκσ = κ(σ(ρ(I)))
            Expr::Variable(name) => {
                let value = env.get(&name)
                    .ok_or_else(|| LambdustError::undefined_variable(name))?;
                self.apply_continuation_pure(cont, value)
            }
            
            // 関数適用: E[(E0 E1 ...)]ρκσ
            Expr::List(exprs) if !exprs.is_empty() => {
                self.eval_application_pure(exprs, env, cont)
            }
            
            // 空リスト
            Expr::List(_) => {
                self.apply_continuation_pure(cont, Value::List(vec![]))
            }
        }
    }
    
    /// 継続適用（純粋版）
    fn apply_continuation_pure(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        match cont {
            Continuation::Done => Ok(value),
            
            // 特殊形式継続の処理
            Continuation::IfTest { consequent, alternate, env, parent } => {
                if value.is_truthy() {
                    self.eval_pure(consequent, env, *parent)
                } else if let Some(alt) = alternate {
                    self.eval_pure(alt, env, *parent)
                } else {
                    self.apply_continuation_pure(*parent, Value::Undefined)
                }
            }
            
            // その他の継続...
            _ => self.apply_other_continuations_pure(cont, value),
        }
    }
    
    /// 特殊形式の純粋評価
    fn eval_special_form_pure(
        &mut self,
        name: &str,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        match name {
            "lambda" => self.eval_lambda_pure(operands, env, cont),
            "if" => self.eval_if_pure(operands, env, cont),
            "define" => self.eval_define_pure(operands, env, cont),
            "begin" => self.eval_begin_pure(operands, env, cont),
            "set!" => self.eval_set_pure(operands, env, cont),
            // ... 他の特殊形式
            _ => Err(LambdustError::unknown_special_form(name)),
        }
    }
}
```

---

## 🎛️ Step 2: RuntimeExecutor設計

### 統合アーキテクチャ
```rust
/// 実行時最適化と検証を統合する実行器
pub struct RuntimeExecutor {
    /// 純粋R7RS評価器（参照実装）
    semantic_evaluator: SemanticEvaluator,
    
    /// 最適化制御システム
    optimization_controller: OptimizationController,
    
    /// 実行時検証システム
    verification_system: VerificationSystem,
    
    /// デバッグ・プロファイリング
    debug_mode: bool,
    verification_level: VerificationLevel,
}

#[derive(Debug, Clone)]
pub enum VerificationLevel {
    /// デバッグ: 全て純粋評価（最適化なし）
    Debug,
    /// 標準: 証明済み最適化のみ + 実行時検証
    Standard,
    /// 高速: 証明済み最適化 + 軽量検証
    Performance,
    /// 信頼: 証明済み最適化のみ（検証なし）
    Trusted,
}
```

### メイン実行ロジック
```rust
impl RuntimeExecutor {
    /// メイン実行エントリーポイント
    pub fn execute(&mut self, expr: Expr) -> Result<Value> {
        match self.verification_level {
            VerificationLevel::Debug => {
                // デバッグモード: 純粋評価のみ
                self.semantic_evaluator.eval_pure(
                    expr, 
                    self.semantic_evaluator.global_env.clone(), 
                    Continuation::Done
                )
            }
            
            VerificationLevel::Standard => {
                // 標準モード: 最適化 + 検証
                self.execute_with_verification(expr)
            }
            
            VerificationLevel::Performance => {
                // 高速モード: 最適化 + 軽量検証
                self.execute_with_light_verification(expr)
            }
            
            VerificationLevel::Trusted => {
                // 信頼モード: 最適化のみ
                self.execute_optimized(expr)
            }
        }
    }
    
    /// 検証付き実行
    fn execute_with_verification(&mut self, expr: Expr) -> Result<Value> {
        // 1. 純粋評価で参照結果を取得
        let reference_result = self.semantic_evaluator.eval_pure(
            expr.clone(),
            self.semantic_evaluator.global_env.clone(),
            Continuation::Done,
        )?;
        
        // 2. 最適化適用判定
        if let Some(optimization) = self.optimization_controller.select_optimization(&expr) {
            // 3. Agda証明確認
            if !optimization.has_formal_proof() {
                return Ok(reference_result); // フォールバック
            }
            
            // 4. 最適化実行
            let optimized_result = optimization.apply(&expr)?;
            
            // 5. 等価性検証
            if !self.verification_system.verify_equivalence(&reference_result, &optimized_result) {
                return Err(LambdustError::optimization_violation(
                    format!("Optimization {} violated semantics", optimization.name())
                ));
            }
            
            return Ok(optimized_result);
        }
        
        Ok(reference_result)
    }
}
```

---

## 🔧 Step 3: 最適化システム再構築

### 検証可能最適化トレイト
```rust
/// Agda証明付き最適化の統一インターフェース
pub trait VerifiedOptimization {
    /// 最適化名
    fn name(&self) -> &'static str;
    
    /// Agda証明ファイルへの参照
    fn agda_proof_file(&self) -> &'static str;
    
    /// 安全適用条件
    fn is_safe_to_apply(&self, expr: &Expr, env: &Environment) -> bool;
    
    /// 最適化適用
    fn apply(&self, expr: &Expr) -> Result<Value>;
    
    /// Agda証明の存在確認
    fn has_formal_proof(&self) -> bool {
        std::path::Path::new(self.agda_proof_file()).exists()
    }
}
```

### 定数畳み込み最適化（例）
```rust
/// 定数畳み込み最適化（Agda証明済み）
pub struct ConstantFoldingOptimization;

impl VerifiedOptimization for ConstantFoldingOptimization {
    fn name(&self) -> &'static str {
        "constant-folding"
    }
    
    fn agda_proof_file(&self) -> &'static str {
        "agda/Optimizations/ConstantFolding.agda"
    }
    
    fn is_safe_to_apply(&self, expr: &Expr, env: &Environment) -> bool {
        // 重要: Lambda環境内では適用しない
        !env.is_lambda_context() && 
        expr.is_arithmetic_application() &&
        expr.all_operands_are_constants()
    }
    
    fn apply(&self, expr: &Expr) -> Result<Value> {
        // Agdaで証明された変換のみ適用
        match expr {
            Expr::List(exprs) if exprs.len() >= 2 => {
                if let (Expr::Variable(op), operands) = exprs.split_first().unwrap() {
                    match op.as_str() {
                        "+" => self.fold_addition(operands),
                        "*" => self.fold_multiplication(operands),
                        _ => Err(LambdustError::optimization_not_applicable()),
                    }
                } else {
                    Err(LambdustError::optimization_not_applicable())
                }
            }
            _ => Err(LambdustError::optimization_not_applicable()),
        }
    }
}
```

---

## 🧪 Step 4: 検証システム設計

### プロパティベースド検証
```rust
pub struct VerificationSystem {
    property_checker: PropertyChecker,
    counterexample_generator: CounterexampleGenerator,
}

impl VerificationSystem {
    /// 最適化結果の等価性検証
    pub fn verify_equivalence(&mut self, reference: &Value, optimized: &Value) -> bool {
        // 1. 直接比較
        if reference == optimized {
            return true;
        }
        
        // 2. 数値の等価性（型は異なる可能性）
        if let (Value::Number(ref_num), Value::Number(opt_num)) = (reference, optimized) {
            return self.numbers_equivalent(ref_num, opt_num);
        }
        
        // 3. プロパティベース検証
        self.property_checker.verify_semantic_equivalence(reference, optimized)
    }
    
    /// 反例生成
    pub fn generate_counterexample(
        &mut self,
        optimization: &dyn VerifiedOptimization,
        failing_expr: &Expr,
    ) -> CounterExample {
        self.counterexample_generator.generate(optimization, failing_expr)
    }
}
```

---

## 📊 Step 5: 統合API設計

### 新しい統一インターフェース
```rust
/// 新Lambdust統一API
pub struct Lambdust {
    runtime_executor: RuntimeExecutor,
}

impl Lambdust {
    /// 標準コンストラクタ
    pub fn new() -> Self {
        Self {
            runtime_executor: RuntimeExecutor::new(VerificationLevel::Standard),
        }
    }
    
    /// デバッグモード（純粋評価のみ）
    pub fn debug() -> Self {
        Self {
            runtime_executor: RuntimeExecutor::new(VerificationLevel::Debug),
        }
    }
    
    /// 高速モード（証明済み最適化）
    pub fn performance() -> Self {
        Self {
            runtime_executor: RuntimeExecutor::new(VerificationLevel::Performance),
        }
    }
    
    /// メイン評価API
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        let expr = crate::parser::parse(input)?;
        self.runtime_executor.execute(expr)
    }
    
    /// セマンティック評価（デバッグ用）
    pub fn eval_semantic(&mut self, input: &str) -> Result<Value> {
        let expr = crate::parser::parse(input)?;
        self.runtime_executor.semantic_evaluator.eval_pure(
            expr,
            self.runtime_executor.semantic_evaluator.global_env.clone(),
            Continuation::Done,
        )
    }
}
```

---

## 🎯 実装スケジュール

### Week 1: SemanticEvaluator抽出
- [ ] `src/evaluator/semantic.rs` 作成
- [ ] 純粋評価ロジックの抽出
- [ ] 基本テストの移植

### Week 2: RuntimeExecutor統合
- [ ] `src/runtime_executor.rs` 作成
- [ ] 最適化システムの再構築
- [ ] 検証システムの実装

### Week 3: API統合・テスト
- [ ] 新統一APIの実装
- [ ] 既存テストの互換性確保
- [ ] プロパティベースドテスト追加

---

## ⚠️ リスク軽減策

### 1. 段階的移行
```rust
// 移行期間中の互換性API
impl Evaluator {
    /// 旧API互換性のため残置（deprecated）
    #[deprecated(note = "Use Lambdust::eval instead")]
    pub fn eval_string(&mut self, input: &str) -> Result<Value> {
        // 内部でRuntimeExecutorに委譲
        let mut runtime = RuntimeExecutor::new(VerificationLevel::Standard);
        runtime.execute_string(input)
    }
}
```

### 2. フェイルセーフ機能
```rust
impl RuntimeExecutor {
    /// 緊急時フォールバック
    pub fn emergency_fallback(&mut self, expr: Expr) -> Result<Value> {
        self.semantic_evaluator.eval_pure(
            expr,
            self.semantic_evaluator.global_env.clone(),
            Continuation::Done,
        )
    }
}
```

---

この設計により、現在のSRFI 69問題のような**意味論的整合性の喪失**を根本的に防止し、同時に証明済み最適化による高性能を実現できます。

Agdaインストール完了次第、この設計に基づいて実装を開始しましょう。