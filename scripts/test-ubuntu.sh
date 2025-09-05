#!/bin/bash
set -e

echo "🐳 Ubuntu Docker環境でのテスト実行"

# Docker環境の構築
cd docker
docker compose build ubuntu-test

# テスト実行
docker compose run --rm ubuntu-test bash -c "
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