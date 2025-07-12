# Semantic Reduction Design for SemanticEvaluator

## 🎯 重要な設計方針

**指摘**: 意味論的評価器が「最適化完全排除」とされているが、意味論を保持した状態でのS式としての「簡約」は必要である。

## 📐 簡約 vs 最適化の明確な区別

### 🔬 **簡約 (Reduction)** - SemanticEvaluatorで必要
R7RS形式的意味論に基づく正当な式変換：

#### A. β簡約 (Beta Reduction)
```scheme
;; Lambda適用の簡約
((lambda (x) (+ x 1)) 5) → (+ 5 1) → 6
```

#### B. 定数畳み込み (Constant Folding) - 意味論準拠
```scheme
;; 数学的同値性に基づく簡約
(+ 2 3) → 5
(* 0 expr) → 0  ;; exprが副作用なしの場合
```

#### C. 同一性簡約 (Identity Reduction)
```scheme
;; 数学的同一性の適用
(+ x 0) → x
(* x 1) → x
(and #t expr) → expr
(or #f expr) → expr
```

#### D. 条件式簡約 (Conditional Reduction)
```scheme
;; テスト条件が定数の場合
(if #t then-expr else-expr) → then-expr
(if #f then-expr else-expr) → else-expr
```

#### E. Let束縛簡約 (Let Binding Reduction)
```scheme
;; 使用されない束縛の除去（副作用なしの場合）
(let ((x 5) (y (complex-expr))) x) → 5  ;; complex-exprが副作用なしの場合
```

### ⚡ **最適化 (Optimization)** - RuntimeExecutorで実装
実行効率向上のための変換（意味論を超える）：

#### A. インライン展開 (Inlining)
```rust
// Rust実装レベルでの関数呼び出し除去
function_call() → inline_code_block
```

#### B. ループアンローリング (Loop Unrolling)
```scheme
;; 小さなループの展開
(do ((i 0 (+ i 1))) ((= i 3)) (display i))
→ (begin (display 0) (display 1) (display 2))
```

#### C. 継続最適化 (Continuation Optimization)
```rust
// トランポリン評価器の継続チェーン短縮
long_continuation_chain → optimized_direct_call
```

## 🔧 SemanticEvaluatorでの簡約実装方針

### 1. 純粋簡約関数群
```rust
impl SemanticEvaluator {
    /// R7RS意味論準拠の式簡約
    pub fn reduce_expression_pure(&self, expr: Expr) -> Result<Expr> {
        match expr {
            // β簡約: Lambda適用
            Expr::List(exprs) if self.is_lambda_application(&exprs) => {
                self.beta_reduce(exprs)
            }
            
            // 定数畳み込み: 算術式
            Expr::List(exprs) if self.is_arithmetic_expression(&exprs) => {
                self.fold_constants_pure(exprs)
            }
            
            // 条件式簡約
            Expr::List(exprs) if self.is_conditional(&exprs) => {
                self.reduce_conditional_pure(exprs)
            }
            
            // その他は変更なし
            _ => Ok(expr)
        }
    }
    
    /// β簡約の実装
    fn beta_reduce(&self, exprs: Vec<Expr>) -> Result<Expr> {
        // ((lambda (params) body) args...) → body[params := args]
        // 変数置換を実行
    }
    
    /// 定数畳み込み（R7RS意味論準拠）
    fn fold_constants_pure(&self, exprs: Vec<Expr>) -> Result<Expr> {
        // (+ 2 3) → 5
        // (* x 0) → 0 (xが副作用なしの場合)
        // 数学的同値性のみ適用
    }
    
    /// 条件式簡約
    fn reduce_conditional_pure(&self, exprs: Vec<Expr>) -> Result<Expr> {
        // (if #t then else) → then
        // (if #f then else) → else
    }
}
```

### 2. 副作用解析
```rust
impl SemanticEvaluator {
    /// 式が副作用を持つかR7RS準拠で判定
    fn has_side_effects_pure(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Variable(_) | Expr::Literal(_) => false,
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) => {
                        // R7RS定義に基づく副作用関数判定
                        self.is_side_effect_procedure(name) || 
                        exprs[1..].iter().any(|e| self.has_side_effects_pure(e))
                    }
                    _ => exprs.iter().any(|e| self.has_side_effects_pure(e))
                }
            }
            _ => false // 保守的判定
        }
    }
    
    /// R7RS準拠の副作用手続き判定
    fn is_side_effect_procedure(&self, name: &str) -> bool {
        matches!(name,
            "set!" | "set-car!" | "set-cdr!" | "vector-set!" |
            "display" | "write" | "newline" | "read" |
            "call-with-output-file" | "call-with-input-file"
            // R7RS標準の副作用手続きリスト
        )
    }
}
```

### 3. 段階的簡約適用
```rust
impl SemanticEvaluator {
    /// 評価前の式簡約
    pub fn eval_pure_with_reduction(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // 1. R7RS準拠の簡約適用
        let reduced_expr = self.reduce_expression_pure(expr)?;
        
        // 2. 通常の純粋評価
        self.eval_pure(reduced_expr, env, cont)
    }
}
```

## 📊 簡約効果の測定

### A. 意味論的正確性保証
- **同値性テスト**: 簡約前後で評価結果が同一
- **副作用保持**: 副作用のある式は簡約しない
- **R7RS準拠性**: 標準仕様との完全一致

### B. 形式的検証対応
```agda
-- Agda証明における簡約正当性
reduction-preserves-semantics : ∀ (e e' : Expr) → 
  reduce e ≡ e' → eval e ≡ eval e'
```

### C. 簡約統計
```rust
#[derive(Debug, Default)]
pub struct ReductionStats {
    pub beta_reductions: usize,
    pub constant_folds: usize,
    pub conditional_reductions: usize,
    pub identity_reductions: usize,
    pub expressions_analyzed: usize,
    pub expressions_reduced: usize,
}
```

## 🎯 実装優先度

### HIGH: 基本簡約
1. **定数畳み込み**: 算術式・真偽値式
2. **同一性簡約**: +0, *1, and #t, or #f
3. **条件式簡約**: if定数条件

### MEDIUM: 高度簡約
4. **β簡約**: Lambda適用（小規模）
5. **Let束縛簡約**: 未使用変数除去
6. **リスト操作簡約**: car/cdr組み合わせ

### LOW: 特殊簡約
7. **マクロ展開後簡約**: cond→if変換後の簡約
8. **再帰パターン簡約**: 末尾再帰の識別

## 💡 重要な制約

### ✅ 許可される簡約
- **数学的同値性**: R7RS仕様で保証される変換
- **構文糖衣の展開**: マクロ展開と同等の変換
- **定数式の評価**: コンパイル時に決定可能な値

### ❌ 禁止される最適化
- **実行順序の変更**: 副作用がある場合
- **関数呼び出しの除去**: 副作用の可能性
- **メモリレイアウト最適化**: 実装依存の変更

この設計により、SemanticEvaluatorは**R7RS形式的意味論を保持した状態での適切な簡約**を実行し、数学的参照実装としての役割を果たしながら、不要な計算を除去できます。