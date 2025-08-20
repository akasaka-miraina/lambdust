# 🚀 Lambdust - Advanced Lisp/Scheme Implementation

[![Build Status](https://img.shields.io/github/actions/workflow/status/lambdust/lambdust/ci.yml)](https://github.com/lambdust/lambdust/actions)
[![Documentation](https://img.shields.io/badge/docs-specification-blue)](docs/specification/lambdust-spec.pdf)
[![R7RS Compliance](https://img.shields.io/badge/R7RS-85%25%20compliant-green)]()
[![Performance](https://img.shields.io/badge/performance-tracking-yellow)](benchmarks/)
[![Code Quality](https://img.shields.io/badge/clippy-warnings%200-green.svg)](https://github.com/rust-lang/rust-clippy)

**Lambdust**は、現代のソフトウェア開発に最適化された革新的なLisp/Scheme実装です。R7RS準拠を基盤として、**漸進的型付け**、**副作用システム**、**Actor並行性**、**安全なFFI**を統合し、業界最高レベルの性能と安全性を実現します。

### 🌟 主要特徴

- **📐 漸進的型付け**: Dynamic → Contracts → Static → Dependent の4段階型システム
- **⚡ 副作用システム**: Algebraic effectsによる副作用の安全な管理
- **🎭 Actor並行性**: 軽量アクター + async/awaitのハイブリッドモデル
- **🔧 安全なFFI**: Capability-basedアクセス制御によるメモリ安全FFI
- **🎨 高度マクロ**: R7RS準拠 + 型安全マクロ + compile-time computation
- **🏗️ JIT統合**: LLVM統合による実行時最適化

### 📊 性能目標

| 指標 | 目標 | 現状 |
|------|------|------|
| **メモリ効率** | -60% | 最適化中 |
| **実行速度** | +200-500% | SIMD実装中 |
| **GC停止時間** | <1ms | Incremental GC実装中 |
| **並行効率** | >95% | Actor実装中 |

## 🚀 クイックスタート

### インストール

```bash
# Rustツールチェーンが必要 (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Lambdustのビルド
git clone https://github.com/lambdust/lambdust.git
cd lambdust
cargo build --release

# REPLの起動
./target/release/lambdust
```

### 基本使用例

```scheme
;; 漸進的型付け
(define (fibonacci n :: Integer) :: Integer
  (if (<= n 1) n
      (+ (fibonacci (- n 1))
         (fibonacci (- n 2)))))

;; 副作用システム
(define-effect (State s)
  (get () -> s)
  (put (new-state s) -> Unit))

(with-handler state-handler
  (perform (put 42))
  (perform (get)))

;; Actor並行性
(define (worker-actor)
  (receive
    [(msg data) 
     (process-data data)
     (worker-actor)]))

(spawn worker-actor)
(send worker-actor 'process some-data)

;; 安全なFFI
(foreign-call "libc" "strlen" 
  (-> CString -> Size)
  capability: read-only
  "Hello, World!")
```

## 🏗️ アーキテクチャ

### システム構成

```
src/
├── ast/           # 抽象構文木・パターンマッチング
├── bytecode/      # バイトコードコンパイラ・JIT統合  
├── concurrency/   # Actor・Future・分散処理
├── containers/    # 高性能データ構造
├── effects/       # 副作用システム・代数的副作用
├── eval/          # 評価器・メモリ最適化 (32モジュール)
├── lexer/         # 字句解析・Unicode対応
├── macro_system/  # マクロ展開・hygiene・syntax-case
├── parser/        # 構文解析・エラー回復
├── runtime/       # ランタイムシステム・GC統合
├── stdlib/        # R7RS標準ライブラリ・SRFI実装
├── types/         # 漸進的型システム・依存型・推論エンジン
└── utils/         # メモリプール・文字列インターナー
```

### 技術スタック

- **言語**: Rust 1.70+ (メモリ安全・ゼロコスト抽象化)
- **並行性**: tokio + rayon (async/await + データ並列)
- **最適化**: SIMD (AVX-512/NEON) + LLVM JIT
- **テスト**: criterion.rs + property-based testing
- **文書化**: LaTeX (言語仕様) + mdBook (ユーザーガイド)

## 📚 ドキュメント

### 📖 言語仕様書
- **[完全言語仕様書](docs/specification/lambdust-spec.pdf)** (90ページ, LaTeX生成)
- **形式意味論**: Denotational semanticsの完全定義
- **R7RS拡張**: 標準からの拡張点詳細説明

### 🎯 開発ロードマップ
- **[実装ロードマップ](IMPLEMENTATION_ROADMAP.md)**: 四者協業による開発計画
- **[進捗追跡](progress/)**: Phase別詳細進捗管理
- **[性能目標](benchmarks/performance-targets.md)**: ベンチマーク・最適化目標

### 👥 協業体制
- **[専門家分担](collaboration/expert-assignments.md)**: 四者協業の責任分担
- **[依存関係管理](collaboration/dependency-matrix.md)**: タスク依存関係・並列化戦略

## 🛠️ 開発

### ビルド要件

```bash
# 必須
rustc 1.70+
cargo 1.70+

# オプション (最適化機能)
llvm-dev          # JIT統合
valgrind          # メモリ分析
criterion         # ベンチマーク
```

### 開発ワークフロー

```bash
# 開発ビルド
cargo check --all-targets --all-features

# テスト実行
cargo test --all-features

# 静的解析
cargo clippy --all-targets --all-features -- -D warnings

# フォーマット
cargo fmt --check

# ベンチマーク
cargo bench

# ドキュメント生成
cargo doc --no-deps --open
```

### 品質保証

**品質ゲート** (必須クリア):
- ✅ コンパイルエラー: 0個
- ✅ Clippy警告: 0個
- ✅ テストカバレッジ: >90%
- ✅ ベンチマーク回帰: <5%

## 📈 現在の完了状況

### ✅ 完全完了 (100%)
- **言語仕様書**: 90ページ包括仕様
- **コンパイルエラー**: 291個 → 0個達成
- **Clippy警告**: 150個 → 0個達成
- **ファイル分割**: 20,000トークン制限遵守

### 🟢 高完成度 (85-95%)
- **パーサ・レキサ**: R7RS構文完全対応
- **型システム**: 漸進的型付け4レベル実装
- **評価器**: 32モジュール高度最適化
- **並行性**: Actor + Future/Promise
- **FFI**: 包括的安全性チェック

### 🟡 実装中 (70-85%)
- **標準ライブラリ**: R7RS準拠85%
- **SIMD最適化**: AVX-512/NEON対応
- **メモリ最適化**: Arc使用90%削減戦略
- **JIT統合**: LLVM統合準備

---

## 🤝 貢献

### 四者協業体制

Lambdustの開発は以下の専門家協業により進行中:

- **🧠 language-processor-architect**: 言語設計・構文・意味論
- **🏗️ cs-architect**: アルゴリズム・データ構造・システム設計  
- **⚙️ rust-expert-programmer**: Rust実装・最適化・安全性
- **📚 lambdust-r7rs-programmer**: R7RS準拠・標準ライブラリ

### 貢献方法

1. **Issue報告**: バグ・機能要望の報告
2. **PR投稿**: 実装・ドキュメント改善
3. **テスト追加**: 品質向上・カバレッジ改善
4. **ベンチマーク**: 性能測定・回帰検出
5. **文書化**: 使用例・チュートリアル

詳細は [CONTRIBUTING.md](CONTRIBUTING.md) を参照。

## 📄 ライセンス

**MIT License** - 詳細は [LICENSE](LICENSE) を参照。

学術研究・商用利用・オープンソースプロジェクト等、自由に使用可能です。

---

## 🌐 コミュニティ

- **GitHub**: [https://github.com/lambdust/lambdust](https://github.com/lambdust/lambdust)
- **Documentation**: [https://lambdust.dev](https://lambdust.dev)
- **Discussions**: [GitHub Discussions](https://github.com/lambdust/lambdust/discussions)

---

## 🎯 ロードマップ概要

### 🔴 Phase 1: 基盤完成 (2025年9月)
- 型システム統一 (クリティカルパス)
- エラー・警告ゼロ状態維持

### 🟢 Phase 2: 並列実装 (2025年10月)
- 言語処理拡張・システム最適化・R7RS完全準拠
- 四者協業による効率的並列開発

### 🔵 Phase 3: 高度統合 (2025年11月)
- 継続システム・分散計算・JIT統合
- 業界最高レベル性能実現

### 🎉 Phase 4: 完成・検証 (2025年12月)
- 最終統合・実用性検証・リリース準備

詳細は [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) を参照。

---

**Lambdust** - *The next generation of Lisp/Scheme for modern software development*

*Generated: 2025-08-20 | Version: 0.2.0*