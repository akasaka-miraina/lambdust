# Lambdust評価器アーキテクチャ詳細ドキュメント

## 現在の問題概要

SRFI 69のlambda関数内で`(+ v acc)`式の評価が正常に行われず、`v`の値のみが返される問題が発生。
トレースシステムによる調査で、`(+ v acc)`式が`eval()`メソッドに到達していないことが判明。

## 評価器アーキテクチャ

### 1. CPS（継続渡しスタイル）評価器

Lambdustは、R7RS形式的意味論に準拠したCPS評価器を採用：

```rust
pub fn eval(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value>
```

#### 核心原理
- **式（Expression）**: 評価対象のS式
- **環境（Environment）**: レキシカルスコープの変数束縛
- **継続（Continuation）**: 評価完了後の処理

### 2. 継続（Continuation）システム

```rust
#[derive(Debug, Clone)]
pub enum Continuation {
    Identity,                    // 恒等継続：結果をそのまま返す
    Application { ... },         // 関数適用継続
    Operator { ... },           // 演算子評価継続
    Begin { ... },              // begin構文継続
    Define { ... },             // define構文継続
    Values { ... },             // 多値継続
    // ... その他
}
```

### 3. 環境（Environment）システム

```rust
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}
```

#### レキシカルスコープ実装
- **継承連鎖**: 親環境への参照チェーン
- **変数探索**: 現在の環境→親環境の順序で探索
- **束縛作成**: `define`で現在環境に束縛、`set!`で既存束縛を更新

## 現在の評価フロー

### 4. Lambda関数評価

```rust
// 1. Lambda関数作成
(lambda (k v acc) (+ v acc))
↓
Procedure::Lambda {
    params: ["k", "v", "acc"],
    body: [List([Variable("+"), Variable("v"), Variable("acc")])]
}

// 2. Lambda関数呼び出し
apply_procedure_with_evaluator()
↓
// パラメータバインディング
new_env.define("k", arg0);
new_env.define("v", arg1); 
new_env.define("acc", arg2);
↓
// Body評価
eval_sequence(body, new_env, cont)
```

### 5. 式評価フロー

```rust
eval_sequence() → single expression
↓
eval(expr, env, cont)
↓
match expr {
    List(exprs) => eval_application(exprs, env, cont)
    Variable(name) => eval_variable(name, env, cont)
    // ...
}
```

## 問題分析

### 6. 現在の問題箇所

**症状**: `(+ v acc)`の評価で`acc`が無視され、`v`の値のみ返される

**トレース調査結果**:
1. ✅ Lambda関数作成は正常
2. ✅ パラメータバインディング（`k`, `v`, `acc`）は正常
3. ✅ `eval_sequence`は`(+ v acc)`式を正しく受け取り
4. ❌ `eval()`内で`(+ v acc)`のList処理が実行されない
5. ❌ 代わりに`Number(Integer(1))`（`v`の値）が直接返される

### 7. 推定原因

#### A. 最適化システムの干渉
- **Inline Evaluation**: Phase 6-B-Step3の継続最適化
- **Expression Analyzer**: Phase 5-Step1の式解析最適化
- **JIT最適化**: Phase 6-Cのループ最適化

#### B. 継続チェーンの短絡
```rust
eval_sequence() 
→ eval((+ v acc), env, Identity)
→ ? (何らかの最適化で短絡)
→ apply_continuation(Identity, Number(Integer(1)))
```

## R7RS形式的意味論の要件

### 8. S式評価の正確性

R7RSにおけるS式評価は以下を満たす必要がある：

#### リスト評価
```scheme
(operator arg1 arg2 ...)
```
1. **演算子評価**: `operator`を評価してprocedureを取得
2. **引数評価**: `arg1`, `arg2`, ... を順次評価
3. **手続き適用**: procedureに評価済み引数を適用

#### レキシカルスコープ
```scheme
(lambda (x y) (+ x y))
```
- **環境作成**: lambda作成時の環境を保持（クロージャ）
- **パラメータ束縛**: 呼び出し時に新環境でパラメータを束縛
- **Body評価**: 新環境でbodyを評価

### 9. 現在の実装の問題点

#### A. S式評価後のリスト持ち方
```rust
// 現在の問題: List処理で最適化が過度に適用される
match expr {
    List(exprs) => {
        // 何らかの最適化で短絡される？
        eval_application(exprs, env, cont)
    }
}
```

#### B. レキシカル束縛変数の参照
```rust
// 環境チェーンは正常だが、評価時に問題
Environment::get(&name) → Some(value) // 正常
↓
apply_continuation(cont, value) // ここで最適化？
```

## アーキテクチャ改善提案

### 10. 評価器の分離と明確化

#### A. 純粋評価器と最適化の分離
```rust
pub struct CoreEvaluator {
    // R7RS形式的意味論の純粋実装
}

pub struct OptimizedEvaluator {
    core: CoreEvaluator,
    optimizations: Vec<OptimizationPass>,
}
```

#### B. 継続処理の透明性確保
```rust
impl Evaluator {
    fn eval_with_trace(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // 必ず全ての評価ステップをトレース
        // 最適化による短絡を防止
    }
}
```

### 11. レキシカルスコープの改善

#### A. 環境システムの強化
```rust
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
    scope_id: ScopeId,  // デバッグ用
    created_by: String, // 作成コンテキスト
}
```

#### B. 変数束縛の追跡
```rust
pub struct VariableBinding {
    name: String,
    value: Value,
    scope: ScopeId,
    created_at: Location,
}
```

### 12. S式評価の堅牢化

#### A. List評価の段階的処理
```rust
fn eval_application(&mut self, exprs: Vec<Expr>, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
    // 1. 必ず演算子を評価
    // 2. 必ず引数を評価
    // 3. 最適化は明示的に制御
    
    if self.should_optimize(&exprs) {
        self.try_optimized_evaluation(exprs, env, cont)
    } else {
        self.standard_evaluation(exprs, env, cont)
    }
}
```

#### B. 最適化の条件判定
```rust
fn should_optimize(&self, exprs: &[Expr]) -> bool {
    // lambda内では最適化を無効化
    if self.is_in_lambda_context() {
        return false;
    }
    
    // 他の条件...
}
```

## 次のステップ

### 13. 短期的修正

1. **最適化の一時無効化**: 問題箇所特定のため
2. **トレースの詳細化**: 評価フローの完全可視化
3. **Lambda評価の分離**: 専用評価パスの作成

### 14. 長期的リファクタリング

1. **評価器の階層化**: Core/Optimized分離
2. **継続システムの再設計**: より透明な処理
3. **環境システムの強化**: デバッグ機能充実
4. **テストシステムの拡充**: R7RS適合性保証

## 結論

現在の問題は、最適化システムがR7RS形式的意味論の純粋性を損なっていることが原因と推測される。
アーキテクチャの分離と透明性の確保により、正確性と性能の両立を図る必要がある。