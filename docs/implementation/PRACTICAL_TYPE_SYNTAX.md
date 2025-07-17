# Practical Type Syntax for Lambdust
# Lambdust実用型構文設計

## 🎯 設計原則

1. **キーボード入力容易性**: 標準ASCIIキーボードで全て入力可能
2. **視覚的明確性**: 型レベルと値レベルの明確な区別
3. **R7RS保存性**: 既存Scheme構文と衝突なし
4. **数学的一貫性**: 理論的基盤との対応関係維持

## 📝 型抽象化構文

### System F Style Type Abstraction

```scheme
;; 型抽象化 (Type Lambda)
;; 理論: Λα.T
;; 実装: (type-lambda (alpha) body)
(type-lambda (alpha) 
  (lambda ((x : alpha)) x))

;; 複数型変数
(type-lambda (alpha beta)
  (lambda ((f : (-> alpha beta)) (x : alpha))
    (f x)))
```

### Universal Quantification

```scheme
;; 全称量化
;; 理論: ∀α.T
;; 実装: (forall (alpha) type-expr)
(forall (alpha) 
  (-> alpha alpha))

;; 制約付き量化
(forall (alpha) 
  (=> (Eq alpha) (-> alpha alpha Bool)))
```

### Type Application

```scheme
;; 型適用
;; 理論: e[T]
;; 実装: (type-apply expr type)
(type-apply identity-func Int)

;; または中置記法風
(identity @ Int)
```

## 🏗️ 依存型構文

### Dependent Functions (Pi Types)

```scheme
;; 依存関数型
;; 理論: Π(x:A).B(x)
;; 実装: (pi (x A) B)
(pi (n Nat) (Vector Int n))

;; 関数定義での使用
(define vec-length : (pi (n Nat) (pi (A Type) (-> (Vector A n) Nat)))
  (type-lambda (n A)
    (lambda ((vec : (Vector A n)))
      n)))
```

### Dependent Pairs (Sigma Types)

```scheme
;; 依存対型
;; 理論: Σ(x:A).B(x)
;; 実装: (sigma (x A) B)
(sigma (n Nat) (Vector Int n))

;; 値構築
(make-sigma 5 (vector 1 2 3 4 5))
```

## 🔧 特殊形式一覧

### 型レベル操作

| 特殊形式 | 数学記法 | 説明 | 例 |
|---------|----------|------|-----|
| `type-lambda` | `Λα.T` | 型抽象化 | `(type-lambda (a) (List a))` |
| `forall` | `∀α.T` | 全称量化 | `(forall (a) (-> a a))` |
| `exists` | `∃α.T` | 存在量化 | `(exists (a) (Pair a String))` |
| `pi` | `Π(x:A).B(x)` | 依存積 | `(pi (n Nat) (Vec Int n))` |
| `sigma` | `Σ(x:A).B(x)` | 依存和 | `(sigma (n Nat) (Vec a n))` |
| `type-apply` | `e[T]` | 型適用 | `(type-apply f Int)` |
| `type-of` | `typeof(e)` | 型取得 | `(type-of expr)` |

### 型制約

| 特殊形式 | 説明 | 例 |
|---------|------|-----|
| `=>` | 制約含意 | `(=> (Eq a) (-> a a Bool))` |
| `where` | 制約付き定義 | `(where ((Eq a)) (-> a a Bool))` |
| `instance` | 型クラスインスタンス | `(instance (Eq Int) ...)` |

## 🎯 実装優先度

### Phase 1: 基本型抽象化 (高優先度)
- [x] 基本lambda型注釈: `(lambda ((x : Int)) ...)`
- [ ] 型抽象化: `(type-lambda (alpha) ...)`
- [ ] 型適用: `(type-apply expr type)`
- [ ] 全称量化: `(forall (alpha) ...)`

### Phase 2: 依存型 (中優先度)  
- [ ] Pi型: `(pi (x A) B)`
- [ ] Sigma型: `(sigma (x A) B)`
- [ ] 依存型関数定義
- [ ] 型レベル計算

### Phase 3: 高度機能 (低優先度)
- [ ] 存在量化: `(exists (alpha) ...)`
- [ ] 型制約システム
- [ ] 高階種型
- [ ] ユニバレンス

## 💻 実装戦略

### 1. パーサー拡張

```rust
// 新しい特殊形式の認識
pub fn is_type_level_form(name: &str) -> bool {
    matches!(name, 
        "type-lambda" | "forall" | "exists" | 
        "pi" | "sigma" | "type-apply" |
        "type-of" | "instance"
    )
}
```

### 2. 型システム拡張

```rust
pub enum PolynomialType {
    // 既存...
    
    /// Universal quantification: (forall (alpha) body)
    Forall {
        type_vars: Vec<String>,
        body: Box<PolynomialType>,
    },
    
    /// Type lambda: (type-lambda (alpha) body)  
    TypeLambda {
        type_vars: Vec<String>,
        body: Box<PolynomialType>,
    },
    
    /// Type application: (type-apply function argument)
    TypeApplication {
        function: Box<PolynomialType>,
        arguments: Vec<PolynomialType>,
    },
}
```

### 3. 評価器統合

```rust
impl Evaluator {
    pub fn eval_type_lambda(&mut self, operands: &[Expr], env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // 型抽象化の評価実装
    }
    
    pub fn eval_forall(&mut self, operands: &[Expr], env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // 全称量化の評価実装  
    }
    
    pub fn eval_type_apply(&mut self, operands: &[Expr], env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // 型適用の評価実装
    }
}
```

## 🧪 テスト構文更新

### 更新前（入力困難）
```scheme
(Λ [α] (lambda ([x : α]) x))
```

### 更新後（実装可能）
```scheme
(type-lambda (alpha)
  (lambda ((x : alpha)) x))
```

## 🔬 理論的正当性

この構文設計は以下の理論的基盤に基づいています：

1. **System F完全性**: 全てのSystem F構成要素をカバー
2. **Polynomial Universe対応**: 依存型理論との整合性
3. **HoTT準拠**: ホモトピー型理論への拡張可能性
4. **実装可能性**: 段階的実装とテスト可能

---

この設計により、**理論的厳密性**と**実装可能性**を両立し、Lambdustの型システムを段階的に構築できます。