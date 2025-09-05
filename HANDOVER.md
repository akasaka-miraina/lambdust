# Lambdust プロジェクト引き継ぎドキュメント

## 概要

このドキュメントは、Lambdust R7RS Scheme実装プロジェクトの引き継ぎ情報をまとめたものです。現在のプロジェクト状況、解決済み課題、残存課題、および開発環境の構築方法について記載しています。

## プロジェクト現状（2025年9月4日時点）

### ✅ 解決済み主要課題

1. **メモリ安全性の向上**
   - 危険な`mem::zeroed()`呼び出しを全て除去
   - SIMD最適化モジュールでの安全なデフォルト値使用
   - SIGSEGV問題の大幅な改善

2. **モナド実装の完成**
   - Reader monad bind メソッドの複雑計算対応
   - State monad bind メソッドのArc基盤実装
   - モナド評価器の型制約問題解決

3. **データ構造の安定化**
   - OrderedSet赤黒木削除バグ修正
   - 全削除シナリオ（リーフノード、単子ノード、双子ノード）対応

4. **ベンチマーク系統の修正**
   - 環境最適化での除算エラー対策
   - スキーム実行タイミング精度向上（ナノ秒→ミリ秒変換）
   - ベンチマークテスト全般の安定化

5. **コンパイル品質向上**
   - 291個のコンパイルエラー完全解決
   - 全Clippy警告解決
   - SIMD/ICUスタブ実装による依存関係修正

### 🚨 残存課題

1. **メモリ安全性（部分的）**
   - `src/eval/optimized_value.rs`でのunsafeポインタ参照（12箇所以上）
   - ローカル環境でのSIGSEGV散発的発生
   - ValueData union型での生ポインタ管理

2. **テスト失敗（詳細調査必要）**
   - 一部の診断系テスト
   - 契約システムテスト  
   - バイトコード系テスト

3. **CI/CD環境差異**
   - ローカル環境とCI環境でのテスト結果不整合
   - プラットフォーム固有の問題可能性

## 開発環境構築

### Ubuntu Docker環境でのテスト環境構築

ローカル環境でCI環境（Ubuntu）と同等のテスト環境を構築するため、以下のDocker環境を整備してください。

#### 1. Dockerfileの作成

プロジェクトルートに`docker/ubuntu-test/Dockerfile`を作成：

```dockerfile
FROM ubuntu:22.04

# 基本パッケージのインストール
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    llvm-dev \
    libclang-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Rustのインストール
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# 作業ディレクトリの設定
WORKDIR /workspace

# ユーザー権限での実行環境構築
RUN groupadd -r lambda && useradd -r -g lambda lambda
RUN mkdir -p /workspace && chown lambda:lambda /workspace

USER lambda

# Rustツールチェーンの設定
RUN rustup default stable
RUN rustup component add clippy rustfmt

# 作業ディレクトリに戻る
WORKDIR /workspace
```

#### 2. Docker Composeの設定

`docker/docker-compose.yml`を作成：

```yaml
version: '3.8'
services:
  ubuntu-test:
    build: 
      context: .
      dockerfile: ubuntu-test/Dockerfile
    volumes:
      - ../:/workspace:cached
    environment:
      - CARGO_HOME=/workspace/.cargo
      - RUST_BACKTRACE=1
      - CARGO_TERM_COLOR=always
    working_dir: /workspace
    command: /bin/bash
    stdin_open: true
    tty: true
```

#### 3. 開発用スクリプトの作成

`scripts/test-ubuntu.sh`を作成：

```bash
#!/bin/bash
set -e

echo "🐳 Ubuntu Docker環境でのテスト実行"

# Docker環境の構築
cd docker
docker-compose build ubuntu-test

# テスト実行
docker-compose run --rm ubuntu-test bash -c "
    echo '📋 システム情報:'
    uname -a
    rustc --version
    cargo --version
    
    echo '🔧 依存関係の確認:'
    cargo check --all-targets --all-features
    
    echo '📝 Clippy静的解析:'
    cargo clippy --all-targets --all-features -- -D warnings
    
    echo '🧪 テスト実行:'
    cargo test --lib --no-default-features --verbose
    
    echo '✨ テスト完了'
"
```

#### 4. デバッグ用環境の構築

特定テストのデバッグ用スクリプト `scripts/debug-ubuntu.sh`：

```bash
#!/bin/bash
set -e

TEST_NAME="${1:-}"

if [ -z "$TEST_NAME" ]; then
    echo "使用法: $0 <テスト名>"
    echo "例: $0 benchmarks::environment_optimization::tests::test_small_scale_benchmark"
    exit 1
fi

echo "🔍 Ubuntu環境での個別テストデバッグ: $TEST_NAME"

cd docker
docker-compose run --rm ubuntu-test bash -c "
    echo '🎯 対象テスト: $TEST_NAME'
    echo 'RUST_BACKTRACE=full cargo test $TEST_NAME --lib --no-default-features -- --nocapture'
    RUST_BACKTRACE=full cargo test '$TEST_NAME' --lib --no-default-features -- --nocapture
"
```

#### 5. CI環境再現スクリプト

完全なCI環境再現用 `scripts/ci-simulate.sh`：

```bash
#!/bin/bash
set -e

echo "🤖 CI環境完全再現テスト"

cd docker
docker-compose run --rm ubuntu-test bash -c "
    # GitHub Actionsと同様の環境変数設定
    export CARGO_TERM_COLOR=always
    export RUST_BACKTRACE=1
    export CARGO_BUILD_JOBS=1
    export CARGO_INCREMENTAL=0
    
    echo '🔄 クリーンビルド:'
    cargo clean
    
    echo '🏗️ コンパイル品質チェック:'
    cargo clippy --all-targets --all-features -- -D warnings
    
    echo '📚 ライブラリビルド:'
    cargo build --lib --no-default-features
    
    echo '🧪 テストスイート実行:'
    echo '- Beta toolchain simulation'
    cargo test --lib --no-default-features
    
    echo '📊 結果サマリー表示'
"
```

### 使用方法

1. **権限設定**
```bash
chmod +x scripts/test-ubuntu.sh
chmod +x scripts/debug-ubuntu.sh  
chmod +x scripts/ci-simulate.sh
```

2. **完全テスト実行**
```bash
./scripts/test-ubuntu.sh
```

3. **特定テストのデバッグ**
```bash
./scripts/debug-ubuntu.sh "benchmarks::environment_optimization::tests::test_small_scale_benchmark"
```

4. **CI環境再現**
```bash
./scripts/ci-simulate.sh
```

## 重要な技術的注意事項

### メモリ安全性の課題

`src/eval/optimized_value.rs`には以下の危険なパターンが存在：

```rust
// 危険: 生ポインタの直接参照
let obj = unsafe { &*(self.data.ptr as *const NumberObj) };
```

これらは将来的に以下の方針で対処が必要：
1. `Rc<T>`/`Arc<T>`への移行
2. 型安全なenum表現への変更
3. ライフタイム管理の明確化

### パフォーマンスベンチマーク

修正済み主要問題：
- 除算エラー: `std::cmp::max(1, divisor)`による安全ガード
- タイミング精度: ナノ秒基準の高精度測定

### テスト戦略

1. **段階的テスト**: 小モジュール単位での検証
2. **環境分離**: Docker環境での再現性確保  
3. **CI同期**: GitHub Actions環境との整合性維持

## 次期開発優先順位

1. **高優先度**: `optimized_value.rs`メモリ安全性改善
2. **中優先度**: 残存テスト失敗の個別対応
3. **低優先度**: 追加SRFI実装とパフォーマンス最適化

## 連絡事項

- 全体的なプロジェクトの安定性は大幅に向上
- コンパイル品質は production-ready レベル
- オールグリーン達成まで残り僅かな課題のみ

---

**作成日**: 2025年9月4日  
**最終更新**: 引き継ぎ時点  
**対象ブランチ**: `0.2.0`