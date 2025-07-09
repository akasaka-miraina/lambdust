# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Lambdust (λust) - Rust Scheme Interpreter

このファイルは、このリポジトリで作業する際のClaude Code（claude.ai/code）へのガイダンスを提供します。

## 📚 ドキュメント構成

プロジェクトのドキュメントは以下のように整理されています：

- **[PROJECT_OVERVIEW.md](docs/PROJECT_OVERVIEW.md)**: プロジェクト概要・開発状況・基本方針
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)**: アーキテクチャ設計・モジュール構成・技術的詳細
- **[DEVELOPMENT_FLOW.md](docs/DEVELOPMENT_FLOW.md)**: 開発フロー・作業手順・品質チェック
- **[BUILD_COMMANDS.md](docs/BUILD_COMMANDS.md)**: ビルドコマンド・テスト実行・開発ツール
- **[CURRENT_TASKS.md](docs/CURRENT_TASKS.md)**: 現在のタスク・優先度・技術課題
- **[R7RS_IMPLEMENTATION.md](docs/R7RS_IMPLEMENTATION.md)**: R7RS実装状況・SRFI対応・機能完成度

## 🚀 現在の最重要タスク

### 📊 最新状況（2025年7月）
- **✅ Bridge Module Tests完全実装**: 78テスト・48.97%カバレッジ・外部統合API包括検証完了
- **🎯 現在の目標**: 核心部分テスト強化・80%カバレッジ達成
- **次期優先度**: evaluator・value・parser系重要モジュールテスト実装

### 🧪 技術的コンテキスト
- **評価器**: R7RS準拠CPS評価器・完全統合済み
- **テスト**: 564/564テスト全通過・Bridge Module Tests追加
- **アーキテクチャ**: モジュール化完了・production ready
- **品質**: pre-commitフック・Clippy・自動品質チェック

## 💡 重要な開発原則

1. **核心部分優先**: evaluator・value・parser系の重要モジュール優先テスト
2. **品質保証**: 継続・値システム・構文解析器の包括的テスト実装
3. **カバレッジ向上**: Bridge Module Tests成功パターンを核心部分に適用
4. **コード品質**: Clippy警告ゼロ・pre-commitフック通過必須

## 🔄 作業フロー

1. **現状確認**: 各ドキュメントで最新状況確認
2. **優先度決定**: 核心部分テスト・カバレッジ向上を最優先
3. **実装**: 品質チェック通過・comprehensive testing
4. **更新**: 作業完了時にドキュメント更新

重要：コードコメントやCLAUDE.md以外のmarkdownドキュメントは英語で、CLAUDE.mdやチャットは日本語で行います。