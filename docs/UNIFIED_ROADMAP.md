# Lambdust (λust) 統一開発ロードマップ

## 🎯 概要

このドキュメントは、Lambdust Scheme interpreterの全開発タスクを**単一の軸**で直列化した統一ロードマップです。従来の混在していたPhase番号を廃止し、明確な進捗管理を実現します。

## 📊 現在の完成状況（2025年7月最新）

### ✅ 完成済み基盤システム (Completed Foundation)

1. **Core Interpreter Infrastructure** ✅ 100%
   - Lexer・Parser・AST system (完全実装)
   - CPS-based evaluator (569テスト通過)
   - Environment management (COW最適化完成)
   - Built-in functions library (R7RS準拠完成)

2. **Advanced Type System** ✅ 100%
   - Value system with comprehensive types
   - Record types (SRFI 9完成)
   - Promise/delay system (完全実装)
   - External object integration (Bridge API完成)
   - UniqueTypeInstance (SRFI 137完成)

3. **Memory Management System** ✅ 100%
   - RAII-based automatic memory management
   - Copy-on-Write environments (SharedEnvironment完成)
   - Memory pool optimization (continuation pooling実装)
   - Garbage collection elimination (RAII store完成)

4. **3段階評価器分離アーキテクチャ** ✅ 67% (2/3完成)
   - **SemanticEvaluator**: ✅ 完成 (純粋R7RS形式的意味論・12テスト通過)
   - **RuntimeExecutor**: ✅ 完成 (最適化統合・10テスト通過・warning free)
   - **EvaluatorInterface**: ⏳ 未実装 (統一API・透明切り替え)

5. **コンビネータ理論統合** ✅ 100%
   - SKIコンビネータシステム (基本・拡張完全実装)
   - Lambda→コンビネータ変換 (bracket abstraction完成)
   - SemanticEvaluator統合 (15テスト通過)
   - 定理証明基盤 (Church-Rosser性保証)

6. **SRFI実装システム** ✅ 80%
   - SRFI 137-141: ✅ 完成 (R7RS Large基盤)
   - SRFI レジストリ: ✅ 完成 (動的管理システム)
   - 拡張SRFI: ⏳ 継続中 (SRFI 142+)

## 🚀 統一開発タスク体系 (Unified Task System)

### 🎯 最優先タスク (Critical Priority)

#### **T1: EvaluatorInterface実装** ⚡ CRITICAL
- **Goal**: 3段階評価器アーキテクチャ完成
- **Status**: SemanticEvaluator (✅) → RuntimeExecutor (✅) → EvaluatorInterface (⏳ 0%)
- **Dependencies**: 完成済みSemanticEvaluator・RuntimeExecutor
- **Deliverables**: 
  - 統一API設計・実装
  - 意味論↔実行の透明切り替え
  - verification system (SemanticEvaluator基準)
- **Impact**: 全最適化システムの基盤・形式的検証可能性確保
- **Estimated**: 1-2週間

#### **T2: Tail Call Optimization完成** 🔧 HIGH
- **Goal**: 完全な末尾呼び出し最適化
- **Status**: 基本フレームワーク実装済み (⏳ 40%)
- **Dependencies**: RuntimeExecutor統合
- **Deliverables**:
  - LLVM backend統合
  - 再帰関数最適化
  - Stack overflow完全解決
- **Impact**: 大規模プログラム実行・production stability
- **Estimated**: 2-3週間

#### **T3: パフォーマンス測定システム** 📊 HIGH
- **Goal**: 最適化効果定量評価
- **Status**: 基盤準備完了 (⏳ 20%)
- **Dependencies**: RuntimeExecutor・EvaluatorInterface
- **Deliverables**:
  - ベンチマークシステム
  - 最適化効果測定
  - パフォーマンス回帰検出
- **Impact**: 最適化の正当性保証・継続的改善
- **Estimated**: 1-2週間

### 🔧 高優先度タスク (High Priority)

#### **T4: JIT統合・最適化完成** ⚡ HIGH
- **Goal**: ネイティブレベルパフォーマンス実現
- **Status**: JIT loop system実装済み (⏳ 60%)
- **Dependencies**: EvaluatorInterface・パフォーマンス測定システム
- **Deliverables**:
  - Native code generation統合
  - Hot path detection強化
  - Dynamic optimization strategy
- **Impact**: Native実装に匹敵するパフォーマンス
- **Estimated**: 3-4週間

#### **T5: 形式的検証システム強化** 🔬 HIGH
- **Goal**: 数学的正当性保証の完成
- **Status**: コンビネータ理論基盤完成 (⏳ 70%)
- **Dependencies**: SemanticEvaluator・定理証明基盤
- **Deliverables**:
  - Agda/Coq統合
  - Correctness guarantee system
  - 形式的プログラム検証
- **Impact**: 学術的価値・産業応用における信頼性
- **Estimated**: 4-6週間

#### **T6: 拡張SRFI実装継続** 📚 HIGH
- **Goal**: R7RS Large完全対応
- **Status**: SRFI 137-141完成 (⏳ 80%)
- **Dependencies**: SRFI基盤システム
- **Deliverables**:
  - SRFI 142-150実装
  - R7RS Large compatibility完成
  - 包括的テストスイート
- **Impact**: R7RS Large準拠・エコシステム互換性
- **Estimated**: 2-3週間

### 🚀 中優先度タスク (Medium Priority)

#### **T7: LLVM Backend完成** 🎯 MEDIUM
- **Goal**: 機械語コンパイルシステム
- **Status**: 基盤準備完了 (⏳ 30%)
- **Dependencies**: Tail call optimization・JIT統合
- **Deliverables**:
  - LLVM IR生成
  - 最適化パイプライン
  - Machine code output
- **Impact**: 最高性能実現・ネイティブ並み実行速度
- **Estimated**: 6-8週間

#### **T8: 完全Hygienic Macros** 🔄 MEDIUM
- **Goal**: 高度メタプログラミング能力
- **Status**: 基盤・syntax parameters完成 (⏳ 60%)
- **Dependencies**: マクロシステム基盤
- **Deliverables**:
  - 完全hygiene実装
  - Macro expansion最適化
  - Advanced syntax-rules
- **Impact**: 言語表現力の飛躍的向上
- **Estimated**: 4-5週間

### 🌟 将来展開タスク (Future Development)

#### **T9: 開発ツール・デバッガ** 🛠️ FUTURE
- **Goal**: 完全開発環境
- **Status**: エラー報告基盤完成 (⏳ 25%)
- **Dependencies**: EvaluatorInterface・形式的検証
- **Impact**: 開発者体験・エコシステム成長

#### **T10: パッケージエコシステム** 📦 FUTURE
- **Goal**: Schemeパッケージ管理
- **Status**: 設計段階 (⏳ 10%)
- **Dependencies**: R7RS Large対応完成
- **Impact**: コミュニティ成長・産業応用拡大

#### **T11: 高度言語機能** 🎭 FUTURE
- **Goal**: 現代的言語機能
- **Status**: 設計段階 (⏳ 5%)
- **Dependencies**: 全基盤システム完成
- **Impact**: 言語競争力・研究価値向上

## 🎯 実行計画・優先度マトリクス

### ⚡ 即時実行 (Next 1-2 weeks)
**Critical Path - 全後続タスクの基盤**
1. **T1: EvaluatorInterface実装** - 統一API・transparent switching・verification
2. **T3: パフォーマンス測定システム** - ベンチマーク・最適化効果測定・回帰検出

### 🔧 短期実行 (Next 2-4 weeks)  
**High Impact - 重要機能完成**
3. **T2: Tail call optimization完成** - LLVM統合・stack overflow解決
4. **T6: 拡張SRFI実装継続** - SRFI 142-150・R7RS Large完成

### 🚀 中期実行 (Next 1-3 months)
**Advanced Features - 競争力強化**
5. **T4: JIT統合・最適化完成** - Native code generation・hot path detection
6. **T5: 形式的検証システム強化** - Agda/Coq統合・correctness guarantee
7. **T8: 完全Hygienic Macros** - 高度メタプログラミング・syntax-rules最適化

### 🌟 長期実行 (Next 3-6 months)
**Ecosystem Building - エコシステム拡張**
8. **T7: LLVM Backend完成** - 機械語コンパイル・最適化パイプライン
9. **T9: 開発ツール・デバッガ** - 完全開発環境・デバッガ・プロファイラ
10. **T10: パッケージエコシステム** - パッケージマネージャ・コミュニティ成長

## 🔄 統一開発フロー原則

### 🛡️ 品質保証体系
- **Test-Driven**: 全新機能に先行テスト実装必須
- **Backward Compatible**: 既存機能の互換性絶対保持
- **Warning-Free**: コンパイル時警告完全除去維持
- **Formal Verification**: SemanticEvaluator基準の正当性検証
- **Continuous Integration**: 自動テスト・ベンチマーク・回帰検出

### ⚡ パフォーマンス原則
- **Memory Efficiency**: RAII・COW・pooling優先
- **Zero-Cost Abstractions**: 抽象化コスト完全除去
- **Benchmark-Driven**: 定量的パフォーマンス改善
- **Production Ready**: 大規模実用に耐える品質
- **Optimization Transparency**: 最適化効果の可視化・検証

### 🏗️ アーキテクチャ原則
- **Modular Design**: 疎結合・高凝集モジュール設計
- **Clean API**: 明確で一貫したインターフェース
- **Separation of Concerns**: 意味論・実行・最適化の分離
- **Extensibility**: 将来拡張に対する柔軟性
- **Documentation-First**: 設計文書・API文書の先行作成

## 📈 定量的成功指標・マイルストーン

### 🎯 短期達成目標 (1-3ヶ月)
**Critical Milestones - 基盤完成**
- [ ] **EvaluatorInterface完成** (T1) - 完全な3層分離アーキテクチャ
  - [ ] 統一API実装 - 意味論↔実行透明切り替え
  - [ ] Verification system - SemanticEvaluator基準検証
  - [ ] 10+テスト通過 - API互換性・機能正当性確認
- [ ] **Stack overflow完全解決** (T2) - 大規模プログラム実行可能
  - [ ] Tail call optimization完成 - 無限再帰実行可能
  - [ ] LLVM backend統合 - ネイティブ末尾呼び出し
  - [ ] 1M+ 反復耐性テスト通過
- [ ] **パフォーマンス可視化** (T3) - 最適化効果定量評価
  - [ ] ベンチマークスイート - 継続的性能測定
  - [ ] 回帰検出システム - 性能劣化自動検出
  - [ ] 最適化効果追跡 - RuntimeExecutor vs SemanticEvaluator

### 🚀 中期達成目標 (3-6ヶ月)  
**Advanced Features - 競争力確立**
- [ ] **R7RS Large完全対応** (T6) - 標準準拠達成
  - [ ] SRFI 142-150実装完成 - 全標準ライブラリ対応
  - [ ] 互換性テスト100%通過 - 他R7RS実装との互換
  - [ ] Performance parity - 同等以上の実行性能
- [ ] **JIT compilation実用化** (T4) - ネイティブレベルパフォーマンス
  - [ ] 2-10x性能改善 - インタープリタ比較
  - [ ] Hot path自動検出 - 動的最適化
  - [ ] Memory efficiency保持 - 最適化コスト最小化
- [ ] **形式的検証基盤** (T5) - 数学的正当性保証
  - [ ] Agda/Coq統合 - 外部証明器連携
  - [ ] Correctness theorem証明 - SemanticEvaluator正当性
  - [ ] Program verification例 - 実用的検証ケース

### 🌟 長期達成目標 (6-12ヶ月)
**Ecosystem Leadership - 生態系主導**
- [ ] **Production Ready達成** (T7) - 産業応用可能
  - [ ] LLVM backend完成 - 機械語コンパイル実現
  - [ ] Native performance - C/Rust並み実行速度
  - [ ] Enterprise stability - 大規模システム運用可能
- [ ] **Developer Experience最高級** (T9) - 開発者体験完成
  - [ ] IDE統合 - シンタックスハイライト・自動補完
  - [ ] 統合デバッガ - ステップ実行・変数監視
  - [ ] 性能プロファイラ - ボトルネック可視化
- [ ] **Community Ecosystem** (T10) - コミュニティ主導成長
  - [ ] パッケージマネージャ - 依存解決・バージョン管理
  - [ ] 公開レジストリ - パッケージ共有・発見
  - [ ] 自立的成長 - コミュニティ主導開発

## 📊 定量指標・KPI

### 技術的KPI
- **テスト通過率**: 95%+ (現在: 569テスト通過)
- **コンパイル警告**: 0件維持 (Warning-free)
- **メモリ効率**: RAII完全適用・leak detection 100%
- **パフォーマンス**: SemanticEvaluator比2-10x改善目標

### 品質KPI  
- **Backward compatibility**: 100%維持
- **R7RS準拠**: Large完全対応 (現在: Small完成・Large 80%)
- **形式的検証**: 主要アルゴリズム正当性証明
- **文書品質**: API documentation完全性・実例豊富

---

## 🎯 統一ロードマップ完成宣言

このロードマップは**従来のPhase番号混在問題を完全解決**し、明確で追跡可能な直列開発進捗管理を実現します。各タスクは:

✅ **独立評価可能** - 明確な成果物・成功指標  
✅ **依存関係明確** - 前提条件・実行順序の可視化  
✅ **定量的追跡** - 進捗率・完成時期の客観的測定  
✅ **優先度体系** - Critical→High→Medium→Future の明確な階層

**Next Action**: T1 EvaluatorInterface実装開始 - 統一API設計・透明切り替え・verification system構築