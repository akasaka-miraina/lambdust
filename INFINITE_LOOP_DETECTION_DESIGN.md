# 無限ループ検出システム設計書

## 🎯 目的
パーサーレベルで循環参照や脱出契機のない無限ループを検出し、ParseErrorとしてユーザーに返すシステム。

## 🔍 検出対象パターン

### 1. 循環参照パターン
```scheme
;; 直接循環参照
(define x x)

;; 間接循環参照
(define x y)
(define y x)

;; 複雑な循環参照
(define a (+ b 1))
(define b (+ c 1))
(define c (+ a 1))
```

### 2. 自己再帰パターン
```scheme
;; 脱出条件のない再帰関数
(define (loop x) (loop x))

;; 条件があるが到達不可能な再帰
(define (loop x) (if #f x (loop x)))

;; 相互再帰での循環
(define (foo x) (bar x))
(define (bar x) (foo x))
```

### 3. 構造的循環パターン
```scheme
;; リスト構造の循環
(define lst '(1 2 3))
(set-cdr! (cddr lst) lst)

;; quote内の循環参照
'(x . x)  ; これは正常
```

## 🛠️ 実装戦略

### Phase 1: 静的解析フレームワーク
- **DependencyAnalyzer**: 変数・関数間の依存関係を構築
- **CyclicDependencyDetector**: 循環依存の検出
- **LoopAnalyzer**: 条件分岐・脱出条件の分析

### Phase 2: AST拡張
- **ExpressionContext**: 式の解析コンテキスト
- **VariableScope**: 変数スコープと依存関係追跡
- **FunctionAnalysis**: 関数定義の静的解析

### Phase 3: Parser統合
- **ParseContext**: パーサーのコンテキスト拡張
- **LoopDetectionPass**: 無限ループ検出パス
- **ErrorReporting**: 詳細なエラーメッセージ

## 📊 検出アルゴリズム

### 1. 依存関係グラフ構築
```rust
struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
    edges: Vec<(String, String)>,
}

struct DependencyNode {
    name: String,
    expr: Expr,
    depends_on: Vec<String>,
}
```

### 2. 循環検出アルゴリズム
- **Tarjan's Algorithm**: 強連結成分検出
- **DFS Based**: 単純な循環検出
- **Topological Sort**: 依存関係の整列

### 3. 条件分析
- **Reachability Analysis**: 脱出条件の到達可能性
- **Control Flow Analysis**: 制御フロー解析
- **Termination Analysis**: 終了条件の存在確認

## 🎨 実装アーキテクチャ

```
src/
├── parser/
│   ├── mod.rs
│   ├── loop_detection.rs      # 無限ループ検出
│   ├── dependency_analyzer.rs # 依存関係解析
│   └── cycle_detector.rs      # 循環検出
├── ast/
│   ├── analysis.rs            # AST解析拡張
│   └── context.rs             # 解析コンテキスト
└── error/
    └── loop_errors.rs         # ループエラー定義
```

## 🚀 実装ステップ

### Step 1: 基本インフラ
1. `DependencyAnalyzer`の作成
2. `CycleDetector`の実装
3. `LoopDetectionError`の定義

### Step 2: 循環参照検出
1. 変数定義の依存関係グラフ構築
2. 強連結成分検出
3. 循環参照エラー生成

### Step 3: 関数循環検出
1. 関数呼び出しの依存関係解析
2. 脱出条件の分析
3. 無限再帰検出

### Step 4: Parser統合
1. パーサーへの検出システム統合
2. エラーメッセージ強化
3. テストスイート作成

## 📋 エラーメッセージ例

```
ParseError: Infinite loop detected
  --> code.scm:3:1
   |
 1 | (define x y)
 2 | (define y z)
 3 | (define z x)
   |         ^ Circular dependency detected: x → y → z → x
   |
   = help: Remove the circular dependency by breaking the cycle

ParseError: Infinite recursion detected
  --> code.scm:1:1
   |
 1 | (define (loop x) (loop x))
   |  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |         
   = help: Add a base case or termination condition to prevent infinite recursion
```

## 🔧 設定オプション

```rust
pub struct LoopDetectionConfig {
    /// 循環検出を有効化
    pub enable_cycle_detection: bool,
    /// 再帰深度制限
    pub max_recursion_depth: usize,
    /// 依存関係解析深度
    pub max_dependency_depth: usize,
    /// 警告のみ（エラーにしない）
    pub warn_only: bool,
}
```

## 🎯 パフォーマンス考慮

- **遅延評価**: 必要時のみ解析実行
- **キャッシュ**: 解析結果のキャッシュ
- **増分解析**: 部分的な再解析
- **スケーラビリティ**: 大規模コードベース対応

## 📈 将来拡張

- **デッドコード検出**: 到達不可能コード
- **パフォーマンス解析**: ホットスポット検出
- **最適化ヒント**: コンパイラ最適化支援
- **IDE統合**: リアルタイム解析