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
docker compose run --rm ubuntu-test bash -c "
    echo '🎯 対象テスト: $TEST_NAME'
    echo 'RUST_BACKTRACE=full cargo test $TEST_NAME --lib --no-default-features -- --nocapture'
    RUST_BACKTRACE=full cargo test '$TEST_NAME' --lib --no-default-features -- --nocapture
"