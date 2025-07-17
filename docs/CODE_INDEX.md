# Lambdust Code Index

この文書はLambdustコードベースの構造体・関数・メソッドの完全なインデックスです。実装時は必ずこのインデックスを参照し、変更時は対応するインデックス項目を更新してください。

## インデックス化ルール

1. **構造体定義**: 全てのpublic構造体とそのメンバフィールド
2. **関数シグネチャ**: 全てのpublic関数・メソッドの完全なシグネチャ
3. **ファイル位置**: ファイルパス + 行番号 (最終更新時点)
4. **依存関係**: 使用されるtrait、型、モジュール
5. **更新履歴**: 最終更新日時とchangelog

## 🏗️ Core Infrastructure

### error.rs - エラー処理システム

#### LambdustError enum
**Location**: `src/error.rs:177`  
**Last Updated**: 2025-01-12

```rust
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LambdustError {
    LexerError { message: String, location: SourceSpan },
    ParseError { message: String, location: SourceSpan },
    RuntimeError { message: String, context: Box<ErrorContext> },
    TypeError { message: String, context: Box<ErrorContext> },
    UndefinedVariable { variable: String, context: Box<ErrorContext> },
    ArityError { expected: usize, actual: usize, function: String, context: Box<ErrorContext> },
    DivisionByZero { context: Box<ErrorContext> },
    IoError { message: String, location: Option<SourceSpan> },
    StackOverflow { context: Box<ErrorContext> },
    MacroError { message: String, context: Box<ErrorContext> },
    SyntaxError { message: String, location: SourceSpan },
}
```

**Helper Methods**:
- `runtime_error(message: impl Into<String>) -> Self` - Line 289
- `type_error_old(message: String) -> Self` - Line 297  
- `undefined_variable_old(variable: String) -> Self` - Line 302
- `syntax_error_old(message: String) -> Self` - Line 307

#### ErrorContext struct
**Location**: `src/error.rs:130`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    pub source: Option<SourceSpan>,
    pub call_stack: Vec<String>,
    pub stack_trace: Vec<String>,
}
```

#### SourceSpan struct
**Location**: `src/error.rs:52`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub filename: Option<String>,
}
```

### value/mod.rs - Value System

#### Value enum
**Location**: `src/value/mod.rs:24`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(SchemeNumber),
    Character(char),
    String(String),
    Symbol(String),
    Keyword(String),
    Pair(std::rc::Rc<std::cell::RefCell<PairData>>),
    Vector(std::rc::Rc<std::cell::RefCell<Vec<Value>>>),
    HashTable(std::rc::Rc<std::cell::RefCell<std::collections::HashMap<Value, Value>>>),
    Procedure(crate::value::Procedure),
    Port(crate::value::Port),
    Record(crate::value::Record),
    Promise(std::rc::Rc<std::cell::RefCell<crate::value::Promise>>),
    Undefined,
}
```

#### PairData struct
**Location**: `src/value/pair.rs:10`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PairData {
    pub car: Value,
    pub cdr: Value,
}
```

### value/procedure.rs - Procedure Types

#### Procedure enum
**Location**: `src/value/procedure.rs:10`  
**Last Updated**: 2025-01-12

```rust
#[derive(Clone)]
pub enum Procedure {
    Lambda {
        params: Vec<String>,
        variadic: bool,
        body: Vec<Expr>,
        closure: Rc<Environment>,
    },
    Builtin {
        name: String,
        arity: Option<usize>,
        func: fn(&[Value]) -> crate::Result<Value>,
    },
    HostFunction {
        name: String,
        arity: Option<usize>,
        func: crate::host::HostFunc,
    },
    Continuation {
        continuation: Box<Continuation>,
    },
    CapturedContinuation {
        continuation: Box<crate::evaluator::Continuation>,
    },
    // ... other variants
}
```

### environment.rs - Environment System

#### Environment struct  
**Location**: `src/environment.rs:25`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug)]
pub struct Environment {
    bindings: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Environment>>,
    metadata: RefCell<EnvironmentMetadata>,
}
```

**Key Methods**:
- `new() -> Self` - Line 45
- `extend(parent: Rc<Environment>) -> Self` - Line 52
- `define(&self, name: String, value: Value)` - Line 70
- `get(&self, name: &str) -> Option<Value>` - Line 85
- `set(&self, name: &str, value: Value) -> Result<()>` - Line 105

## 🚀 Evaluator System

### evaluator/mod.rs - Main Evaluator Interface

#### Key Exports
**Location**: `src/evaluator/mod.rs:84-200`  
**Last Updated**: 2025-01-12

```rust
// Core evaluator types
pub use continuation::{Continuation, DoLoopState, DynamicPoint, EnvironmentRef};
pub use continuation_pooling::{ContinuationPoolManager, ContinuationType};
pub use execution_context::{ExecutionContext, ExecutionContextBuilder, ExecutionPriority};
pub use semantic::SemanticEvaluator;
pub use runtime_executor::{RuntimeExecutor, RuntimeOptimizationLevel};

// R7RS-pico exports (feature gated)
#[cfg(feature = "pico")]
pub use pico_evaluator::PicoEvaluator;
#[cfg(feature = "pico")]
pub use pico_environment::{create_pico_initial_environment, get_pico_features, PicoFeatures};
```

### evaluator/execution_context.rs - Context System

#### ExecutionContext struct
**Location**: `src/evaluator/execution_context.rs:24`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub expression: Expr,
    pub environment: Rc<Environment>,
    pub continuation: Continuation,
    pub static_analysis: StaticAnalysisResult,
    pub optimization_hints: OptimizationHints,
    pub execution_metadata: ExecutionMetadata,
    pub constant_bindings: FxHashMap<String, Value>,
    pub macro_expansion_state: MacroExpansionState,
}
```

**Key Methods**:
- `new(expression: Expr, environment: Rc<Environment>, continuation: Continuation) -> Self` - Line 492
- `derive_optimization_hints(&mut self)` - Line 535
- `add_constant_binding(&mut self, name: String, value: Value)` - Line 590

### evaluator/pico_evaluator.rs - R7RS-pico Implementation

#### PicoEvaluator struct
**Location**: `src/evaluator/pico_evaluator.rs:43`  
**Last Updated**: 2025-01-12 (⚠️ Error type fixes needed)

```rust
#[cfg(feature = "pico")]
#[derive(Debug, Clone)]
pub struct PicoEvaluator {
    max_recursion_depth: usize,
    current_depth: usize,
}
```

**Key Methods**:
- `new() -> Self` - Line 49
- `with_recursion_limit(max_depth: usize) -> Self` - Line 55
- `evaluate(&mut self, expr: &Expr, env: Rc<Environment>) -> Result<Value>` - Line 66
- `apply_builtin(&self, name: &str, args: &[Value]) -> Result<Value>` - Line 391

**⚠️ Known Issues**: Error type compatibility needs fixing for current LambdustError structure

### evaluator/runtime_executor.rs - Runtime Optimization

#### RuntimeExecutor struct
**Location**: `src/evaluator/runtime_executor.rs:280`  
**Last Updated**: 2025-01-12

```rust
pub struct RuntimeExecutor {
    semantic_evaluator: SemanticEvaluator,
    jit_optimizer: JitLoopOptimizer,
    continuation_pool: ContinuationPoolManager,
    inline_evaluator: InlineEvaluator,
    optimization_manager: IntegratedOptimizationManager,
    hotpath_detector: AdvancedHotPathDetector,
    // ... performance tracking fields with FxHashMap
}
```

## 🎯 AST and Parsing

### ast.rs - Abstract Syntax Tree

#### Expr enum
**Location**: `src/ast.rs:15`  
**Last Updated**: 2025-01-12

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    List(Vec<Expr>),
    Quote(Box<Expr>),
    Quasiquote(Box<Expr>),
    Unquote(Box<Expr>),
    UnquoteSplicing(Box<Expr>),
    Lambda(LambdaExpr),
    If(IfExpr),
    Begin(Vec<Expr>),
    Set(SetExpr),
    Define(DefineExpr),
    // ... other expression types
}
```

### parser.rs - Parser Implementation

#### Parser struct
**Location**: `src/parser.rs:25`  
**Last Updated**: 2025-01-12

```rust
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    recursion_depth: usize,
    max_recursion_depth: usize,
}
```

**Key Methods**:
- `new(tokens: Vec<Token>) -> Self` - Line 45
- `parse(&mut self) -> Result<Expr>` - Line 65
- `parse_list(&mut self) -> Result<Expr>` - Line 120

## 🧮 Memory Management

### memory_pool.rs - Memory Optimization

#### ValuePool struct
**Location**: `src/memory_pool.rs:63`  
**Last Updated**: 2025-01-12

```rust
pub struct ValuePool {
    boolean_cache: [Value; 2],
    nil_singleton: Value,
    small_integer_pool: Vec<Value>,
    value_recycle_pool: Vec<Value>,
}
```

#### ContinuationPool struct
**Location**: `src/memory_pool.rs:223`  
**Last Updated**: 2025-01-12

```rust
pub struct ContinuationPool {
    identity_pool: Vec<Continuation>,
    recycled_count: usize,
    created_count: usize,
}
```

## 📝 Macro System

### macros/hygiene/generator.rs - Symbol Generation

#### SymbolGenerator struct
**Location**: `src/macros/hygiene/generator.rs:50`  
**Last Updated**: 2025-01-12

**⚠️ Known Issue**: Uses `gen` variable name which conflicts with Rust 2024 edition reserved keyword

## 🧪 Testing Infrastructure

### tests/ - Test Organization

**Integration Tests**:
- `tests/integration/execution_context_integration_tests.rs`
- `tests/integration/module_system_integration_tests.rs`
- `tests/integration/srfi46_tests.rs`

**Unit Tests**:
- `tests/unit/environment/cow_tests.rs`
- `tests/unit/evaluator/runtime_executor_jit_tests.rs`
- `tests/unit/macros/hygienic_integration_tests.rs`

## 🔧 Build Configuration

### Cargo.toml - Feature Flags

**Core Features**:
- `pico` - R7RS-pico ultra-minimal implementation
- `embedded` - Embedded systems support (includes pico)
- `minimal` - Minimal configuration
- `standard` - Standard configuration (default)

## 📋 Update Protocol

### インデックス更新手順

1. **構造体・enum変更時**:
   ```bash
   # 1. 該当するインデックス項目を更新
   # 2. ファイル位置と行番号を確認
   # 3. Last Updated日付を更新
   # 4. 関連する依存項目も確認・更新
   ```

2. **関数シグネチャ変更時**:
   ```bash
   # 1. 関数シグネチャをインデックスで確認
   # 2. 変更内容をインデックスに反映
   # 3. 呼び出し元の影響も評価
   # 4. 関連テストの更新が必要か確認
   ```

3. **新機能追加時**:
   ```bash
   # 1. 新しい構造体・関数をインデックスに追加
   # 2. 適切なカテゴリに分類
   # 3. 依存関係を明記
   # 4. サンプルコードも追加
   ```

## 🚨 重要な既知の問題

1. **R7RS-pico Error Types**: `LambdustError`の現在の構造に合わせた修正が必要
2. **Rust 2024 Edition**: `gen`変数名の衝突問題
3. **Procedure Types**: `Procedure::Lambda`の`body`フィールドが`Vec<Expr>`に変更されている

## 📊 統計情報

- **総ファイル数**: 100+ Rustファイル
- **主要構造体数**: 50+ public structs/enums
- **Public関数数**: 500+ functions/methods
- **Feature Flags**: 20+ conditional compilation features
- **Test Files**: 50+ test modules

---

**最終更新**: 2025-01-12  
**メンテナ**: Claude Code Assistant  
**バージョン**: Lambdust v0.3.0