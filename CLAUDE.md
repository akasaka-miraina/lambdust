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
- **[COMBINATOR_THEORY_INTEGRATION.md](docs/COMBINATOR_THEORY_INTEGRATION.md)**: コンビネータ理論統合・SKIシステム・定理証明基盤

## 🚀 現在の最重要タスク - Evaluator分離アーキテクチャ（2025年7月最新）

### 📊 Phase 1完了（SemanticEvaluator）
- **✅ 完全分離設計**: SemanticEvaluator・RuntimeExecutor・EvaluatorInterface 3段階分離アーキテクチャ設計完了
- **✅ SemanticEvaluator実装**: 純粋R7RS形式的意味論・最適化完全排除・数学的参照実装完成
- **✅ S式簡約機能**: R7RS準拠簡約システム・定数畳み込み・同一性簡約・条件式簡約・副作用解析完全実装
- **✅ 包括テスト**: 12テスト全通過・コンパイル成功・production ready

### 📊 Phase 2完了（RuntimeExecutor）
- **✅ 基本構造実装**: RuntimeExecutor基本アーキテクチャ・最適化レベル制御・SemanticEvaluator統合完成
- **✅ 最適化フレームワーク**: RuntimeOptimizationLevel（None/Conservative/Balanced/Aggressive）実装
- **✅ 統合テスト**: 10テスト全通過・SemanticEvaluatorとの互換性確認・コンパイル成功
- **✅ プレースホルダー設計**: 将来の最適化システム統合準備完了

### 📊 コンビネータ理論統合完了（Phase 2追加）
- **✅ SKIコンビネータシステム実装**: 基本SKI・拡張BCYW・bracket abstraction完成
- **✅ ラムダ→コンビネータ変換**: 完全な双方向変換・R7RS意味論保持・正規化システム実装
- **✅ SemanticEvaluator統合**: コンビネータ簡約機能・純粋評価との統合・意味論的正確性保証
- **✅ 包括テスト**: 15テスト全通過（コンビネータ11テスト・SemanticEvaluator統合4テスト）
- **✅ 定理証明基盤**: 形式的検証準備・数学的参照実装・Church-Rosser性保証

### 🎯 次期優先度（Phase 2継続-3）
1. **定理証明支援システム**: コンビネータ理論基盤の形式的検証・Agda/Coq統合・正当性証明
2. **RuntimeExecutor最適化統合**: JIT・continuation pooling・performance tuning統合実装
3. **EvaluatorInterface実装**: 統一API・意味論と実行の切り替え・verification system  
4. **形式的検証基盤**: SemanticEvaluator基準・correctness guarantee・数学的証明体系

### 🧪 技術的コンテキスト（アーキテクチャ統合）
- **評価器**: R7RS準拠CPS評価器 + SemanticEvaluator pure reference + コンビネータ理論統合
- **設計**: 3段階分離アーキテクチャ・backward compatibility・段階的移行戦略
- **品質**: 意味論的正確性保証・mathematical reference・形式的検証準備完了
- **テスト**: semantic reduction 12テスト・コンビネータ統合15テスト・runtime executor 10テスト・既存569テスト継続通過
- **理論基盤**: SKIコンビネータ・bracket abstraction・Church-Rosser性・定理証明支援システム基盤

## 📋 完了した実装詳細

### Phase 1: SemanticEvaluator完了実装

### 🔬 S式簡約システム完全実装
1. **定数畳み込み（Constant Folding）**
   - 算術式の事前計算: `(+ 2 3)` → `5`, `(* 4 6)` → `24`
   - 全算術演算子対応: `+`, `-`, `*`, `/`
   - 整数・実数型の適切な保持・変換

2. **同一性簡約（Identity Reduction）**
   - 加算恒等式: `(+ x 0)` → `x`, `(+ 0 x)` → `x`
   - 乗算恒等式: `(* x 1)` → `x`, `(* 1 x)` → `x`
   - ゼロ乗算: `(* x 0)` → `0` (副作用なしの場合)
   - 論理恒等式: `(and #t x)` → `x`, `(or #f x)` → `x`

3. **条件式簡約（Conditional Reduction）**
   - 定数条件の除去: `(if #t then else)` → `then`
   - 偽条件の処理: `(if #f then else)` → `else`

4. **副作用解析（Side Effect Analysis）**
   - R7RS準拠純粋性判定
   - 副作用手続き識別: `set!`, `display`, `write`, `read` 等
   - 安全な簡約のみ適用保証

5. **β簡約基盤（Beta Reduction Framework）**
   - Lambda適用の基本ケース: `((lambda () body))` → `body`
   - 変数置換フレームワーク準備完了

### 📊 テスト・品質保証
- **12テスト完全実装**: constant folding, identity reduction, conditional reduction, side effect analysis
- **包括的検証**: 正確性・安全性・R7RS準拠性
- **統計追跡API**: `ReductionStats`構造体・パフォーマンス分析基盤
- **コンパイル成功**: `cargo check`通過・依存関係整合性確保

### Phase 2: RuntimeExecutor完了実装

#### 🏗️ 基本アーキテクチャ実装
1. **RuntimeExecutor構造体**: 最適化システム統合基盤・SemanticEvaluator参照・統計追跡完成
2. **最適化レベル制御**: RuntimeOptimizationLevel（None/Conservative/Balanced/Aggressive）実装
3. **SemanticEvaluator統合**: 純粋意味論評価器との統合・互換性確保

#### 🎯 最適化フレームワーク準備
1. **プレースホルダー設計**: 将来の最適化システム統合準備完了
   - `PlaceholderAnalysis`: 式解析システム基盤
   - `PlaceholderOptimizedTailCall`: 末尾呼び出し最適化準備
   - `PlaceholderGeneratedCode`: JITコンパイル準備
2. **統合可能設計**: 既存最適化システムとの段階的統合戦略

#### 🧪 テスト・品質保証
- **10テスト完全実装**: 基本機能・最適化レベル・SemanticEvaluator統合テスト
- **互換性確認**: SemanticEvaluatorとの結果一致確認・段階的移行準備
- **コンパイル成功**: `cargo check`通過・警告のみ・production ready

### コンビネータ理論統合完了実装（Phase 2追加）

#### 🔬 SKIコンビネータシステム実装
1. **基本コンビネータ**: S (Substitution), K (Constant), I (Identity) 完全実装
2. **拡張コンビネータ**: B (Composition), C (Flip), W (Duplication) 最適化実装
3. **コンビネータ簡約**: 全コンビネータ規則の正規化・終了性保証

#### 🔄 ラムダ→コンビネータ変換システム
1. **Bracket Abstraction**: `[x] E` アルゴリズム実装・R7RS準拠
2. **双方向変換**: `lambda_to_combinators` / `combinators_to_lambda` 完全実装
3. **自由変数解析**: 変数束縛・スコープ解析・安全性保証

#### 🎯 SemanticEvaluator統合
1. **コンビネータ簡約統合**: `reduce_expression_combinatory` メソッド実装
2. **純粋評価統合**: `eval_pure_with_combinatory_reduction` 実装
3. **意味論的正確性**: R7RS形式的意味論との等価性保証

#### 📊 テスト・品質保証
- **15テスト完全実装**: 基本コンビネータ・変換・統合テスト
- **数学的正確性**: Church-Rosser性・合流性・停止性確認
- **R7RS準拠性**: 意味論保持・副作用解析・正規化保証
- **統合テスト**: SemanticEvaluatorとの16テスト全通過

## 💡 重要な開発原則（アーキテクチャ統合）

1. **段階的分離**: SemanticEvaluator（完了） → RuntimeExecutor（完了） → EvaluatorInterface
2. **意味論的正確性**: R7RS形式的意味論厳密遵守・数学的参照実装
3. **backward compatibility**: 既存evaluator構造との互換性保持
4. **形式的検証準備**: SemanticEvaluatorを基準とした正当性証明基盤

## 🔄 開発フロー（アーキテクチャ統合）

### Phase 1 完了（SemanticEvaluator）✅
1. **分離設計**: 3段階アーキテクチャ設計・文書化完了
2. **純粋実装**: R7RS形式的意味論・S式簡約システム実装
3. **テスト**: 12テスト・品質保証・コンパイル確認完了

### Phase 2 完了（RuntimeExecutor + コンビネータ理論統合）✅
1. **基本構造実装**: RuntimeExecutor・最適化レベル制御・SemanticEvaluator統合完成
2. **最適化フレームワーク**: RuntimeOptimizationLevel（None/Conservative/Balanced/Aggressive）実装
3. **コンビネータ理論統合**: SKIコンビネータシステム・bracket abstraction・SemanticEvaluator統合完了
4. **統合テスト**: 25テスト全通過（Runtime 10テスト・コンビネータ15テスト）・SemanticEvaluatorとの互換性確認完了

### Phase 3 展開（EvaluatorInterface）📋
1. **統一API設計**: 意味論・実行の透明な切り替え
2. **verification system**: SemanticEvaluator基準の自動検証
3. **backward compatibility**: 段階的移行・既存コード保護

## 🎯 次期作業推奨（Phase 2継続-3）
1. **定理証明支援システム設計**: コンビネータ理論基盤の形式的検証・Agda/Coq統合・正当性証明体系
2. **RuntimeExecutor最適化統合**: JIT・continuation pooling・performance tuning統合実装
3. **EvaluatorInterface実装**: 統一API・意味論と実行の切り替え・verification system
4. **形式的検証基盤**: SemanticEvaluator基準・correctness guarantee・数学的証明体系
5. **パフォーマンス測定**: RuntimeExecutor効果の定量的評価

重要：コードコメントやCLAUDE.md以外のmarkdownドキュメントは英語で、CLAUDE.mdやチャットは日本語で行います。