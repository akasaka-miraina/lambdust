# ビルド・テストコマンド

## 基本コマンド

```bash
# ビルド
cargo build

# テスト実行
cargo test

# 特定のテストファイル実行
cargo test phase6a_trampoline_tests

# 特定のテスト関数実行
cargo test test_trampoline_prevents_stack_overflow

# リリースビルド
cargo build --release

# ドキュメント生成
cargo doc --open

# フォーマット
cargo fmt

# リント
cargo clippy

# 開発用: フォーマット・リント・テスト一括実行
make dev-check

# カバレッジ生成・表示
make coverage-open

# 全CI確認
make ci-check

# REPLの実行
cargo run --features repl

# ベンチマーク実行
cargo bench

# 特定のベンチマーク実行
cargo bench --bench performance_benchmark
```

## テストカバレッジ

```bash
# カバレッジ生成
cargo tarpaulin --out html --output-dir coverage

# カバレッジ表示
open coverage/tarpaulin-report.html
```

## 開発状況

- [x] 基本設計完了
- [x] 字句解析器実装
- [x] 構文解析器実装
- [x] **評価器統合完了**（R7RS形式的意味論準拠CPS評価器に統一・従来evaluator完全削除）
- [x] **🎯 評価器モジュール化完了（2025年1月）**: 2752行の巨大evaluator.rsを7つの機能別モジュールに分割・可読性と保守性向上
- [x] 組み込み関数実装（99%完了：103個の標準関数）
- [x] **例外処理システム完成**（raise, with-exception-handler, guard構文実装）
- [x] マクロシステム実装（SRFI 9, 45, 46対応）
- [x] **外部API完全実装**（ホスト連携・マーシャリング・型安全性確保）
- [x] **テスト完備**（564テスト全パス）
- [x] ドキュメント整備
- [x] CI/CD パイプライン構築（GitHub Actions）
- [x] 開発フロー整備（Issue/PRテンプレート、GitHub Copilot統合）
- [x] **アーキテクチャ統合**（公開API完全formal evaluator移行）
- [x] **パフォーマンス最適化Phase 1-3完了**（継続インライン・メモリ効率・GC最適化）
- [x] **🎯 R7RS最終機能完成（2025年1月）**: doループ・call/cc・guard構文完全実装
- [x] **🎯 SRFIモジュール統合（2025年1月）**: SRFI 1・13・69統一SrfiModule trait実装
- [x] **🎯 RAII統合メモリ管理完成（2025年1月）**: Rust特性活用・Drop trait自動cleanup・unified memory strategy
- [x] **🎯 Phase 6-C統合完了（2025年7月マージ）**: JIT最適化・SRFI 141・stack overflow解決完成・production ready達成
- [x] **🎯 Bridge Module Tests完全実装（2025年7月最新成果）**: 78テスト・48.97%カバレッジ・外部統合API包括検証・coverage improvement基盤確立