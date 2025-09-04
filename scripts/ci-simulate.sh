#!/bin/bash
set -e

echo "🤖 CI環境完全再現テスト"

cd docker
docker compose run --rm ubuntu-test bash -c "
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