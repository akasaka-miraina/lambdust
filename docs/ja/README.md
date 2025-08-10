# Lambdust (λust) - モダンなScheme処理系：段階的型付けとエフェクトシステム

[![Rust](https://img.shields.io/badge/rust-2024%20edition-blue)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#ライセンス)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#ビルド)

Schemeのシンプルさと美しさを保ちながら、段階的型付け、エフェクトシステム、高性能実装などの先進的なプログラミング言語機能を組み合わせたモダンなScheme処理系です。

## ✨ 特徴

### 🎯 **コア言語機能**
- **R7RS-large準拠** 豊富なSRFI対応
- **完全に健全なマクロシステム** R7RS互換のsyntax-rules
- **末尾呼び出し最適化** と適切な字句スコープ
- **42のコアプリミティブ** でシステム全体をブートストラップ

### 🏗️ **先進的な型システム**
- **4段階の段階的型付け**: 動的 → 契約 → 静的 → 依存型
- **Hindley-Milner型推論** ランクn多相型対応
- **代数的データ型** とパターンマッチング
- **型クラス** Haskell風の制約
- **行多相** 拡張可能なレコード

### 🎭 **エフェクトシステム**  
- **透過的なエフェクト追跡** Schemeの意味論を保持
- **モナディックプログラミング** IO、State、Error、カスタムエフェクト
- **エフェクトハンドラー** カスタムエフェクト管理
- **世代環境** ミューテーションの関数型処理

### ⚡ **パフォーマンスと並行性**
- **バイトコードコンパイル** 最適化パス付き
- **マルチスレッド並列評価** アクターモデル
- **ロックフリーデータ構造** STM対応
- **SIMD最適化** 数値演算向け
- **プロファイル誘導最適化** JITコンパイル対応

### 🔗 **相互運用性**
- **外部関数インターフェース** C/Rust相互運用
- **動的ライブラリローディング** 型安全バインディング
- **包括的I/Oシステム** async/await、ネットワーク操作
- **モジュールシステム** R7RS互換ライブラリ

## 🚀 クイックスタート

### インストール

```bash
# リポジトリをクローン
git clone https://github.com/username/lambdust.git
cd lambdust

# プロジェクトをビルド
cargo build --release

# REPLを実行
cargo run --features repl
```

### Hello, World!

```scheme
;; 基本的なR7RS Scheme
(display "Hello, World!")
(newline)

;; 型注釈とエフェクト付き
(define (greet name)
  #:type (-> String (IO Unit))
  #:pure #f
  (display "Hello, ")
  (display name)
  (newline))

(greet "Lambdust")
```

### 高度な機能

```scheme
;; 段階的型付け - 動的から開始して段階的に型を追加
(define (factorial n)
  #:type (-> Number Number)
  #:contract (-> (and Number (>= 0)) Number)
  #:pure #t
  (if (zero? n) 1 (* n (factorial (- n 1)))))

;; モナディックプログラミングのエフェクトシステム
(define (safe-divide x y)
  #:type (-> Number Number (Either String Number))
  (if (zero? y)
      (Left "Division by zero")
      (Right (/ x y))))

;; アクターを使った並行プログラミング
(define counter-actor
  (actor
    [(initial-state 0)]
    [(increment n) (+ state n)]
    [(get) state]))

;; 代数的データ型のパターンマッチング
(define-type Maybe (a)
  Nothing
  (Just a))

(define (maybe-map f maybe-val)
  (match maybe-val
    [Nothing Nothing]
    [(Just x) (Just (f x))]))

;; Rust/C関数とのFFI
(define-ffi "libmath.so"
  [fast-sqrt (-> Number Number) "sqrt"])

(fast-sqrt 16) ; => 4.0
```

## 📁 プロジェクト構成

```
lambdust/
├── 🎯 コア実装
│   ├── src/lexer/              # トークン化と字句解析
│   ├── src/parser/             # 式の解析とAST生成
│   ├── src/ast/                # 抽象構文木定義
│   ├── src/eval/               # コア評価エンジンと環境
│   └── src/diagnostics/        # エラーハンドリングと報告
│
├── 🏗️ 言語システム  
│   ├── src/types/              # 4段階の段階的型システム
│   ├── src/effects/            # モナディックプログラミングのエフェクトシステム
│   ├── src/macro_system/       # 健全なマクロ展開
│   ├── src/module_system/      # R7RSモジュールシステムとライブラリローディング
│   └── src/metaprogramming/    # リフレクションとコード生成
│
├── ⚡ ランタイムとパフォーマンス
│   ├── src/runtime/            # ランタイムシステム協調  
│   ├── src/bytecode/           # バイトコードコンパイラと仮想マシン
│   ├── src/concurrency/        # アクターシステムと並列評価
│   ├── src/containers/         # 高性能データ構造
│   └── src/benchmarks/         # パフォーマンス分析と退行検出
│
├── 🔗 相互運用性
│   ├── src/ffi/                # 外部関数インターフェース
│   ├── src/stdlib/             # R7RS標準ライブラリ＋拡張
│   ├── src/numeric/            # 高度な数値タワー
│   └── src/utils/              # メモリ管理とプロファイリング
│
├── 🎮 ユーザーインターフェース
│   ├── src/repl/               # デバッグ機能付き拡張REPL
│   ├── src/main.rs             # CLIアプリケーション エントリーポイント
│   └── src/lib.rs              # ライブラリインターフェース
│
└── 📚 リソース
    ├── stdlib/                 # Scheme標準ライブラリモジュール
    ├── examples/               # サンプルプログラムとデモ
    ├── docs/                   # 包括的なドキュメント
    └── tests/                  # テストスイートとベンチマーク
```

## 🏛️ アーキテクチャの特徴

### 🧠 **クリーンなモジュラー設計** 
- **226+の構造体** one-structure-per-fileの原則で整理
- **ゼロコンパイルエラー** とゼロclippyワーニング維持
- **プロフェッショナルなドキュメント** すべてのパブリックインターフェース
- **インクリメンタル開発** 継続的品質保証

### 🎯 **型システム統合**
- **統合ブリッジ** 動的・静的型付けを seamlessly 接続  
- **段階的型付け** 型レベル間のスムーズな移行
- **R7RS統合** 後方互換性を維持

### 🎭 **エフェクトシステムアーキテクチャ**
- **エフェクト協調** 副作用とI/Oの透過的管理
- **モナディックアーキテクチャ** 包括的エフェクトハンドラー
- **世代環境** 状態変更の関数型処理

## 🏃‍♂️ はじめに

### 前提条件

- Rust 2024 editionまたは以降
- オプション機能用のシステム依存関係（[BUILDING.md](../BUILDING.md)を参照）

### 基本的な使い方

```bash
# インタラクティブREPL
cargo run --features enhanced-repl

# ファイルを実行
cargo run examples/fibonacci.scm

# 式を評価  
cargo run -- --eval "(map (lambda (x) (* x x)) '(1 2 3 4 5))"

# 型チェックを有効化
cargo run -- --type-level static examples/typed-program.scm

# 並列評価
cargo run --bin native-benchmark-runner -- --parallel 4
```

### 開発

```bash
# エラーチェック付き開発ビルド
cargo check --lib

# 全テスト実行
cargo test

# パフォーマンスベンチマーク  
cargo bench --features benchmarks

# コード品質（貢献に必要）
cargo clippy
cargo fmt
```

## 📊 パフォーマンス

Lambdustは複数の最適化戦略により優秀なパフォーマンスを実現：

- **バイトコードコンパイル** マルチパス最適化
- **プリミティブ特殊化** 型情報に基づく  
- **SIMDベクタ化** 数値演算用
- **メモリプーリング** 割り当て重いワークロード用
- **ロックフリー並行性** ワークスティーリングスケジューラー

詳細なベンチマークと最適化ガイドについては[PERFORMANCE.md](../PERFORMANCE.md)を参照。

## 🤝 貢献

貢献を歓迎します！以下については[CONTRIBUTING.md](../CONTRIBUTING.md)を参照：

- コード組織標準
- 開発ワークフローと品質要件
- テストガイドラインとドキュメント標準
- 提出プロセスとレビュー基準

### クイック貢献チェックリスト

1. ✅ `cargo check --lib` で0エラー
2. ✅ `cargo clippy` で0エラー・0ワーニング  
3. ✅ `cargo test` で全テストパス
4. ✅ 新機能のドキュメント更新
5. ✅ プルリクエストごとに1つの集中した変更

## 📚 ドキュメント

| ドキュメント | 説明 |
|----------|-------------|
| [ARCHITECTURE.md](../ARCHITECTURE.md) | システムアーキテクチャとコンポーネント設計 |
| [BUILDING.md](../BUILDING.md) | ビルド手順と開発環境構築 |  
| [API_REFERENCE.md](../API_REFERENCE.md) | コアAPI ドキュメントと例 |
| [TYPE_SYSTEM.md](../TYPE_SYSTEM.md) | 段階的型付けと型推論 |
| [EFFECT_SYSTEM.md](../EFFECT_SYSTEM.md) | エフェクトシステムとモナディックプログラミング |
| [CONCURRENCY.md](../CONCURRENCY.md) | 並行性モデルと並列評価 |
| [FFI.md](../FFI.md) | 外部関数インターフェースガイド |
| [PERFORMANCE.md](../PERFORMANCE.md) | パフォーマンス最適化とベンチマーキング |

## 🧪 テスト

信頼性を保証する包括的テスト戦略：

- **ユニットテスト** 95%+カバレッジを持つ各モジュール
- **統合テスト** エンドツーエンド機能
- **R7RS準拠テスト** 標準準拠を保証  
- **SRFI実装テスト** 拡張機能用
- **パフォーマンス退行テスト** 最適化の成果を維持
- **プロパティベーステスト** ランダム入力

```bash
cargo test                    # 全テスト
cargo test --release          # リリースモードテスト  
cargo test integration::      # 統合テストのみ
cargo bench                   # パフォーマンスベンチマーク
```

## 🔖 バージョン情報

- **言語バージョン**: 0.1.0
- **R7RS準拠**: 完全なR7RS-largeサポート
- **Rust Edition**: 2024
- **SRFIサポート**: 50+のSRFI実装

## 📄 ライセンス

以下のいずれかのライセンスで提供：

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) または http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) または http://opensource.org/licenses/MIT)

お好みの方を選択してください。

## 🌟 謝辞

- Scheme仕様のR7RS作業部会
- 優秀なツールとライブラリのRustコミュニティ
- 型理論とエフェクトシステムの学術研究
- この作業にインスピレーションを与えたオープンソースScheme実装

## 🎯 今後のロードマップ

- **依存型** 証明アシスタント機能付き
- **JITコンパイル** パフォーマンスクリティカルなコードパス用  
- **分散コンピューティング** 透過的なリモート評価
- **IDE統合** Language Server Protocolサポート
- **WebAssemblyターゲット** ブラウザベース実行用

---

*Lambdust: Lispの美しさと現代的型理論が出会う場所* ✨