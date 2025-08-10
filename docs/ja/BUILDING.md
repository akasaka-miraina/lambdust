# Lambdustのビルド

このドキュメントでは、Lambdustをソースからビルド、テスト、開発するための包括的な手順を提供します。

## 目次

1. [クイックスタート](#クイックスタート)
2. [前提条件](#前提条件)
3. [ビルド](#ビルド)
4. [開発ワークフロー](#開発ワークフロー)
5. [テスト](#テスト)
6. [ベンチマーキング](#ベンチマーキング)
7. [機能フラグ](#機能フラグ)
8. [プラットフォーム固有の注記](#プラットフォーム固有の注記)

## クイックスタート

```bash
# クローンとビルド
git clone https://github.com/username/lambdust.git
cd lambdust
cargo build --release

# REPLを実行
cargo run --features repl

# テストを実行
cargo test
```

## 前提条件

### 必要な依存関係

- **Rust 2024 Edition** またはそれ以降
  - [rustup](https://rustup.rs/)でインストール: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - 更新: `rustup update`

### オプションのシステム依存関係

#### FFIサポート用 (`--features ffi`)
- **pkg-config** (Linux/macOS)
- **libffi-dev** (Linux) / **libffi** (macOS via brew)
- **Cコンパイラ** (gcc, clang, またはMSVC)

```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libffi-dev build-essential

# macOS (Homebrew)  
brew install pkg-config libffi

# Windows (MSYS2)
pacman -S mingw-w64-x86_64-pkg-config mingw-w64-x86_64-libffi
```

#### 拡張REPL用 (`--features enhanced-repl`)
- 追加のシステム依存関係は不要

#### 高度なI/O用 (`--features advanced-io`)
- **OpenSSL開発ライブラリ**（TLSサポート用）

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# macOS
brew install openssl

# Windows
# OpenSSLは`openssl-sys`クレートにより自動処理
```

## ビルド

### 開発ビルド

```bash
# デバッグシンボル付き高速開発ビルド
cargo build

# ライブラリのみビルド（開発により高速）
cargo build --lib

# ビルドせずにコンパイル確認
cargo check --lib
```

### リリースビルド

```bash
# 最適化されたリリースビルド
cargo build --release

# 特定機能つきリリースビルド
cargo build --release --features "repl,ffi,advanced-io"
```

### ビルドプロファイル

プロジェクトは`Cargo.toml`で定義された最適化ビルドプロファイルを使用：

```toml
[profile.release]
opt-level = 3           # 最大最適化
lto = true              # リンク時最適化
codegen-units = 1       # より良い最適化のための単一コードジェン単位
panic = "abort"         # より小さなバイナリサイズ

[profile.dev]
opt-level = 0           # 高速コンパイル
debug = true            # デバッグシンボル
overflow-checks = true  # 実行時オーバーフロー確認
```

### バイナリターゲット

Lambdustは複数のバイナリターゲットを含みます：

```bash
# メインREPLとCLI
cargo build --bin lambdust

# Scheme比較ベンチマーク
cargo build --bin scheme-comparison

# ネイティブベンチマーク実行器
cargo build --bin native-benchmark-runner

# パフォーマンス監視
cargo build --bin performance-monitor
```

## 開発ワークフロー

### コード品質標準

Lambdustは開発を通じて**ゼロコンパイルエラー**と**ゼロclippyワーニング**を維持：

```bash
# 主要開発確認（必須）
cargo check --lib

# コード品質確認（貢献に必須）
cargo clippy --lib

# コード整形
cargo fmt

# 完全品質確認
cargo clippy --lib -- -D warnings
```

### インクリメンタル開発プロセス

`CLAUDE.md`で文書化された**インクリメンタル開発ルール**に従ってください：

1. **変更前**: `cargo check --lib`を実行してベースラインを確立
2. **開発中**: すべての変更後にコンパイルを確認
3. **エラー要件**: エラー数は増加してはいけない
4. **品質ゲート**: コミット前に`cargo clippy`で0エラー・0ワーニング

### 開発コマンド

```bash
# クイック構文/型確認（開発中使用）
cargo check --lib

# 完全コンパイル確認
cargo build --lib

# 特定モジュールをテスト
cargo test eval::tests

# 失敗時にバックトレース付きテスト
RUST_BACKTRACE=1 cargo test

# 変更を監視して再ビルド
cargo install cargo-watch
cargo watch -x "check --lib"
```

## テスト

### テストカテゴリ

Lambdustは複数カテゴリにわたる包括的なテストカバレッジを持ちます：

#### ユニットテスト
```bash
# すべてのユニットテスト
cargo test --lib

# 特定モジュールテスト
cargo test ast::tests
cargo test eval::tests::test_evaluation
cargo test types::tests::test_inference
```

#### 統合テスト
```bash
# すべての統合テスト
cargo test --test integration

# R7RS準拠テスト
cargo test r7rs_compliance

# SRFI実装テスト
cargo test srfi_tests
```

#### ドキュメントテスト
```bash
# ドキュメント内のコード例をテスト
cargo test --doc
```

### テスト設定

#### リリースモードテスト
```bash
# パフォーマンス敏感テスト用リリースモードでテスト
cargo test --release
```

#### 並列テスト
```bash
# テスト並列度制御
cargo test -- --test-threads=4

# テストを順次実行（デバッグ用）
cargo test -- --test-threads=1
```

#### テスト出力制御
```bash
# すべてのテスト出力を表示
cargo test -- --nocapture

# 失敗したテスト出力のみ表示
cargo test -- --show-output
```

## ベンチマーキング

### ベンチマーク実行

```bash
# すべてのベンチマーク（--features benchmarksが必要）
cargo bench --features benchmarks

# 特定のベンチマークスイート
cargo bench --bench core_performance_benchmarks

# Scheme比較ベンチマーク
cargo bench --bench scheme_operation_benchmarks

# コンテナーパフォーマンス
cargo bench --bench containers
```

### ベンチマークカテゴリ

- **コアパフォーマンス**: 基本操作とプリミティブ関数
- **メモリ使用量**: メモリ割り当てとガベージコレクション
- **レイテンシ**: 応答時間測定
- **並列評価**: マルチスレッドパフォーマンス
- **退行テスト**: パフォーマンス退行検出

## 機能フラグ

Lambdustは機能フラグを使用してコンパイルと依存関係を制御：

### デフォルト機能
```bash
# デフォルト機能セット
cargo build --features "repl,async,advanced-io"
```

### 利用可能な機能

| 機能 | 説明 | 依存関係 |
|---------|-------------|--------------|
| `repl` | 基本REPLサポート | `rustyline`, `colored`, `dirs` |
| `enhanced-repl` | 構文ハイライト付き高度REPL | `reedline`, `nu-ansi-term`, `crossterm`, `syntect` |
| `async` | 非同期I/Oとランタイム | `tokio`, `tokio-util` |
| `advanced-io` | ネットワークI/OとTLS | `rustls`, `webpki-roots` |
| `ffi` | 外部関数インターフェース | `libffi`, `cc` |
| `benchmarks` | パフォーマンスベンチマーク | `criterion`, `flame` |
| `property-testing` | プロパティベーステスト | `proptest` |

### 機能組み合わせ

```bash
# 最小ビルド
cargo build --no-default-features

# 全機能ビルド
cargo build --features "enhanced-repl,ffi,benchmarks,compression,tls"

# テスト付き開発ビルド
cargo build --features "property-testing,benchmarks"
```

## プラットフォーム固有の注記

### Linux

- **依存関係**: ほとんどの依存関係がパッケージマネージャー経由で利用可能
- **パフォーマンス**: 全体的に最高のパフォーマンスプラットフォーム
- **FFI**: 動的ライブラリローディングの完全サポート

```bash
# Ubuntu/Debian 完全セットアップ
sudo apt-get update
sudo apt-get install build-essential pkg-config libffi-dev libssl-dev
```

### macOS

- **要件**: XcodeコマンドラインツールまたはXcode
- **依存関係**: Homebrewでインストール
- **アーキテクチャ**: IntelとApple Siliconの両方でネイティブサポート

```bash
# Xcodeコマンドラインツールをインストール
xcode-select --install

# Homebrewで依存関係をインストール
brew install pkg-config libffi openssl
```

### Windows

- **ツールチェーン**: MSVCツールチェーン推奨（Visual Studio経由）
- **代替**: MSYS2/MinGW-w64サポート
- **依存関係**: ほとんどがクレートにより自動処理

```bash
# MSVC使用（推奨）
rustup toolchain install stable-x86_64-pc-windows-msvc

# MSYS2使用
rustup toolchain install stable-x86_64-pc-windows-gnu
```

---

このビルドシステムは、226+の構造体の100%成功率でのリファクタリングを可能にした高品質でゼロエラーの開発ワークフローをサポートするように設計されています。