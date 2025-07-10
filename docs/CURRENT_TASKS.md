# 🚀 次期開発予定

## 🆘 **URGENT PRIORITY（核心部分テスト強化）**
- **核心部分テスト充実**: evaluator・value・parser系の重要モジュール優先テスト実装
- **80%カバレッジ達成**: Bridge Module Tests（48.97%）成功をベースに核心部分カバレッジ向上
- **品質保証**: 継続・値システム・構文解析器の包括的テスト実装・production readiness確保
- **🎯 品質管理継続**: Warning Free状態維持・missing docs解決・コード品質向上（[品質管理方針](DEVELOPMENT_FLOW.md#品質管理方針)遵守）

## 🎯 **HIGH PRIORITY（次期マイルストーン）**
- **🔥 衛生的マクロシステム実装**: シンボル衝突防止・HygienicSymbol・SymbolGenerator・真のR7RS準拠マクロ（[設計書](HYGIENIC_MACRO_DESIGN.md)）
- **高度SRFIサポート継続**: SRFI 137-141順次実装・R7RS Large完全対応
- **Phase 6-D tail call高度化**: LLVM backend・recursive function optimization・performance tuning

## 📈 **MEDIUM PRIORITY（機能拡張）**
- **REPL機能拡張**: タブ補完・シンタックスハイライト・デバッガー統合・プロファイラー
- **エコシステム拡張**: VS Code 拡張・Language Server Protocol

## 🌟 **LONG-TERM VISION（長期ビジョン）**
- **🚀 Dustpanエコシステム構想**: Cargo/npm相当パッケージマネージャー・dustpan.dev・コミュニティ機能（[詳細](DUSTPAN_ECOSYSTEM_VISION.md)）

## 🚨 重要技術課題: CPS評価器スタックオーバーフロー問題

**現状分析**: 継続渡しスタイル（CPS）評価器の構造的限界により、反復処理（do-loop）で深い再帰が発生し、Rustスタックオーバーフローが頻発 ⚠️

**技術的対策方針**:
- **Phase 6-A: トランポリン評価器**: 継続unwindingによるstack削減・iterative continuation処理
- **Phase 6-B: CompactContinuation活用**: 軽量継続によるstack frame削減・inline continuation拡張
- **Phase 6-C: 式事前分析JIT**: ExpressionAnalyzer活用・loop→iterative code変換・compile-time最適化 ✅
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

3. **Phase 6-C: JIT反復処理変換 (完了)** ✅
   - ✅ ExpressionAnalyzer統合: loop pattern検出・iterative code生成・compile-time最適化
   - ✅ native iteration: Rust for-loop生成・CPS変換回避・zero stack overhead 
   - ✅ hot path detection: 高頻度loop識別・JIT compilation・runtime最適化
   - ✅ Phase 5 ExpressionAnalyzer活用: 静的解析→最適化code generation
   - ✅ 13テスト全通過: loop pattern detection・native code generation・performance characteristics

4. **Phase 6-D: Tail Call最適化 (MEDIUM)** ⏳
   - LLVM backend: Rust tail call支援・system-level最適化・compiler integration
   - continuation optimization: tail継続識別・stack frame除去・memory効率化
   - recursive function support: 深い再帰処理対応・関数型プログラミング完全支援
   - 長期目標: compiler-level stack optimization・zero-cost反復処理実現