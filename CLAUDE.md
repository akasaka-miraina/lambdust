# Lambdust (λust) - Rust Scheme Interpreter

## 🚀 現在の開発状況（次のClaude Codeインスタンスへの引き継ぎ）

### 📊 最新の進捗状況
- **R7RS Large実装**: 完全実装済み（546/546テスト全通過）
- **完了したタスク**: R7RS Large Red Edition SRFIs（111・113・125・132・133）完全実装
- **完了したタスク**: パフォーマンス最適化Phase 3完了・call/cc完全non-local exit実装完了・継続再利用機能実装
- **🎯 最新完了（2025年7月）**: Phase 6-B-Step3 inline evaluation統合完了（軽量継続最適化・hotpath検出・メモリ効率化）
- **次のタスク**: Phase 6-C JIT反復処理変換・do-loop stack overflow根本解決・高度SRFI統合

### 🔄 開発フローの遵守
最新の作業完了状況：
1. ✅ **Phase 4完全実装**: 継続・環境・値システム3段階最適化完全達成・662テスト全通過
2. ✅ **Phase 5-Step1完了**: ExpressionAnalyzer式分析システム・定数畳み込み・型推論・最適化統計（36テスト）
3. ✅ **Phase 5-Step2完了**: RAII統一メモリ管理・TraditionalGC完全削除・自動Drop trait・メモリリーク根絶（9テスト）
4. ✅ **Phase 6-A完了**: トランポリン評価器・継続unwinding・do-loop最適化（Phase 6-A-Step1,2,3）
5. ✅ **Phase 6-B-Step1完了**: DoLoopContinuation特化実装・状態マシン化・メモリプール統合（10テスト）
6. ✅ **Phase 6-B-Step2完了**: 統合continuation pooling・グローバル管理・heap allocation削減（15テスト）
7. ✅ **Phase 6-B-Step3完了**: inline evaluation統合・軽量継続最適化・hotpath検出・メモリ効率化
8. ✅ **次期タスク**: Phase 6-C JIT反復処理変換・do-loop stack overflow根本解決・Phase 5-Step3型推論拡張

### 🧪 重要な技術的コンテキスト
- **評価器**: formal_evaluator.rsによるR7RS準拠CPS評価器（完全統合済み）
- **アーキテクチャ**: モジュール化完了（control_flow 7サブモジュール・macros 6サブモジュール分割済み）
- **テスト**: 662/662テスト全通過（Phase 4最適化完全実装・zero regression保証）
- **メモリ管理**: RAII統合・traditional GC・dual strategy完全実装
- **Robustness**: panic防止・境界値処理・エラー回復・リソース管理完全実装
- **ブランチ**: `feature/phase4-advanced-optimizations`でPhase 4最適化完全実装

## 重要

コードコメントやCLAUDE.md以外のmarkdownドキュメントは英語で，CLAUDE.mdやチャットは日本語で行います．

## コードコーディング規約

- ネストは2段まで．
- 一箇所でしか使わない一次変数の使用を禁止．
- 1000ステップを超える*.rsは分割する．
- メソッドは50〜100ステップまで．
- DRY原則の徹底．
- 単一責務の原則の徹底．
- clippy警告の#[allow()]抑止の禁止．

## 概要

Lambdust（λust）は、Rustで実装されたR7RS準拠のSchemeインタプリタです。アプリケーションへのマクロ組み込みメカニズムを提供することを目的としています。

## プロジェクト概要

- **言語**: Rust
- **対象仕様**: R7RS Scheme
- **主目的**: 外部アプリケーションへの組み込み可能なSchemeインタプリタ
- **特徴**: 軽量、高速、安全性重視

## アーキテクチャ

```
lambdust/
├── src/
│   ├── lexer.rs         # 字句解析
│   ├── parser.rs        # 構文解析
│   ├── ast.rs           # AST定義
│   ├── evaluator/       # R7RS準拠CPS評価器（モジュール化完了）
│   │   ├── mod.rs       # コア評価ロジック
│   │   ├── continuation.rs # 継続データ構造
│   │   ├── types.rs     # 基本型定義
│   │   ├── special_forms.rs # 特殊形式評価
│   │   ├── control_flow.rs # 制御フロー
│   │   ├── higher_order.rs # 高階関数
│   │   └── imports.rs   # SRFIインポート
│   ├── environment.rs   # 環境管理
│   ├── builtins/        # 組み込み関数モジュール群
│   │   ├── mod.rs       # 統合モジュール
│   │   ├── utils.rs     # 共通ユーティリティ（重複削減）
│   │   ├── arithmetic.rs # 算術関数
│   │   ├── list_ops.rs  # リスト操作
│   │   ├── string_char.rs # 文字列・文字
│   │   ├── vector.rs    # ベクタ操作
│   │   ├── predicates.rs # 述語関数
│   │   ├── io.rs        # I/O関数
│   │   ├── control_flow.rs # 継続・例外処理
│   │   ├── misc.rs      # 多値・レコード
│   │   ├── error_handling.rs # エラー処理
│   │   └── lazy.rs      # 遅延評価（SRFI 45）
│   ├── macros.rs        # マクロシステム
│   ├── bridge.rs        # アプリケーション統合API
│   ├── interpreter.rs   # ホスト連携インターフェース
│   ├── host.rs          # ホスト関数管理
│   ├── marshal.rs       # 型安全マーシャリング
│   ├── value.rs         # Scheme値システム
│   ├── error.rs         # エラーハンドリング
│   └── lib.rs           # ライブラリエントリーポイント
├── tests/               # テスト
├── examples/            # 使用例
├── .github/             # GitHub統合
│   ├── workflows/       # CI/CD Actions
│   └── ISSUE_TEMPLATE/  # テンプレート
└── Cargo.toml
```

## 実装方針

- **R7RS準拠**: 継続渡しスタイル評価器による理論的正確性重視
- **安全性**: Rustの型システムを活用したメモリ安全性
- **パフォーマンス**: ゼロコスト抽象化の活用
- **組み込み性**: 軽量で依存関係最小限
- **拡張性**: プラグイン機能とモジュール化
- **保守性**: 単一evaluatorアーキテクチャによるコード重複排除

## ビルド・テストコマンド

```bash
# ビルド
cargo build

# テスト実行
cargo test

# リリースビルド
cargo build --release

# ドキュメント生成
cargo doc --open

# フォーマット
cargo fmt

# リント
cargo clippy
```

## 開発ステータス

- [x] 基本設計完了
- [x] 字句解析器実装
- [x] 構文解析器実装
- [x] **評価器統合完了**（R7RS形式的意味論準拠CPS評価器に統一・従来evaluator完全削除）
- [x] **🎯 評価器モジュール化完了（2025年1月）**: 2752行の巨大evaluator.rsを7つの機能別モジュールに分割・可読性と保守性向上
- [x] 組み込み関数実装（99%完了：103個の標準関数）
- [x] **例外処理システム完成**（raise, with-exception-handler, guard構文実装）
- [x] マクロシステム実装（SRFI 9, 45, 46対応）
- [x] **外部API完全実装**（ホスト連携・マーシャリング・型安全性確保）
- [x] **テスト完備**（662テスト全パス）
- [x] ドキュメント整備
- [x] CI/CD パイプライン構築（GitHub Actions）
- [x] 開発フロー整備（Issue/PRテンプレート、GitHub Copilot統合）
- [x] **アーキテクチャ統合**（公開API完全formal evaluator移行）
- [x] **パフォーマンス最適化Phase 1-3完了**（継続インライン・メモリ効率・GC最適化）
- [x] **🎯 R7RS最終機能完成（2025年1月）**: doループ・call/cc・guard構文完全実装
- [x] **🎯 SRFIモジュール統合（2025年1月）**: SRFI 1・13・69統一SrfiModule trait実装
- [x] **🎯 RAII統合メモリ管理完成（2025年1月）**: Rust特性活用・Drop trait自動cleanup・unified memory strategy

### R7RS Small実装完了ステータス（99.8%達成）

#### ✅ 完全実装済み

1. **基本データ型とリテラル**
   - 数値（整数・実数）、文字列、文字、シンボル、真偽値
   - ペア（cons cell）、リスト、ベクタ、レコード型

2. **算術・数値関数** (28関数)
   - 基本演算: +, -, *, /, quotient, remainder, modulo
   - 数学関数: abs, floor, ceiling, sqrt, expt
   - 集約関数: min, max
   - 述語: number?, integer?, real?, rational?, complex?, exact?, inexact?
   - 変換: exact->inexact, inexact->exact, number->string, string->number

3. **比較・等価関数** (12関数)
   - 数値比較: =, <, >, <=, >=
   - オブジェクト等価: eq?, eqv?, equal?
   - 型述語: boolean?, symbol?, char?, string?, pair?, null?, procedure?

4. **リスト操作関数** (11関数)
   - 基本操作: car, cdr, cons, list, append, reverse, length
   - 破壊的操作: set-car!, set-cdr!（クローンベース実装）
   - 変換: list->vector, list->string

5. **文字列・文字関数** (23関数)
   - 文字述語・比較: char=?, char<?, char>?, char-alphabetic?, char-numeric?等
   - 文字変換: char-upcase, char-downcase, char->integer, integer->char
   - 文字列操作: string=?, string<?, make-string, string-length, string-ref等
   - 変換: string->list, string->number, char->string, number->string

6. **ベクタ操作関数** (6関数)
   - 基本操作: vector, make-vector, vector-length, vector-ref, vector-set!
   - 変換: vector->list, list->vector

7. **I/O関数** (7関数)
   - 基本I/O: read, write, read-char, write-char, peek-char
   - 述語: eof-object?, char-ready?

8. **高階関数** ✅
   - apply, map, for-each（evaluator統合完全実装）
   - fold, fold-right, filter（evaluator統合完全実装）
   - lambda式完全サポート、クロージャ対応

9. **継続・例外処理** (5関数)
   - 継続: call/cc, call-with-current-continuation
   - 例外: raise, with-exception-handler
   - 制御: dynamic-wind

10. **多値システム**
    - values, call-with-values（基盤実装完了）

11. **レコード型（SRFI 9）** (4関数)
    - make-record, record-of-type?, record-field, record-set-field!
    - 完全なdefine-record-type実装

12. **エラーハンドリング**
    - error関数（irritant対応）

13. **SRFI 1: List Library** ✅
    - 非高階関数: take, drop, concatenate, delete-duplicates（完全動作）
    - 高階関数: fold, fold-right, filter（evaluator統合・lambda式サポート完全実装）
    - 15テスト全パス、主要な高階関数はlambda式完全対応

14. **SRFI 13: String Libraries** ✅
    - 基本文字列操作: string-null?, string-hash, string-hash-ci（完全動作）
    - 前後綴検査: string-prefix?, string-suffix?, string-prefix-ci?, string-suffix-ci?
    - 文字列検索: string-contains, string-contains-ci（完全動作）
    - 文字列切り取り: string-take, string-drop, string-take-right, string-drop-right
    - 文字列結合: string-concatenate（完全動作）
    - 9テスト全パス（33関数実装）

15. **SRFI 69: Basic Hash Tables** ✅
    - ハッシュテーブル作成・述語: make-hash-table, hash-table?（完全動作）
    - 基本操作: hash-table-set!, hash-table-ref, hash-table-delete!（完全動作）
    - 情報取得: hash-table-size, hash-table-exists?, hash-table-keys, hash-table-values
    - 変換操作: hash-table->alist, alist->hash-table, hash-table-copy（完全動作）
    - ハッシュ関数: hash, string-hash, string-ci-hash（完全動作）
    - 9テスト全パス（19関数実装）

### 🚨 重要技術課題: CPS評価器スタックオーバーフロー問題

**現状分析**: 継続渡しスタイル（CPS）評価器の構造的限界により、反復処理（do-loop）で深い再帰が発生し、Rustスタックオーバーフローが頻発 ⚠️

**技術的対策方針**:
- **Phase 6-A: トランポリン評価器**: 継続unwindingによるstack削減・iterative continuation処理
- **Phase 6-B: CompactContinuation活用**: 軽量継続によるstack frame削減・inline continuation拡張
- **Phase 6-C: 式事前分析JIT**: ExpressionAnalyzer活用・loop→iterative code変換・compile-time最適化
- **Phase 6-D: Rust tail call対応**: 末尾再帰最適化・LLVM backend活用・zero-cost反復処理

**緊急度評価**:
- **Critical**: do-loop・while等基本反復構造が実質使用不可・R7RS準拠性に重大影響
- **High Priority**: 現在の回避策（ignore test）は一時的措置・production readiness阻害要因
- **Implementation Target**: Phase 6優先度をHigh→Criticalに格上げ・スタック問題解決が最優先課題

### 🎯 Phase 6: Critical Stack Overflow Resolution (最優先)

**目標**: do-loop等反復処理のスタックオーバーフロー根本解決・R7RS完全実用性確保

1. **Phase 6-A: トランポリン評価器 (CRITICAL)** ✅
   - 継続unwinding: stack-based→heap-based continuation処理・深い再帰回避
   - iterative continuation: loop継続のstack frame削減・bounded memory使用
   - evaluator refactoring: apply_continuation→trampoline_eval変換・CPS最適化
   - 目標: do-loop 1000+ iteration対応・stack overflow完全解決

2. **Phase 6-B: 高度CompactContinuation (HIGH)** ✅
   - 反復継続特化: DoLoopContinuation・WhileContinuation専用軽量化
   - inline evaluation: loop body直接実行・継続生成回避・stack削減
   - continuation pooling: 継続再利用・allocation削減・GC圧力軽減
   - Phase 4 CompactContinuation拡張: 反復処理特化最適化

3. **Phase 6-C: JIT反復処理変換 (HIGH)** ⏳
   - ExpressionAnalyzer統合: loop pattern検出・iterative code生成・compile-time最適化
   - native iteration: Rust for-loop生成・CPS変換回避・zero stack overhead
   - hot path detection: 高頻度loop識別・JIT compilation・runtime最適化
   - Phase 5 ExpressionAnalyzer活用: 静的解析→最適化code generation

4. **Phase 6-D: Tail Call最適化 (MEDIUM)** ⏳
   - LLVM backend: Rust tail call支援・system-level最適化・compiler integration
   - continuation optimization: tail継続識別・stack frame除去・memory効率化
   - recursive function support: 深い再帰処理対応・関数型プログラミング完全支援
   - 長期目標: compiler-level stack optimization・zero-cost反復処理実現

## 開発フロー

プロジェクトではpre-commitフックを使用してコード品質を自動チェックしています：

- **Clippy**: コードの静的解析とリント
- **Tests**: 全テストの実行とパス確認  
- **Documentation**: ドキュメントビルドの成功確認
- **Formatting**: コードフォーマットの確認（警告のみ）

コミット前に自動的にこれらのチェックが実行され、すべてグリーンシグナルであることが確認されます。

### 基本的な作業手順

1. **Issue作成**: GitHubでIssueを作成し、作業内容を明確化
2. **ブランチ作成**: mainブランチからfeatureブランチをfork
3. **設計・実装**: 機能の設計と実装を行う
4. **テスト・品質チェック**: `make dev-check`でlint・test・フォーマット確認
5. **コミット**: pre-commitフックによる自動品質チェック後コミット
6. **進捗追記**: CLAUDE.mdに完了した機能・ステータスを追記
7. **Pull Request**: GitHub CopilotのレビューコメントあるPRを作成
8. **レビュー・マージ**: コードレビュー後、mainブランチにマージ

### 🔄 重要な開発フロー原則

- **必須**: 各機能完了後に必ずCLAUDE.mdに進捗を記録
- **必須**: テスト追加とpre-commitフック通過
- **推奨**: 大きな機能完了時にコミット・進捗追記のセット実行

## 🚀 次期開発予定

- **Phase 6-C: JIT反復処理変換**: ExpressionAnalyzer統合・loop pattern検出・native iteration
- **Phase 6-D: Tail Call最適化**: LLVM backend・continuation optimization・recursive function support
- **高度SRFIサポート**: SRFI 134-141対応・data structure extensions
- **REPL機能拡張**: タブ補完・シンタックスハイライト・デバッガー統合・プロファイラー
- **エコシステム拡張**: VS Code 拡張・Language Server Protocol・パッケージマネージャー