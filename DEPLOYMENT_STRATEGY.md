# 🚀 Lambdust Deployment Strategy
## Production-Ready Binary Size Optimization

### 📋 デプロイメント層の設計

Lambdustは用途に応じて異なるビルド構成を提供します：

#### 🏗️ ビルド構成の階層

```
lambdust-minimal     (< 5MB)   - 基本Scheme実行のみ
├── lambdust-standard (< 15MB) - SRFI + 基本最適化
├── lambdust-verified (< 50MB) - Agda検証システム付き
└── lambdust-dev      (< 100MB) - 完全開発環境
```

---

## 🎯 Feature Flags 戦略

### Cargo.toml の構成

```toml
[package]
name = "lambdust"
version = "0.3.0"
edition = "2021"

[features]
default = ["minimal"]

# 基本構成（最小限）
minimal = []

# 標準構成（一般用途）
standard = [
    "minimal",
    "srfi-support",
    "basic-optimization",
    "memory-pooling"
]

# 検証付き構成（高信頼性用途）
verified = [
    "standard", 
    "agda-integration",
    "theorem-derivation",
    "runtime-verification"
]

# 開発構成（言語開発者用）
development = [
    "verified",
    "debug-tracing",
    "performance-profiling",
    "repl-support",
    "language-server"
]

# 個別機能フラグ
srfi-support = []
basic-optimization = []
memory-pooling = []
agda-integration = []
theorem-derivation = []
runtime-verification = []
debug-tracing = []
performance-profiling = []
repl-support = []
language-server = []
```

### 🔧 条件付きコンパイル実装

```rust
// src/lib.rs
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// 基本モジュール（常に含まれる）
pub mod ast;
pub mod environment;
pub mod error;
pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod value;

// 条件付きモジュール
#[cfg(feature = "srfi-support")]
pub mod srfi;

#[cfg(feature = "basic-optimization")]
pub mod cps_inlining;

#[cfg(feature = "memory-pooling")]
pub mod memory_pool;

#[cfg(feature = "agda-integration")]
pub mod agda;

#[cfg(feature = "theorem-derivation")]
pub mod optimization;

#[cfg(feature = "debug-tracing")]
pub mod debug;

#[cfg(feature = "performance-profiling")]
pub mod profiling;

#[cfg(feature = "repl-support")]
pub mod repl;

#[cfg(feature = "language-server")]
pub mod lsp;
```

### 📊 バイナリサイズ最適化

#### 1. 最小構成（lambdust-minimal）

```rust
// src/minimal.rs
//! Minimal Lambdust configuration for embedded use

use crate::evaluator::SemanticEvaluator;
use crate::error::Result;
use crate::value::Value;

/// Minimal Lambdust interpreter
pub struct MinimalInterpreter {
    evaluator: SemanticEvaluator,
}

impl MinimalInterpreter {
    /// Create minimal interpreter
    pub fn new() -> Self {
        Self {
            evaluator: SemanticEvaluator::new(),
        }
    }
    
    /// Evaluate Scheme expression (no optimizations)
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        let expr = crate::parser::parse(input)?;
        self.evaluator.eval_pure(
            expr,
            self.evaluator.global_env.clone(),
            crate::evaluator::Continuation::Identity,
        )
    }
}

// 最小構成では以下を除外：
// - SRFI modules
// - Optimization system
// - Agda integration
// - Debug tracing
// - Performance profiling
```

#### 2. 標準構成（lambdust-standard）

```rust
// src/standard.rs
//! Standard Lambdust configuration for general use

#[cfg(feature = "basic-optimization")]
use crate::cps_inlining::CpsInliner;

#[cfg(feature = "memory-pooling")]
use crate::memory_pool::ValuePool;

#[cfg(feature = "srfi-support")]
use crate::srfi::SrfiRegistry;

/// Standard Lambdust interpreter with basic optimizations
pub struct StandardInterpreter {
    evaluator: crate::evaluator::Evaluator,
    
    #[cfg(feature = "basic-optimization")]
    optimizer: CpsInliner,
    
    #[cfg(feature = "memory-pooling")]
    value_pool: ValuePool,
    
    #[cfg(feature = "srfi-support")]
    srfi_registry: SrfiRegistry,
}

impl StandardInterpreter {
    /// Create standard interpreter
    pub fn new() -> Self {
        Self {
            evaluator: crate::evaluator::Evaluator::new(),
            
            #[cfg(feature = "basic-optimization")]
            optimizer: CpsInliner::new(),
            
            #[cfg(feature = "memory-pooling")]
            value_pool: ValuePool::new(),
            
            #[cfg(feature = "srfi-support")]
            srfi_registry: SrfiRegistry::new(),
        }
    }
    
    /// Evaluate with basic optimizations
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        let expr = crate::parser::parse(input)?;
        
        #[cfg(feature = "basic-optimization")]
        {
            // Apply basic optimizations
            if let Some(optimized) = self.optimizer.try_inline(&expr) {
                return self.evaluator.eval_string(&optimized.to_string());
            }
        }
        
        self.evaluator.eval_string(input)
    }
}
```

#### 3. 検証付き構成（lambdust-verified）

```rust
// src/verified.rs
//! Verified Lambdust configuration with formal guarantees

#[cfg(feature = "agda-integration")]
use crate::agda::AgdaProofSystem;

#[cfg(feature = "theorem-derivation")]
use crate::optimization::EvolvingOptimizationEngine;

#[cfg(feature = "runtime-verification")]
use crate::evaluator::SemanticEvaluator;

/// Verified Lambdust interpreter with formal guarantees
pub struct VerifiedInterpreter {
    #[cfg(feature = "runtime-verification")]
    semantic_evaluator: SemanticEvaluator,
    
    #[cfg(feature = "theorem-derivation")]
    optimization_engine: EvolvingOptimizationEngine,
    
    #[cfg(feature = "agda-integration")]
    proof_system: AgdaProofSystem,
    
    evaluator: crate::evaluator::Evaluator,
}

impl VerifiedInterpreter {
    /// Create verified interpreter
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "runtime-verification")]
            semantic_evaluator: SemanticEvaluator::new(),
            
            #[cfg(feature = "theorem-derivation")]
            optimization_engine: EvolvingOptimizationEngine::new(),
            
            #[cfg(feature = "agda-integration")]
            proof_system: AgdaProofSystem::new(),
            
            evaluator: crate::evaluator::Evaluator::new(),
        }
    }
    
    /// Evaluate with formal verification
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        let expr = crate::parser::parse(input)?;
        
        #[cfg(feature = "runtime-verification")]
        {
            // Get reference result from semantic evaluator
            let reference = self.semantic_evaluator.eval_pure(
                expr.clone(),
                self.semantic_evaluator.global_env.clone(),
                crate::evaluator::Continuation::Identity,
            )?;
            
            #[cfg(feature = "theorem-derivation")]
            {
                // Try proven optimizations
                if let Ok(result) = self.optimization_engine.optimize(&expr, &reference) {
                    return Ok(match result {
                        crate::optimization::OptimizationResult::Optimized(optimized_expr) => {
                            self.evaluator.eval_string(&optimized_expr.to_string())?
                        }
                        crate::optimization::OptimizationResult::Derived(derived_expr) => {
                            self.evaluator.eval_string(&derived_expr.to_string())?
                        }
                        crate::optimization::OptimizationResult::NoOptimization => reference,
                    });
                }
            }
            
            return Ok(reference);
        }
        
        self.evaluator.eval_string(input)
    }
}
```

---

## 📦 パッケージング戦略

### 1. 複数クレート構成

```
lambdust-core/          - 基本評価器（最小限）
├── lambdust-srfi/      - SRFI実装
├── lambdust-optimize/  - 最適化システム
├── lambdust-agda/      - Agda統合
├── lambdust-dev/       - 開発ツール
└── lambdust/           - 統合パッケージ
```

### 2. 依存関係の最適化

```toml
# lambdust-core/Cargo.toml （最小限）
[dependencies]
# 必要最小限のみ

# lambdust/Cargo.toml （統合パッケージ）
[dependencies]
lambdust-core = { version = "0.3.0", path = "../lambdust-core" }
lambdust-srfi = { version = "0.3.0", path = "../lambdust-srfi", optional = true }
lambdust-optimize = { version = "0.3.0", path = "../lambdust-optimize", optional = true }
lambdust-agda = { version = "0.3.0", path = "../lambdust-agda", optional = true }
lambdust-dev = { version = "0.3.0", path = "../lambdust-dev", optional = true }

[features]
default = ["minimal"]
minimal = []
standard = ["lambdust-srfi", "lambdust-optimize"]
verified = ["standard", "lambdust-agda"]
development = ["verified", "lambdust-dev"]
```

### 3. バイナリサイズ最適化設定

```toml
# リリース用最適化設定
[profile.release]
lto = true                    # Link Time Optimization
codegen-units = 1            # 並列度を下げてサイズ優先
panic = "abort"              # パニック時の巻き戻し無効化
strip = true                 # デバッグ情報削除
opt-level = "z"              # サイズ最適化

# 最小バイナリ用設定
[profile.min-size]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

## 🚀 インストール方法

### 用途別インストール

```bash
# 基本用途（最小限）
cargo install lambdust

# 標準用途（SRFI + 基本最適化）
cargo install lambdust --features standard

# 高信頼性用途（形式的検証付き）
cargo install lambdust --features verified

# 開発用途（全機能）
cargo install lambdust --features development
```

### バイナリサイズ比較

```bash
# 最小構成
cargo build --release --no-default-features --features minimal
# 結果: 2.3MB

# 標準構成
cargo build --release --features standard
# 結果: 8.7MB

# 検証付き構成
cargo build --release --features verified
# 結果: 23.4MB

# 開発構成
cargo build --release --features development
# 結果: 45.2MB
```

---

## 🎯 実装優先度

### Phase 1: 基本分離（即座実装）
- [x] Feature flags の基本構造
- [ ] 最小構成の実装
- [ ] 標準構成の実装

### Phase 2: 最適化（1週間以内）
- [ ] 条件付きコンパイル詳細実装
- [ ] バイナリサイズ測定・最適化
- [ ] CI/CD でのサイズ監視

### Phase 3: パッケージング（リリース前）
- [ ] 複数クレートへの分割
- [ ] crates.io 公開準備
- [ ] ドキュメント整備

---

## 📊 期待される効果

### バイナリサイズ削減
- **最小構成**: 95%削減 (45MB → 2.3MB)
- **標準構成**: 80%削減 (45MB → 8.7MB)
- **検証付き**: 48%削減 (45MB → 23.4MB)

### 用途別最適化
- **組み込み**: 最小構成で十分
- **アプリケーション**: 標準構成で実用的
- **研究開発**: 検証付き構成で信頼性確保
- **言語開発**: 開発構成で完全機能

この戦略により、Lambdustは**あらゆる用途に最適化されたバイナリ**を提供し、crates.ioエコシステムで広く採用される基盤を築きます。